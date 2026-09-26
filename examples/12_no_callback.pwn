// 12_no_callback.pwn - non-blocking queries WITHOUT a callback (fire-and-forget).
//
// Non-blocking does not mean "write a callback for everything". Every native
// that takes a callback takes it with a default of "", and passing nothing
// runs the query and discards the result. That is one line, no forward, no
// public - and the server still never blocks.
//
// Use it for writes: UPDATE, INSERT, DELETE. You need a callback only to read
// a result back, because a value that arrives later has nowhere else to go.
// The two password natives are the exception: their callback is required.
//
// A failure is NOT silenced - it still reaches OnQueryError.
//
// Careful with ordering: queries submitted one after the other run
// CONCURRENTLY, each on its own connection. See 13 below for what to use when
// one statement depends on another.

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

public OnPlayerDisconnect(playerid, reason)
{
    new name[MAX_PLAYER_NAME];
    GetPlayerName(playerid, name, sizeof(name));

    new query[256];
    mysql_format(g_MysqlConn, query, sizeof(query),
                 "UPDATE players SET last_seen = NOW() WHERE name = '%e'", name);

    // No callback: the write is dispatched and the result discarded.
    mysql_query(g_MysqlConn, query);
    return 1;
}

public OnPlayerDeath(playerid, killerid, reason)
{
    new query[128];
    mysql_format(g_MysqlConn, query, sizeof(query),
                 "UPDATE players SET deaths = deaths + 1 WHERE id = %d", playerid);

    // Independent writes have no reason to be ordered: mysql_pquery skips the
    // FIFO buffering entirely. Also callback-free.
    mysql_pquery(g_MysqlConn, query);
    return 1;
}

// Errors still arrive, even with no callback on the query that failed.
public OnQueryError(errorid, const error[], const callback[], const query[], connId)
{
    printf("[MySQL] error %d on '%s': %s", errorid, query, error);
    return 1;
}
