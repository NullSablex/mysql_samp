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
            unquote(value.trim()).to_string(),
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
}
