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
        host: &AmxString,
        user: &AmxString,
        password: &AmxString,
        database: &AmxString,
        options_id: i32,
    ) -> i32 {
        let opts = if options_id == 0 {
            MysqlOptions::default()
        } else {
            match self.options.get(options_id) {
                Some(o) => o.clone(),
                None => {
                    Logger::error("Connection failed: invalid options handle.");
                    self.connections.global_error =
                        ErrorState::new(MysqlError::InvalidOptions, "Invalid options handle.");
                    return 0;
                }
            }
        };

        let id = self
            .connections
            .connect(host, user, password, database, &opts);

        if id > 0 {
            Logger::info(&format!("Connection {} established.", id));
        } else {
            Logger::info("Connection failed.");
        }

        id
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
                dest.write_str(dest_len, &status)?;
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
    pub fn mysql_close(&mut self, _amx: &Amx, connection_id: i32) -> bool {
        if self.connections.disconnect(connection_id) {
            // Statements and transactions bound to this connection can never
            // execute again; dropping them keeps the maps from growing.
            self.stmts.destroy_by_conn(connection_id);
            self.transactions.destroy_by_conn(connection_id);
            Logger::info(&format!("Connection {} closed.", connection_id));
            true
        } else {
            Logger::warn("Connection not found.");
            false
        }
    }

    #[native(name = "mysql_set_charset")]
    pub fn mysql_set_charset(&mut self, _amx: &Amx, conn_id: i32, charset: &AmxString) -> bool {
        self.connections.set_charset(conn_id, charset)
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
                dest.write_str(dest_len, &charset)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    #[native(name = "mysql_unprocessed_queries")]
    pub fn mysql_unprocessed_queries(&mut self, _amx: &Amx) -> i32 {
        i32::try_from(self.queries.pending_count()).unwrap_or(i32::MAX)
    }

    #[native(name = "mysql_log")]
    pub fn mysql_log(&mut self, _amx: &Amx, log_level: i32) -> bool {
        Logger::set_log_level(log_level);
        true
    }
}
