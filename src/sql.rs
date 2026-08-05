//! Lexical scanning of SQL text.
//!
//! Both the placeholder counter and the statement splitter need the same
//! thing: to look at SQL and ignore anything inside a string literal, a quoted
//! identifier or a comment. A `?` inside `'...'` is data, not a placeholder; a
//! `;` inside `/* ... */` does not end a statement. One scanner serves both so
//! the two cannot drift apart.

/// Walks `sql` and calls `on_code_byte(index, byte)` for every byte that sits
/// in code position — outside `'...'`, `"..."`, `` `...` ``, `-- …`, `# …` and
/// `/* … */`.
///
/// Unterminated literals and comments simply consume the rest of the input:
/// malformed SQL must not make this hang or panic, and the server will reject
/// it anyway.
fn scan_code<F: FnMut(usize, u8)>(sql: &str, mut on_code_byte: F) {
    let bytes = sql.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        match bytes[i] {
            quote @ (b'\'' | b'"' | b'`') => {
                i += 1;
                while i < bytes.len() {
                    // Backslash escapes apply to string literals only; a
                    // backtick-quoted identifier has no such escape.
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
            byte => {
                on_code_byte(i, byte);
                i += 1;
            }
        }
    }
}

/// Number of `?` placeholders in code position.
pub fn count_placeholders(query: &str) -> usize {
    let mut count = 0usize;
    scan_code(query, |_, byte| {
        if byte == b'?' {
            count += 1;
        }
    });
    count
}

/// Splits a script into individual statements on `;` in code position.
///
/// Empty fragments — a trailing semicolon, a comment-only line, blank space —
/// are dropped, so the result contains only statements worth sending.
pub fn split_statements(script: &str) -> Vec<String> {
    let mut boundaries = Vec::new();
    scan_code(script, |idx, byte| {
        if byte == b';' {
            boundaries.push(idx);
        }
    });

    let mut out = Vec::new();
    let mut start = 0usize;
    for end in boundaries {
        push_statement(&mut out, &script[start..end]);
        start = end + 1;
    }
    push_statement(&mut out, &script[start..]);
    out
}

/// Keeps a fragment only if it carries actual SQL, not just blanks or comments.
fn push_statement(out: &mut Vec<String>, fragment: &str) {
    let trimmed = fragment.trim();
    if trimmed.is_empty() {
        return;
    }

    // A fragment made only of comments has no code bytes.
    let mut has_code = false;
    scan_code(trimmed, |_, byte| {
        if !byte.is_ascii_whitespace() {
            has_code = true;
        }
    });

    if has_code {
        out.push(trimmed.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Placeholder counting

    #[test]
    fn counts_plain_placeholders() {
        assert_eq!(
            count_placeholders("SELECT * FROM t WHERE a = ? AND b = ?"),
            2
        );
    }

    #[test]
    fn ignores_placeholders_inside_quotes_and_comments() {
        assert_eq!(count_placeholders("SELECT '?' FROM t WHERE a = ?"), 1);
        assert_eq!(count_placeholders(r#"SELECT "?" FROM t WHERE a = ?"#), 1);
        assert_eq!(count_placeholders("SELECT `we?rd` FROM t WHERE a = ?"), 1);
        assert_eq!(count_placeholders("SELECT /* ? */ 1 WHERE a = ?"), 1);
        assert_eq!(count_placeholders("SELECT 1 -- ?\nWHERE a = ?"), 1);
        assert_eq!(count_placeholders("SELECT 1 # ?\nWHERE a = ?"), 1);
    }

    #[test]
    fn handles_escaped_and_doubled_quotes() {
        assert_eq!(
            count_placeholders(r"SELECT 'it\'s ?' FROM t WHERE a = ?"),
            1
        );
        assert_eq!(count_placeholders("SELECT 'it''s ?' FROM t WHERE a = ?"), 1);
    }

    #[test]
    fn unterminated_input_does_not_hang_or_panic() {
        assert_eq!(count_placeholders("SELECT 'unterminated ?"), 0);
        assert_eq!(count_placeholders("SELECT /* unterminated ?"), 0);
    }

    // Statement splitting

    #[test]
    fn splits_on_semicolons() {
        let stmts = split_statements("SELECT 1; SELECT 2; SELECT 3");
        assert_eq!(stmts, vec!["SELECT 1", "SELECT 2", "SELECT 3"]);
    }

    #[test]
    fn a_trailing_semicolon_does_not_add_an_empty_statement() {
        assert_eq!(split_statements("SELECT 1;"), vec!["SELECT 1"]);
        assert_eq!(split_statements("SELECT 1;\n\n"), vec!["SELECT 1"]);
    }

    #[test]
    fn does_not_split_on_a_semicolon_inside_a_literal() {
        let stmts = split_statements("INSERT INTO t VALUES ('a;b'); SELECT 2");
        assert_eq!(stmts, vec!["INSERT INTO t VALUES ('a;b')", "SELECT 2"]);
    }

    #[test]
    fn does_not_split_on_a_semicolon_inside_a_comment() {
        let stmts = split_statements("SELECT 1 /* ; not a break */; SELECT 2");
        assert_eq!(stmts.len(), 2);
        assert!(stmts[0].starts_with("SELECT 1"));
        assert_eq!(stmts[1], "SELECT 2");
    }

    #[test]
    fn comment_only_fragments_are_dropped() {
        let stmts = split_statements("-- header comment\nSELECT 1;\n# trailing note\n");
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].ends_with("SELECT 1"));
    }

    #[test]
    fn an_empty_or_comment_only_script_yields_nothing() {
        assert!(split_statements("").is_empty());
        assert!(split_statements("   \n\t\n").is_empty());
        assert!(split_statements("-- just a comment\n").is_empty());
        assert!(split_statements("/* only a block comment */").is_empty());
    }

    #[test]
    fn a_multiline_statement_survives_intact() {
        let script = "CREATE TABLE t (\n  id INT,\n  name VARCHAR(24)\n);\nSELECT 1;";
        let stmts = split_statements(script);
        assert_eq!(stmts.len(), 2);
        assert!(stmts[0].contains("VARCHAR(24)"));
    }
}
