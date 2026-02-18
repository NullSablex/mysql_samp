use std::collections::HashMap;
use std::time::Duration;

use mysql::{prelude::Queryable, Conn, Opts, OptsBuilder};

use crate::error::{ErrorState, MysqlError};
use crate::logger::Logger;
use crate::options::MysqlOptions;

struct ConnectionEntry {
    conn: Conn,
    last_error: ErrorState,
}

pub struct ConnectionManager {
    connections: HashMap<i32, ConnectionEntry>,
    next_id: i32,
    pub global_error: ErrorState,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            next_id: 1,
            global_error: ErrorState::ok(),
        }
    }

    pub fn connect(&mut self, host: &str, user: &str, password: &str, database: &str, options: &MysqlOptions) -> i32 {
        let builder = if host.starts_with('/') {
            OptsBuilder::new()
                .socket(Some(host))
                .user(Some(user))
                .pass(Some(password))
                .db_name(Some(database))
        } else {
            OptsBuilder::new()
                .ip_or_hostname(Some(host))
                .tcp_port(options.port)
                .user(Some(user))
                .pass(Some(password))
                .db_name(Some(database))
        };

        let builder = if let Some(timeout) = options.connect_timeout {
            builder.tcp_connect_timeout(Some(Duration::from_secs(timeout as u64)))
        } else {
            builder
        };

        // TODO: SSL configuration when mysql crate exposes rustls options

        let opts: Opts = builder.into();

        let mut conn = match Conn::new(opts) {
            Ok(c) => c,
            Err(e) => {
                let detail = format!("Connection failed: {}", e);
                let code = MysqlError::ConnectionFailed.code();
                Logger::error_detail(
                    &format!("Connection failed (error {}). See logs/mysql.log for details.", code),
                    &detail,
                );
                self.global_error = ErrorState::new(MysqlError::ConnectionFailed, detail);
                return 0;
            }
        };

        if let Err(e) = conn.ping() {
            let detail = format!("Ping failed: {}", e);
            let code = MysqlError::PingFailed.code();
            Logger::error_detail(
                &format!("Ping failed (error {}). See logs/mysql.log for details.", code),
                &detail,
            );
            self.global_error = ErrorState::new(MysqlError::PingFailed, detail);
            return 0;
        }

        self.global_error = ErrorState::ok();

        let id = self.next_id;
        self.next_id += 1;
        self.connections.insert(id, ConnectionEntry {
            conn,
            last_error: ErrorState::ok(),
        });

        id
    }

    pub fn disconnect(&mut self, id: i32) -> bool {
        self.connections.remove(&id).is_some()
    }

    pub fn get_status(&mut self, conn_id: i32) -> Option<String> {
        let entry = self.connections.get_mut(&conn_id)?;
        let keys = [
            "Uptime", "Threads_connected", "Questions",
            "Slow_queries", "Opens", "Flush_tables",
            "Open_tables", "Queries_per_second_avg",
        ];
        let rows: Vec<(String, String)> = entry.conn.query("SHOW GLOBAL STATUS").ok()?;
        let mut parts = Vec::new();
        for key in &keys {
            if let Some((_, v)) = rows.iter().find(|(k, _)| k == key) {
                parts.push(format!("{}: {}", key, v));
            }
        }
        Some(parts.join("  "))
    }

    pub fn get_error(&self, conn_id: i32) -> &ErrorState {
        if conn_id == 0 {
            return &self.global_error;
        }
        self.connections
            .get(&conn_id)
            .map(|e| &e.last_error)
            .unwrap_or(&self.global_error)
    }
}
