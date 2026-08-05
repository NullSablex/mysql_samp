use mysql::Value;
use samp::args::Args;
use samp::native;
use samp::prelude::*;

use crate::error::{ErrorState, MysqlError};
use crate::logger::Logger;
use crate::natives::query::parse_variadic_params;
use crate::plugin::MysqlPlugin;
use crate::query::CallbackInfo;

impl MysqlPlugin {
    /// mysql_stmt_new(connId, const query[])
    ///
    /// `query` uses `?` placeholders. Returns a statement ID, or 0 on failure.
    #[native(name = "mysql_stmt_new")]
    pub fn mysql_stmt_new(&mut self, amx: &Amx, conn_id: i32, query: &AmxString) -> i32 {
        if !self.connections.exists(conn_id) {
            Logger::warn("mysql_stmt_new failed: invalid connection ID.");
            self.connections.global_error = ErrorState::new(
                MysqlError::InvalidConnection,
                "mysql_stmt_new failed: invalid connection ID.",
            );
            return 0;
        }

        self.stmts.create(conn_id, amx.ident(), query.to_string())
    }

    /// mysql_stmt_close(stmtId)
    #[native(name = "mysql_stmt_close")]
    pub fn mysql_stmt_close(&mut self, _amx: &Amx, stmt_id: i32) -> bool {
        self.stmts.destroy(stmt_id)
    }

    /// mysql_stmt_reset(stmtId) — drops bound values, keeps the statement.
    #[native(name = "mysql_stmt_reset")]
    pub fn mysql_stmt_reset(&mut self, _amx: &Amx, stmt_id: i32) -> bool {
        self.stmts.reset(stmt_id)
    }

    /// mysql_stmt_bind_int(stmtId, value)
    #[native(name = "mysql_stmt_bind_int")]
    pub fn mysql_stmt_bind_int(&mut self, _amx: &Amx, stmt_id: i32, value: i32) -> bool {
        self.stmts.bind(stmt_id, Value::Int(i64::from(value)))
    }

    /// mysql_stmt_bind_float(stmtId, Float:value)
    #[native(name = "mysql_stmt_bind_float")]
    pub fn mysql_stmt_bind_float(&mut self, _amx: &Amx, stmt_id: i32, value: f32) -> bool {
        self.stmts.bind(stmt_id, Value::Float(value))
    }

    /// mysql_stmt_bind_str(stmtId, const value[])
    #[native(name = "mysql_stmt_bind_str")]
    pub fn mysql_stmt_bind_str(&mut self, _amx: &Amx, stmt_id: i32, value: &AmxString) -> bool {
        self.stmts
            .bind(stmt_id, Value::Bytes(value.to_string().into_bytes()))
    }

    /// mysql_stmt_bind_null(stmtId)
    #[native(name = "mysql_stmt_bind_null")]
    pub fn mysql_stmt_bind_null(&mut self, _amx: &Amx, stmt_id: i32) -> bool {
        self.stmts.bind(stmt_id, Value::NULL)
    }

    /// mysql_stmt_execute(stmtId, const callback[] = "", const format[] = "", {Float,_}:...)
    ///
    /// Non-blocking, FIFO-ordered like `mysql_query`. The result reaches the
    /// callback through the usual cache stack.
    #[native(name = "mysql_stmt_execute", raw)]
    pub fn mysql_stmt_execute(&mut self, _amx: &Amx, mut args: Args) -> bool {
        let Some(stmt_id) = args.next_arg::<i32>() else {
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

        let Some(stmt) = self.stmts.get(stmt_id) else {
            Logger::warn("mysql_stmt_execute failed: invalid statement ID.");
            return false;
        };

        // Catching the arity mismatch here turns a confusing server-side error
        // into a precise one, and names both numbers.
        let expected = stmt.placeholder_count();
        if stmt.params.len() != expected {
            let msg = format!(
                "mysql_stmt_execute failed: statement has {expected} placeholder(s) but {} value(s) were bound.",
                stmt.params.len()
            );
            Logger::warn(&msg);
            self.connections.global_error = ErrorState::new(MysqlError::QueryFailed, msg);
            return false;
        }

        let query = stmt.query.clone();
        let params = stmt.params.clone();
        let conn_id = stmt.conn_id;

        let Some(pool) = self.connections.get_pool(conn_id) else {
            Logger::warn("mysql_stmt_execute failed: invalid connection ID.");
            self.connections.global_error = ErrorState::new(
                MysqlError::InvalidConnection,
                "mysql_stmt_execute failed: invalid connection ID.",
            );
            return false;
        };

        let callback_info = if callback.is_empty() {
            None
        } else {
            let params = parse_variadic_params(&mut args, &format, 3);
            Some(CallbackInfo {
                name: callback,
                format,
                params,
            })
        };

        self.queries.submit_prepared(
            pool,
            query,
            params,
            callback_info,
            conn_id,
            self.connections.get_auto_reconnect(conn_id),
        );
        true
    }
}
