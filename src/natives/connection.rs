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
        let opts = match self.resolve_options(options_id) {
            Some(opts) => opts,
            None => return 0,
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

    /// mysql_connect_file(const path[] = "mysql.ini", options = 0)
    ///
    /// Reads host / user / password / database from a `key = value` file and
    /// connects. Keeps credentials out of the gamemode source, which is
    /// usually in version control while the config file is not.
    ///
    /// Options stay with `mysql_options_new` — the file carries credentials
    /// only, so there is one place to look for connection tuning.
    #[native(name = "mysql_connect_file")]
    pub fn mysql_connect_file(&mut self, _amx: &Amx, path: &AmxString, options_id: i32) -> i32 {
        let path = path.to_string();

        let cfg = match crate::config::load(std::path::Path::new(&path)) {
            Ok(cfg) => cfg,
            Err(err) => {
                // The detail names the file and the offending key, never a
                // value — the file holds a password.
                let detail = format!("Connection file '{path}' rejected: {}", err.message());
                Logger::error_detail(
                    &format!(
                        "Connection failed (error {}). See logs/mysql.log for details.",
                        MysqlError::InvalidOptions.code()
                    ),
                    &detail,
                );
                self.connections.global_error = ErrorState::new(MysqlError::InvalidOptions, detail);
                return 0;
            }
        };

        let opts = match self.resolve_options(options_id) {
            Some(opts) => opts,
            None => return 0,
        };

        let id =
            self.connections
                .connect(&cfg.host, &cfg.user, &cfg.password, &cfg.database, &opts);

        if id > 0 {
            Logger::info(&format!("Connection {} established from '{}'.", id, path));
        } else {
            Logger::info("Connection failed.");
        }

        id
    }

    /// Resolves an options handle, reporting and returning `None` when it is
    /// unknown. `0` means "no options" and yields the defaults.
    fn resolve_options(&mut self, options_id: i32) -> Option<MysqlOptions> {
        if options_id == 0 {
            return Some(MysqlOptions::default());
        }
        match self.options.get(options_id) {
            Some(o) => Some(o.clone()),
            None => {
                Logger::error("Connection failed: invalid options handle.");
                self.connections.global_error =
                    ErrorState::new(MysqlError::InvalidOptions, "Invalid options handle.");
                None
            }
        }
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

    /// Whether the session is encrypted, as the handshake settled it.
    ///
    /// This is not "was TLS asked for": `MYSQL_OPT_SSL` is the request, and a
    /// request can be refused. Reading what was negotiated is what lets a
    /// gamemode verify its own deployment instead of trusting the option it
    /// passed.
    #[native(name = "mysql_tls_active")]
    pub fn mysql_tls_active(&mut self, _amx: &Amx, conn_id: i32) -> bool {
        self.connections.tls_active(conn_id)
    }

    /// Writes the negotiated cipher suite, empty when the session is not
    /// encrypted.
    #[native(name = "mysql_tls_cipher")]
    pub fn mysql_tls_cipher(
        &mut self,
        _amx: &Amx,
        conn_id: i32,
        dest: UnsizedBuffer,
        dest_len: usize,
    ) -> AmxResult<bool> {
        // An unencrypted session is not a failure to report through the error
        // state: it is the answer. The empty string says it, and the return
        // value distinguishes it from a connection that does not exist.
        let cipher = self
            .connections
            .tls_cipher(conn_id)
            .unwrap_or("")
            .to_owned();
        dest.write_str(dest_len, &cipher)?;
        Ok(!cipher.is_empty())
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
