use samp::args::Args;
use samp::cell::Ref;
use samp::native;
use samp::prelude::*;

use crate::connection::escape_string;
use crate::error::{ErrorState, MysqlError};
use crate::logger::Logger;
use crate::plugin::MysqlPlugin;
use crate::query::{CallbackInfo, CallbackParam};

/// Parameters bundled for [`MysqlPlugin::submit_query`].
/// Groups every value that describes a single query submission so the
/// internal helper has one data argument instead of seven positional ones.
struct QueryRequest<'a> {
    conn_id: i32,
    query: &'a str,
    callback: &'a str,
    format: &'a str,
    variadic_start: usize,
    /// `true` for FIFO-ordered submission, `false` for parallel.
    ordered: bool,
}

impl MysqlPlugin {
    /// mysql_query(connId, const query[], const callback[] = "", const format[] = "", {Float,_}:...)
    /// Non-blocking threaded query with FIFO ordering.
    #[native(name = "mysql_query", raw)]
    pub fn mysql_query(&mut self, _amx: &Amx, mut args: Args) -> bool {
        let Some(conn_id) = args.next_arg::<i32>() else {
            return false;
        };
        let Some(query_str) = args.next_arg::<AmxString>() else {
            return false;
        };

        let callback = args
            .next_arg::<AmxString>()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let format = args
            .next_arg::<AmxString>()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let variadic_start = if callback.is_empty() || format.is_empty() {
            3
        } else {
            4
        };

        self.submit_query(
            QueryRequest {
                conn_id,
                query: &query_str.to_string(),
                callback: &callback,
                format: &format,
                variadic_start,
                ordered: true,
            },
            &mut args,
        )
    }

    /// mysql_pquery(connId, const query[], const callback[] = "", const format[] = "", {Float,_}:...)
    /// Non-blocking parallel query (no order guarantee).
    #[native(name = "mysql_pquery", raw)]
    pub fn mysql_pquery(&mut self, _amx: &Amx, mut args: Args) -> bool {
        let Some(conn_id) = args.next_arg::<i32>() else {
            return false;
        };
        let Some(query_str) = args.next_arg::<AmxString>() else {
            return false;
        };

        let callback = args
            .next_arg::<AmxString>()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let format = args
            .next_arg::<AmxString>()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let variadic_start = if callback.is_empty() || format.is_empty() {
            3
        } else {
            4
        };

        self.submit_query(
            QueryRequest {
                conn_id,
                query: &query_str.to_string(),
                callback: &callback,
                format: &format,
                variadic_start,
                ordered: false,
            },
            &mut args,
        )
    }

    /// Internal: submits a query (ordered or parallel).
    fn submit_query(&mut self, req: QueryRequest<'_>, args: &mut Args) -> bool {
        let Some(pool) = self.connections.get_pool(req.conn_id) else {
            Logger::warn("Query failed: invalid connection ID.");
            self.connections.global_error = ErrorState::new(
                MysqlError::InvalidConnection,
                "Query failed: invalid connection ID.",
            );
            return false;
        };

        let auto_reconnect = self.connections.get_auto_reconnect(req.conn_id);

        let callback_info = if req.callback.is_empty() {
            None
        } else {
            let params = parse_variadic_params(args, req.format, req.variadic_start);
            Some(CallbackInfo {
                name: req.callback.to_string(),
                format: req.format.to_string(),
                params,
            })
        };

        if req.ordered {
            self.queries.submit_query(
                pool,
                req.query.to_string(),
                callback_info,
                req.conn_id,
                auto_reconnect,
            );
        } else {
            self.queries.submit_pquery(
                pool,
                req.query.to_string(),
                callback_info,
                req.conn_id,
                auto_reconnect,
            );
        }

        true
    }

    /// mysql_tick()
    /// Kept only for backwards compatibility — since rust-samp v3.0.0 the
    /// unified `on_tick` already dispatches callbacks automatically on both
    /// SA-MP and native Open Multiplayer.
    #[native(name = "mysql_tick")]
    pub fn mysql_tick(&mut self, _amx: &Amx) -> bool {
        self.process_pending_queries();
        true
    }

    /// mysql_escape_string(const src[], dest[], max_len = sizeof(dest))
    #[native(name = "mysql_escape_string")]
    pub fn mysql_escape_string(
        &mut self,
        _amx: &Amx,
        src: &AmxString,
        dest: UnsizedBuffer,
        dest_len: usize,
    ) -> AmxResult<bool> {
        let escaped = escape_string(src);
        dest.write_str(dest_len, &escaped)?;
        Ok(true)
    }

    /// mysql_format(connId, dest[], max_len, const format[], {Float,_}:...)
    ///
    /// Truncates the rendered string to `max_len - 1` characters (leaving room
    /// for the AMX NUL terminator) instead of aborting when the buffer is too
    /// small. A warning is logged once per call when truncation occurs.
    #[native(name = "mysql_format", raw)]
    pub fn mysql_format(&mut self, _amx: &Amx, mut args: Args) -> i32 {
        let Some(_conn_id) = args.next_arg::<i32>() else {
            return 0;
        };
        let Some(dest) = args.next_arg::<UnsizedBuffer>() else {
            return 0;
        };
        let Some(max_len) = args.next_arg::<usize>() else {
            return 0;
        };
        let Some(format_str) = args.next_arg::<AmxString>() else {
            return 0;
        };

        let fmt = format_str.to_string();
        let specs = parse_format(&fmt);
        let values = collect_format_values(&args, &specs, 4);
        let rendered = render_format(&specs, &values);

        if rendered.unknown_specs > 0 {
            Logger::warn(&format!(
                "mysql_format: {} unknown format specifier(s) in pattern.",
                rendered.unknown_specs
            ));
        }

        let (output, truncated) = truncate_to_buffer(&rendered.output, max_len);
        if truncated {
            Logger::warn(&format!(
                "mysql_format: output truncated to fit destination buffer ({} of {} bytes).",
                output.len(),
                rendered.output.len()
            ));
        }

        if let Err(err) = dest.write_str(max_len, output) {
            Logger::warn(&format!("mysql_format: write failed: {:?}", err));
            return 0;
        }
        i32::try_from(output.len()).unwrap_or(i32::MAX)
    }
}

/// Parses variadic callback parameters based on the format string.
/// Unknown format characters trigger a warning but are otherwise skipped.
pub fn parse_variadic_params(args: &mut Args, format: &str, start: usize) -> Vec<CallbackParam> {
    let mut params = Vec::new();
    let mut offset = start;
    let mut unknown = 0usize;

    for ch in format.chars() {
        match ch {
            'd' | 'i' => {
                let val: i32 = args.get::<Ref<i32>>(offset).map(|r| *r).unwrap_or(0);
                params.push(CallbackParam::Int(val));
                offset += 1;
            }
            'f' => {
                let val: f32 = args.get::<Ref<f32>>(offset).map(|r| *r).unwrap_or(0.0);
                params.push(CallbackParam::Float(val));
                offset += 1;
            }
            's' => {
                let val: String = args
                    .get::<AmxString>(offset)
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                params.push(CallbackParam::String(val));
                offset += 1;
            }
            _ => unknown += 1,
        }
    }

    if unknown > 0 {
        Logger::warn(&format!(
            "callback format string contains {} unknown specifier(s) — only 'd', 'i', 'f', 's' are recognized.",
            unknown
        ));
    }

    params
}

// ---------------------------------------------------------------------------
// Pure helpers (tested in #[cfg(test)] below).
// ---------------------------------------------------------------------------

/// A single token produced by [`parse_format`].
#[derive(Debug, Clone, PartialEq)]
pub enum FormatSpec {
    Literal(String),
    Int,
    Float,
    /// SQL-escaped string (`%s`, `%e`).
    EscapedStr,
    /// Raw string (`%r`) — inserted without escaping.
    RawStr,
    Percent,
    Unknown(char),
}

/// Value supplied for a single [`FormatSpec`] that needs one.
#[derive(Debug, Clone, PartialEq)]
pub enum FormatValue {
    Int(i32),
    Float(f32),
    Str(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormatResult {
    pub output: String,
    pub unknown_specs: usize,
}

pub fn parse_format(fmt: &str) -> Vec<FormatSpec> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut chars = fmt.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '%' {
            buf.push(ch);
            continue;
        }
        if !buf.is_empty() {
            out.push(FormatSpec::Literal(std::mem::take(&mut buf)));
        }
        match chars.next() {
            Some('d') | Some('i') => out.push(FormatSpec::Int),
            Some('f') => out.push(FormatSpec::Float),
            Some('s') | Some('e') => out.push(FormatSpec::EscapedStr),
            Some('r') => out.push(FormatSpec::RawStr),
            Some('%') => out.push(FormatSpec::Percent),
            Some(other) => out.push(FormatSpec::Unknown(other)),
            None => out.push(FormatSpec::Literal("%".to_string())),
        }
    }
    if !buf.is_empty() {
        out.push(FormatSpec::Literal(buf));
    }
    out
}

pub fn render_format(specs: &[FormatSpec], values: &[FormatValue]) -> FormatResult {
    let mut output = String::new();
    let mut values_iter = values.iter();
    let mut unknown = 0usize;

    for spec in specs {
        match spec {
            FormatSpec::Literal(text) => output.push_str(text),
            FormatSpec::Percent => output.push('%'),
            FormatSpec::Int => {
                if let Some(FormatValue::Int(v)) = values_iter.next() {
                    output.push_str(&v.to_string());
                }
            }
            FormatSpec::Float => {
                if let Some(FormatValue::Float(v)) = values_iter.next() {
                    output.push_str(&format!("{:.4}", v));
                }
            }
            FormatSpec::EscapedStr => {
                if let Some(FormatValue::Str(s)) = values_iter.next() {
                    output.push_str(&escape_string(s));
                }
            }
            FormatSpec::RawStr => {
                if let Some(FormatValue::Str(s)) = values_iter.next() {
                    output.push_str(s);
                }
            }
            FormatSpec::Unknown(ch) => {
                output.push('%');
                output.push(*ch);
                unknown += 1;
            }
        }
    }

    FormatResult {
        output,
        unknown_specs: unknown,
    }
}

/// Fetches one [`FormatValue`] per [`FormatSpec`] that requires a value,
/// reading positional args starting at `start_offset`.
fn collect_format_values(
    args: &Args,
    specs: &[FormatSpec],
    start_offset: usize,
) -> Vec<FormatValue> {
    let mut values = Vec::new();
    let mut offset = start_offset;

    for spec in specs {
        match spec {
            FormatSpec::Int => {
                let v: i32 = args.get::<Ref<i32>>(offset).map(|r| *r).unwrap_or(0);
                values.push(FormatValue::Int(v));
                offset += 1;
            }
            FormatSpec::Float => {
                let v: f32 = args.get::<Ref<f32>>(offset).map(|r| *r).unwrap_or(0.0);
                values.push(FormatValue::Float(v));
                offset += 1;
            }
            FormatSpec::EscapedStr | FormatSpec::RawStr => {
                let v: String = args
                    .get::<AmxString>(offset)
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                values.push(FormatValue::Str(v));
                offset += 1;
            }
            FormatSpec::Literal(_) | FormatSpec::Percent | FormatSpec::Unknown(_) => {}
        }
    }
    values
}

/// Truncates `s` so it fits in `max_len` (counting the AMX NUL terminator).
/// Returns the truncated slice and a flag indicating whether truncation
/// actually occurred. Respects UTF-8 char boundaries.
fn truncate_to_buffer(s: &str, max_len: usize) -> (&str, bool) {
    if max_len == 0 {
        return ("", !s.is_empty());
    }
    let cap = max_len.saturating_sub(1);
    if s.len() <= cap {
        return (s, false);
    }
    // Walk back to the previous char boundary so we don't slice through UTF-8.
    let mut end = cap;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    (&s[..end], true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_format_plain() {
        assert_eq!(
            parse_format("hello"),
            vec![FormatSpec::Literal("hello".into())]
        );
    }

    #[test]
    fn parse_format_mix() {
        let specs = parse_format("id=%d, name='%s', raw=%r, pct=%%, dunno=%z");
        assert_eq!(
            specs,
            vec![
                FormatSpec::Literal("id=".into()),
                FormatSpec::Int,
                FormatSpec::Literal(", name='".into()),
                FormatSpec::EscapedStr,
                FormatSpec::Literal("', raw=".into()),
                FormatSpec::RawStr,
                FormatSpec::Literal(", pct=".into()),
                FormatSpec::Percent,
                FormatSpec::Literal(", dunno=".into()),
                FormatSpec::Unknown('z'),
            ]
        );
    }

    #[test]
    fn parse_format_trailing_percent() {
        assert_eq!(
            parse_format("ends with %"),
            vec![
                FormatSpec::Literal("ends with ".into()),
                FormatSpec::Literal("%".into()),
            ]
        );
    }

    #[test]
    fn render_format_basic() {
        let specs = parse_format("INSERT INTO t (id, name) VALUES (%d, '%s')");
        let values = vec![FormatValue::Int(42), FormatValue::Str("O'Brien".into())];
        let r = render_format(&specs, &values);
        assert_eq!(
            r.output,
            "INSERT INTO t (id, name) VALUES (42, 'O\\'Brien')"
        );
        assert_eq!(r.unknown_specs, 0);
    }

    #[test]
    fn render_format_float_precision() {
        let specs = parse_format("v=%f");
        let r = render_format(&specs, &[FormatValue::Float(1.5)]);
        assert_eq!(r.output, "v=1.5000");
    }

    #[test]
    fn render_format_raw_string_not_escaped() {
        let specs = parse_format("raw=%r");
        let r = render_format(&specs, &[FormatValue::Str("a'b".into())]);
        assert_eq!(r.output, "raw=a'b");
    }

    #[test]
    fn render_format_escaped_string_is_escaped() {
        let specs = parse_format("v=%e");
        let r = render_format(&specs, &[FormatValue::Str("a'b".into())]);
        assert_eq!(r.output, "v=a\\'b");
    }

    #[test]
    fn render_format_percent_literal() {
        let specs = parse_format("100%%");
        let r = render_format(&specs, &[]);
        assert_eq!(r.output, "100%");
    }

    #[test]
    fn render_format_unknown_spec_counted() {
        let specs = parse_format("%z %q");
        let r = render_format(&specs, &[]);
        assert_eq!(r.unknown_specs, 2);
        assert_eq!(r.output, "%z %q");
    }

    #[test]
    fn render_format_missing_value_skipped() {
        let specs = parse_format("%d-%d");
        let r = render_format(&specs, &[FormatValue::Int(5)]);
        assert_eq!(r.output, "5-");
    }

    #[test]
    fn truncate_within_capacity() {
        let (s, t) = truncate_to_buffer("hello", 10);
        assert_eq!(s, "hello");
        assert!(!t);
    }

    #[test]
    fn truncate_oversized_ascii() {
        // max_len = 5 means 4 chars + NUL — output should be "hell".
        let (s, t) = truncate_to_buffer("hello world", 5);
        assert_eq!(s, "hell");
        assert!(t);
    }

    #[test]
    fn truncate_respects_utf8_boundary() {
        // "café" = 5 bytes (c=1, a=1, f=1, é=2). cap = 4 would split é → walks back to 3.
        let (s, t) = truncate_to_buffer("café", 5);
        assert_eq!(s, "caf");
        assert!(t);
    }

    #[test]
    fn truncate_zero_capacity() {
        let (s, t) = truncate_to_buffer("anything", 0);
        assert_eq!(s, "");
        assert!(t);
    }

    #[test]
    fn truncate_zero_capacity_empty_input() {
        let (s, t) = truncate_to_buffer("", 0);
        assert_eq!(s, "");
        assert!(!t);
    }
}
