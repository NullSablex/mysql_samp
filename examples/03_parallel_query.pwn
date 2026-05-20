// 03_parallel_query.pwn — mysql_pquery: parallel, no ordering guarantee.
//
// Use mysql_pquery when the result does NOT depend on a previously queued
// query for the same row. Typical fit: independent UPDATEs, write-only logging,
// or read-only lookups whose order doesn't matter.
//
// If you fire mysql_pquery() for the same key (e.g. same playerid) multiple
// times in a row and need ordering, switch to mysql_query() instead.

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

// Fire-and-forget log entry. We don't care when it lands, only that it lands.
LogPlayerAction(playerid, const action[])
{
    new name[MAX_PLAYER_NAME];
    GetPlayerName(playerid, name, sizeof(name));

    new query[256];
    mysql_format(g_MysqlConn, query, sizeof(query),
                 "INSERT INTO action_log (player, action, ts) VALUES ('%e', '%e', NOW())",
                 name, action);

    // Empty callback string = run and discard the result.
    mysql_pquery(g_MysqlConn, query);
}

public OnPlayerDeath(playerid, killerid, reason)
{
    LogPlayerAction(playerid, "death");
    return 1;
}

public OnPlayerSpawn(playerid)
{
    LogPlayerAction(playerid, "spawn");
    return 1;
}

public OnGameModeExit()
{
    if (g_MysqlConn != 0) mysql_close(g_MysqlConn);
    return 1;
}
