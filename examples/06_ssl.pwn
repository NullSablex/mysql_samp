// 06_ssl.pwn — TLS connection. Requires mysql_samp v1.2.0 or later.
//
// IMPORTANT: before v1.2.0 these options did NOT encrypt anything. The plugin
// shipped without a TLS backend, so enabling SSL aborted the connection rather
// than securing it. If you ran an older build with MYSQL_OPT_SSL on and it
// appeared to work, that traffic was plaintext — rotate those credentials.
//
//   - MYSQL_OPT_SSL = 1 turns on TLS (rustls, compiled into the plugin).
//   - MYSQL_OPT_SSL_CA points at the root certificate that signed the server.
//     WITHOUT it, only the webpki root bundle compiled into the plugin is
//     trusted — NOT your operating system's trust store. A self-signed or
//     internal-CA server will fail to connect until you set this.
//   - MYSQL_OPT_SSL_CERT + MYSQL_OPT_SSL_KEY provide a client certificate when
//     the server requires mutual TLS.
//   - MYSQL_OPT_SSL_VERIFY_CERT = 0 disables verification. See the note below.

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

    // --- Trust anchor -------------------------------------------------------
    // Path is resolved relative to the server's working directory.
    // Required for a self-signed certificate or an internal CA. Omit it ONLY if
    // the server's certificate chains to a public CA (RDS, Cloud SQL, ...).
    mysql_options_set_str(opts, MYSQL_OPT_SSL_CA, "certs/ca.pem");

    // --- Optional: mutual TLS -----------------------------------------------
    // Both are required together; setting only one logs a warning and the
    // client certificate is ignored.
    // mysql_options_set_str(opts, MYSQL_OPT_SSL_CERT, "certs/client-cert.pem");
    // mysql_options_set_str(opts, MYSQL_OPT_SSL_KEY,  "certs/client-key.pem");

    // --- DANGEROUS: disable verification ------------------------------------
    // Accepts ANY certificate and skips the hostname check. The traffic stays
    // encrypted, but anyone able to intercept the connection can present their
    // own certificate and read or rewrite every query — which is exactly what
    // TLS exists to prevent. Set MYSQL_OPT_SSL_CA above instead.
    // mysql_options_set_int(opts, MYSQL_OPT_SSL_VERIFY_CERT, 0);

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
