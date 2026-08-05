use samp::args::Args;
use samp::native;
use samp::prelude::*;

use crate::error::{ErrorState, MysqlError};
use crate::logger::Logger;
use crate::natives::query::parse_variadic_params;
use crate::plugin::MysqlPlugin;
use crate::query::CallbackInfo;
use crate::transaction::TxStep;

impl MysqlPlugin {
    /// mysql_transaction_new(connId)
    #[native(name = "mysql_transaction_new")]
    pub fn mysql_transaction_new(&mut self, amx: &Amx, conn_id: i32) -> i32 {
        if !self.connections.exists(conn_id) {
            Logger::warn("mysql_transaction_new failed: invalid connection ID.");
            self.connections.global_error = ErrorState::new(
                MysqlError::InvalidConnection,
                "mysql_transaction_new failed: invalid connection ID.",
            );
            return 0;
        }

        self.transactions.create(conn_id, amx.ident())
    }

    /// mysql_transaction_destroy(txId) — discards a transaction that was never executed.
    #[native(name = "mysql_transaction_destroy")]
    pub fn mysql_transaction_destroy(&mut self, _amx: &Amx, tx_id: i32) -> bool {
        self.transactions.destroy(tx_id)
    }

    /// mysql_transaction_add(txId, const query[]) — appends a plain SQL step.
    #[native(name = "mysql_transaction_add")]
    pub fn mysql_transaction_add(&mut self, _amx: &Amx, tx_id: i32, query: &AmxString) -> bool {
        self.transactions.add_step(
            tx_id,
            TxStep {
                query: query.to_string(),
                params: Vec::new(),
            },
        )
    }

    /// mysql_transaction_add_stmt(txId, stmtId)
    ///
    /// Appends a prepared statement, copying its query and currently bound
    /// values. The statement itself is untouched and can be reset and reused.
    #[native(name = "mysql_transaction_add_stmt")]
    pub fn mysql_transaction_add_stmt(&mut self, _amx: &Amx, tx_id: i32, stmt_id: i32) -> bool {
        let Some(stmt) = self.stmts.get(stmt_id) else {
            Logger::warn("mysql_transaction_add_stmt failed: invalid statement ID.");
            return false;
        };

        let expected = stmt.placeholder_count();
        if stmt.params.len() != expected {
            Logger::warn(&format!(
                "mysql_transaction_add_stmt failed: statement has {expected} placeholder(s) but {} value(s) were bound.",
                stmt.params.len()
            ));
            return false;
        }

        let step = TxStep {
            query: stmt.query.clone(),
            params: stmt.params.clone(),
        };
        self.transactions.add_step(tx_id, step)
    }

    /// mysql_transaction_execute(txId, const callback[] = "", const format[] = "", {Float,_}:...)
    ///
    /// Runs every step atomically on one connection and destroys the
    /// transaction. The callback receives the cache of the last step; a failure
    /// rolls everything back and fires `OnQueryError`.
    #[native(name = "mysql_transaction_execute", raw)]
    pub fn mysql_transaction_execute(&mut self, _amx: &Amx, mut args: Args) -> bool {
        let Some(tx_id) = args.next_arg::<i32>() else {
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

        let Some(tx) = self.transactions.get(tx_id) else {
            Logger::warn("mysql_transaction_execute failed: invalid transaction ID.");
            return false;
        };

        if tx.steps.is_empty() {
            Logger::warn("mysql_transaction_execute failed: transaction has no steps.");
            return false;
        }

        let conn_id = tx.conn_id;
        let Some(pool) = self.connections.get_pool(conn_id) else {
            Logger::warn("mysql_transaction_execute failed: invalid connection ID.");
            self.connections.global_error = ErrorState::new(
                MysqlError::InvalidConnection,
                "mysql_transaction_execute failed: invalid connection ID.",
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

        // Ownership moves to the worker; the handle is consumed by execution so
        // a transaction cannot be accidentally run twice.
        let Some(tx) = self.transactions.take(tx_id) else {
            return false;
        };

        self.queries
            .submit_transaction(pool, tx.steps, callback_info, conn_id);
        true
    }
}
