use std::collections::HashMap;
use std::time::Duration;

use mysql::prelude::Queryable;
use mysql::{Opts, OptsBuilder, Pool, PooledConn};

use crate::cache::{CacheEntry, CacheRow};
use crate::error::{ErrorState, MysqlError};
use crate::logger::Logger;
use crate::options::MysqlOptions;

struct ConnectionEntry {
    pool: Pool,
    last_error: ErrorState,
    auto_reconnect: bool,
}

pub struct QueryError {
    pub code: u16,
    pub message: String,
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

    pub fn connect(
        &mut self,
        host: &str,
        user: &str,
        password: &str,
        database: &str,
        options: &MysqlOptions,
    ) -> i32 {
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

        // Force UTF-8 encoding on every connection for safe string escaping
        let builder = builder.init(vec!["SET NAMES utf8mb4"]);

        let opts: Opts = builder.into();

        let pool = match Pool::new(opts) {
            Ok(p) => p,
            Err(e) => {
                let detail = format!("Pool creation failed: {}", e);
                let code = MysqlError::ConnectionFailed.code();
                Logger::error_detail(
                    &format!(
                        "Connection failed (error {}). See logs/mysql.log for details.",
                        code
                    ),
                    &detail,
                );
                self.global_error = ErrorState::new(MysqlError::ConnectionFailed, detail);
                return 0;
            }
        };

        // Validate by getting a connection (Pool connects lazily on first get_conn)
        match pool.get_conn() {
            Ok(_) => {}
            Err(e) => {
                let detail = format!("Connection failed: {}", e);
                let code = MysqlError::ConnectionFailed.code();
                Logger::error_detail(
                    &format!(
                        "Connection failed (error {}). See logs/mysql.log for details.",
                        code
                    ),
                    &detail,
                );
                self.global_error = ErrorState::new(MysqlError::ConnectionFailed, detail);
                return 0;
            }
        }

        self.global_error = ErrorState::ok();

        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        self.connections.insert(
            id,
            ConnectionEntry {
                pool,
                last_error: ErrorState::ok(),
                auto_reconnect: options.auto_reconnect,
            },
        );

        id
    }

    pub fn disconnect(&mut self, id: i32) -> bool {
        self.connections.remove(&id).is_some()
    }

    pub fn get_status(&mut self, conn_id: i32) -> Option<String> {
        let entry = self.connections.get(&conn_id)?;
        let mut conn = entry.pool.get_conn().ok()?;
        let keys = [
            "Uptime",
            "Threads_connected",
            "Questions",
            "Slow_queries",
            "Opens",
            "Flush_tables",
            "Open_tables",
            "Queries_per_second_avg",
        ];
        let rows: Vec<(String, String)> = conn.query("SHOW GLOBAL STATUS").ok()?;
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

    /// Returns a clone of the Pool for a given connection (for use in threads).
    pub fn get_pool(&self, conn_id: i32) -> Option<Pool> {
        self.connections.get(&conn_id).map(|e| e.pool.clone())
    }

    /// Returns the auto_reconnect setting for a connection (defaults to true if not found).
    pub fn get_auto_reconnect(&self, conn_id: i32) -> bool {
        self.connections
            .get(&conn_id)
            .map(|e| e.auto_reconnect)
            .unwrap_or(true)
    }

    /// Sets the last error for a connection.
    pub fn set_error(&mut self, conn_id: i32, error: ErrorState) {
        if let Some(entry) = self.connections.get_mut(&conn_id) {
            entry.last_error = error;
        }
    }

    /// Checks if a connection ID exists.
    pub fn exists(&self, conn_id: i32) -> bool {
        self.connections.contains_key(&conn_id)
    }

    /// Sets the character set for a connection by executing `SET NAMES`.
    pub fn set_charset(&mut self, conn_id: i32, charset: &str) -> bool {
        let entry = match self.connections.get(&conn_id) {
            Some(e) => e,
            None => return false,
        };

        match entry.pool.get_conn() {
            Ok(mut conn) => {
                let query = format!("SET NAMES '{}'", escape_string(charset));
                conn.query_drop(&query).is_ok()
            }
            Err(_) => false,
        }
    }

    /// Gets the current character set for a connection.
    pub fn get_charset(&self, conn_id: i32) -> Option<String> {
        let entry = self.connections.get(&conn_id)?;
        let mut conn = entry.pool.get_conn().ok()?;
        let result: Option<String> = conn
            .query_first("SELECT @@character_set_connection")
            .ok()?;
        result
    }
}

/// Escapes a SQL identifier (table/column name) by removing backticks.
/// Used for safe backtick-quoting: `escape_identifier(name)` -> `` `safe_name` ``
pub fn escape_identifier(input: &str) -> String {
    input.replace('`', "")
}

/// Escapes a string for safe use in SQL queries.
/// Pure function — no connection required.
pub fn escape_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 2);
    for ch in input.chars() {
        match ch {
            '\0' => out.push_str("\\0"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '"' => out.push_str("\\\""),
            '\x1a' => out.push_str("\\Z"),
            _ => out.push(ch),
        }
    }
    out
}

/// Executes a query on a Pool, retrying once on connection-lost errors when auto_reconnect is true.
/// Connection-lost errors are identified by error code 0 (non-MySQL errors such as IO errors,
/// which the Rust mysql crate returns when the TCP connection is dropped by the server).
pub fn attempt_query(pool: &Pool, query: &str, auto_reconnect: bool) -> Result<CacheEntry, QueryError> {
    let mut conn = pool.get_conn().map_err(|e| QueryError {
        code: 0,
        message: e.to_string(),
    })?;

    match execute_query(&mut conn, query) {
        Err(ref e) if auto_reconnect && e.code == 0 => {
            drop(conn);
            let mut conn2 = pool.get_conn().map_err(|e2| QueryError {
                code: 0,
                message: e2.to_string(),
            })?;
            execute_query(&mut conn2, query)
        }
        other => other,
    }
}

/// Maximum number of rows stored in a single CacheEntry to prevent memory exhaustion.
const MAX_RESULT_ROWS: usize = 100_000;

/// Executes a query on a PooledConn and returns a CacheEntry with results.
pub fn execute_query(conn: &mut PooledConn, query: &str) -> Result<CacheEntry, QueryError> {
    let start = std::time::Instant::now();

    let result = match conn.query_iter(query) {
        Ok(r) => r,
        Err(e) => {
            return Err(QueryError {
                code: extract_mysql_errno(&e),
                message: e.to_string(),
            });
        }
    };

    let cols_ref = result.columns();
    let columns: Vec<String> = cols_ref.as_ref().iter()
        .map(|c| c.name_str().to_string())
        .collect();
    let field_types: Vec<u8> = cols_ref.as_ref().iter()
        .map(|c| c.column_type() as u8)
        .collect();

    let mut rows: Vec<CacheRow> = Vec::new();
    let mut truncated = false;
    for row_result in result {
        match row_result {
            Ok(row) => {
                if rows.len() >= MAX_RESULT_ROWS {
                    truncated = true;
                    continue; // drain remaining to avoid protocol desync
                }
                let mut cells = Vec::with_capacity(columns.len());
                for i in 0..columns.len() {
                    let val: Option<String> = row.get(i);
                    cells.push(val);
                }
                rows.push(cells);
            }
            Err(e) => {
                return Err(QueryError {
                    code: extract_mysql_errno(&e),
                    message: e.to_string(),
                });
            }
        }
    }

    if truncated {
        crate::logger::Logger::warn(&format!(
            "Query result truncated to {} rows.",
            MAX_RESULT_ROWS
        ));
    }

    let exec_time = start.elapsed().as_micros();

    let affected_rows = conn.affected_rows();
    let insert_id = conn.last_insert_id();
    let warning_count = conn.warnings();

    Ok(CacheEntry::new(
        rows,
        columns,
        field_types,
        affected_rows,
        insert_id,
        warning_count,
        exec_time,
        query.to_string(),
    ))
}

/// Extracts the MySQL error number from a mysql::Error.
fn extract_mysql_errno(err: &mysql::Error) -> u16 {
    match err {
        mysql::Error::MySqlError(e) => e.code,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // escape_string tests

    #[test]
    fn escape_empty_string() {
        assert_eq!(escape_string(""), "");
    }

    #[test]
    fn escape_no_special_chars() {
        assert_eq!(escape_string("hello world"), "hello world");
    }

    #[test]
    fn escape_single_quote() {
        assert_eq!(escape_string("it's"), "it\\'s");
    }

    #[test]
    fn escape_double_quote() {
        assert_eq!(escape_string(r#"say "hi""#), r#"say \"hi\""#);
    }

    #[test]
    fn escape_backslash() {
        assert_eq!(escape_string(r"path\to"), r"path\\to");
    }

    #[test]
    fn escape_null_byte() {
        assert_eq!(escape_string("a\0b"), "a\\0b");
    }

    #[test]
    fn escape_newline() {
        assert_eq!(escape_string("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn escape_carriage_return() {
        assert_eq!(escape_string("line1\rline2"), "line1\\rline2");
    }

    #[test]
    fn escape_ctrl_z() {
        assert_eq!(escape_string("data\x1aend"), "data\\Zend");
    }

    #[test]
    fn escape_multiple_special_chars() {
        assert_eq!(
            escape_string("it's a \"test\"\nwith\\stuff"),
            "it\\'s a \\\"test\\\"\\nwith\\\\stuff"
        );
    }

    #[test]
    fn escape_utf8_passthrough() {
        assert_eq!(escape_string("café ñ 日本語"), "café ñ 日本語");
    }

    #[test]
    fn escape_sql_injection_attempt() {
        assert_eq!(
            escape_string("'; DROP TABLE users; --"),
            "\\'; DROP TABLE users; --"
        );
    }

    // escape_identifier tests

    #[test]
    fn identifier_no_backticks() {
        assert_eq!(escape_identifier("users"), "users");
    }

    #[test]
    fn identifier_with_backticks() {
        assert_eq!(escape_identifier("us`ers"), "users");
    }

    #[test]
    fn identifier_all_backticks() {
        assert_eq!(escape_identifier("```"), "");
    }

    #[test]
    fn identifier_empty() {
        assert_eq!(escape_identifier(""), "");
    }

    // ConnectionManager tests (without MySQL connection)

    #[test]
    fn connection_manager_new() {
        let mgr = ConnectionManager::new();
        assert_eq!(mgr.global_error.code, MysqlError::Ok);
        assert!(!mgr.exists(1));
    }

    #[test]
    fn connection_manager_exists_false() {
        let mgr = ConnectionManager::new();
        assert!(!mgr.exists(0));
        assert!(!mgr.exists(1));
        assert!(!mgr.exists(999));
    }

    #[test]
    fn connection_manager_disconnect_nonexistent() {
        let mut mgr = ConnectionManager::new();
        assert!(!mgr.disconnect(1));
    }

    #[test]
    fn connection_manager_get_error_global() {
        let mgr = ConnectionManager::new();
        let err = mgr.get_error(0);
        assert_eq!(err.code, MysqlError::Ok);
    }

    #[test]
    fn connection_manager_get_error_nonexistent_falls_back() {
        let mgr = ConnectionManager::new();
        let err = mgr.get_error(999);
        assert_eq!(err.code, MysqlError::Ok); // falls back to global
    }

    #[test]
    fn connection_manager_get_pool_nonexistent() {
        let mgr = ConnectionManager::new();
        assert!(mgr.get_pool(1).is_none());
    }
}
