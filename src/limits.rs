//! Runtime caps on what the plugin will hold in memory.
//!
//! These exist so a gamemode cannot exhaust the server's memory by accident -
//! a query that matches a million rows, a loop that saves a cache per tick.
//! The defaults suit an ordinary game server; a job that legitimately needs
//! more can raise them, and a deployment that wants to be stricter can lower
//! them.
//!
//! They are atomics rather than fields on the plugin because
//! [`crate::connection::collect_result`] enforces the row cap from a worker
//! thread, which has no route to plugin state. Reads happen per row, so they
//! use `Relaxed`: a change racing with a query in flight lands on the next
//! one, which is all the ordering this needs.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::logger::Logger;

/// Saved caches held by `cache_save` at once.
pub const DEFAULT_SAVED_CACHES: usize = 1024;
/// Rows a single result set may carry.
pub const DEFAULT_RESULT_ROWS: usize = 100_000;
/// Bytes an ORM string variable may bind.
pub const DEFAULT_ORM_STRING_LEN: usize = 4096;

static SAVED_CACHES: AtomicUsize = AtomicUsize::new(DEFAULT_SAVED_CACHES);
static RESULT_ROWS: AtomicUsize = AtomicUsize::new(DEFAULT_RESULT_ROWS);
static ORM_STRING_LEN: AtomicUsize = AtomicUsize::new(DEFAULT_ORM_STRING_LEN);

/// Which cap a `mysql_limit_*` native is addressing.
///
/// The numbering is the Pawn enum in the include and is part of the API: add
/// to the end, never renumber.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum Limit {
    SavedCaches = 0,
    ResultRows = 1,
    OrmStringLen = 2,
}

impl Limit {
    /// The Pawn constant, or `None` when the script passed something else.
    #[must_use]
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::SavedCaches),
            1 => Some(Self::ResultRows),
            2 => Some(Self::OrmStringLen),
            _ => None,
        }
    }

    fn cell(self) -> &'static AtomicUsize {
        match self {
            Self::SavedCaches => &SAVED_CACHES,
            Self::ResultRows => &RESULT_ROWS,
            Self::OrmStringLen => &ORM_STRING_LEN,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::SavedCaches => "MYSQL_LIMIT_SAVED_CACHES",
            Self::ResultRows => "MYSQL_LIMIT_RESULT_ROWS",
            Self::OrmStringLen => "MYSQL_LIMIT_ORM_STRING_LEN",
        }
    }
}

/// Current value of a cap.
#[must_use]
pub fn get(limit: Limit) -> usize {
    limit.cell().load(Ordering::Relaxed)
}

/// Sets a cap, refusing a value that would disable it.
///
/// Zero and negatives are refused rather than treated as "unlimited": a cap of
/// zero would break every save and every query, and "unlimited" is exactly the
/// state these exist to prevent. Lowering a cap below what is already held
/// keeps what is there and refuses the next one, which is the behaviour that
/// does not throw away a gamemode's data mid-round.
pub fn set(limit: Limit, value: i32) -> bool {
    let Ok(value) = usize::try_from(value) else {
        Logger::warn(&format!(
            "{}: a limit must be positive; {value} was ignored.",
            limit.name()
        ));
        return false;
    };
    if value == 0 {
        Logger::warn(&format!(
            "{}: a limit of 0 would refuse everything; it was ignored.",
            limit.name()
        ));
        return false;
    }

    limit.cell().store(value, Ordering::Relaxed);
    true
}

/// Saved caches held at once.
#[must_use]
pub fn saved_caches() -> usize {
    get(Limit::SavedCaches)
}

/// Rows a single result set may carry.
#[must_use]
pub fn result_rows() -> usize {
    get(Limit::ResultRows)
}

/// Bytes an ORM string variable may bind.
#[must_use]
pub fn orm_string_len() -> usize {
    get(Limit::OrmStringLen)
}

/// Same as [`orm_string_len`] where the caller compares against a Pawn cell.
#[must_use]
pub fn orm_string_len_i32() -> i32 {
    i32::try_from(orm_string_len()).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The statics are process-wide, so each test restores what it changed.

    #[test]
    fn unknown_id_is_rejected() {
        assert!(Limit::from_id(-1).is_none());
        assert!(Limit::from_id(3).is_none());
        assert_eq!(Limit::from_id(0), Some(Limit::SavedCaches));
    }

    #[test]
    fn a_limit_round_trips() {
        let before = get(Limit::SavedCaches);
        assert!(set(Limit::SavedCaches, 7));
        assert_eq!(get(Limit::SavedCaches), 7);
        assert!(set(
            Limit::SavedCaches,
            i32::try_from(before).expect("default fits in i32")
        ));
    }

    #[test]
    fn zero_and_negative_are_refused() {
        let before = get(Limit::ResultRows);
        assert!(!set(Limit::ResultRows, 0));
        assert!(!set(Limit::ResultRows, -5));
        assert_eq!(
            get(Limit::ResultRows),
            before,
            "a refused set must not change the cap"
        );
    }

    #[test]
    fn defaults_match_the_documented_numbers() {
        assert_eq!(DEFAULT_SAVED_CACHES, 1024);
        assert_eq!(DEFAULT_RESULT_ROWS, 100_000);
        assert_eq!(DEFAULT_ORM_STRING_LEN, 4096);
    }
}
