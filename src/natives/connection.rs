use samp::native;
use samp::prelude::*;

use crate::error::{ErrorState, MysqlError};
use crate::logger::Logger;
use crate::options::MysqlOptions;
use crate::plugin::MysqlPlugin;

impl MysqlPlugin {
    #[native(name = "mysql_connect")]
    pub fn mysql_connect(
        &mut self,
        _amx: &Amx,
        host: AmxString,
        user: AmxString,
        password: AmxString,
        database: AmxString,
        options_id: i32,
    ) -> AmxResult<i32> {
        let opts = if options_id == 0 {
            MysqlOptions::default()
        } else {
            match self.options.get(options_id) {
                Some(o) => o.clone(),
                None => {
                    Logger::error("Connection failed: invalid options handle.");
                    self.connections.global_error = ErrorState::new(
                        MysqlError::InvalidOptions,
                        "Invalid options handle.",
                    );
                    return Ok(0);
                }
            }
        };

        let id = self.connections.connect(
            &host.to_string(),
            &user.to_string(),
            &password.to_string(),
            &database.to_string(),
            &opts,
        );

        if id > 0 {
            Logger::info(&format!("Connection {} established.", id));
        } else {
            Logger::info("Connection failed.");
        }

        Ok(id)
    }

    #[native(name = "mysql_status")]
    pub fn mysql_status(
        &mut self,
        _amx: &Amx,
        conn_id: i32,
        dest: UnsizedBuffer,
        dest_len: usize,
    ) -> AmxResult<bool> {
        match self.connections.get_status(conn_id) {
            Some(status) => {
                let mut buf = dest.into_sized_buffer(dest_len);
                let _ = samp::cell::string::put_in_buffer(&mut buf, &status);
                Ok(true)
            }
            None => {
                Logger::warn("Failed to retrieve server status.");
                self.connections.set_error(
                    conn_id,
                    ErrorState::new(MysqlError::PingFailed, "Failed to retrieve server status."),
                );
                Ok(false)
            }
        }
    }

    #[native(name = "mysql_close")]
    pub fn mysql_close(&mut self, _amx: &Amx, connection_id: i32) -> AmxResult<bool> {
        if self.connections.disconnect(connection_id) {
            Logger::info(&format!("Connection {} closed.", connection_id));
            Ok(true)
        } else {
            Logger::warn("Connection not found.");
            Ok(false)
        }
    }

    #[native(name = "mysql_set_charset")]
    pub fn mysql_set_charset(
        &mut self,
        _amx: &Amx,
        conn_id: i32,
        charset: AmxString,
    ) -> AmxResult<bool> {
        Ok(self.connections.set_charset(conn_id, &charset.to_string()))
    }

    #[native(name = "mysql_get_charset")]
    pub fn mysql_get_charset(
        &mut self,
        _amx: &Amx,
        conn_id: i32,
        dest: UnsizedBuffer,
        dest_len: usize,
    ) -> AmxResult<bool> {
        match self.connections.get_charset(conn_id) {
            Some(charset) => {
                let mut buf = dest.into_sized_buffer(dest_len);
                let _ = samp::cell::string::put_in_buffer(&mut buf, &charset);
                Ok(true)
            }
            None => Ok(false),
        }
    }

    #[native(name = "mysql_unprocessed_queries")]
    pub fn mysql_unprocessed_queries(&mut self, _amx: &Amx) -> AmxResult<i32> {
        Ok(self.queries.pending_count() as i32)
    }

    #[native(name = "mysql_log")]
    pub fn mysql_log(&mut self, _amx: &Amx, log_level: i32) -> AmxResult<bool> {
        Logger::set_log_level(log_level);
        Ok(true)
    }
}
