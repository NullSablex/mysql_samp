// 08_prepared_statements.pwn — the safe way to put player input in a query.
//
// mysql_format escapes values into the SQL text. That works, but its
// correctness depends on matching the server's sql_mode: under
// NO_BACKSLASH_ESCAPES the backslash is not an escape character and the usual
// rules stop protecting you.
//
// A prepared statement removes the problem instead of defending against it.
// The statement and the values travel to the server separately, so there is no
// SQL text for a value to break out of and no escaping involved at all.
//
// Bonus property: because the values never enter the query text, they also
// never appear in logs/mysql.log or in cache_get_query_string() — only the
// template with its ? placeholders does.

#include <a_samp>
#include <mysql_samp>

#define MYSQL_HOST     "127.0.0.1"
#define MYSQL_USER     "samp"
#define MYSQL_PASSWORD "secret"
#define MYSQL_DATABASE "samp_server"

new g_MysqlConn = 0;

public OnGameModeInit()
{
    g_MysqlConn = mysql_connect(MYSQL_HOST, MYSQL_USER, MYSQL_PASSWORD, MYSQL_DATABASE);
    return 1;
}

public OnGameModeExit()
{
    if (g_MysqlConn != 0) mysql_close(g_MysqlConn);
    return 1;
}

// --- SELECT with two bound values -------------------------------------------

FindPlayersByName(playerid, const name[], minScore)
{
    new stmt = mysql_stmt_new(g_MysqlConn,
        "SELECT id, name, score FROM players WHERE name LIKE ? AND score > ?");

    if (stmt == 0) return 0;

    // Bind in the same order as the ? placeholders.
    mysql_stmt_bind_str(stmt, name);
    mysql_stmt_bind_int(stmt, minScore);

    // Non-blocking and FIFO-ordered, exactly like mysql_query.
    mysql_stmt_execute(stmt, "OnPlayersFound", "d", playerid);

    // The values were copied at execute time, so closing here is safe — the
    // query is already on its way.
    mysql_stmt_close(stmt);
    return 1;
}

forward OnPlayersFound(playerid);
public OnPlayersFound(playerid)
{
    new rows = cache_get_row_count();
    printf("[mysql] %d player(s) matched for playerid %d", rows, playerid);

    for (new i = 0; i < rows; i++)
    {
        new name[MAX_PLAYER_NAME + 1];
        cache_get_value_name(i, "name", name);
        printf("  #%d  %s  score=%d", cache_get_value_name_int(i, "id"), name,
            cache_get_value_name_int(i, "score"));
    }
    return 1;
}

// --- INSERT, reusing one statement for several rows --------------------------
//
// mysql_stmt_reset drops the bound values but keeps the statement, so a bulk
// insert does not have to rebuild it every time.

InsertScores(const names[][], const scores[], count)
{
    new stmt = mysql_stmt_new(g_MysqlConn,
        "INSERT INTO players (name, score) VALUES (?, ?)");

    if (stmt == 0) return 0;

    for (new i = 0; i < count; i++)
    {
        mysql_stmt_bind_str(stmt, names[i]);
        mysql_stmt_bind_int(stmt, scores[i]);
        mysql_stmt_execute(stmt);
        mysql_stmt_reset(stmt);   // clear the values, keep the statement
    }

    mysql_stmt_close(stmt);
    return 1;
}

// --- NULL --------------------------------------------------------------------

ClearPlayerClan(playerid, dbId)
{
    new stmt = mysql_stmt_new(g_MysqlConn, "UPDATE players SET clan_id = ? WHERE id = ?");
    if (stmt == 0) return 0;

    mysql_stmt_bind_null(stmt);      // SQL NULL, not the string "NULL" and not 0
    mysql_stmt_bind_int(stmt, dbId);
    mysql_stmt_execute(stmt);
    mysql_stmt_close(stmt);
    return 1;
}

// --- What placeholders cannot do ---------------------------------------------
//
// A ? stands for a VALUE. It cannot stand for a table name, a column name, or a
// keyword like ASC/DESC. This does NOT work:
//
//     mysql_stmt_new(conn, "SELECT * FROM ? ORDER BY ? ?");
//
// For those, build the SQL from values you control — a whitelist, never raw
// player input:
//
//     new const sortColumns[][] = { "score", "name", "created_at" };
//     new query[128];
//     if (0 <= sortIndex < sizeof(sortColumns))
//     {
//         format(query, sizeof(query),
//             "SELECT * FROM players ORDER BY `%s` LIMIT ?", sortColumns[sortIndex]);
//         new stmt = mysql_stmt_new(g_MysqlConn, query);
//         mysql_stmt_bind_int(stmt, limit);
//         mysql_stmt_execute(stmt, "OnPlayersFound", "d", playerid);
//         mysql_stmt_close(stmt);
//     }
//
// Note the placeholder count must match the number of bound values, or
// mysql_stmt_execute returns false and logs both numbers.
