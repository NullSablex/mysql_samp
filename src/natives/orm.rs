use samp::args::Args;
use samp::native;
use samp::prelude::*;

use crate::error::{ErrorState, MysqlError};
use crate::logger::Logger;
use crate::natives::query::parse_variadic_params;
use crate::orm::{OrmError, OrmVarBinding};
use crate::plugin::MysqlPlugin;
use crate::query::CallbackInfo;

impl MysqlPlugin {
    /// orm_create(const table[], connId)
    #[native(name = "orm_create")]
    pub fn orm_create(
        &mut self,
        amx: &Amx,
        table: AmxString,
        conn_id: i32,
    ) -> AmxResult<i32> {
        if !self.connections.exists(conn_id) {
            Logger::warn("ORM create failed: invalid connection ID.");
            self.connections.global_error = ErrorState::new(
                MysqlError::InvalidConnection,
                "ORM create failed: invalid connection ID.",
            );
            return Ok(0);
        }

        let ident = amx.ident();
        let id = self.orm.create(table.to_string(), conn_id, ident);
        Ok(id)
    }

    /// orm_destroy(orm_id)
    #[native(name = "orm_destroy")]
    pub fn orm_destroy(&mut self, _amx: &Amx, orm_id: i32) -> AmxResult<bool> {
        Ok(self.orm.destroy(orm_id))
    }

    /// orm_errno(orm_id)
    #[native(name = "orm_errno")]
    pub fn orm_errno(&mut self, _amx: &Amx, orm_id: i32) -> AmxResult<i32> {
        match self.orm.get(orm_id) {
            Some(inst) => Ok(inst.errno as i32),
            None => Ok(-1),
        }
    }

    /// orm_select(orm_id, const callback[] = "", const format[] = "", {Float,_}:...)
    #[native(name = "orm_select", raw)]
    pub fn orm_select(&mut self, _amx: &Amx, mut args: Args) -> AmxResult<bool> {
        let orm_id: i32 = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };

        let callback_str: String = match args.next_arg::<AmxString>() {
            Some(v) => v.to_string(),
            None => String::new(),
        };

        let format_str: String = match args.next_arg::<AmxString>() {
            Some(v) => v.to_string(),
            None => String::new(),
        };

        let inst = match self.orm.get(orm_id) {
            Some(i) => i,
            None => {
                Logger::warn("ORM select failed: invalid ORM ID.");
                self.connections.global_error = ErrorState::new(
                    MysqlError::InvalidOrm,
                    "ORM select failed: invalid ORM ID.",
                );
                return Ok(false);
            }
        };

        let query = match inst.build_select() {
            Some(q) => q,
            None => {
                Logger::warn("ORM select failed: key column not set or no variables bound.");
                self.connections.global_error = ErrorState::new(
                    MysqlError::OrmKeyNotSet,
                    "ORM select failed: key column not set or no variables bound.",
                );
                return Ok(false);
            }
        };

        let conn_id = inst.conn_id;
        let pool = match self.connections.get_pool(conn_id) {
            Some(p) => p,
            None => {
                Logger::warn("ORM select failed: invalid connection ID.");
                return Ok(false);
            }
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

        self.queries
            .submit_query(pool, query, callback_info, conn_id, self.connections.get_auto_reconnect(conn_id));
        Ok(true)
    }

    /// orm_update(orm_id, const callback[] = "", const format[] = "", {Float,_}:...)
    #[native(name = "orm_update", raw)]
    pub fn orm_update(&mut self, _amx: &Amx, mut args: Args) -> AmxResult<bool> {
        let orm_id: i32 = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };

        let callback_str: String = match args.next_arg::<AmxString>() {
            Some(v) => v.to_string(),
            None => String::new(),
        };

        let format_str: String = match args.next_arg::<AmxString>() {
            Some(v) => v.to_string(),
            None => String::new(),
        };

        let inst = match self.orm.get(orm_id) {
            Some(i) => i,
            None => {
                Logger::warn("ORM update failed: invalid ORM ID.");
                self.connections.global_error = ErrorState::new(
                    MysqlError::InvalidOrm,
                    "ORM update failed: invalid ORM ID.",
                );
                return Ok(false);
            }
        };

        let query = match inst.build_update() {
            Some(q) => q,
            None => {
                Logger::warn("ORM update failed: key column not set or no variables bound.");
                self.connections.global_error = ErrorState::new(
                    MysqlError::OrmKeyNotSet,
                    "ORM update failed: key column not set or no variables bound.",
                );
                return Ok(false);
            }
        };

        let conn_id = inst.conn_id;
        let pool = match self.connections.get_pool(conn_id) {
            Some(p) => p,
            None => {
                Logger::warn("ORM update failed: invalid connection ID.");
                return Ok(false);
            }
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

        self.queries
            .submit_query(pool, query, callback_info, conn_id, self.connections.get_auto_reconnect(conn_id));
        Ok(true)
    }

    /// orm_insert(orm_id, const callback[] = "", const format[] = "", {Float,_}:...)
    #[native(name = "orm_insert", raw)]
    pub fn orm_insert(&mut self, _amx: &Amx, mut args: Args) -> AmxResult<bool> {
        let orm_id: i32 = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };

        let callback_str: String = match args.next_arg::<AmxString>() {
            Some(v) => v.to_string(),
            None => String::new(),
        };

        let format_str: String = match args.next_arg::<AmxString>() {
            Some(v) => v.to_string(),
            None => String::new(),
        };

        let inst = match self.orm.get(orm_id) {
            Some(i) => i,
            None => {
                Logger::warn("ORM insert failed: invalid ORM ID.");
                self.connections.global_error = ErrorState::new(
                    MysqlError::InvalidOrm,
                    "ORM insert failed: invalid ORM ID.",
                );
                return Ok(false);
            }
        };

        let query = match inst.build_insert() {
            Some(q) => q,
            None => {
                Logger::warn("ORM insert failed: no variables bound.");
                self.connections.global_error = ErrorState::new(
                    MysqlError::InvalidOrm,
                    "ORM insert failed: no variables bound.",
                );
                return Ok(false);
            }
        };

        let conn_id = inst.conn_id;
        let pool = match self.connections.get_pool(conn_id) {
            Some(p) => p,
            None => {
                Logger::warn("ORM insert failed: invalid connection ID.");
                return Ok(false);
            }
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

        self.queries
            .submit_query(pool, query, callback_info, conn_id, self.connections.get_auto_reconnect(conn_id));
        Ok(true)
    }

    /// orm_delete(orm_id, const callback[] = "", const format[] = "", {Float,_}:...)
    #[native(name = "orm_delete", raw)]
    pub fn orm_delete(&mut self, _amx: &Amx, mut args: Args) -> AmxResult<bool> {
        let orm_id: i32 = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };

        let callback_str: String = match args.next_arg::<AmxString>() {
            Some(v) => v.to_string(),
            None => String::new(),
        };

        let format_str: String = match args.next_arg::<AmxString>() {
            Some(v) => v.to_string(),
            None => String::new(),
        };

        let inst = match self.orm.get(orm_id) {
            Some(i) => i,
            None => {
                Logger::warn("ORM delete failed: invalid ORM ID.");
                self.connections.global_error = ErrorState::new(
                    MysqlError::InvalidOrm,
                    "ORM delete failed: invalid ORM ID.",
                );
                return Ok(false);
            }
        };

        let query = match inst.build_delete() {
            Some(q) => q,
            None => {
                Logger::warn("ORM delete failed: key column not set.");
                self.connections.global_error = ErrorState::new(
                    MysqlError::OrmKeyNotSet,
                    "ORM delete failed: key column not set.",
                );
                return Ok(false);
            }
        };

        let conn_id = inst.conn_id;
        let pool = match self.connections.get_pool(conn_id) {
            Some(p) => p,
            None => {
                Logger::warn("ORM delete failed: invalid connection ID.");
                return Ok(false);
            }
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

        self.queries
            .submit_query(pool, query, callback_info, conn_id, self.connections.get_auto_reconnect(conn_id));
        Ok(true)
    }

    /// orm_save(orm_id, const callback[] = "", const format[] = "", {Float,_}:...)
    /// If key value is 0/empty, does INSERT. Otherwise does UPDATE.
    #[native(name = "orm_save", raw)]
    pub fn orm_save(&mut self, _amx: &Amx, mut args: Args) -> AmxResult<bool> {
        let orm_id: i32 = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };

        let callback_str: String = match args.next_arg::<AmxString>() {
            Some(v) => v.to_string(),
            None => String::new(),
        };

        let format_str: String = match args.next_arg::<AmxString>() {
            Some(v) => v.to_string(),
            None => String::new(),
        };

        let inst = match self.orm.get(orm_id) {
            Some(i) => i,
            None => {
                Logger::warn("ORM save failed: invalid ORM ID.");
                self.connections.global_error = ErrorState::new(
                    MysqlError::InvalidOrm,
                    "ORM save failed: invalid ORM ID.",
                );
                return Ok(false);
            }
        };

        let is_insert = inst.is_key_empty();
        let query = if is_insert {
            inst.build_insert()
        } else {
            inst.build_update()
        };

        let query = match query {
            Some(q) => q,
            None => {
                Logger::warn("ORM save failed: cannot build query.");
                self.connections.global_error = ErrorState::new(
                    MysqlError::InvalidOrm,
                    "ORM save failed: cannot build query.",
                );
                return Ok(false);
            }
        };

        let conn_id = inst.conn_id;
        let pool = match self.connections.get_pool(conn_id) {
            Some(p) => p,
            None => {
                Logger::warn("ORM save failed: invalid connection ID.");
                return Ok(false);
            }
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

        self.queries
            .submit_query(pool, query, callback_info, conn_id, self.connections.get_auto_reconnect(conn_id));
        Ok(true)
    }

    /// orm_apply_cache(orm_id, row = 0)
    #[native(name = "orm_apply_cache")]
    pub fn orm_apply_cache(
        &mut self,
        amx: &Amx,
        orm_id: i32,
        row: i32,
    ) -> AmxResult<bool> {
        let cache = match self.cache.get_active() {
            Some(c) => c,
            None => {
                Logger::warn("ORM apply_cache failed: no active cache.");
                self.connections.global_error = ErrorState::new(
                    MysqlError::NoCacheActive,
                    "ORM apply_cache failed: no active cache.",
                );
                return Ok(false);
            }
        };

        let inst = match self.orm.get_mut(orm_id) {
            Some(i) => i,
            None => {
                Logger::warn("ORM apply_cache failed: invalid ORM ID.");
                self.connections.global_error = ErrorState::new(
                    MysqlError::InvalidOrm,
                    "ORM apply_cache failed: invalid ORM ID.",
                );
                return Ok(false);
            }
        };

        if row as usize >= cache.row_count() {
            inst.errno = OrmError::NoData;
            return Ok(false);
        }

        inst.apply_cache(amx, cache, row as usize);
        inst.errno = OrmError::Ok;
        Ok(true)
    }

    /// orm_addvar_int(orm_id, &var, const column_name[])
    #[native(name = "orm_addvar_int", raw)]
    pub fn orm_addvar_int(&mut self, _amx: &Amx, mut args: Args) -> AmxResult<bool> {
        let orm_id: i32 = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };

        let var_ref: Ref<i32> = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };

        let column: AmxString = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };

        let inst = match self.orm.get_mut(orm_id) {
            Some(i) => i,
            None => return Ok(false),
        };

        inst.variables.push(OrmVarBinding::Int {
            amx_addr: var_ref.address(),
            column: column.to_string(),
        });
        Ok(true)
    }

    /// orm_addvar_float(orm_id, &Float:var, const column_name[])
    #[native(name = "orm_addvar_float", raw)]
    pub fn orm_addvar_float(&mut self, _amx: &Amx, mut args: Args) -> AmxResult<bool> {
        let orm_id: i32 = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };

        let var_ref: Ref<i32> = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };

        let column: AmxString = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };

        let inst = match self.orm.get_mut(orm_id) {
            Some(i) => i,
            None => return Ok(false),
        };

        inst.variables.push(OrmVarBinding::Float {
            amx_addr: var_ref.address(),
            column: column.to_string(),
        });
        Ok(true)
    }

    /// orm_addvar_string(orm_id, var[], var_max_len, const column_name[])
    #[native(name = "orm_addvar_string", raw)]
    pub fn orm_addvar_string(&mut self, _amx: &Amx, mut args: Args) -> AmxResult<bool> {
        let orm_id: i32 = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };

        let var_ref: Ref<i32> = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };

        let max_len: i32 = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };

        let column: AmxString = match args.next_arg() {
            Some(v) => v,
            None => return Ok(false),
        };

        if max_len <= 0 || max_len > 4096 {
            Logger::warn("ORM addvar_string failed: max_len must be between 1 and 4096.");
            return Ok(false);
        }

        let inst = match self.orm.get_mut(orm_id) {
            Some(i) => i,
            None => return Ok(false),
        };

        inst.variables.push(OrmVarBinding::String {
            amx_addr: var_ref.address(),
            max_len,
            column: column.to_string(),
        });
        Ok(true)
    }

    /// orm_delvar(orm_id, const column_name[])
    #[native(name = "orm_delvar")]
    pub fn orm_delvar(
        &mut self,
        _amx: &Amx,
        orm_id: i32,
        column_name: AmxString,
    ) -> AmxResult<bool> {
        let name = column_name.to_string();
        let inst = match self.orm.get_mut(orm_id) {
            Some(i) => i,
            None => return Ok(false),
        };

        let before = inst.variables.len();
        inst.variables.retain(|v| v.column_name() != name);
        Ok(inst.variables.len() < before)
    }

    /// orm_clear_vars(orm_id)
    #[native(name = "orm_clear_vars")]
    pub fn orm_clear_vars(&mut self, _amx: &Amx, orm_id: i32) -> AmxResult<bool> {
        let inst = match self.orm.get_mut(orm_id) {
            Some(i) => i,
            None => return Ok(false),
        };

        inst.variables.clear();
        Ok(true)
    }

    /// orm_setkey(orm_id, const column_name[])
    #[native(name = "orm_setkey")]
    pub fn orm_setkey(
        &mut self,
        _amx: &Amx,
        orm_id: i32,
        column_name: AmxString,
    ) -> AmxResult<bool> {
        let inst = match self.orm.get_mut(orm_id) {
            Some(i) => i,
            None => return Ok(false),
        };

        inst.key_column = Some(column_name.to_string());
        Ok(true)
    }
}
