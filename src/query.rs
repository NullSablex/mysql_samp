use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;

use mysql::Pool;

use crate::cache::CacheEntry;
use crate::connection::{
    QueryError, attempt_prepared, attempt_query, attempt_script, attempt_transaction,
};

/// Parameter type for callback invocation.
#[derive(Debug, Clone)]
pub enum CallbackParam {
    Int(i32),
    Float(f32),
    String(String),
}

/// Information needed to invoke a Pawn callback after query completion.
#[derive(Debug, Clone)]
pub struct CallbackInfo {
    pub name: String,
    pub format: String,
    pub params: Vec<CallbackParam>,
}

/// Result of a completed query (received from worker thread).
pub struct QueryResult {
    pub cache: CacheEntry,
    pub callback: Option<CallbackInfo>,
    pub conn_id: i32,
    pub error: Option<QueryError>,
    pub ordered: bool,
    pub sequence: u64,
}

/// Sequence threshold used to mark parallel-query results inside the same
/// `pending_ordered` map. Ordered queries use sequences starting at 0 and
/// growing by one; parallel results are stored at `u64::MAX` and counting
/// downward, so anything `>= PARALLEL_KEY_THRESHOLD` is a parallel entry.
const PARALLEL_KEY_THRESHOLD: u64 = u64::MAX / 2;

/// Manages threaded query execution and result collection.
pub struct QueryManager {
    sender: mpsc::Sender<QueryResult>,
    receiver: mpsc::Receiver<QueryResult>,
    next_sequence: u64,
    pending_ordered: BTreeMap<u64, QueryResult>,
    next_dispatch: u64,
    in_flight: Arc<AtomicU64>,
}

impl QueryManager {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            next_sequence: 0,
            pending_ordered: BTreeMap::new(),
            next_dispatch: 0,
            in_flight: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Returns the number of queries currently in-flight (submitted but not yet dispatched).
    pub fn pending_count(&self) -> u64 {
        self.in_flight.load(Ordering::Relaxed)
            + u64::try_from(self.pending_ordered.len()).unwrap_or(u64::MAX)
    }

    /// Submits an ordered query (FIFO — callbacks dispatched in submission order).
    pub fn submit_query(
        &mut self,
        pool: Pool,
        query: String,
        callback: Option<CallbackInfo>,
        conn_id: i32,
        auto_reconnect: bool,
    ) {
        let sequence = self.next_sequence;
        self.next_sequence += 1;
        let sender = self.sender.clone();
        self.in_flight.fetch_add(1, Ordering::Relaxed);
        let in_flight = self.in_flight.clone();

        thread::spawn(move || {
            let result = match attempt_query(&pool, &query, auto_reconnect) {
                Ok(cache) => QueryResult {
                    cache,
                    callback,
                    conn_id,
                    error: None,
                    ordered: true,
                    sequence,
                },
                Err(e) => QueryResult {
                    cache: CacheEntry::empty(query),
                    callback,
                    conn_id,
                    error: Some(e),
                    ordered: true,
                    sequence,
                },
            };
            let _ = sender.send(result);
            in_flight.fetch_sub(1, Ordering::Relaxed);
        });
    }

    /// Submits a prepared statement. Ordered like [`Self::submit_query`] — the
    /// values are bound server-side rather than pasted into the SQL text.
    pub fn submit_prepared(
        &mut self,
        pool: Pool,
        query: String,
        params: Vec<mysql::Value>,
        callback: Option<CallbackInfo>,
        conn_id: i32,
        auto_reconnect: bool,
    ) {
        let sequence = self.next_sequence;
        self.next_sequence += 1;
        let sender = self.sender.clone();
        self.in_flight.fetch_add(1, Ordering::Relaxed);
        let in_flight = self.in_flight.clone();

        thread::spawn(move || {
            let result = match attempt_prepared(&pool, &query, params, auto_reconnect) {
                Ok(cache) => QueryResult {
                    cache,
                    callback,
                    conn_id,
                    error: None,
                    ordered: true,
                    sequence,
                },
                Err(e) => QueryResult {
                    cache: CacheEntry::empty(query),
                    callback,
                    conn_id,
                    error: Some(e),
                    ordered: true,
                    sequence,
                },
            };
            let _ = sender.send(result);
            in_flight.fetch_sub(1, Ordering::Relaxed);
        });
    }

    /// Parallel counterpart of [`Self::submit_prepared`] — no ordering
    /// guarantee, dispatched as soon as it completes.
    pub fn submit_pprepared(
        &mut self,
        pool: Pool,
        query: String,
        params: Vec<mysql::Value>,
        callback: Option<CallbackInfo>,
        conn_id: i32,
        auto_reconnect: bool,
    ) {
        let sender = self.sender.clone();
        self.in_flight.fetch_add(1, Ordering::Relaxed);
        let in_flight = self.in_flight.clone();

        thread::spawn(move || {
            let result = match attempt_prepared(&pool, &query, params, auto_reconnect) {
                Ok(cache) => QueryResult {
                    cache,
                    callback,
                    conn_id,
                    error: None,
                    ordered: false,
                    sequence: 0,
                },
                Err(e) => QueryResult {
                    cache: CacheEntry::empty(query),
                    callback,
                    conn_id,
                    error: Some(e),
                    ordered: false,
                    sequence: 0,
                },
            };
            let _ = sender.send(result);
            in_flight.fetch_sub(1, Ordering::Relaxed);
        });
    }

    /// Submits a script (several statements from a file), ordered.
    pub fn submit_script(
        &mut self,
        pool: Pool,
        statements: Vec<String>,
        callback: Option<CallbackInfo>,
        conn_id: i32,
    ) {
        let sequence = self.next_sequence;
        self.next_sequence += 1;
        let sender = self.sender.clone();
        self.in_flight.fetch_add(1, Ordering::Relaxed);
        let in_flight = self.in_flight.clone();

        thread::spawn(move || {
            let result = match attempt_script(&pool, &statements) {
                Ok(cache) => QueryResult {
                    cache,
                    callback,
                    conn_id,
                    error: None,
                    ordered: true,
                    sequence,
                },
                Err(e) => QueryResult {
                    cache: CacheEntry::empty(String::new()),
                    callback,
                    conn_id,
                    error: Some(e),
                    ordered: true,
                    sequence,
                },
            };
            let _ = sender.send(result);
            in_flight.fetch_sub(1, Ordering::Relaxed);
        });
    }

    /// Submits a transaction batch. Ordered, and never auto-reconnects — see
    /// [`attempt_transaction`].
    pub fn submit_transaction(
        &mut self,
        pool: Pool,
        steps: Vec<crate::transaction::TxStep>,
        callback: Option<CallbackInfo>,
        conn_id: i32,
    ) {
        let sequence = self.next_sequence;
        self.next_sequence += 1;
        let sender = self.sender.clone();
        self.in_flight.fetch_add(1, Ordering::Relaxed);
        let in_flight = self.in_flight.clone();

        thread::spawn(move || {
            let result = match attempt_transaction(&pool, &steps) {
                Ok(cache) => QueryResult {
                    cache,
                    callback,
                    conn_id,
                    error: None,
                    ordered: true,
                    sequence,
                },
                Err(e) => QueryResult {
                    cache: CacheEntry::empty(String::from("TRANSACTION")),
                    callback,
                    conn_id,
                    error: Some(e),
                    ordered: true,
                    sequence,
                },
            };
            let _ = sender.send(result);
            in_flight.fetch_sub(1, Ordering::Relaxed);
        });
    }

    /// Submits a parallel query (no order guarantee — dispatched as soon as complete).
    pub fn submit_pquery(
        &mut self,
        pool: Pool,
        query: String,
        callback: Option<CallbackInfo>,
        conn_id: i32,
        auto_reconnect: bool,
    ) {
        let sender = self.sender.clone();
        self.in_flight.fetch_add(1, Ordering::Relaxed);
        let in_flight = self.in_flight.clone();

        thread::spawn(move || {
            let result = match attempt_query(&pool, &query, auto_reconnect) {
                Ok(cache) => QueryResult {
                    cache,
                    callback,
                    conn_id,
                    error: None,
                    ordered: false,
                    sequence: 0,
                },
                Err(e) => QueryResult {
                    cache: CacheEntry::empty(query),
                    callback,
                    conn_id,
                    error: Some(e),
                    ordered: false,
                    sequence: 0,
                },
            };
            let _ = sender.send(result);
            in_flight.fetch_sub(1, Ordering::Relaxed);
        });
    }

    /// Collects completed results and returns them in correct dispatch order.
    /// Ordered queries are buffered and dispatched in FIFO sequence.
    /// Parallel queries are dispatched immediately.
    pub fn poll_results(&mut self) -> Vec<QueryResult> {
        // Drain all available results from the channel
        while let Ok(result) = self.receiver.try_recv() {
            if result.ordered {
                self.pending_ordered.insert(result.sequence, result);
            } else {
                // Parallel queries go to pending with a special high sequence
                // so they are dispatched after ordered ones in this tick
                let key = u64::MAX - u64::try_from(self.pending_ordered.len()).unwrap_or(u64::MAX);
                self.pending_ordered.insert(key, result);
            }
        }

        let mut ready = Vec::new();

        // Dispatch ordered results in sequence
        while let Some(entry) = self.pending_ordered.remove(&self.next_dispatch) {
            self.next_dispatch += 1;
            ready.push(entry);
        }

        // Dispatch any parallel results (high sequence keys)
        let parallel_keys: Vec<u64> = self
            .pending_ordered
            .keys()
            .filter(|k| **k >= PARALLEL_KEY_THRESHOLD)
            .copied()
            .collect();

        for key in parallel_keys {
            if let Some(result) = self.pending_ordered.remove(&key) {
                ready.push(result);
            }
        }

        ready
    }

    /// Returns a clone of the sender for testing (inject results without threads).
    #[cfg(test)]
    pub fn test_sender(&self) -> mpsc::Sender<QueryResult> {
        self.sender.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_result(sequence: u64, ordered: bool) -> QueryResult {
        QueryResult {
            cache: CacheEntry::empty(format!("query_{}", sequence)),
            callback: None,
            conn_id: 1,
            error: None,
            ordered,
            sequence,
        }
    }

    #[test]
    fn new_manager_is_empty() {
        let mgr = QueryManager::new();
        assert_eq!(mgr.pending_count(), 0);
    }

    #[test]
    fn poll_results_empty() {
        let mut mgr = QueryManager::new();
        let results = mgr.poll_results();
        assert!(results.is_empty());
    }

    #[test]
    fn ordered_results_fifo() {
        let mut mgr = QueryManager::new();
        let sender = mgr.test_sender();

        // Send results out of order: 2, 0, 1
        sender.send(empty_result(2, true)).unwrap();
        sender.send(empty_result(0, true)).unwrap();
        sender.send(empty_result(1, true)).unwrap();

        let results = mgr.poll_results();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].sequence, 0);
        assert_eq!(results[1].sequence, 1);
        assert_eq!(results[2].sequence, 2);
    }

    #[test]
    fn ordered_results_partial_dispatch() {
        let mut mgr = QueryManager::new();
        let sender = mgr.test_sender();

        // Send sequence 1 but not 0 — should block dispatch
        sender.send(empty_result(1, true)).unwrap();

        let results = mgr.poll_results();
        assert!(results.is_empty()); // sequence 0 is missing

        // Now send sequence 0
        sender.send(empty_result(0, true)).unwrap();

        let results = mgr.poll_results();
        assert_eq!(results.len(), 2); // 0 and 1 dispatched
        assert_eq!(results[0].sequence, 0);
        assert_eq!(results[1].sequence, 1);
    }

    #[test]
    fn parallel_results_dispatch_immediately() {
        let mut mgr = QueryManager::new();
        let sender = mgr.test_sender();

        sender.send(empty_result(0, false)).unwrap();
        sender.send(empty_result(0, false)).unwrap();

        let results = mgr.poll_results();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn mixed_ordered_and_parallel() {
        let mut mgr = QueryManager::new();
        let sender = mgr.test_sender();

        // Ordered sequence 0
        sender.send(empty_result(0, true)).unwrap();
        // Parallel (no order)
        sender.send(empty_result(0, false)).unwrap();

        let results = mgr.poll_results();
        assert_eq!(results.len(), 2);
        // First should be ordered (sequence 0), second parallel
        assert!(results[0].ordered);
        assert!(!results[1].ordered);
    }

    #[test]
    fn callback_info_construction() {
        let info = CallbackInfo {
            name: "OnData".to_string(),
            format: "dis".to_string(),
            params: vec![
                CallbackParam::Int(42),
                CallbackParam::Int(1),
                CallbackParam::String("test".to_string()),
            ],
        };
        assert_eq!(info.name, "OnData");
        assert_eq!(info.format, "dis");
        assert_eq!(info.params.len(), 3);
    }
}
