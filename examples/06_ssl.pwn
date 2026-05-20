// 06_ssl.pwn — TLS connection. Requires mysql_samp v1.1.1 or later.
//
// Up to v1.1.0 the SSL options were silently ignored and every connection was
// plaintext. From v1.1.1 onwards:
//   - MYSQL_OPT_SSL = 1 turns on TLS via rustls (SslOpts::default()).
//   - MYSQL_OPT_SSL_CA is an optional path to a root certificate (.pem or .der).
//     Without it, the OS trust store is used.

#include <a_samp>
#include <mysql_samp>

#define MYSQL_HOST     "db.example.com"
#define MYSQL_USER     "samp"
#define MYSQL_PASSWORD "secret"
#define MYSQL_DATABASE "samp_server"

new g_MysqlConn = 0;

public OnGameModeInit()
{
    new opts = mysql_options_new();
    mysql_options_set_int(opts, MYSQL_OPT_PORT, 3306);
    mysql_options_set_int(opts, MYSQL_OPT_CONNECT_TIMEOUT, 5);

    // --- Enable TLS ---------------------------------------------------------
    mysql_options_set_int(opts, MYSQL_OPT_SSL, 1);

    // --- Optional: pin a CA certificate -------------------------------------
    // Path is resolved relative to the server's working directory.
    // Comment this line out to fall back to the platform's trust store.
    mysql_options_set_str(opts, MYSQL_OPT_SSL_CA, "certs/ca.pem");

    g_MysqlConn = mysql_connect(MYSQL_HOST, MYSQL_USER, MYSQL_PASSWORD, MYSQL_DATABASE, opts);

    if (g_MysqlConn == 0)
    {
        new err[256];
        mysql_error(0, err);
        printf("[mysql] TLS connect failed: %s", err);
        return 1;
    }

    printf("[mysql] TLS connection established to %s", MYSQL_HOST);
    return 1;
}

public OnGameModeExit()
{
    if (g_MysqlConn != 0) mysql_close(g_MysqlConn);
    return 1;
}
