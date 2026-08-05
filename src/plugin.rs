use samp::amx::AmxIdent;
use samp::plugin::TickContext;
use samp::prelude::*;

use crate::cache::CacheManager;
use crate::callback;
use crate::connection::ConnectionManager;
use crate::error::{ErrorState, MysqlError};
use crate::logger::Logger;
use crate::options::OptionsManager;
use crate::orm::OrmManager;
use crate::password::{PasswordManager, PasswordOutcome};
use crate::query::{CallbackParam, QueryManager};
use crate::stmt::StmtManager;
use crate::transaction::TransactionManager;

pub struct MysqlPlugin {
    pub connections: ConnectionManager,
    pub options: OptionsManager,
    pub cache: CacheManager,
    pub queries: QueryManager,
    pub orm: OrmManager,
    pub passwords: PasswordManager,
    pub stmts: StmtManager,
    pub transactions: TransactionManager,
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
            passwords: PasswordManager::new(),
            stmts: StmtManager::new(),
            transactions: TransactionManager::new(),
            amx_list: Vec::new(),
        }
    }

    /// Processes completed threaded queries and dispatches callbacks.
    pub fn process_pending_queries(&mut self) {
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
                    ErrorState::new(MysqlError::QueryFailed, error.message.clone()),
                );

                callback::fire_on_query_error(
                    &self.amx_list,
                    i32::from(error.code),
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
            if let Some(ref info) = result.callback
                && !info.name.is_empty()
            {
                callback::invoke_callback(&self.amx_list, info);
            }

            // Pop cache after callback returns
            self.cache.pop_active();
        }
    }

    /// Dispatches finished Argon2id work to its Pawn callback.
    pub fn process_pending_passwords(&mut self) {
        for result in self.passwords.poll_results() {
            let mut info = result.callback;

            match result.outcome {
                PasswordOutcome::Hash(hash) => {
                    info.params.insert(0, CallbackParam::String(hash));
                }
                PasswordOutcome::Verify(matched) => {
                    info.params
                        .insert(0, CallbackParam::Int(i32::from(matched)));
                }
                PasswordOutcome::Failed(detail) => {
                    Logger::error_detail(
                        "Password operation failed. See logs/mysql.log for details.",
                        &detail,
                    );
                    // Report the failure as "did not match" / empty hash rather
                    // than dropping the callback: a gamemode waiting on it would
                    // otherwise leave the player stuck forever.
                    match info.format.chars().next() {
                        Some('s') => info.params.insert(0, CallbackParam::String(String::new())),
                        _ => info.params.insert(0, CallbackParam::Int(0)),
                    }
                }
            }

            callback::invoke_callback(&self.amx_list, &info);
        }
    }
}

impl SampPlugin for MysqlPlugin {
    fn on_load(&mut self) {}

    fn on_unload(&mut self) {
        Logger::info("Plugin unloaded.");
        Logger::flush();
    }

    fn on_amx_load(&mut self, amx: &Amx) {
        let ident = amx.ident();
        self.amx_list.push(ident);
    }

    fn on_amx_unload(&mut self, amx: &Amx) {
        let ident = amx.ident();
        self.amx_list.retain(|id| *id != ident);

        // Reclaim everything the unloaded script owned. Without this a
        // gamemode restart leaks every handle it ever created.
        self.orm.destroy_by_amx(ident);
        self.stmts.destroy_by_amx(ident);
        self.transactions.destroy_by_amx(ident);
    }

    /// Unified tick callback (v3.0.0+): fires on both SA-MP (ProcessTick) and
    /// Open Multiplayer native mode (ITimersComponent timer). Drives query
    /// dispatch automatically — no Pawn timer required anymore.
    fn on_tick(&mut self, _ctx: TickContext) {
        self.process_pending_queries();
        self.process_pending_passwords();
    }

    fn on_omp_ready(&mut self) {
        Logger::info("Open Multiplayer native mode: all components ready.");
    }

    /// Fires when any Open Multiplayer component is being released (not just
    /// ours). We don't query other components, so there's nothing to
    /// invalidate — the log line just helps correlate "mysql_samp misbehaved
    /// after plugin X was unloaded" reports.
    fn on_component_free(&mut self) {
        Logger::info("Open Multiplayer: a neighbouring component is being unloaded.");
    }
}
