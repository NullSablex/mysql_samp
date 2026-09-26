//! Connection credentials loaded from a file.
//!
//! Keeping credentials out of the gamemode source is the point: the `.pwn` is
//! usually in version control, the config file is not. The parser is
//! deliberately minimal — `key = value`, one per line — because anything
//! richer would be a second configuration language to document and to get
//! wrong.
//!
//! Nothing read here is ever logged. A parse failure reports the *key* that
//! was missing or unknown, never the value.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::logger::Logger;

/// Credentials read from a config file. Options (port, SSL, …) are not part of
/// it — they stay with `mysql_options_new`, so there is one place to look.
pub struct ConnectionFile {
    pub host: String,
    pub user: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// The file could not be read. Holds the OS error, not the contents.
    Unreadable(String),
    /// A required key is absent.
    MissingKey(&'static str),
}

impl ConfigError {
    pub fn message(&self) -> String {
        match self {
            Self::Unreadable(err) => format!("could not read the file: {err}"),
            Self::MissingKey(key) => format!("required key '{key}' is missing"),
        }
    }
}

/// Parses `key = value` lines.
///
/// - `#` and `;` start a comment, to the end of the line.
/// - Keys are case-insensitive; surrounding whitespace is trimmed.
/// - A value may be wrapped in single or double quotes, which is the only way
///   to keep leading or trailing spaces in a password.
/// - Unknown keys are ignored rather than rejected, so a file shared with
///   another tool does not break the connect.
/// - `${NAME}` in a value is replaced by that environment variable, which is
///   how a password stays out of the file as well as out of the gamemode.
pub fn parse(contents: &str) -> Result<ConnectionFile, ConfigError> {
    let mut values: HashMap<String, String> = HashMap::new();

    for line in contents.lines() {
        let line = strip_comment(line).trim();
        if line.is_empty() {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        values.insert(
            key.trim().to_ascii_lowercase(),
            expand_env(unquote(value.trim())),
        );
    }

    let take = |key: &'static str| values.get(key).cloned().ok_or(ConfigError::MissingKey(key));

    Ok(ConnectionFile {
        host: take("host")?,
        user: take("user")?,
        // An empty password is legitimate (a local socket account often has
        // none), so only a *missing* key is an error.
        password: values.get("password").cloned().unwrap_or_default(),
        database: take("database")?,
    })
}

pub fn load(path: &Path) -> Result<ConnectionFile, ConfigError> {
    let contents = fs::read_to_string(path).map_err(|e| ConfigError::Unreadable(e.to_string()))?;
    parse(&contents)
}

/// Replaces every `${NAME}` with the environment variable of that name.
///
/// This is what lets the file name a secret instead of holding one: the
/// operator exports `MYSQL_PASSWORD`, the repository carries
/// `password = ${MYSQL_PASSWORD}`, and a copy of the file is worth nothing on
/// its own.
///
/// An unset variable expands to nothing rather than being left as the literal
/// `${NAME}`, and says so in the log. Sending `${MYSQL_PASSWORD}` to the
/// server as if it were the password would fail with a confusing "access
/// denied" instead of naming the real problem; an empty password at least
/// fails for a reason the operator can read. A name that is not there is
/// almost always a variable the service unit forgot to pass.
///
/// Only `${NAME}` is recognised. A bare `$NAME` is left alone, so a password
/// that happens to contain a dollar sign keeps working.
fn expand_env(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];

        let Some(end) = after.find('}') else {
            // No closing brace: not a reference, just text that starts with
            // `${`. Keep it verbatim.
            out.push_str(&rest[start..]);
            return out;
        };

        let name = &after[..end];
        match std::env::var(name) {
            Ok(found) => out.push_str(&found),
            Err(_) => Logger::warn(&format!(
                "Connection file: ${{{name}}} is not set in the environment; \
                 it expanded to nothing."
            )),
        }
        rest = &after[end + 1..];
    }

    out.push_str(rest);
    out
}

fn strip_comment(line: &str) -> &str {
    match line.find(['#', ';']) {
        Some(idx) => &line[..idx],
        None => line,
    }
}

fn unquote(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' || first == b'\'') && first == last {
            return &value[1..value.len() - 1];
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
host = 127.0.0.1
user = samp
password = s3cr3t
database = samp_server
";

    #[test]
    fn parses_the_four_required_keys() {
        let cfg = parse(SAMPLE).expect("valid file");
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.user, "samp");
        assert_eq!(cfg.password, "s3cr3t");
        assert_eq!(cfg.database, "samp_server");
    }

    #[test]
    fn keys_are_case_insensitive_and_trimmed() {
        let cfg =
            parse("  HOST =  db.local \nUser=samp\nPassword=x\nDATABASE=d\n").expect("valid file");
        assert_eq!(cfg.host, "db.local");
        assert_eq!(cfg.user, "samp");
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        let cfg = parse("# comment\n\nhost=h ; trailing\nuser=u\npassword=p\ndatabase=d\n")
            .expect("valid file");
        assert_eq!(cfg.host, "h");
    }

    #[test]
    fn quotes_preserve_surrounding_spaces() {
        let cfg =
            parse("host=h\nuser=u\npassword=\"  spaced  \"\ndatabase=d\n").expect("valid file");
        assert_eq!(cfg.password, "  spaced  ");
    }

    #[test]
    fn a_password_may_contain_an_equals_sign() {
        // split_once stops at the first '=', so the rest of the line survives.
        let cfg = parse("host=h\nuser=u\npassword=a=b=c\ndatabase=d\n").expect("valid file");
        assert_eq!(cfg.password, "a=b=c");
    }

    #[test]
    fn an_empty_password_is_accepted() {
        let cfg = parse("host=h\nuser=u\npassword=\ndatabase=d\n").expect("valid file");
        assert_eq!(cfg.password, "");
    }

    #[test]
    fn a_missing_password_key_is_also_accepted() {
        let cfg = parse("host=h\nuser=u\ndatabase=d\n").expect("valid file");
        assert_eq!(cfg.password, "");
    }

    #[test]
    fn a_missing_required_key_names_it() {
        assert!(matches!(
            parse("user=u\npassword=p\ndatabase=d\n"),
            Err(ConfigError::MissingKey("host"))
        ));
        assert!(matches!(
            parse("host=h\nuser=u\npassword=p\n"),
            Err(ConfigError::MissingKey("database"))
        ));
    }

    #[test]
    fn unknown_keys_do_not_break_the_file() {
        let cfg = parse("host=h\nuser=u\npassword=p\ndatabase=d\nport=3307\ncharset=utf8\n")
            .expect("valid file");
        assert_eq!(cfg.host, "h");
    }

    #[test]
    fn the_error_message_never_contains_a_value() {
        let Err(err) = parse("user=u\npassword=hunter2\ndatabase=d\n") else {
            panic!("expected the missing host key to be reported");
        };
        assert!(!err.message().contains("hunter2"));
    }
    // ${ENV} expansion. The variable names are unique per test: the
    // environment is process-wide and the suite runs in parallel.

    #[test]
    fn env_reference_is_expanded() {
        unsafe { std::env::set_var("MYSQL_SAMP_T1_PASS", "s3cr3t") };
        let cfg = parse("host=h\nuser=u\npassword=${MYSQL_SAMP_T1_PASS}\ndatabase=d\n")
            .expect("valid file");
        assert_eq!(cfg.password, "s3cr3t");
    }

    #[test]
    fn env_reference_can_sit_inside_a_larger_value() {
        unsafe { std::env::set_var("MYSQL_SAMP_T2_HOST", "db.internal") };
        let cfg = parse("host=${MYSQL_SAMP_T2_HOST}\nuser=u\npassword=p\ndatabase=d\n")
            .expect("valid file");
        assert_eq!(cfg.host, "db.internal");
    }

    #[test]
    fn several_references_in_one_value() {
        unsafe { std::env::set_var("MYSQL_SAMP_T3_A", "aa") };
        unsafe { std::env::set_var("MYSQL_SAMP_T3_B", "bb") };
        let cfg =
            parse("host=h\nuser=u\npassword=x${MYSQL_SAMP_T3_A}y${MYSQL_SAMP_T3_B}z\ndatabase=d\n")
                .expect("valid file");
        assert_eq!(cfg.password, "xaaybbz");
    }

    #[test]
    fn an_unset_variable_expands_to_nothing() {
        let cfg = parse("host=h\nuser=u\npassword=${MYSQL_SAMP_T4_MISSING}\ndatabase=d\n")
            .expect("valid file");
        assert_eq!(
            cfg.password, "",
            "an unset name must not reach the server as a literal reference"
        );
    }

    #[test]
    fn a_dollar_without_braces_is_literal() {
        // A password is allowed to contain a dollar sign.
        let cfg = parse("host=h\nuser=u\npassword=pa$$word\ndatabase=d\n").expect("valid file");
        assert_eq!(cfg.password, "pa$$word");
    }

    #[test]
    fn an_unclosed_reference_is_left_alone() {
        let cfg = parse("host=h\nuser=u\npassword=${OPEN\ndatabase=d\n").expect("valid file");
        assert_eq!(cfg.password, "${OPEN");
    }
}
