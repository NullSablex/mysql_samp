// 01_basic_connection.pwn — open a connection at OnGameModeInit, close it on exit.
//
// Two variants are shown:
//   1. The minimal call (default port 3306, no SSL, auto_reconnect on).
//   2. The same call routed through mysql_options_new() so you can tweak port,
//      connect timeout, auto-reconnect, etc.

#include <a_samp>
#include <mysql_samp>

#define MYSQL_HOST     "127.0.0.1"
#define MYSQL_USER     "samp"
#define MYSQL_PASSWORD "secret"
#define MYSQL_DATABASE "samp_server"

new g_MysqlConn = 0;

public OnGameModeInit()
{
    // --- Variant 1: no options ----------------------------------------------
    // g_MysqlConn = mysql_connect(MYSQL_HOST, MYSQL_USER, MYSQL_PASSWORD, MYSQL_DATABASE);

    // --- Variant 2: with options --------------------------------------------
    new opts = mysql_options_new();
    mysql_options_set_int(opts, MYSQL_OPT_PORT, 3306);
    mysql_options_set_int(opts, MYSQL_OPT_CONNECT_TIMEOUT, 5);   // seconds
    mysql_options_set_int(opts, MYSQL_OPT_AUTO_RECONNECT, 1);

    g_MysqlConn = mysql_connect(MYSQL_HOST, MYSQL_USER, MYSQL_PASSWORD, MYSQL_DATABASE, opts);

    if (g_MysqlConn == 0)
    {
        new err[256];
        mysql_error(0, err);  // 0 = global error slot (set when connect itself fails)
        printf("[mysql] connect failed: %s", err);
        return 1;
    }

    // Optional: ask the server how it's doing.
    new status[256];
    if (mysql_status(g_MysqlConn, status))
    {
        printf("[mysql] connected. %s", status);
    }

    return 1;
}

public OnGameModeExit()
{
    if (g_MysqlConn != 0)
    {
        mysql_close(g_MysqlConn);
        g_MysqlConn = 0;
    }
    return 1;
}
