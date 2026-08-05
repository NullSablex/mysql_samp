//! Prepared statements.
//!
//! The structural answer to SQL injection. `mysql_format` and the ORM build a
//! query by pasting escaped values into SQL text — correct only as long as the
//! escaping matches the server's `sql_mode` (see [`crate::connection::EscapeMode`]).
//! Here the values travel to the server separately from the statement, over the
//! binary protocol, so there is no text for a value to break out of and no
//! escaping to get wrong.

use std::collections::HashMap;

use mysql::Value;
use samp::amx::AmxIdent;

/// Upper bound on parameters bound to one statement. MySQL's own limit is
/// 65535; this is a sanity ceiling so a runaway loop in Pawn cannot grow the
/// vector without bound.
pub const MAX_STMT_PARAMS: usize = 4096;

pub struct Statement {
    pub conn_id: i32,
    /// AMX that created the statement, so it can be reclaimed when that script
    /// is unloaded — a gamemode restart would otherwise leak every handle.
    pub amx_ident: AmxIdent,
    pub query: String,
    pub params: Vec<Value>,
}

impl Statement {
    fn new(conn_id: i32, amx_ident: AmxIdent, query: String) -> Self {
        Self {
            conn_id,
            amx_ident,
            query,
            params: Vec::new(),
        }
    }

    /// Number of `?` placeholders in the statement text.
    ///
    /// Counted outside of string literals and comments so that a `?` inside
    /// `'...'`, `"..."`, a backtick-quoted identifier, `-- ...`, `# ...` or
    /// `/* ... */` is not mistaken for a placeholder.
    pub fn placeholder_count(&self) -> usize {
        count_placeholders(&self.query)
    }
}

pub struct StmtManager {
    statements: HashMap<i32, Statement>,
    next_id: i32,
}

impl StmtManager {
    pub fn new() -> Self {
        Self {
            statements: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create(&mut self, conn_id: i32, amx_ident: AmxIdent, query: String) -> i32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        self.statements
            .insert(id, Statement::new(conn_id, amx_ident, query));
        id
    }

    /// Drops every statement owned by an unloaded AMX.
    pub fn destroy_by_amx(&mut self, ident: AmxIdent) {
        self.statements.retain(|_, stmt| stmt.amx_ident != ident);
    }

    /// Drops every statement bound to a closed connection. They could never
    /// execute again, so keeping them would only grow the map.
    pub fn destroy_by_conn(&mut self, conn_id: i32) {
        self.statements.retain(|_, stmt| stmt.conn_id != conn_id);
    }

    pub fn get(&self, id: i32) -> Option<&Statement> {
        self.statements.get(&id)
    }

    pub fn destroy(&mut self, id: i32) -> bool {
        self.statements.remove(&id).is_some()
    }

    /// Appends a bound value. Fails when the statement is unknown or the
    /// parameter ceiling is reached.
    pub fn bind(&mut self, id: i32, value: Value) -> bool {
        let Some(stmt) = self.statements.get_mut(&id) else {
            return false;
        };
        if stmt.params.len() >= MAX_STMT_PARAMS {
            return false;
        }
        stmt.params.push(value);
        true
    }

    /// Drops the bound values, keeping the statement for reuse.
    pub fn reset(&mut self, id: i32) -> bool {
        let Some(stmt) = self.statements.get_mut(&id) else {
            return false;
        };
        stmt.params.clear();
        true
    }
}

/// Counts `?` placeholders, skipping quoted regions and comments.
fn count_placeholders(query: &str) -> usize {
    let bytes = query.as_bytes();
    let mut count = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        match bytes[i] {
            b'\'' | b'"' | b'`' => {
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() {
                    // Backslash escapes are skipped for '/" only; a backtick
                    // identifier has no backslash escaping.
                    if quote != b'`' && bytes[i] == b'\\' {
                        i += 2;
                        continue;
                    }
                    if bytes[i] == quote {
                        // A doubled quote is a literal quote, not a terminator.
                        if i + 1 < bytes.len() && bytes[i + 1] == quote {
                            i += 2;
                            continue;
                        }
                        break;
                    }
                    i += 1;
                }
                i += 1;
            }
            b'-' if bytes.get(i + 1) == Some(&b'-') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'#' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i += 2;
            }
            b'?' => {
                count += 1;
                i += 1;
            }
            _ => i += 1,
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_ident() -> AmxIdent {
        AmxIdent::from(std::ptr::dangling_mut::<samp::raw::types::AMX>())
    }

    fn dummy_ident_2() -> AmxIdent {
        AmxIdent::from(2usize as *mut samp::raw::types::AMX)
    }

    fn count(q: &str) -> usize {
        count_placeholders(q)
    }

    #[test]
    fn counts_plain_placeholders() {
        assert_eq!(count("SELECT * FROM t WHERE a = ? AND b = ?"), 2);
    }

    #[test]
    fn ignores_placeholder_inside_single_quotes() {
        assert_eq!(count("SELECT '?' FROM t WHERE a = ?"), 1);
    }

    #[test]
    fn ignores_placeholder_inside_double_quotes() {
        assert_eq!(count(r#"SELECT "?" FROM t WHERE a = ?"#), 1);
    }

    #[test]
    fn ignores_placeholder_inside_backticks() {
        assert_eq!(count("SELECT `we?rd` FROM t WHERE a = ?"), 1);
    }

    #[test]
    fn handles_escaped_quote_in_literal() {
        assert_eq!(count(r"SELECT 'it\'s ?' FROM t WHERE a = ?"), 1);
    }

    #[test]
    fn handles_doubled_quote_in_literal() {
        assert_eq!(count("SELECT 'it''s ?' FROM t WHERE a = ?"), 1);
    }

    #[test]
    fn ignores_placeholder_in_line_comment() {
        assert_eq!(count("SELECT 1 -- ? comment\nWHERE a = ?"), 1);
        assert_eq!(count("SELECT 1 # ? comment\nWHERE a = ?"), 1);
    }

    #[test]
    fn ignores_placeholder_in_block_comment() {
        assert_eq!(count("SELECT /* ? */ 1 WHERE a = ?"), 1);
    }

    #[test]
    fn unterminated_literal_does_not_hang_or_panic() {
        assert_eq!(count("SELECT 'unterminated ?"), 0);
        assert_eq!(count("SELECT /* unterminated ?"), 0);
    }

    // StmtManager

    #[test]
    fn bind_respects_the_parameter_ceiling() {
        let mut mgr = StmtManager::new();
        let id = mgr.create(1, dummy_ident(), "SELECT ?".into());

        for _ in 0..MAX_STMT_PARAMS {
            assert!(mgr.bind(id, Value::Int(1)));
        }
        assert!(!mgr.bind(id, Value::Int(1)), "ceiling must be enforced");
    }

    #[test]
    fn reset_clears_params_but_keeps_the_statement() {
        let mut mgr = StmtManager::new();
        let id = mgr.create(1, dummy_ident(), "SELECT ?".into());

        assert!(mgr.bind(id, Value::Int(7)));
        assert!(mgr.reset(id));
        assert_eq!(mgr.get(id).expect("still alive").params.len(), 0);
    }

    #[test]
    fn operations_on_unknown_ids_fail() {
        let mut mgr = StmtManager::new();
        assert!(!mgr.bind(999, Value::Int(1)));
        assert!(!mgr.reset(999));
        assert!(!mgr.destroy(999));
        assert!(mgr.get(999).is_none());
    }

    #[test]
    fn destroy_by_amx_only_touches_that_script() {
        let mut mgr = StmtManager::new();
        let mine = mgr.create(1, dummy_ident(), "SELECT 1".into());
        let other = mgr.create(1, dummy_ident_2(), "SELECT 2".into());

        mgr.destroy_by_amx(dummy_ident());

        assert!(mgr.get(mine).is_none(), "unloaded script's handle must go");
        assert!(mgr.get(other).is_some(), "other script must be untouched");
    }

    #[test]
    fn destroy_by_conn_only_touches_that_connection() {
        let mut mgr = StmtManager::new();
        let on_one = mgr.create(1, dummy_ident(), "SELECT 1".into());
        let on_two = mgr.create(2, dummy_ident(), "SELECT 2".into());

        mgr.destroy_by_conn(1);

        assert!(mgr.get(on_one).is_none());
        assert!(mgr.get(on_two).is_some());
    }

    #[test]
    fn destroy_removes_the_statement() {
        let mut mgr = StmtManager::new();
        let id = mgr.create(1, dummy_ident(), "SELECT 1".into());
        assert!(mgr.destroy(id));
        assert!(mgr.get(id).is_none());
    }

    #[test]
    fn placeholder_count_is_exposed_on_the_statement() {
        let mut mgr = StmtManager::new();
        let id = mgr.create(
            1,
            dummy_ident(),
            "SELECT * FROM t WHERE a = ? AND b = ?".into(),
        );
        assert_eq!(mgr.get(id).expect("exists").placeholder_count(), 2);
    }
}
