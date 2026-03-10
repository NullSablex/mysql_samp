use std::collections::HashMap;

use crate::logger::Logger;

/// Maximum number of saved caches allowed to prevent memory exhaustion.
const MAX_SAVED_CACHES: usize = 1024;

/// A single row: Vec of Option<String> where None represents SQL NULL.
pub type CacheRow = Vec<Option<String>>;

/// Stores the result of a single query execution.
pub struct CacheEntry {
    rows: Vec<CacheRow>,
    field_names: Vec<String>,
    field_types: Vec<u8>,
    affected_rows: u64,
    insert_id: u64,
    warning_count: u16,
    exec_time_us: u128,
    query_string: String,
}

impl CacheEntry {
    pub fn new(
        rows: Vec<CacheRow>,
        field_names: Vec<String>,
        field_types: Vec<u8>,
        affected_rows: u64,
        insert_id: u64,
        warning_count: u16,
        exec_time_us: u128,
        query_string: String,
    ) -> Self {
        Self {
            rows,
            field_names,
            field_types,
            affected_rows,
            insert_id,
            warning_count,
            exec_time_us,
            query_string,
        }
    }

    pub fn empty(query_string: String) -> Self {
        Self {
            rows: Vec::new(),
            field_names: Vec::new(),
            field_types: Vec::new(),
            affected_rows: 0,
            insert_id: 0,
            warning_count: 0,
            exec_time_us: 0,
            query_string,
        }
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn field_count(&self) -> usize {
        self.field_names.len()
    }

    pub fn field_name(&self, index: usize) -> Option<&str> {
        self.field_names.get(index).map(|s| s.as_str())
    }

    pub fn field_index(&self, name: &str) -> Option<usize> {
        self.field_names
            .iter()
            .position(|f| f.eq_ignore_ascii_case(name))
    }

    pub fn get_value(&self, row: usize, col: usize) -> Option<&Option<String>> {
        self.rows.get(row).and_then(|r| r.get(col))
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
        self.field_types.get(index).copied()
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
            CacheEntry::new(
                active.rows.clone(),
                active.field_names.clone(),
                active.field_types.clone(),
                active.affected_rows,
                active.insert_id,
                active.warning_count,
                active.exec_time_us,
                active.query_string.clone(),
            )
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
mod tests {
    use super::*;

    fn sample_entry() -> CacheEntry {
        CacheEntry::new(
            vec![
                vec![Some("1".to_string()), Some("Alice".to_string()), None],
                vec![Some("2".to_string()), Some("Bob".to_string()), Some("bob@test.com".to_string())],
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
        let entry = CacheEntry::new(
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
        let entry = CacheEntry::new(
            vec![],
            vec![],
            vec![],
            0,
            0,
            3,
            0,
            "".to_string(),
        );
        assert_eq!(entry.warning_count(), 3);
    }

    #[test]
    fn entry_exec_time_conversion() {
        let entry = CacheEntry::new(
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
        assert_eq!(mgr.get_active().unwrap().query_string(), "SELECT * FROM users");

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
