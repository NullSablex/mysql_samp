mod cache;
mod callback;
mod connection;
mod error;
mod logger;
mod natives;
mod options;
mod orm;
mod plugin;
mod query;

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
        // Query
        MysqlPlugin::mysql_query,
        MysqlPlugin::mysql_pquery,
        MysqlPlugin::mysql_escape_string,
        MysqlPlugin::mysql_format,
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
        samp::plugin::enable_process_tick();
        return MysqlPlugin::new();
    }
);
