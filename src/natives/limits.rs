use samp::native;
use samp::prelude::*;

use crate::limits::{self, Limit};
use crate::logger::Logger;
use crate::plugin::MysqlPlugin;

impl MysqlPlugin {
    /// Raises or lowers one of the memory caps.
    ///
    /// Global on purpose, not per connection: the thing being protected is the
    /// server's memory, which every connection shares.
    #[native(name = "mysql_limit_set")]
    pub fn mysql_limit_set(&mut self, _amx: &Amx, limit: i32, value: i32) -> bool {
        let Some(limit) = Limit::from_id(limit) else {
            Logger::warn(&format!(
                "mysql_limit_set: unknown limit {limit}. Use one of the MYSQL_LIMIT_* constants."
            ));
            return false;
        };
        limits::set(limit, value)
    }

    /// Reads a cap, so a script can report it or restore it later.
    #[native(name = "mysql_limit_get")]
    pub fn mysql_limit_get(&mut self, _amx: &Amx, limit: i32) -> i32 {
        let Some(limit) = Limit::from_id(limit) else {
            Logger::warn(&format!(
                "mysql_limit_get: unknown limit {limit}. Use one of the MYSQL_LIMIT_* constants."
            ));
            return 0;
        };
        i32::try_from(limits::get(limit)).unwrap_or(i32::MAX)
    }
}
