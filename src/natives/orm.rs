use samp::args::Args;
use samp::native;
use samp::prelude::*;

use crate::connection::EscapeMode;
use crate::error::{ErrorState, MysqlError};
use crate::logger::Logger;
use crate::natives::query::parse_variadic_params;
use crate::orm::{MAX_ORM_STRING_LEN, OrmError, OrmInstance, OrmVarBinding};
use crate::plugin::MysqlPlugin;
use crate::query::CallbackInfo;

/// One of the five threaded CRUD operations exposed to Pawn.
///
/// Bundles every per-operation detail (display name, build error code,
/// build error message, query builder) in one place so that
/// [`MysqlPlugin::run_orm_op`] can stay generic over the operation.
#[derive(Clone, Copy)]
enum OrmOp {
    Select,
    Update,
    Insert,
    Delete,
    /// `Save` is `Insert` when the key column is empty, `Update` otherwise.
    Save,
}

impl OrmOp {
    fn name(self) -> &'static str {
        match self {
            Self::Select => "select",
            Self::Update => "update",
            Self::Insert => "insert",
            Self::Delete => "delete",
            Self::Save => "save",
        }
    }

    /// Error code reported via `mysql_errno`/global state when
    /// [`OrmOp::build`] returns `None`.
    fn build_error_code(self) -> MysqlError {
        match self {
            Self::Select | Self::Update | Self::Delete => MysqlError::OrmKeyNotSet,
            Self::Insert | Self::Save => MysqlError::InvalidOrm,
        }
    }

    /// Human-readable cause appended to the warning when
    /// [`OrmOp::build`] returns `None`.
    fn build_error_detail(self) -> &'static str {
        match self {
            Self::Select | Self::Update => "key column not set or no variables bound",
            Self::Insert => "no variables bound",
            Self::Delete => "key column not set",
            Self::Save => "cannot build query",
        }
    }

    fn build(self, inst: &OrmInstance, mode: EscapeMode) -> Option<String> {
        match self {
            Self::Select => inst.build_select(mode),
            Self::Update => inst.build_update(mode),
            Self::Insert => inst.build_insert(mode),
            Self::Delete => inst.build_delete(mode),
            Self::Save => {
                if inst.is_key_empty() {
                    inst.build_insert(mode)
                } else {
                    inst.build_update(mode)
                }
            }
        }
    }
}

impl MysqlPlugin {
    /// orm_create(const table[], connId)
    #[native(name = "orm_create")]
    pub fn orm_create(&mut self, amx: &Amx, table: &AmxString, conn_id: i32) -> i32 {
        if !self.connections.exists(conn_id) {
            Logger::warn("ORM create failed: invalid connection ID.");
            self.connections.global_error = ErrorState::new(
                MysqlError::InvalidConnection,
                "ORM create failed: invalid connection ID.",
            );
            return 0;
        }

        let ident = amx.ident();
        self.orm.create(table.to_string(), conn_id, ident)
    }

    /// orm_destroy(orm_id)
    #[native(name = "orm_destroy")]
    pub fn orm_destroy(&mut self, _amx: &Amx, orm_id: i32) -> bool {
        self.orm.destroy(orm_id)
    }

    /// orm_errno(orm_id)
    #[native(name = "orm_errno")]
    pub fn orm_errno(&mut self, _amx: &Amx, orm_id: i32) -> i32 {
        self.orm.get(orm_id).map_or(-1, |inst| inst.errno as i32)
    }

    /// orm_select(orm_id, const callback[] = "", const format[] = "", {Float,_}:...)
    #[native(name = "orm_select", raw)]
    pub fn orm_select(&mut self, _amx: &Amx, args: Args) -> bool {
        self.run_orm_op(OrmOp::Select, args)
    }

    /// orm_update(orm_id, const callback[] = "", const format[] = "", {Float,_}:...)
    #[native(name = "orm_update", raw)]
    pub fn orm_update(&mut self, _amx: &Amx, args: Args) -> bool {
        self.run_orm_op(OrmOp::Update, args)
    }

    /// orm_insert(orm_id, const callback[] = "", const format[] = "", {Float,_}:...)
    #[native(name = "orm_insert", raw)]
    pub fn orm_insert(&mut self, _amx: &Amx, args: Args) -> bool {
        self.run_orm_op(OrmOp::Insert, args)
    }

    /// orm_delete(orm_id, const callback[] = "", const format[] = "", {Float,_}:...)
    #[native(name = "orm_delete", raw)]
    pub fn orm_delete(&mut self, _amx: &Amx, args: Args) -> bool {
        self.run_orm_op(OrmOp::Delete, args)
    }

    /// orm_save(orm_id, const callback[] = "", const format[] = "", {Float,_}:...)
    /// INSERT when the key column is empty, UPDATE otherwise.
    #[native(name = "orm_save", raw)]
    pub fn orm_save(&mut self, _amx: &Amx, args: Args) -> bool {
        self.run_orm_op(OrmOp::Save, args)
    }

    /// Shared pipeline for the five threaded CRUD natives. Parses the
    /// common prelude (orm_id, callback, format), looks up the instance,
    /// asks `op` to build the SQL string, resolves the pool, builds the
    /// callback descriptor and submits the query.
    fn run_orm_op(&mut self, op: OrmOp, mut args: Args) -> bool {
        let Some(orm_id) = args.next_arg::<i32>() else {
            return false;
        };
        let callback_str: String = args
            .next_arg::<AmxString>()
            .map(|v| v.to_string())
            .unwrap_or_default();
        let format_str: String = args
            .next_arg::<AmxString>()
            .map(|v| v.to_string())
            .unwrap_or_default();

        let name = op.name();

        let (query, conn_id) = {
            let Some(inst) = self.orm.get(orm_id) else {
                let msg = format!("ORM {name} failed: invalid ORM ID.");
                Logger::warn(&msg);
                self.connections.global_error = ErrorState::new(MysqlError::InvalidOrm, msg);
                return false;
            };

            let Some(query) = op.build(inst, self.connections.escape_mode(inst.conn_id)) else {
                let msg = format!("ORM {name} failed: {}.", op.build_error_detail());
                Logger::warn(&msg);
                self.connections.global_error = ErrorState::new(op.build_error_code(), msg);
                return false;
            };

            (query, inst.conn_id)
        };

        let Some(pool) = self.connections.get_pool(conn_id) else {
            Logger::warn(&format!("ORM {name} failed: invalid connection ID."));
            return false;
        };

        let callback_info = if callback_str.is_empty() {
            None
        } else {
            let params = parse_variadic_params(&mut args, &format_str, 3);
            Some(CallbackInfo {
                name: callback_str,
                format: format_str,
                params,
            })
        };

        self.queries.submit_query(
            pool,
            query,
            callback_info,
            conn_id,
            self.connections.get_auto_reconnect(conn_id),
        );
        true
    }

    /// orm_apply_cache(orm_id, row = 0)
    #[native(name = "orm_apply_cache")]
    pub fn orm_apply_cache(&mut self, amx: &Amx, orm_id: i32, row: i32) -> bool {
        let Some(cache) = self.cache.get_active() else {
            Logger::warn("ORM apply_cache failed: no active cache.");
            self.connections.global_error = ErrorState::new(
                MysqlError::NoCacheActive,
                "ORM apply_cache failed: no active cache.",
            );
            return false;
        };

        let Some(inst) = self.orm.get_mut(orm_id) else {
            Logger::warn("ORM apply_cache failed: invalid ORM ID.");
            self.connections.global_error = ErrorState::new(
                MysqlError::InvalidOrm,
                "ORM apply_cache failed: invalid ORM ID.",
            );
            return false;
        };

        let Ok(row_idx) = usize::try_from(row) else {
            inst.errno = OrmError::NoData;
            return false;
        };

        if row_idx >= cache.row_count() {
            inst.errno = OrmError::NoData;
            return false;
        }

        inst.apply_cache(amx, cache, row_idx);
        inst.errno = OrmError::Ok;
        true
    }

    /// orm_addvar_int(orm_id, &var, const column_name[])
    #[native(name = "orm_addvar_int", raw)]
    pub fn orm_addvar_int(&mut self, _amx: &Amx, mut args: Args) -> bool {
        let Some(orm_id) = args.next_arg::<i32>() else {
            return false;
        };
        let Some(var_ref) = args.next_arg::<Ref<i32>>() else {
            return false;
        };
        let Some(column) = args.next_arg::<AmxString>() else {
            return false;
        };
        let Some(inst) = self.orm.get_mut(orm_id) else {
            return false;
        };

        inst.variables.push(OrmVarBinding::Int {
            amx_addr: var_ref.address(),
            column: column.to_string(),
        });
        true
    }

    /// orm_addvar_float(orm_id, &Float:var, const column_name[])
    #[native(name = "orm_addvar_float", raw)]
    pub fn orm_addvar_float(&mut self, _amx: &Amx, mut args: Args) -> bool {
        let Some(orm_id) = args.next_arg::<i32>() else {
            return false;
        };
        let Some(var_ref) = args.next_arg::<Ref<i32>>() else {
            return false;
        };
        let Some(column) = args.next_arg::<AmxString>() else {
            return false;
        };
        let Some(inst) = self.orm.get_mut(orm_id) else {
            return false;
        };

        inst.variables.push(OrmVarBinding::Float {
            amx_addr: var_ref.address(),
            column: column.to_string(),
        });
        true
    }

    /// orm_addvar_string(orm_id, var[], var_max_len, const column_name[])
    #[native(name = "orm_addvar_string", raw)]
    pub fn orm_addvar_string(&mut self, _amx: &Amx, mut args: Args) -> bool {
        let Some(orm_id) = args.next_arg::<i32>() else {
            return false;
        };
        let Some(var_ref) = args.next_arg::<Ref<i32>>() else {
            return false;
        };
        let Some(max_len) = args.next_arg::<i32>() else {
            return false;
        };
        let Some(column) = args.next_arg::<AmxString>() else {
            return false;
        };

        if max_len <= 0 || max_len > MAX_ORM_STRING_LEN {
            Logger::warn(&format!(
                "ORM addvar_string failed: max_len must be between 1 and {}.",
                MAX_ORM_STRING_LEN
            ));
            return false;
        }

        let Some(inst) = self.orm.get_mut(orm_id) else {
            return false;
        };

        inst.variables.push(OrmVarBinding::String {
            amx_addr: var_ref.address(),
            max_len,
            column: column.to_string(),
        });
        true
    }

    /// orm_delvar(orm_id, const column_name[])
    #[native(name = "orm_delvar")]
    pub fn orm_delvar(&mut self, _amx: &Amx, orm_id: i32, column_name: &AmxString) -> bool {
        let Some(inst) = self.orm.get_mut(orm_id) else {
            return false;
        };

        let before = inst.variables.len();
        inst.variables.retain(|v| v.column_name() != &**column_name);
        inst.variables.len() < before
    }

    /// orm_clear_vars(orm_id)
    #[native(name = "orm_clear_vars")]
    pub fn orm_clear_vars(&mut self, _amx: &Amx, orm_id: i32) -> bool {
        let Some(inst) = self.orm.get_mut(orm_id) else {
            return false;
        };

        inst.variables.clear();
        true
    }

    /// orm_setkey(orm_id, const column_name[])
    #[native(name = "orm_setkey")]
    pub fn orm_setkey(&mut self, _amx: &Amx, orm_id: i32, column_name: &AmxString) -> bool {
        let Some(inst) = self.orm.get_mut(orm_id) else {
            return false;
        };

        inst.key_column = Some(column_name.to_string());
        true
    }
}
