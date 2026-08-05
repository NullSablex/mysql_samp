mod cache;
mod callback;
mod connection;
mod error;
mod logger;
mod natives;
mod options;
mod orm;
mod password;
mod plugin;
mod query;
mod stmt;
mod transaction;

use plugin::MysqlPlugin;
use samp::initialize_plugin;

initialize_plugin!(
    natives: [
        // Connection
        MysqlPlugin::mysql_connect,
        MysqlPlugin::mysql_close,
        MysqlPlugin::mysql_status,
        // Options
        MysqlPlugin::mysql_options_new,
        MysqlPlugin::mysql_options_set_int,
        MysqlPlugin::mysql_options_set_str,
        // Error
        MysqlPlugin::mysql_errno,
        MysqlPlugin::mysql_error,
        // Charset
        MysqlPlugin::mysql_set_charset,
        MysqlPlugin::mysql_get_charset,
        // Utility
        MysqlPlugin::mysql_unprocessed_queries,
        MysqlPlugin::mysql_log,
        // Password hashing
        MysqlPlugin::mysql_hash_password,
        MysqlPlugin::mysql_verify_password,
        // Query
        MysqlPlugin::mysql_query,
        MysqlPlugin::mysql_pquery,
        MysqlPlugin::mysql_tick,
        MysqlPlugin::mysql_escape_string,
        MysqlPlugin::mysql_format,
        // Prepared statements
        MysqlPlugin::mysql_stmt_new,
        MysqlPlugin::mysql_stmt_close,
        MysqlPlugin::mysql_stmt_reset,
        MysqlPlugin::mysql_stmt_bind_int,
        MysqlPlugin::mysql_stmt_bind_float,
        MysqlPlugin::mysql_stmt_bind_str,
        MysqlPlugin::mysql_stmt_bind_null,
        MysqlPlugin::mysql_stmt_execute,
        // Transactions
        MysqlPlugin::mysql_transaction_new,
        MysqlPlugin::mysql_transaction_destroy,
        MysqlPlugin::mysql_transaction_add,
        MysqlPlugin::mysql_transaction_add_stmt,
        MysqlPlugin::mysql_transaction_execute,
        // Cache
        MysqlPlugin::cache_get_row_count,
        MysqlPlugin::cache_get_field_count,
        MysqlPlugin::cache_get_field_name,
        MysqlPlugin::cache_get_value_index,
        MysqlPlugin::cache_get_value_index_int,
        MysqlPlugin::cache_get_value_index_float,
        MysqlPlugin::cache_get_value_name,
        MysqlPlugin::cache_get_value_name_int,
        MysqlPlugin::cache_get_value_name_float,
        MysqlPlugin::cache_is_value_index_null,
        MysqlPlugin::cache_is_value_name_null,
        MysqlPlugin::cache_affected_rows,
        MysqlPlugin::cache_insert_id,
        MysqlPlugin::cache_get_query_exec_time,
        MysqlPlugin::cache_get_query_string,
        MysqlPlugin::cache_save,
        MysqlPlugin::cache_delete,
        MysqlPlugin::cache_set_active,
        MysqlPlugin::cache_unset_active,
        MysqlPlugin::cache_is_any_active,
        MysqlPlugin::cache_is_valid,
        MysqlPlugin::cache_warning_count,
        MysqlPlugin::cache_get_field_type,
        // ORM
        MysqlPlugin::orm_create,
        MysqlPlugin::orm_destroy,
        MysqlPlugin::orm_errno,
        MysqlPlugin::orm_select,
        MysqlPlugin::orm_update,
        MysqlPlugin::orm_insert,
        MysqlPlugin::orm_delete,
        MysqlPlugin::orm_save,
        MysqlPlugin::orm_apply_cache,
        MysqlPlugin::orm_addvar_int,
        MysqlPlugin::orm_addvar_float,
        MysqlPlugin::orm_addvar_string,
        MysqlPlugin::orm_delvar,
        MysqlPlugin::orm_clear_vars,
        MysqlPlugin::orm_setkey,
    ],
    {
        samp::plugin::enable_tick();
        return MysqlPlugin::new();
    }
);
