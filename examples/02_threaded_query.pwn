// 02_threaded_query.pwn — non-blocking SELECT with a callback, FIFO-ordered.
//
// mysql_query() runs the query on a worker thread and dispatches the result
// to the named callback. Inside the callback the result is the *active cache*
// — read it with the cache_* natives. The cache is freed automatically when
// the callback returns (use cache_save() to keep it longer).
//
// Format spec for the variadic args: each letter maps to one extra param.
//   "d" int, "f" float, "s" string. Order must match the callback signature.

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

public OnPlayerConnect(playerid)
{
    new name[MAX_PLAYER_NAME];
    GetPlayerName(playerid, name, sizeof(name));

    new query[256];
    mysql_format(g_MysqlConn, query, sizeof(query),
                 "SELECT id, money, score FROM players WHERE name = '%e' LIMIT 1",
                 name);

    // Pass playerid through to the callback so we know which session to apply
    // the result to. FIFO ensures the result lands before any later query for
    // the same player.
    mysql_query(g_MysqlConn, query, "OnPlayerLoad", "d", playerid);
    return 1;
}

forward OnPlayerLoad(playerid);
public  OnPlayerLoad(playerid)
{
    if (cache_get_row_count() == 0)
    {
        printf("[mysql] new player, no row yet (id=%d)", playerid);
        return 1;
    }

    new playerDbId = cache_get_value_name_int(0, "id");
    new money      = cache_get_value_name_int(0, "money");
    new score      = cache_get_value_name_int(0, "score");

    GivePlayerMoney(playerid, money);
    SetPlayerScore(playerid, score);

    printf("[mysql] loaded player %d (db_id=%d, money=%d, score=%d)",
           playerid, playerDbId, money, score);
    return 1;
}

public OnGameModeExit()
{
    if (g_MysqlConn != 0) mysql_close(g_MysqlConn);
    return 1;
}
