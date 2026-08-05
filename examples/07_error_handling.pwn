// 07_error_handling.pwn — OnQueryError, mysql_errno, mysql_error.
//
// Every threaded query that fails (syntax error, constraint violation, lost
// connection, ...) triggers OnQueryError BEFORE the regular callback.
//
//   errorid     — native MySQL error code (e.g. 1062 = duplicate key) or
//                 one of the MYSQL_ERROR_* codes (1..=8) for plugin-internal
//                 failures.
//   error[]     — human-readable message.
//   callback[]  — the callback that would have been invoked on success.
//   query[]     — the SQL text (may be truncated for very long queries).
//   connId      — the connection that produced the error.

#include <a_samp>
#include <mysql_samp>

#define MYSQL_HOST     "127.0.0.1"
#define MYSQL_USER     "samp"
#define MYSQL_PASSWORD "secret"
#define MYSQL_DATABASE "samp_server"

new g_MysqlConn = 0;

public OnGameModeInit()
{
    // Crank logging up while debugging.
    mysql_log(MYSQL_LOG_ALL);

    g_MysqlConn = mysql_connect(MYSQL_HOST, MYSQL_USER, MYSQL_PASSWORD, MYSQL_DATABASE);

    if (g_MysqlConn == 0)
    {
        new err[256];
        mysql_error(0, err);
        printf("[mysql] connect failed (code=%d): %s", mysql_errno(0), err);
        return 1;
    }

    // Trigger a deliberate failure to exercise OnQueryError.
    mysql_query(g_MysqlConn, "SELECT * FROM table_that_does_not_exist", "OnNeverCalled");
    return 1;
}

forward OnNeverCalled();
public  OnNeverCalled()
{
    // Won't run — OnQueryError fires first and the regular callback is skipped.
    return 1;
}

public OnQueryError(errorid, const error[], const callback[], const query[], connId)
{
    printf("[mysql] error %d on connId %d: %s", errorid, connId, error);
    printf("[mysql]   callback: %s", callback);
    // Careful: this puts the whole statement in server_log.txt, including any
    // value interpolated by mysql_format. Drop this line when the query may
    // carry sensitive data — or use prepared statements, where the values are
    // never part of the query text (see 08_prepared_statements.pwn).
    printf("[mysql]   query:    %s", query);

    // errorid is the native MySQL code (1146 = base table or view not found,
    // 1062 = duplicate key, 1045 = access denied, ...) — handle the ones you
    // care about explicitly.
    switch (errorid)
    {
        case 1062: printf("[mysql] duplicate key — ignoring");
        case 1146: printf("[mysql] missing table — schema migration pending?");
        default:   printf("[mysql] unhandled MySQL error %d", errorid);
    }
    return 1;
}

public OnGameModeExit()
{
    if (g_MysqlConn != 0) mysql_close(g_MysqlConn);
    return 1;
}
