// 11_config_and_scripts.pwn — credentials in a file, schema in a file, and
// queries that return more than one result set.
//
// Three things that usually show up together when a server grows past the
// "one hardcoded connection" stage:
//
//   1. mysql_connect_file  — credentials outside the gamemode source.
//   2. mysql_query_file    — schema and migrations as .sql files.
//   3. cache_set_result    — reading a stored procedure that returns several
//                            result sets.
//
// Companion files in this folder: mysql.ini.example and schema.sql.

#include <a_samp>
#include <mysql_samp>

new g_MysqlConn = 0;

// --- 1. Connecting from a config file ----------------------------------------
//
// The credentials live in mysql.ini, which your repository does not carry.
// Connection OPTIONS stay here in code — there is one place to look for
// tuning, and the file holds nothing but credentials.

public OnGameModeInit()
{
    new opts = mysql_options_new();
    mysql_options_set_int(opts, MYSQL_OPT_CONNECT_TIMEOUT, 5);

    // Cap the pool when several game servers share one MySQL instance with a
    // tight max_connections. Without this the driver allows up to 100.
    mysql_options_set_int(opts, MYSQL_OPT_POOL_SIZE, 16);

    g_MysqlConn = mysql_connect_file("mysql.ini", opts);

    if (g_MysqlConn == 0)
    {
        // The log names the missing key (never the value). A missing file and
        // a malformed file both land here.
        new err[256];
        mysql_error(0, err);
        printf("[mysql] connect_file failed: %s", err);
        return 1;
    }

    ApplySchema();
    return 1;
}

public OnGameModeExit()
{
    if (g_MysqlConn != 0) mysql_close(g_MysqlConn);
    return 1;
}

// --- 2. Running a .sql file ---------------------------------------------------
//
// Statements run in order on one connection, non-blocking like every other
// query. Splitting happens on `;` outside string literals and comments.

ApplySchema()
{
    mysql_query_file(g_MysqlConn, "schema.sql", "OnSchemaReady");
    return 1;
}

forward OnSchemaReady();
public OnSchemaReady()
{
    // The callback receives the cache of the LAST statement in the file.
    printf("[mysql] schema applied (last statement affected %d row(s))", cache_affected_rows());
    return 1;
}

// A failure stops at the offending statement and fires OnQueryError. Note what
// this does NOT mean: the script is not a transaction, so every statement
// before the failure stays applied. DDL commits implicitly in MySQL, which is
// why wrapping the file in a transaction would be misleading rather than
// helpful.
public OnQueryError(errorid, const error[], const callback[], const query[], connId)
{
    if (strcmp(callback, "OnSchemaReady") == 0)
    {
        // The message names the position, e.g. "statement 3 of 4: ...".
        printf("[mysql] schema failed (%d): %s", errorid, error);
        printf("[mysql] statements before it are already applied — fix and re-run");
    }
    return 1;
}

// --- 3. Reading several result sets -------------------------------------------
//
// A CALL to a stored procedure that runs more than one SELECT returns one
// result set per SELECT. Same for a script that ends in several SELECTs.
//
//   CREATE PROCEDURE player_overview(IN pid INT)
//   BEGIN
//       SELECT name, score FROM accounts WHERE id = pid;
//       SELECT note, at FROM audit_log ORDER BY at DESC LIMIT 5;
//   END

LoadPlayerOverview(playerid, dbId)
{
    new stmt = mysql_stmt_new(g_MysqlConn, "CALL player_overview(?)");
    if (stmt == 0) return 0;

    mysql_stmt_bind_int(stmt, dbId);
    mysql_stmt_execute(stmt, "OnOverviewLoaded", "d", playerid);
    mysql_stmt_close(stmt);
    return 1;
}

forward OnOverviewLoaded(playerid);
public OnOverviewLoaded(playerid)
{
    new sets = cache_get_result_count();
    printf("[mysql] procedure returned %d result set(s)", sets);

    // Result 0 is selected by default, so a single-set query needs none of
    // this — existing code keeps working unchanged.

    // First set: the account row.
    if (cache_set_result(0) && cache_get_row_count() > 0)
    {
        new name[MAX_PLAYER_NAME + 1];
        cache_get_value_name(0, "name", name);
        printf("  account: %s (score %d)", name, cache_get_value_name_int(0, "score"));
    }

    // Second set: the recent audit rows.
    if (sets > 1 && cache_set_result(1))
    {
        new rows = cache_get_row_count();
        for (new i = 0; i < rows; i++)
        {
            new note[256];
            cache_get_value_name(i, "note", note);
            printf("  log: %s", note);
        }
    }

    return 1;
}

// --- Notes --------------------------------------------------------------------
//
// * cache_set_result returns false for an out-of-range index and leaves the
//   current selection alone, so the guards above are enough.
//
// * cache_save() copies EVERY result set together with the current selection.
//   Saving a cache and later switching sets does not lose data.
//
// * mysql_query_file reads a path your gamemode chose. Treat it like any other
//   file it opens: never build that path from player input.
