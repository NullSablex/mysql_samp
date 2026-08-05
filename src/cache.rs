use std::collections::HashMap;

use crate::logger::Logger;

/// Maximum number of saved caches allowed to prevent memory exhaustion.
const MAX_SAVED_CACHES: usize = 1024;

/// A single row: Vec of Option<String> where None represents SQL NULL.
pub type CacheRow = Vec<Option<String>>;

/// One result set: rows plus the column metadata that describes them.
///
/// A single `mysql_query` normally produces one of these. A script run through
/// `mysql_query_file`, or a `CALL` to a stored procedure, produces several —
/// hence the vector in [`CacheEntry`].
#[derive(Clone, Default)]
pub struct ResultSet {
    pub rows: Vec<CacheRow>,
    pub field_names: Vec<String>,
    pub field_types: Vec<u8>,
}

/// Stores the result of a query execution.
///
/// Every reader below (`row_count`, `field_name`, `get_value`, …) reports on
/// the **selected** result set, which is the first one until `set_result`
/// changes it. Queries that return a single set therefore behave exactly as
/// they always did.
pub struct CacheEntry {
    results: Vec<ResultSet>,
    active_result: usize,
    affected_rows: u64,
    insert_id: u64,
    warning_count: u16,
    exec_time_us: u128,
    query_string: String,
}

impl CacheEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn with_results(
        results: Vec<ResultSet>,
        affected_rows: u64,
        insert_id: u64,
        warning_count: u16,
        exec_time_us: u128,
        query_string: String,
    ) -> Self {
        Self {
            // An empty vector would make every reader a special case; one empty
            // set keeps them uniform.
            results: if results.is_empty() {
                vec![ResultSet::default()]
            } else {
                results
            },
            active_result: 0,
            affected_rows,
            insert_id,
            warning_count,
            exec_time_us,
            query_string,
        }
    }

    pub fn empty(query_string: String) -> Self {
        Self::with_results(Vec::new(), 0, 0, 0, 0, query_string)
    }

    /// How many result sets the query produced. Always at least 1.
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Selects which result set the readers report on. Out-of-range indices are
    /// rejected and leave the selection untouched.
    pub fn set_result(&mut self, index: usize) -> bool {
        if index >= self.results.len() {
            return false;
        }
        self.active_result = index;
        true
    }

    /// The selected set. The index is kept in range by `set_result`, and the
    /// vector is never empty, so this always resolves.
    fn active(&self) -> &ResultSet {
        &self.results[self.active_result]
    }

    /// Deep copy, used by `cache_save`. Copies **every** result set, not just
    /// the selected one — saving a cache and then switching result would
    /// otherwise silently lose data.
    pub fn duplicate(&self) -> Self {
        Self {
            results: self.results.clone(),
            active_result: self.active_result,
            affected_rows: self.affected_rows,
            insert_id: self.insert_id,
            warning_count: self.warning_count,
            exec_time_us: self.exec_time_us,
            query_string: self.query_string.clone(),
        }
    }

    pub fn row_count(&self) -> usize {
        self.active().rows.len()
    }

    pub fn field_count(&self) -> usize {
        self.active().field_names.len()
    }

    pub fn field_name(&self, index: usize) -> Option<&str> {
        self.active().field_names.get(index).map(|s| s.as_str())
    }

    pub fn field_index(&self, name: &str) -> Option<usize> {
        self.active()
            .field_names
            .iter()
            .position(|f| f.eq_ignore_ascii_case(name))
    }

    pub fn get_value(&self, row: usize, col: usize) -> Option<&Option<String>> {
        self.active().rows.get(row).and_then(|r| r.get(col))
    }

    pub fn affected_rows(&self) -> u64 {
        self.affected_rows
    }

    pub fn insert_id(&self) -> u64 {
        self.insert_id
    }

    pub fn exec_time_ms(&self) -> u128 {
        self.exec_time_us / 1000
    }

    pub fn query_string(&self) -> &str {
        &self.query_string
    }

    pub fn warning_count(&self) -> u16 {
        self.warning_count
    }

    pub fn field_type(&self, index: usize) -> Option<u8> {
        self.active().field_types.get(index).copied()
    }
}

/// Manages the active cache stack and saved caches.
///
/// The cache system works as a stack: executing a query pushes a CacheEntry,
/// and completing a callback pops it. Saved caches persist independently.
pub struct CacheManager {
    active_stack: Vec<CacheEntry>,
    saved: HashMap<i32, CacheEntry>,
    next_saved_id: i32,
    manual_active: Option<i32>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            active_stack: Vec::new(),
            saved: HashMap::new(),
            next_saved_id: 1,
            manual_active: None,
        }
    }

    /// Pushes a cache entry onto the active stack.
    pub fn push_active(&mut self, entry: CacheEntry) {
        self.active_stack.push(entry);
    }

    /// Pops the top cache entry from the active stack.
    pub fn pop_active(&mut self) -> Option<CacheEntry> {
        self.active_stack.pop()
    }

    /// Returns a reference to the currently active cache.
    /// If a manual cache is set via `set_active`, returns that.
    /// Otherwise returns the top of the stack.
    /// Mutable view of the active cache, needed by `cache_set_result`.
    pub fn get_active_mut(&mut self) -> Option<&mut CacheEntry> {
        if let Some(id) = self.manual_active {
            return self.saved.get_mut(&id);
        }
        self.active_stack.last_mut()
    }

    pub fn get_active(&self) -> Option<&CacheEntry> {
        if let Some(id) = self.manual_active {
            return self.saved.get(&id);
        }
        self.active_stack.last()
    }

    /// Clones the current active cache into saved storage.
    /// Returns the saved cache ID, or 0 if no active cache or limit reached.
    pub fn save(&mut self) -> i32 {
        if self.saved.len() >= MAX_SAVED_CACHES {
            Logger::warn("cache_save failed: maximum saved caches reached (1024).");
            return 0;
        }

        // Clone data from active cache to avoid borrow conflict
        let cloned = {
            let active = match self.get_active() {
                Some(a) => a,
                None => return 0,
            };
            active.duplicate()
        };

        let id = self.next_saved_id;
        self.next_saved_id = self.next_saved_id.wrapping_add(1).max(1);
        self.saved.insert(id, cloned);
        id
    }

    /// Deletes a saved cache by ID.
    pub fn delete(&mut self, id: i32) -> bool {
        if self.manual_active == Some(id) {
            self.manual_active = None;
        }
        self.saved.remove(&id).is_some()
    }

    /// Manually activates a saved cache (overrides stack top).
    pub fn set_active(&mut self, id: i32) -> bool {
        if self.saved.contains_key(&id) {
            self.manual_active = Some(id);
            true
        } else {
            false
        }
    }

    /// Deactivates the manually set cache.
    pub fn unset_active(&mut self) -> bool {
        if self.manual_active.is_some() {
            self.manual_active = None;
            true
        } else {
            false
        }
    }

    /// Checks if a saved cache ID is valid.
    pub fn is_valid(&self, id: i32) -> bool {
        self.saved.contains_key(&id)
    }

    /// Checks if any cache is currently active.
    pub fn is_any_active(&self) -> bool {
        self.get_active().is_some()
    }
}

#[cfg(test)]
mod multi_result_tests {
    use super::*;

    fn set(name: &str, value: &str) -> ResultSet {
        ResultSet {
            rows: vec![vec![Some(value.to_string())]],
            field_names: vec![name.to_string()],
            field_types: vec![253],
        }
    }

    fn two_sets() -> CacheEntry {
        CacheEntry::with_results(
            vec![set("a", "first"), set("b", "second")],
            0,
            0,
            0,
            0,
            "CALL p()".to_string(),
        )
    }

    #[test]
    fn a_single_set_query_reports_one_result() {
        let entry = CacheEntry::with_results(vec![set("a", "x")], 0, 0, 0, 0, String::new());
        assert_eq!(entry.result_count(), 1);
    }

    #[test]
    fn an_empty_entry_still_reports_one_result() {
        // Readers must never hit an empty vector, so `empty` seeds one set.
        let entry = CacheEntry::empty("SELECT 1".to_string());
        assert_eq!(entry.result_count(), 1);
        assert_eq!(entry.row_count(), 0);
    }

    #[test]
    fn the_first_set_is_selected_by_default() {
        let entry = two_sets();
        assert_eq!(entry.result_count(), 2);
        assert_eq!(entry.field_name(0), Some("a"));
        assert_eq!(entry.get_value(0, 0), Some(&Some("first".to_string())));
    }

    #[test]
    fn set_result_switches_every_reader() {
        let mut entry = two_sets();
        assert!(entry.set_result(1));
        assert_eq!(entry.field_name(0), Some("b"));
        assert_eq!(entry.get_value(0, 0), Some(&Some("second".to_string())));
    }

    #[test]
    fn an_out_of_range_index_is_rejected_and_changes_nothing() {
        let mut entry = two_sets();
        assert!(!entry.set_result(2));
        assert!(!entry.set_result(99));
        assert_eq!(
            entry.field_name(0),
            Some("a"),
            "selection must be untouched"
        );
    }

    #[test]
    fn duplicate_keeps_every_set_and_the_selection() {
        let mut entry = two_sets();
        entry.set_result(1);

        let copy = entry.duplicate();
        assert_eq!(copy.result_count(), 2, "cache_save must not drop sets");
        assert_eq!(copy.field_name(0), Some("b"), "selection is preserved");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a single-result-set entry, which is what most of these tests
    /// care about. Multi-set behaviour is covered separately below.
    #[allow(clippy::too_many_arguments)]
    fn entry(
        rows: Vec<CacheRow>,
        field_names: Vec<String>,
        field_types: Vec<u8>,
        affected_rows: u64,
        insert_id: u64,
        warning_count: u16,
        exec_time_us: u128,
        query_string: String,
    ) -> CacheEntry {
        CacheEntry::with_results(
            vec![ResultSet {
                rows,
                field_names,
                field_types,
            }],
            affected_rows,
            insert_id,
            warning_count,
            exec_time_us,
            query_string,
        )
    }

    fn sample_entry() -> CacheEntry {
        entry(
            vec![
                vec![Some("1".to_string()), Some("Alice".to_string()), None],
                vec![
                    Some("2".to_string()),
                    Some("Bob".to_string()),
                    Some("bob@test.com".to_string()),
                ],
            ],
            vec!["id".to_string(), "name".to_string(), "email".to_string()],
            vec![3, 253, 253], // LONG, VAR_STRING, VAR_STRING
            0,
            0,
            0,
            5000, // 5ms
            "SELECT * FROM users".to_string(),
        )
    }

    // CacheEntry tests

    #[test]
    fn entry_new_preserves_fields() {
        let entry = sample_entry();
        assert_eq!(entry.row_count(), 2);
        assert_eq!(entry.field_count(), 3);
        assert_eq!(entry.query_string(), "SELECT * FROM users");
        assert_eq!(entry.exec_time_ms(), 5);
    }

    #[test]
    fn entry_empty() {
        let entry = CacheEntry::empty("INSERT INTO x".to_string());
        assert_eq!(entry.row_count(), 0);
        assert_eq!(entry.field_count(), 0);
        assert_eq!(entry.query_string(), "INSERT INTO x");
        assert_eq!(entry.affected_rows(), 0);
        assert_eq!(entry.insert_id(), 0);
    }

    #[test]
    fn entry_field_name_valid() {
        let entry = sample_entry();
        assert_eq!(entry.field_name(0), Some("id"));
        assert_eq!(entry.field_name(1), Some("name"));
        assert_eq!(entry.field_name(2), Some("email"));
    }

    #[test]
    fn entry_field_name_out_of_bounds() {
        let entry = sample_entry();
        assert!(entry.field_name(3).is_none());
    }

    #[test]
    fn entry_field_index_case_insensitive() {
        let entry = sample_entry();
        assert_eq!(entry.field_index("id"), Some(0));
        assert_eq!(entry.field_index("ID"), Some(0));
        assert_eq!(entry.field_index("Id"), Some(0));
        assert_eq!(entry.field_index("NAME"), Some(1));
    }

    #[test]
    fn entry_field_index_not_found() {
        let entry = sample_entry();
        assert!(entry.field_index("nonexistent").is_none());
    }

    #[test]
    fn entry_get_value_valid() {
        let entry = sample_entry();
        assert_eq!(entry.get_value(0, 0), Some(&Some("1".to_string())));
        assert_eq!(entry.get_value(0, 1), Some(&Some("Alice".to_string())));
    }

    #[test]
    fn entry_get_value_null() {
        let entry = sample_entry();
        assert_eq!(entry.get_value(0, 2), Some(&None)); // Alice's email is NULL
    }

    #[test]
    fn entry_get_value_out_of_bounds() {
        let entry = sample_entry();
        assert!(entry.get_value(5, 0).is_none());
        assert!(entry.get_value(0, 10).is_none());
    }

    #[test]
    fn entry_affected_rows_and_insert_id() {
        let entry = entry(
            vec![],
            vec![],
            vec![],
            42,
            100,
            0,
            0,
            "INSERT INTO x".to_string(),
        );
        assert_eq!(entry.affected_rows(), 42);
        assert_eq!(entry.insert_id(), 100);
    }

    #[test]
    fn entry_warning_count() {
        let entry = entry(vec![], vec![], vec![], 0, 0, 3, 0, "".to_string());
        assert_eq!(entry.warning_count(), 3);
    }

    #[test]
    fn entry_exec_time_conversion() {
        let entry = entry(
            vec![],
            vec![],
            vec![],
            0,
            0,
            0,
            123456, // microseconds
            "".to_string(),
        );
        assert_eq!(entry.exec_time_ms(), 123); // truncated to ms
    }

    #[test]
    fn entry_field_type_valid() {
        let entry = sample_entry();
        assert_eq!(entry.field_type(0), Some(3));
        assert_eq!(entry.field_type(1), Some(253));
    }

    #[test]
    fn entry_field_type_out_of_bounds() {
        let entry = sample_entry();
        assert!(entry.field_type(10).is_none());
    }

    // CacheManager tests

    #[test]
    fn manager_new_is_empty() {
        let mgr = CacheManager::new();
        assert!(!mgr.is_any_active());
        assert!(mgr.get_active().is_none());
    }

    #[test]
    fn manager_push_pop_stack() {
        let mut mgr = CacheManager::new();
        mgr.push_active(sample_entry());
        assert!(mgr.is_any_active());
        assert_eq!(mgr.get_active().unwrap().row_count(), 2);

        let popped = mgr.pop_active().unwrap();
        assert_eq!(popped.row_count(), 2);
        assert!(!mgr.is_any_active());
    }

    #[test]
    fn manager_stack_lifo() {
        let mut mgr = CacheManager::new();

        let entry1 = CacheEntry::empty("query1".to_string());
        let entry2 = CacheEntry::empty("query2".to_string());

        mgr.push_active(entry1);
        mgr.push_active(entry2);

        assert_eq!(mgr.get_active().unwrap().query_string(), "query2");
        mgr.pop_active();
        assert_eq!(mgr.get_active().unwrap().query_string(), "query1");
        mgr.pop_active();
        assert!(!mgr.is_any_active());
    }

    #[test]
    fn manager_pop_empty_returns_none() {
        let mut mgr = CacheManager::new();
        assert!(mgr.pop_active().is_none());
    }

    #[test]
    fn manager_save_and_restore() {
        let mut mgr = CacheManager::new();
        mgr.push_active(sample_entry());

        let saved_id = mgr.save();
        assert!(saved_id >= 1);
        assert!(mgr.is_valid(saved_id));

        mgr.pop_active();
        assert!(!mgr.is_any_active());

        // Restore saved cache
        assert!(mgr.set_active(saved_id));
        assert!(mgr.is_any_active());
        assert_eq!(mgr.get_active().unwrap().row_count(), 2);

        mgr.unset_active();
        assert!(!mgr.is_any_active());
    }

    #[test]
    fn manager_save_without_active_returns_zero() {
        let mut mgr = CacheManager::new();
        assert_eq!(mgr.save(), 0);
    }

    #[test]
    fn manager_save_incremental_ids() {
        let mut mgr = CacheManager::new();
        mgr.push_active(sample_entry());

        let id1 = mgr.save();
        let id2 = mgr.save();
        let id3 = mgr.save();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn manager_delete_cache() {
        let mut mgr = CacheManager::new();
        mgr.push_active(sample_entry());
        let id = mgr.save();

        assert!(mgr.delete(id));
        assert!(!mgr.is_valid(id));
        assert!(!mgr.delete(id)); // already deleted
    }

    #[test]
    fn manager_delete_clears_manual_active() {
        let mut mgr = CacheManager::new();
        mgr.push_active(sample_entry());
        let id = mgr.save();
        mgr.pop_active();

        mgr.set_active(id);
        assert!(mgr.is_any_active());

        mgr.delete(id);
        assert!(!mgr.is_any_active()); // manual_active cleared
    }

    #[test]
    fn manager_set_active_invalid_id() {
        let mut mgr = CacheManager::new();
        assert!(!mgr.set_active(999));
    }

    #[test]
    fn manager_unset_active_when_none() {
        let mut mgr = CacheManager::new();
        assert!(!mgr.unset_active());
    }

    #[test]
    fn manager_manual_active_overrides_stack() {
        let mut mgr = CacheManager::new();

        // Push entry on stack
        mgr.push_active(CacheEntry::empty("stack_query".to_string()));

        // Save a different entry
        mgr.push_active(sample_entry());
        let saved_id = mgr.save();
        mgr.pop_active();

        // Stack top is "stack_query"
        assert_eq!(mgr.get_active().unwrap().query_string(), "stack_query");

        // Manual active overrides
        mgr.set_active(saved_id);
        assert_eq!(
            mgr.get_active().unwrap().query_string(),
            "SELECT * FROM users"
        );

        // Unset manual returns to stack
        mgr.unset_active();
        assert_eq!(mgr.get_active().unwrap().query_string(), "stack_query");
    }

    #[test]
    fn manager_is_valid() {
        let mut mgr = CacheManager::new();
        assert!(!mgr.is_valid(1));

        mgr.push_active(sample_entry());
        let id = mgr.save();
        assert!(mgr.is_valid(id));
        assert!(!mgr.is_valid(id + 1));
    }

    #[test]
    fn manager_wrapping_saved_id() {
        let mut mgr = CacheManager::new();
        mgr.next_saved_id = i32::MAX;
        mgr.push_active(sample_entry());

        let id1 = mgr.save();
        assert_eq!(id1, i32::MAX);

        let id2 = mgr.save();
        assert!(id2 >= 1); // wraps, never 0
    }
}
