mod connection;
mod error;
mod logger;
mod natives;
mod options;
mod plugin;

use plugin::MysqlPlugin;
use samp::initialize_plugin;

initialize_plugin!(
    natives: [
        MysqlPlugin::mysql_connect,
        MysqlPlugin::mysql_close,
        MysqlPlugin::mysql_status,
        MysqlPlugin::mysql_options_new,
        MysqlPlugin::mysql_options_set_int,
        MysqlPlugin::mysql_options_set_str,
        MysqlPlugin::mysql_errno,
    ],
    {
        return MysqlPlugin::new();
    }
);
