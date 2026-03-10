/*
 * mysql_samp - Plugin MySQL para SA:MP escrito em Rust
 * Autor: NullSablex
 * Repositorio: https://github.com/NullSablex/mysql_samp
 * Licenca: GPL-3.0
 */

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
    MYSQL_OPT_AUTO_RECONNECT,
}

// Codigos de erro
enum {
    MYSQL_OK = 0,
    MYSQL_ERROR_CONNECTION_FAILED,
    MYSQL_ERROR_INVALID_OPTIONS,
    MYSQL_ERROR_INVALID_CONNECTION,
    MYSQL_ERROR_PING_FAILED,
    MYSQL_ERROR_QUERY_FAILED,
    MYSQL_ERROR_NO_CACHE_ACTIVE,
    MYSQL_ERROR_INVALID_ORM,
    MYSQL_ERROR_ORM_KEY_NOT_SET,
}

// Niveis de log
enum {
    MYSQL_LOG_NONE = 0,
    MYSQL_LOG_ERROR,
    MYSQL_LOG_WARNING,
    MYSQL_LOG_INFO,
    MYSQL_LOG_ALL,
}

// ORM error codes
enum {
    ORM_OK = 0,
    ORM_NO_DATA,
}

// Forward: chamado quando uma query threaded falha
forward OnQueryError(errorid, const error[], const callback[], const query[], connId);

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
native bool:mysql_error(connId, dest[], max_len = sizeof(dest));

// Charset
native bool:mysql_set_charset(connId, const charset[]);
native bool:mysql_get_charset(connId, dest[], max_len = sizeof(dest));

// Utility
native mysql_unprocessed_queries();
native bool:mysql_log(log_level);

// Query (non-blocking)
native bool:mysql_query(connId, const query[], const callback[] = "", const format[] = "", {Float,_}:...);
native bool:mysql_pquery(connId, const query[], const callback[] = "", const format[] = "", {Float,_}:...);
native bool:mysql_escape_string(const src[], dest[], max_len = sizeof(dest));
native mysql_format(connId, dest[], max_len, const format[], {Float,_}:...);

// Cache
native cache_get_row_count();
native cache_get_field_count();
native bool:cache_get_field_name(field_idx, dest[], max_len = sizeof(dest));
native cache_get_field_type(field_idx);
native bool:cache_get_value_index(row, col, dest[], max_len = sizeof(dest));
native cache_get_value_index_int(row, col);
native Float:cache_get_value_index_float(row, col);
native bool:cache_get_value_name(row, const field_name[], dest[], max_len = sizeof(dest));
native cache_get_value_name_int(row, const field_name[]);
native Float:cache_get_value_name_float(row, const field_name[]);
native bool:cache_is_value_index_null(row, col);
native bool:cache_is_value_name_null(row, const field_name[]);
native cache_affected_rows();
native cache_insert_id();
native cache_warning_count();
native cache_get_query_exec_time();
native bool:cache_get_query_string(dest[], max_len = sizeof(dest));
native cache_save();
native bool:cache_delete(cache_id);
native bool:cache_set_active(cache_id);
native bool:cache_unset_active();
native bool:cache_is_any_active();
native bool:cache_is_valid(cache_id);

// ORM
native orm_create(const table[], connId);
native bool:orm_destroy(orm_id);
native orm_errno(orm_id);
native bool:orm_select(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:orm_update(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:orm_insert(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:orm_delete(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:orm_save(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:orm_apply_cache(orm_id, row = 0);
native bool:orm_addvar_int(orm_id, &var, const column_name[]);
native bool:orm_addvar_float(orm_id, &Float:var, const column_name[]);
native bool:orm_addvar_string(orm_id, var[], var_max_len, const column_name[]);
native bool:orm_delvar(orm_id, const column_name[]);
native bool:orm_clear_vars(orm_id);
native bool:orm_setkey(orm_id, const column_name[]);
