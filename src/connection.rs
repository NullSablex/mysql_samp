use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use mysql::prelude::Queryable;
use mysql::{
    ClientIdentity, Opts, OptsBuilder, Pool, PoolConstraints, PoolOpts, PooledConn, SslOpts,
};

use crate::cache::{CacheEntry, CacheRow, ResultSet};
use crate::error::{ErrorState, MysqlError};
use crate::logger::Logger;
use crate::options::MysqlOptions;

/// Lower bound used when `MYSQL_OPT_POOL_SIZE` sets the ceiling. Matches the
/// driver default, clamped so it never exceeds the requested maximum.
const DEFAULT_POOL_MIN: usize = 10;

struct ConnectionEntry {
    pool: Pool,
    last_error: ErrorState,
    auto_reconnect: bool,
    escape_mode: EscapeMode,
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
            builder.tcp_connect_timeout(Some(Duration::from_secs(u64::from(timeout))))
        } else {
            builder
        };

        let builder = if options.ssl {
            let mut ssl_opts = SslOpts::default();

            // Without a CA the driver trusts the compiled-in webpki roots
            // (the Mozilla bundle), NOT the OS trust store. A server using an
            // internal CA or a self-signed certificate — the common case for a
            // game server — therefore needs MYSQL_OPT_SSL_CA.
            if let Some(ca) = options.ssl_ca.as_deref() {
                ssl_opts = ssl_opts.with_root_cert_path(Some(PathBuf::from(ca)));
            }

            // Mutual TLS: both halves are required, a lone one is a
            // configuration mistake worth reporting rather than ignoring.
            match (options.ssl_cert.as_deref(), options.ssl_key.as_deref()) {
                (Some(cert), Some(key)) => {
                    ssl_opts = ssl_opts.with_client_identity(Some(ClientIdentity::new(
                        PathBuf::from(cert),
                        PathBuf::from(key),
                    )));
                }
                (Some(_), None) => {
                    Logger::warn(
                        "MYSQL_OPT_SSL_CERT was set without MYSQL_OPT_SSL_KEY; \
                         client certificate ignored.",
                    );
                }
                (None, Some(_)) => {
                    Logger::warn(
                        "MYSQL_OPT_SSL_KEY was set without MYSQL_OPT_SSL_CERT; \
                         client certificate ignored.",
                    );
                }
                (None, None) => {}
            }

            if !options.ssl_verify_cert {
                Logger::warn(
                    "MYSQL_OPT_SSL_VERIFY_CERT is disabled: the server certificate and \
                     hostname are NOT verified. The connection is encrypted but open to \
                     man-in-the-middle. Use MYSQL_OPT_SSL_CA instead whenever possible.",
                );
                ssl_opts = ssl_opts
                    .with_danger_accept_invalid_certs(true)
                    .with_danger_skip_domain_validation(true);
            }

            builder.ssl_opts(Some(ssl_opts))
        } else {
            builder
        };

        // Pool ceiling. The driver defaults to min 10 / max 100; a smaller max
        // is the usual reason to touch this, so the minimum is clamped below it
        // rather than left at 10 (which `PoolConstraints::new` would reject).
        let builder = match options.pool_size {
            Some(max) => {
                let max = usize::try_from(max).unwrap_or(usize::MAX);
                let min = DEFAULT_POOL_MIN.min(max);
                match PoolConstraints::new(min, max) {
                    Some(constraints) => {
                        builder.pool_opts(PoolOpts::default().with_constraints(constraints))
                    }
                    None => {
                        Logger::warn(&format!(
                            "MYSQL_OPT_POOL_SIZE={max} is not a usable pool size; \
                             keeping the driver default."
                        ));
                        builder
                    }
                }
            }
            None => builder,
        };

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
        let escape_mode = match pool.get_conn() {
            Ok(mut conn) => detect_escape_mode(&mut conn),
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
        };

        self.global_error = ErrorState::ok();

        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        self.connections.insert(
            id,
            ConnectionEntry {
                pool,
                last_error: ErrorState::ok(),
                auto_reconnect: options.auto_reconnect,
                escape_mode,
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
                let query = format!("SET NAMES '{}'", escape_string(charset, entry.escape_mode));
                conn.query_drop(&query).is_ok()
            }
            Err(_) => false,
        }
    }

    /// Escaping mode of a connection. Unknown IDs fall back to the MySQL
    /// default, which is what a server uses unless configured otherwise.
    pub fn escape_mode(&self, conn_id: i32) -> EscapeMode {
        self.connections
            .get(&conn_id)
            .map_or(EscapeMode::Backslash, |entry| entry.escape_mode)
    }

    /// Gets the current character set for a connection.
    pub fn get_charset(&self, conn_id: i32) -> Option<String> {
        let entry = self.connections.get(&conn_id)?;
        let mut conn = entry.pool.get_conn().ok()?;
        let result: Option<String> = conn.query_first("SELECT @@character_set_connection").ok()?;
        result
    }
}

/// Reads `sql_mode` from a freshly opened connection to decide how string
/// literals must be escaped on it.
///
/// On failure the connection is assumed to be in the default mode. That is the
/// safe fallback: it is also what the server does unless explicitly configured
/// otherwise, and a wrong guess here is loud (broken queries), not silent.
fn detect_escape_mode(conn: &mut PooledConn) -> EscapeMode {
    let sql_mode: Option<String> = conn.query_first("SELECT @@SESSION.sql_mode").ok().flatten();

    let Some(sql_mode) = sql_mode else {
        Logger::warn(
            "Could not read sql_mode; assuming backslash escaping. If the server runs with \
             NO_BACKSLASH_ESCAPES, use prepared statements instead of mysql_format.",
        );
        return EscapeMode::Backslash;
    };

    if sql_mode
        .split(',')
        .any(|flag| flag.trim().eq_ignore_ascii_case("NO_BACKSLASH_ESCAPES"))
    {
        Logger::warn(
            "Server runs with NO_BACKSLASH_ESCAPES: escaping switched to quote doubling. \
             Only single-quoted literals can be escaped safely in this mode.",
        );
        EscapeMode::NoBackslashEscapes
    } else {
        EscapeMode::Backslash
    }
}

/// Escapes a SQL identifier (table/column name) by removing backticks.
/// Used for safe backtick-quoting: `escape_identifier(name)` -> `` `safe_name` ``
pub fn escape_identifier(input: &str) -> String {
    input.replace('`', "")
}

/// How string literals must be escaped on a given connection.
///
/// Under `sql_mode=NO_BACKSLASH_ESCAPES` the backslash stops being an escape
/// character, so `\'` no longer escapes the quote: it becomes a literal
/// backslash followed by a *live* quote, and the value breaks out of the
/// literal. The only valid escape in that mode is doubling the quote.
///
/// Picking the wrong mode reopens SQL injection in **either** direction —
/// emitting `\'` against a `NO_BACKSLASH_ESCAPES` server terminates the
/// string early, and emitting `''` against a normal server lets an input of
/// `\'` do the same. That is why the mode is detected per connection at
/// `connect` and threaded explicitly to every call site instead of defaulting.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EscapeMode {
    /// Default MySQL: backslash escapes are honoured.
    Backslash,
    /// `sql_mode` contains `NO_BACKSLASH_ESCAPES`.
    NoBackslashEscapes,
}

/// Escapes a string for safe interpolation into a **single-quoted** SQL
/// literal, following the same rules as `mysql_real_escape_string`.
///
/// Under [`EscapeMode::NoBackslashEscapes`] only the quote is escaped (by
/// doubling); every other byte is literal. As MySQL itself documents for that
/// mode, the result is only safe inside single quotes — a double-quoted
/// literal cannot be escaped safely there.
pub fn escape_string(input: &str, mode: EscapeMode) -> String {
    if mode == EscapeMode::NoBackslashEscapes {
        return input.replace('\'', "''");
    }

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
/// Runs every step inside one `START TRANSACTION` … `COMMIT` on a single
/// connection. Any failing step aborts the batch and rolls it back, so the
/// database never observes a partially applied transaction.
///
/// Returns the cache of the **last** step, which is where `cache_affected_rows`
/// and `cache_insert_id` are usually wanted.
///
/// There is deliberately no auto-reconnect retry here: replaying a transaction
/// after a mid-flight connection loss could re-apply steps the server already
/// committed. A dropped connection aborts the transaction and reports it.
pub fn attempt_transaction(
    pool: &Pool,
    steps: &[crate::transaction::TxStep],
) -> Result<CacheEntry, QueryError> {
    let mut conn = pool.get_conn().map_err(|e| QueryError {
        code: 0,
        message: e.to_string(),
    })?;

    execute_query(&mut conn, "START TRANSACTION")?;

    let mut last: Option<CacheEntry> = None;
    for step in steps {
        let outcome = if step.params.is_empty() {
            execute_query(&mut conn, &step.query)
        } else {
            execute_prepared(&mut conn, &step.query, step.params.clone())
        };

        match outcome {
            Ok(cache) => last = Some(cache),
            Err(e) => {
                // Best-effort: if the rollback itself fails the connection is
                // already gone, and the server discards the transaction when it
                // closes. The original error is what the caller needs.
                let _ = execute_query(&mut conn, "ROLLBACK");
                return Err(e);
            }
        }
    }

    execute_query(&mut conn, "COMMIT")?;

    Ok(last.unwrap_or_else(|| CacheEntry::empty(String::from("START TRANSACTION"))))
}

/// Runs a script's statements in order on one connection, stopping at the
/// first failure.
///
/// Deliberately **not** wrapped in a transaction: these scripts are usually
/// schema work, and DDL causes an implicit commit in MySQL — a transaction
/// around it would suggest an atomicity that the server does not provide.
///
/// Returns the cache of the last statement.
pub fn attempt_script(pool: &Pool, statements: &[String]) -> Result<CacheEntry, QueryError> {
    let mut conn = pool.get_conn().map_err(|e| QueryError {
        code: 0,
        message: e.to_string(),
    })?;

    let mut last: Option<CacheEntry> = None;
    for (index, statement) in statements.iter().enumerate() {
        match execute_query(&mut conn, statement) {
            Ok(cache) => last = Some(cache),
            Err(mut e) => {
                // Naming the position turns "syntax error" into something
                // actionable in a file with dozens of statements.
                e.message = format!(
                    "statement {} of {}: {}",
                    index + 1,
                    statements.len(),
                    e.message
                );
                return Err(e);
            }
        }
    }

    Ok(last.unwrap_or_else(|| CacheEntry::empty(String::new())))
}

/// Prepared-statement counterpart of [`attempt_query`], with the same
/// single-retry auto-reconnect behaviour.
pub fn attempt_prepared(
    pool: &Pool,
    query: &str,
    params: Vec<mysql::Value>,
    auto_reconnect: bool,
) -> Result<CacheEntry, QueryError> {
    let mut conn = pool.get_conn().map_err(|e| QueryError {
        code: 0,
        message: e.to_string(),
    })?;

    match execute_prepared(&mut conn, query, params.clone()) {
        Err(ref e) if auto_reconnect && e.code == 0 => {
            drop(conn);
            let mut conn2 = pool.get_conn().map_err(|e2| QueryError {
                code: 0,
                message: e2.to_string(),
            })?;
            execute_prepared(&mut conn2, query, params)
        }
        other => other,
    }
}

pub fn attempt_query(
    pool: &Pool,
    query: &str,
    auto_reconnect: bool,
) -> Result<CacheEntry, QueryError> {
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
/// Every result set a query produced, before the connection stats are read.
struct RawResult {
    sets: Vec<ResultSet>,
    truncated: bool,
}

/// Drains a query result, including **every** result set it carries.
///
/// A plain `SELECT` yields one. A script or a `CALL` to a stored procedure
/// yields several, and stopping at the first would both lose data and leave
/// the connection out of sync with the protocol.
///
/// Generic over the protocol so the text protocol (`query_iter`) and the
/// binary/prepared protocol (`exec_iter`) share one implementation.
fn collect_result<P: mysql::prelude::Protocol>(
    mut result: mysql::QueryResult<'_, '_, '_, P>,
) -> Result<RawResult, QueryError> {
    let mut sets = Vec::new();
    let mut total_rows = 0usize;
    let mut truncated = false;

    while let Some(mut set) = result.iter() {
        let cols_ref = set.columns();
        let field_names: Vec<String> = cols_ref
            .as_ref()
            .iter()
            .map(|c| c.name_str().to_string())
            .collect();
        let field_types: Vec<u8> = cols_ref
            .as_ref()
            .iter()
            .map(|c| c.column_type() as u8)
            .collect();

        let mut rows: Vec<CacheRow> = Vec::new();
        for row_result in set.by_ref() {
            match row_result {
                Ok(row) => {
                    // The ceiling spans the whole query, not each set, so a
                    // script cannot bypass it by returning many small sets.
                    if total_rows >= MAX_RESULT_ROWS {
                        truncated = true;
                        continue; // drain the rest to avoid protocol desync
                    }
                    let mut cells = Vec::with_capacity(field_names.len());
                    for i in 0..field_names.len() {
                        let val: Option<String> = row.get(i);
                        cells.push(val);
                    }
                    rows.push(cells);
                    total_rows += 1;
                }
                Err(e) => {
                    return Err(QueryError {
                        code: extract_mysql_errno(&e),
                        message: e.to_string(),
                    });
                }
            }
        }

        sets.push(ResultSet {
            rows,
            field_names,
            field_types,
        });
    }

    Ok(RawResult { sets, truncated })
}

/// Turns a drained result set plus the connection's post-execution stats into
/// a cache entry.
fn build_cache_entry(
    conn: &mut PooledConn,
    raw: RawResult,
    start: std::time::Instant,
    query: &str,
) -> CacheEntry {
    if raw.truncated {
        crate::logger::Logger::warn(&format!(
            "Query result truncated to {} rows.",
            MAX_RESULT_ROWS
        ));
    }

    CacheEntry::with_results(
        raw.sets,
        conn.affected_rows(),
        conn.last_insert_id(),
        conn.warnings(),
        start.elapsed().as_micros(),
        query.to_string(),
    )
}

pub fn execute_query(conn: &mut PooledConn, query: &str) -> Result<CacheEntry, QueryError> {
    let start = std::time::Instant::now();

    let raw = {
        let result = conn.query_iter(query).map_err(|e| QueryError {
            code: extract_mysql_errno(&e),
            message: e.to_string(),
        })?;
        collect_result(result)?
    };

    Ok(build_cache_entry(conn, raw, start, query))
}

/// Runs `query` through the binary protocol with `params` bound server-side.
///
/// The values never enter the SQL text, so no escaping is involved and the
/// server cannot reinterpret a value as syntax — which is what makes this
/// immune to injection rather than merely defended against it.
pub fn execute_prepared(
    conn: &mut PooledConn,
    query: &str,
    params: Vec<mysql::Value>,
) -> Result<CacheEntry, QueryError> {
    let start = std::time::Instant::now();

    let raw = {
        let result = conn
            .exec_iter(query, mysql::Params::Positional(params))
            .map_err(|e| QueryError {
                code: extract_mysql_errno(&e),
                message: e.to_string(),
            })?;
        collect_result(result)?
    };

    Ok(build_cache_entry(conn, raw, start, query))
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
        assert_eq!(escape_string("", EscapeMode::Backslash), "");
    }

    #[test]
    fn escape_no_special_chars() {
        assert_eq!(
            escape_string("hello world", EscapeMode::Backslash),
            "hello world"
        );
    }

    #[test]
    fn escape_single_quote() {
        assert_eq!(escape_string("it's", EscapeMode::Backslash), "it\\'s");
    }

    #[test]
    fn escape_double_quote() {
        assert_eq!(
            escape_string(r#"say "hi""#, EscapeMode::Backslash),
            r#"say \"hi\""#
        );
    }

    #[test]
    fn escape_backslash() {
        assert_eq!(
            escape_string(r"path\to", EscapeMode::Backslash),
            r"path\\to"
        );
    }

    #[test]
    fn escape_null_byte() {
        assert_eq!(escape_string("a\0b", EscapeMode::Backslash), "a\\0b");
    }

    #[test]
    fn escape_newline() {
        assert_eq!(
            escape_string("line1\nline2", EscapeMode::Backslash),
            "line1\\nline2"
        );
    }

    #[test]
    fn escape_carriage_return() {
        assert_eq!(
            escape_string("line1\rline2", EscapeMode::Backslash),
            "line1\\rline2"
        );
    }

    #[test]
    fn escape_ctrl_z() {
        assert_eq!(
            escape_string("data\x1aend", EscapeMode::Backslash),
            "data\\Zend"
        );
    }

    #[test]
    fn escape_multiple_special_chars() {
        assert_eq!(
            escape_string("it's a \"test\"\nwith\\stuff", EscapeMode::Backslash),
            "it\\'s a \\\"test\\\"\\nwith\\\\stuff"
        );
    }

    #[test]
    fn escape_utf8_passthrough() {
        assert_eq!(
            escape_string("café ñ 日本語", EscapeMode::Backslash),
            "café ñ 日本語"
        );
    }

    #[test]
    fn escape_sql_injection_attempt() {
        assert_eq!(
            escape_string("'; DROP TABLE users; --", EscapeMode::Backslash),
            "\\'; DROP TABLE users; --"
        );
    }

    #[test]
    fn escape_consecutive_quotes() {
        // Three quotes in a row must each be escaped individually.
        assert_eq!(escape_string("'''", EscapeMode::Backslash), "\\'\\'\\'");
    }

    #[test]
    fn escape_already_escaped_is_double_escaped() {
        // Important invariant: feeding the function its own output produces
        // a different (deeper-escaped) string. Callers must escape EXACTLY ONCE.
        let once = escape_string("a'b", EscapeMode::Backslash);
        let twice = escape_string(&once, EscapeMode::Backslash);
        assert_ne!(once, twice);
        // After two escapes: 'a', '\\', '\\', '\\', '\'', 'b'  → r"a\\\'b" with each char doubled.
        assert_eq!(twice, "a\\\\\\'b");
    }

    #[test]
    fn escape_all_specials_at_once() {
        // \0 \n \r \\ \' \" \x1a
        let input = "\0\n\r\\\'\"\x1a";
        let expected = "\\0\\n\\r\\\\\\'\\\"\\Z";
        assert_eq!(escape_string(input, EscapeMode::Backslash), expected);
    }

    #[test]
    fn escape_low_control_chars_passthrough() {
        // Only \0 \n \r \x1a are special. Other low-ASCII control bytes
        // pass through unchanged (no \xNN encoding is done).
        assert_eq!(
            escape_string("\x01\x07\x08\x0b", EscapeMode::Backslash),
            "\x01\x07\x08\x0b"
        );
    }

    // NO_BACKSLASH_ESCAPES mode

    #[test]
    fn no_backslash_mode_doubles_quote() {
        assert_eq!(
            escape_string("O'Brien", EscapeMode::NoBackslashEscapes),
            "O''Brien"
        );
    }

    #[test]
    fn no_backslash_mode_leaves_backslash_literal() {
        // The backslash is not an escape character in this mode, so doubling
        // it would corrupt the value instead of protecting it.
        assert_eq!(
            escape_string(r"path\to", EscapeMode::NoBackslashEscapes),
            r"path\to"
        );
    }

    #[test]
    fn no_backslash_mode_blocks_the_backslash_quote_escape() {
        // The attack the mode enables: under NO_BACKSLASH_ESCAPES the server
        // reads `\'` as a literal backslash followed by a LIVE quote, so the
        // backslash escaper would let this input terminate the literal early.
        // Doubling leaves no unpaired quote behind.
        let escaped = escape_string(r"\' OR 1=1 -- ", EscapeMode::NoBackslashEscapes);
        assert_eq!(escaped, r"\'' OR 1=1 -- ");
        assert_eq!(escaped.matches('\'').count() % 2, 0);
    }

    #[test]
    fn backslash_mode_still_escapes_the_quote() {
        // Same input on a default server: here the backslash IS an escape
        // character, so the quote must be backslash-escaped instead.
        assert_eq!(
            escape_string(r"\' OR 1=1 -- ", EscapeMode::Backslash),
            r"\\\' OR 1=1 -- "
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
