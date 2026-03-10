use samp::args::Args;
use samp::cell::Ref;
use samp::native;
use samp::prelude::*;

use crate::connection::escape_string;
use crate::error::{ErrorState, MysqlError};
use crate::logger::Logger;
use crate::plugin::MysqlPlugin;
use crate::query::{CallbackInfo, CallbackParam};

impl MysqlPlugin {
    /// mysql_query(connId, const query[], const callback[] = "", const format[] = "", {Float,_}:...)
    /// Non-blocking threaded query with FIFO ordering.
    #[native(name = "mysql_query", raw)]
    pub fn mysql_query(&mut self, _amx: &Amx, mut args: Args) -> AmxResult<bool> {
        let conn_id: i32 = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };
        let query_str: AmxString = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };
        let callback_str: AmxString = match args.next_arg() {
            Some(v) => v,
            None => {
                // callback is optional, default to empty
                return self.submit_query(conn_id, &query_str.to_string(), "", "", &mut args, 3, true);
            }
        };
        let format_str: AmxString = match args.next_arg() {
            Some(v) => v,
            None => {
                return self.submit_query(
                    conn_id,
                    &query_str.to_string(),
                    &callback_str.to_string(),
                    "",
                    &mut args,
                    3,
                    true,
                );
            }
        };

        self.submit_query(
            conn_id,
            &query_str.to_string(),
            &callback_str.to_string(),
            &format_str.to_string(),
            &mut args,
            4,
            true,
        )
    }

    /// mysql_pquery(connId, const query[], const callback[] = "", const format[] = "", {Float,_}:...)
    /// Non-blocking parallel query (no order guarantee).
    #[native(name = "mysql_pquery", raw)]
    pub fn mysql_pquery(&mut self, _amx: &Amx, mut args: Args) -> AmxResult<bool> {
        let conn_id: i32 = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };
        let query_str: AmxString = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };
        let callback_str: AmxString = match args.next_arg() {
            Some(v) => v,
            None => {
                return self.submit_query(conn_id, &query_str.to_string(), "", "", &mut args, 3, false);
            }
        };
        let format_str: AmxString = match args.next_arg() {
            Some(v) => v,
            None => {
                return self.submit_query(
                    conn_id,
                    &query_str.to_string(),
                    &callback_str.to_string(),
                    "",
                    &mut args,
                    3,
                    false,
                );
            }
        };

        self.submit_query(
            conn_id,
            &query_str.to_string(),
            &callback_str.to_string(),
            &format_str.to_string(),
            &mut args,
            4,
            false,
        )
    }

    /// Internal: submits a query (ordered or parallel).
    #[allow(clippy::too_many_arguments)]
    fn submit_query(
        &mut self,
        conn_id: i32,
        query: &str,
        callback: &str,
        format: &str,
        args: &mut Args,
        variadic_start: usize,
        ordered: bool,
    ) -> AmxResult<bool> {
        let pool = match self.connections.get_pool(conn_id) {
            Some(p) => p,
            None => {
                Logger::warn("Query failed: invalid connection ID.");
                self.connections.global_error = ErrorState::new(
                    MysqlError::InvalidConnection,
                    "Query failed: invalid connection ID.",
                );
                return Ok(false);
            }
        };

        let auto_reconnect = self.connections.get_auto_reconnect(conn_id);

        let callback_info = if callback.is_empty() {
            None
        } else {
            let params = parse_variadic_params(args, format, variadic_start);
            Some(CallbackInfo {
                name: callback.to_string(),
                format: format.to_string(),
                params,
            })
        };

        if ordered {
            self.queries
                .submit_query(pool, query.to_string(), callback_info, conn_id, auto_reconnect);
        } else {
            self.queries
                .submit_pquery(pool, query.to_string(), callback_info, conn_id, auto_reconnect);
        }

        Ok(true)
    }

    /// mysql_escape_string(const src[], dest[], max_len = sizeof(dest))
    #[native(name = "mysql_escape_string")]
    pub fn mysql_escape_string(
        &mut self,
        _amx: &Amx,
        src: AmxString,
        dest: UnsizedBuffer,
        dest_len: usize,
    ) -> AmxResult<bool> {
        let escaped = escape_string(&src.to_string());
        let mut buf = dest.into_sized_buffer(dest_len);
        let _ = samp::cell::string::put_in_buffer(&mut buf, &escaped);
        Ok(true)
    }

    /// mysql_format(connId, dest[], max_len, const format[], {Float,_}:...)
    #[native(name = "mysql_format", raw)]
    pub fn mysql_format(&mut self, _amx: &Amx, mut args: Args) -> AmxResult<i32> {
        let _conn_id: i32 = match args.next_arg() {
            Some(v) => v,
            None => return Ok(0),
        };
        let dest: UnsizedBuffer = match args.next_arg() {
            Some(v) => v,
            None => return Ok(0),
        };
        let max_len: usize = match args.next_arg() {
            Some(v) => v,
            None => return Ok(0),
        };
        let format_str: AmxString = match args.next_arg() {
            Some(v) => v,
            None => return Ok(0),
        };

        let fmt = format_str.to_string();
        let mut output = String::new();
        let mut param_offset = 4; // variadic params start at index 4

        let mut chars = fmt.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '%' {
                match chars.next() {
                    Some('d') | Some('i') => {
                        let val: i32 = args.get::<Ref<i32>>(param_offset).map(|r| *r).unwrap_or(0);
                        output.push_str(&val.to_string());
                        param_offset += 1;
                    }
                    Some('f') => {
                        let val: f32 = args.get::<Ref<f32>>(param_offset).map(|r| *r).unwrap_or(0.0);
                        output.push_str(&format!("{:.4}", val));
                        param_offset += 1;
                    }
                    Some('s') | Some('e') => {
                        // %s and %e both escape strings (safe by default)
                        let val: AmxString = match args.get(param_offset) {
                            Some(v) => v,
                            None => {
                                output.push_str("");
                                param_offset += 1;
                                continue;
                            }
                        };
                        output.push_str(&escape_string(&val.to_string()));
                        param_offset += 1;
                    }
                    Some('r') => {
                        // %r = raw string (no escaping — use only for trusted values)
                        let val: AmxString = match args.get(param_offset) {
                            Some(v) => v,
                            None => {
                                output.push_str("");
                                param_offset += 1;
                                continue;
                            }
                        };
                        output.push_str(&val.to_string());
                        param_offset += 1;
                    }
                    Some('%') => output.push('%'),
                    Some(other) => {
                        output.push('%');
                        output.push(other);
                    }
                    None => output.push('%'),
                }
            } else {
                output.push(ch);
            }
        }

        let mut buf = dest.into_sized_buffer(max_len);
        let _ = samp::cell::string::put_in_buffer(&mut buf, &output);
        Ok(output.len() as i32)
    }
}

/// Parses variadic callback parameters based on the format string.
pub fn parse_variadic_params(args: &mut Args, format: &str, start: usize) -> Vec<CallbackParam> {
    let mut params = Vec::new();
    let mut offset = start;

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
                let val: AmxString = match args.get(offset) {
                    Some(v) => v,
                    None => {
                        params.push(CallbackParam::String(String::new()));
                        offset += 1;
                        continue;
                    }
                };
                params.push(CallbackParam::String(val.to_string()));
                offset += 1;
            }
            _ => {}
        }
    }

    params
}
