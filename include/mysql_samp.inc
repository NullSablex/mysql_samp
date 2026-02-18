#if defined _mysql_samp_included
    #endinput
#endif
#define _mysql_samp_included

// Opcoes de conexao
enum {
    MYSQL_OPT_PORT = 0,
    MYSQL_OPT_SSL,
    MYSQL_OPT_SSL_CA,
    MYSQL_OPT_CONNECT_TIMEOUT,
}

// Codigos de erro
enum {
    MYSQL_OK = 0,
    MYSQL_ERROR_CONNECTION_FAILED,
    MYSQL_ERROR_INVALID_OPTIONS,
    MYSQL_ERROR_INVALID_CONNECTION,
    MYSQL_ERROR_PING_FAILED,
    MYSQL_ERROR_UNKNOWN,
}

// Options
native mysql_options_new();
native bool:mysql_options_set_int(handle, option, value);
native bool:mysql_options_set_str(handle, option, const value[]);

// Conexao
native mysql_connect(const host[], const user[], const password[], const database[], options = 0);
native bool:mysql_close(connId);
native bool:mysql_status(connId, dest[], max_len = sizeof(dest));

// Erro
native mysql_errno(connId = 0);
