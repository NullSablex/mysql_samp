//! Transactions, as an all-or-nothing batch.
//!
//! A transaction must run every statement on the *same* connection, but
//! connections here come from a pool and the plugin is non-blocking: an
//! interactive `begin` / … / `commit` API would have to hold a pooled
//! connection across ticks, and a gamemode that never reaches `commit` (an
//! early `return`, a runtime error, a disconnect) would leak that connection
//! for the lifetime of the server.
//!
//! So a transaction is built up first and executed as one unit on one worker
//! thread: `START TRANSACTION`, every step, `COMMIT`. Any failing step rolls
//! the whole thing back. Nothing is held between ticks and nothing can leak.

use std::collections::HashMap;

use mysql::Value;
use samp::amx::AmxIdent;

/// Upper bound on steps in a single transaction — a sanity ceiling against a
/// runaway loop in Pawn, not a MySQL limit.
pub const MAX_TX_STEPS: usize = 1024;

/// One statement inside a transaction. `params` is empty for plain SQL and
/// populated when the step came from a prepared statement.
pub struct TxStep {
    pub query: String,
    pub params: Vec<Value>,
}

pub struct Transaction {
    pub conn_id: i32,
    /// AMX that created the batch, so an unloaded script does not leave its
    /// half-built transactions behind.
    pub amx_ident: AmxIdent,
    pub steps: Vec<TxStep>,
}

pub struct TransactionManager {
    transactions: HashMap<i32, Transaction>,
    next_id: i32,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create(&mut self, conn_id: i32, amx_ident: AmxIdent) -> i32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        self.transactions.insert(
            id,
            Transaction {
                conn_id,
                amx_ident,
                steps: Vec::new(),
            },
        );
        id
    }

    /// Drops every transaction owned by an unloaded AMX.
    pub fn destroy_by_amx(&mut self, ident: AmxIdent) {
        self.transactions.retain(|_, tx| tx.amx_ident != ident);
    }

    /// Drops every transaction bound to a closed connection.
    pub fn destroy_by_conn(&mut self, conn_id: i32) {
        self.transactions.retain(|_, tx| tx.conn_id != conn_id);
    }

    pub fn get(&self, id: i32) -> Option<&Transaction> {
        self.transactions.get(&id)
    }

    pub fn destroy(&mut self, id: i32) -> bool {
        self.transactions.remove(&id).is_some()
    }

    /// Removes and returns a transaction. Execution consumes the handle so the
    /// same batch cannot be submitted twice by mistake.
    pub fn take(&mut self, id: i32) -> Option<Transaction> {
        self.transactions.remove(&id)
    }

    pub fn add_step(&mut self, id: i32, step: TxStep) -> bool {
        let Some(tx) = self.transactions.get_mut(&id) else {
            return false;
        };
        if tx.steps.len() >= MAX_TX_STEPS {
            return false;
        }
        tx.steps.push(step);
        true
    }
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

    fn plain(q: &str) -> TxStep {
        TxStep {
            query: q.to_string(),
            params: Vec::new(),
        }
    }

    #[test]
    fn steps_are_kept_in_submission_order() {
        let mut mgr = TransactionManager::new();
        let id = mgr.create(1, dummy_ident());

        assert!(mgr.add_step(id, plain("UPDATE a SET x = 1")));
        assert!(mgr.add_step(id, plain("UPDATE b SET y = 2")));

        let steps = &mgr.get(id).expect("exists").steps;
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].query, "UPDATE a SET x = 1");
        assert_eq!(steps[1].query, "UPDATE b SET y = 2");
    }

    #[test]
    fn add_step_respects_the_ceiling() {
        let mut mgr = TransactionManager::new();
        let id = mgr.create(1, dummy_ident());

        for _ in 0..MAX_TX_STEPS {
            assert!(mgr.add_step(id, plain("SELECT 1")));
        }
        assert!(!mgr.add_step(id, plain("SELECT 1")), "ceiling must hold");
    }

    #[test]
    fn operations_on_unknown_ids_fail() {
        let mut mgr = TransactionManager::new();
        assert!(!mgr.add_step(999, plain("SELECT 1")));
        assert!(!mgr.destroy(999));
        assert!(mgr.get(999).is_none());
    }

    #[test]
    fn destroy_by_amx_only_touches_that_script() {
        let mut mgr = TransactionManager::new();
        let mine = mgr.create(1, dummy_ident());
        let other = mgr.create(1, dummy_ident_2());

        mgr.destroy_by_amx(dummy_ident());

        assert!(mgr.get(mine).is_none(), "unloaded script's handle must go");
        assert!(mgr.get(other).is_some(), "other script must be untouched");
    }

    #[test]
    fn destroy_by_conn_only_touches_that_connection() {
        let mut mgr = TransactionManager::new();
        let on_one = mgr.create(1, dummy_ident());
        let on_two = mgr.create(2, dummy_ident());

        mgr.destroy_by_conn(1);

        assert!(mgr.get(on_one).is_none());
        assert!(mgr.get(on_two).is_some());
    }

    #[test]
    fn destroy_removes_the_transaction() {
        let mut mgr = TransactionManager::new();
        let id = mgr.create(1, dummy_ident());
        assert!(mgr.destroy(id));
        assert!(mgr.get(id).is_none());
    }

    #[test]
    fn steps_carry_bound_params() {
        let mut mgr = TransactionManager::new();
        let id = mgr.create(1, dummy_ident());

        assert!(mgr.add_step(
            id,
            TxStep {
                query: "UPDATE a SET x = ? WHERE id = ?".into(),
                params: vec![Value::Int(5), Value::Int(1)],
            }
        ));

        assert_eq!(mgr.get(id).expect("exists").steps[0].params.len(), 2);
    }
}
