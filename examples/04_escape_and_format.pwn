// 04_escape_and_format.pwn — building queries safely.
//
// mysql_format specifiers:
//   %d  int
//   %f  float
//   %s  string, auto-escaped (preferred for user input)
//   %e  alias of %s, explicit "escape"
//   %r  string, raw (NO escape) — use only with values YOU control
//   %%  literal percent sign
//
// If you absolutely need to escape a string outside mysql_format, use
// mysql_escape_string. It is a pure function: no connection required.

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

// Example 1: a SELECT with an escaped user-supplied name.
SearchPlayerByName(const input[])
{
    new query[256];
    mysql_format(g_MysqlConn, query, sizeof(query),
                 "SELECT id, score FROM players WHERE name = '%e' LIMIT 1",
                 input);
    mysql_query(g_MysqlConn, query, "OnSearchResult");
}

forward OnSearchResult();
public  OnSearchResult()
{
    if (cache_get_row_count() == 0) return printf("[mysql] not found");
    new id    = cache_get_value_name_int(0, "id");
    new score = cache_get_value_name_int(0, "score");
    printf("[mysql] id=%d score=%d", id, score);
    return 1;
}

// Example 2: an UPDATE mixing ints, floats and strings.
PersistPlayer(playerid, const note[])
{
    new Float:x, Float:y, Float:z;
    GetPlayerPos(playerid, x, y, z);

    new query[512];
    mysql_format(g_MysqlConn, query, sizeof(query),
                 "UPDATE players SET pos_x=%f, pos_y=%f, pos_z=%f, score=%d, note='%e' WHERE id=%d",
                 x, y, z, GetPlayerScore(playerid), note, playerid);
    mysql_pquery(g_MysqlConn, query);
}

// Example 3: %r — raw, no escape. Only for trusted, hard-coded values.
TruncateActionLog()
{
    new query[128];
    mysql_format(g_MysqlConn, query, sizeof(query),
                 "DELETE FROM %r WHERE ts < NOW() - INTERVAL %d DAY",
                 "action_log", 30);
    mysql_pquery(g_MysqlConn, query);
}

// Example 4: escape outside mysql_format (rare, but available).
ManualEscape()
{
    new src[] = "O'Brien";
    new dest[64];
    mysql_escape_string(src, dest);
    printf("[mysql] escaped: %s", dest);  // O\'Brien
}

public OnGameModeExit()
{
    if (g_MysqlConn != 0) mysql_close(g_MysqlConn);
    return 1;
}
