use samp::amx::AmxIdent;
use samp::prelude::*;

use crate::cache::CacheManager;
use crate::callback;
use crate::connection::ConnectionManager;
use crate::error::{ErrorState, MysqlError};
use crate::logger::Logger;
use crate::options::OptionsManager;
use crate::orm::OrmManager;
use crate::query::QueryManager;

pub struct MysqlPlugin {
    pub connections: ConnectionManager,
    pub options: OptionsManager,
    pub cache: CacheManager,
    pub queries: QueryManager,
    pub orm: OrmManager,
    pub amx_list: Vec<AmxIdent>,
}

impl MysqlPlugin {
    pub fn new() -> Self {
        Logger::init();

        Self {
            connections: ConnectionManager::new(),
            options: OptionsManager::new(),
            cache: CacheManager::new(),
            queries: QueryManager::new(),
            orm: OrmManager::new(),
            amx_list: Vec::new(),
        }
    }

    /// Processes completed threaded queries and dispatches callbacks.
    fn process_pending_queries(&mut self) {
        let results = self.queries.poll_results();

        for result in results {
            let callback_name = result
                .callback
                .as_ref()
                .map(|c| c.name.as_str())
                .unwrap_or("");

            // Handle query errors
            if let Some(ref error) = result.error {
                Logger::error_detail(
                    &format!(
                        "Query failed on connection {} (error {}). See logs/mysql.log for details.",
                        result.conn_id, error.code
                    ),
                    &format!("Query error: {}", error.message),
                );

                // Update the per-connection error state
                self.connections.set_error(
                    result.conn_id,
                    ErrorState::new(
                        MysqlError::QueryFailed,
                        error.message.clone(),
                    ),
                );

                callback::fire_on_query_error(
                    &self.amx_list,
                    error.code as i32,
                    &error.message,
                    callback_name,
                    result.cache.query_string(),
                    result.conn_id,
                );
                continue;
            }

            // Push cache onto the active stack
            self.cache.push_active(result.cache);

            // Invoke callback if specified
            if let Some(ref info) = result.callback {
                if !info.name.is_empty() {
                    callback::invoke_callback(&self.amx_list, info);
                }
            }

            // Pop cache after callback returns
            self.cache.pop_active();
        }
    }
}

impl SampPlugin for MysqlPlugin {
    fn on_load(&mut self) {}

    fn on_unload(&mut self) {
        Logger::info("Plugin unloaded.");
    }

    fn on_amx_load(&mut self, amx: &Amx) {
        let ident = amx.ident();
        self.amx_list.push(ident);
    }

    fn on_amx_unload(&mut self, amx: &Amx) {
        let ident = amx.ident();
        self.amx_list.retain(|id| *id != ident);

        // Destroy ORM instances associated with the unloaded AMX
        self.orm.destroy_by_amx(ident);
    }

    fn process_tick(&mut self) {
        self.process_pending_queries();
    }
}
