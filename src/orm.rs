use std::collections::HashMap;

use samp::amx::{AmxIdent, get as get_amx};
use samp::prelude::*;

use crate::connection::{escape_identifier, escape_string};

/// ORM error codes exposed to Pawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum OrmError {
    Ok = 0,
    NoData = 1,
}

/// Represents a bound Pawn variable mapped to a database column.
#[derive(Debug, Clone)]
pub enum OrmVarBinding {
    Int { amx_addr: i32, column: String },
    Float { amx_addr: i32, column: String },
    String { amx_addr: i32, max_len: i32, column: String },
}

impl OrmVarBinding {
    pub fn column_name(&self) -> &str {
        match self {
            OrmVarBinding::Int { column, .. } => column,
            OrmVarBinding::Float { column, .. } => column,
            OrmVarBinding::String { column, .. } => column,
        }
    }
}

/// A single ORM instance mapping a Pawn struct to a database table.
pub struct OrmInstance {
    pub table: String,
    pub conn_id: i32,
    pub key_column: Option<String>,
    pub variables: Vec<OrmVarBinding>,
    pub errno: OrmError,
    pub amx_ident: AmxIdent,
}

impl OrmInstance {
    pub fn new(table: String, conn_id: i32, amx_ident: AmxIdent) -> Self {
        Self {
            table,
            conn_id,
            key_column: None,
            variables: Vec::new(),
            errno: OrmError::Ok,
            amx_ident,
        }
    }

    /// Reads the current value of an int binding from AMX memory.
    fn read_int(&self, amx: &Amx, amx_addr: i32) -> i32 {
        match amx.get_ref::<i32>(amx_addr) {
            Ok(r) => *r,
            Err(_) => 0,
        }
    }

    /// Reads the current value of a float binding from AMX memory.
    fn read_float(&self, amx: &Amx, amx_addr: i32) -> f32 {
        match amx.get_ref::<f32>(amx_addr) {
            Ok(r) => *r,
            Err(_) => 0.0,
        }
    }

    /// Reads a string from AMX memory at the given address.
    fn read_string(&self, amx: &Amx, amx_addr: i32, max_len: i32) -> String {
        match amx.get_ref::<i32>(amx_addr) {
            Ok(r) => {
                let ptr = r.as_ptr();
                match amx.strlen(ptr) {
                    Ok(len) => {
                        let actual_len = len.min(max_len as usize);
                        let mut chars = Vec::with_capacity(actual_len);
                        for i in 0..actual_len {
                            let cell = unsafe { *ptr.add(i) };
                            if cell == 0 {
                                break;
                            }
                            chars.push(cell as u8 as char);
                        }
                        chars.into_iter().collect()
                    }
                    Err(_) => String::new(),
                }
            }
            Err(_) => String::new(),
        }
    }

    /// Reads a variable value as an SQL-safe string.
    fn read_var_as_sql(&self, amx: &Amx, var: &OrmVarBinding) -> String {
        match var {
            OrmVarBinding::Int { amx_addr, .. } => {
                format!("{}", self.read_int(amx, *amx_addr))
            }
            OrmVarBinding::Float { amx_addr, .. } => {
                format!("{}", self.read_float(amx, *amx_addr))
            }
            OrmVarBinding::String { amx_addr, max_len, .. } => {
                let raw = self.read_string(amx, *amx_addr, *max_len);
                format!("'{}'", escape_string(&raw))
            }
        }
    }

    /// Reads the key column's current value from AMX memory.
    fn read_key_value(&self, amx: &Amx) -> Option<String> {
        let key_col = self.key_column.as_ref()?;
        let var = self.variables.iter().find(|v| v.column_name() == key_col)?;
        Some(self.read_var_as_sql(amx, var))
    }

    /// Builds a SELECT query for this ORM instance.
    pub fn build_select(&self) -> Option<String> {
        let key = self.key_column.as_ref()?;
        let amx = get_amx(self.amx_ident)?;

        let columns: Vec<String> = self
            .variables
            .iter()
            .map(|v| format!("`{}`", escape_identifier(v.column_name())))
            .collect();

        if columns.is_empty() {
            return None;
        }

        let key_value = self.read_key_value(amx)?;

        Some(format!(
            "SELECT {} FROM `{}` WHERE `{}` = {}",
            columns.join(", "),
            escape_identifier(&self.table),
            escape_identifier(key),
            key_value
        ))
    }

    /// Builds an INSERT query for this ORM instance.
    pub fn build_insert(&self) -> Option<String> {
        let amx = get_amx(self.amx_ident)?;

        if self.variables.is_empty() {
            return None;
        }

        let mut cols = Vec::new();
        let mut vals = Vec::new();

        for var in &self.variables {
            cols.push(format!("`{}`", escape_identifier(var.column_name())));
            vals.push(self.read_var_as_sql(amx, var));
        }

        Some(format!(
            "INSERT INTO `{}` ({}) VALUES ({})",
            escape_identifier(&self.table),
            cols.join(", "),
            vals.join(", ")
        ))
    }

    /// Builds an UPDATE query for this ORM instance.
    pub fn build_update(&self) -> Option<String> {
        let key = self.key_column.as_ref()?;
        let amx = get_amx(self.amx_ident)?;

        if self.variables.is_empty() {
            return None;
        }

        let mut sets = Vec::new();
        for var in &self.variables {
            sets.push(format!(
                "`{}` = {}",
                escape_identifier(var.column_name()),
                self.read_var_as_sql(amx, var)
            ));
        }

        let key_value = self.read_key_value(amx)?;

        Some(format!(
            "UPDATE `{}` SET {} WHERE `{}` = {}",
            escape_identifier(&self.table),
            sets.join(", "),
            escape_identifier(key),
            key_value
        ))
    }

    /// Builds a DELETE query for this ORM instance.
    pub fn build_delete(&self) -> Option<String> {
        let key = self.key_column.as_ref()?;
        let amx = get_amx(self.amx_ident)?;
        let key_value = self.read_key_value(amx)?;

        Some(format!(
            "DELETE FROM `{}` WHERE `{}` = {}",
            escape_identifier(&self.table),
            escape_identifier(key),
            key_value
        ))
    }

    /// Checks if the key value is zero/empty (used by orm_save to decide INSERT vs UPDATE).
    pub fn is_key_empty(&self) -> bool {
        let amx = match get_amx(self.amx_ident) {
            Some(a) => a,
            None => return true,
        };

        let key_col = match &self.key_column {
            Some(k) => k,
            None => return true,
        };

        let var = match self.variables.iter().find(|v| v.column_name() == key_col) {
            Some(v) => v,
            None => return true,
        };

        match var {
            OrmVarBinding::Int { amx_addr, .. } => self.read_int(amx, *amx_addr) == 0,
            OrmVarBinding::Float { amx_addr, .. } => self.read_float(amx, *amx_addr) == 0.0,
            OrmVarBinding::String { amx_addr, max_len, .. } => {
                self.read_string(amx, *amx_addr, *max_len).is_empty()
            }
        }
    }

    /// Writes cache values into the bound Pawn variables.
    pub fn apply_cache(
        &self,
        amx: &Amx,
        cache: &crate::cache::CacheEntry,
        row: usize,
    ) {
        for var in &self.variables {
            let col_idx = match cache.field_index(var.column_name()) {
                Some(i) => i,
                None => continue,
            };

            let value = match cache.get_value(row, col_idx) {
                Some(Some(v)) => v,
                _ => continue,
            };

            match var {
                OrmVarBinding::Int { amx_addr, .. } => {
                    if let Ok(mut r) = amx.get_ref::<i32>(*amx_addr) {
                        *r = value.parse::<i32>().unwrap_or(0);
                    }
                }
                OrmVarBinding::Float { amx_addr, .. } => {
                    if let Ok(mut r) = amx.get_ref::<f32>(*amx_addr) {
                        *r = value.parse::<f32>().unwrap_or(0.0);
                    }
                }
                OrmVarBinding::String { amx_addr, max_len, .. } => {
                    // Clamp max_len to a safe upper bound to prevent OOB writes
                    let safe_max = (*max_len).clamp(0, 4096) as usize;
                    if safe_max == 0 {
                        continue;
                    }
                    if let Ok(mut r) = amx.get_ref::<i32>(*amx_addr) {
                        let ptr = r.as_mut_ptr();
                        let max = safe_max.saturating_sub(1);
                        let bytes = value.as_bytes();
                        let write_len = bytes.len().min(max);
                        for (i, &byte) in bytes.iter().enumerate().take(write_len) {
                            unsafe { *ptr.add(i) = byte as i32 };
                        }
                        unsafe { *ptr.add(write_len) = 0 }; // null terminator
                    }
                }
            }
        }
    }
}

/// Manages all ORM instances.
pub struct OrmManager {
    instances: HashMap<i32, OrmInstance>,
    next_id: i32,
}

impl OrmManager {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
            next_id: 1,
        }
    }

    /// Creates a new ORM instance. Returns the ORM ID (>= 1).
    pub fn create(&mut self, table: String, conn_id: i32, amx_ident: AmxIdent) -> i32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        self.instances
            .insert(id, OrmInstance::new(table, conn_id, amx_ident));
        id
    }

    /// Destroys an ORM instance.
    pub fn destroy(&mut self, id: i32) -> bool {
        self.instances.remove(&id).is_some()
    }

    /// Returns a reference to an ORM instance.
    pub fn get(&self, id: i32) -> Option<&OrmInstance> {
        self.instances.get(&id)
    }

    /// Returns a mutable reference to an ORM instance.
    pub fn get_mut(&mut self, id: i32) -> Option<&mut OrmInstance> {
        self.instances.get_mut(&id)
    }

    /// Destroys all ORM instances associated with a given AMX.
    pub fn destroy_by_amx(&mut self, ident: AmxIdent) {
        self.instances.retain(|_, inst| inst.amx_ident != ident);
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

    // OrmVarBinding tests

    #[test]
    fn var_binding_column_name_int() {
        let binding = OrmVarBinding::Int {
            amx_addr: 100,
            column: "score".to_string(),
        };
        assert_eq!(binding.column_name(), "score");
    }

    #[test]
    fn var_binding_column_name_float() {
        let binding = OrmVarBinding::Float {
            amx_addr: 200,
            column: "pos_x".to_string(),
        };
        assert_eq!(binding.column_name(), "pos_x");
    }

    #[test]
    fn var_binding_column_name_string() {
        let binding = OrmVarBinding::String {
            amx_addr: 300,
            max_len: 64,
            column: "name".to_string(),
        };
        assert_eq!(binding.column_name(), "name");
    }

    // OrmError tests

    #[test]
    fn orm_error_values() {
        assert_eq!(OrmError::Ok as i32, 0);
        assert_eq!(OrmError::NoData as i32, 1);
    }

    // OrmManager tests

    #[test]
    fn manager_create_incremental_ids() {
        let mut mgr = OrmManager::new();
        let id1 = mgr.create("users".to_string(), 1, dummy_ident());
        let id2 = mgr.create("vehicles".to_string(), 1, dummy_ident());
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn manager_get() {
        let mut mgr = OrmManager::new();
        let id = mgr.create("users".to_string(), 1, dummy_ident());
        let instance = mgr.get(id).unwrap();
        assert_eq!(instance.table, "users");
        assert_eq!(instance.conn_id, 1);
    }

    #[test]
    fn manager_get_nonexistent() {
        let mgr = OrmManager::new();
        assert!(mgr.get(999).is_none());
    }

    #[test]
    fn manager_get_mut() {
        let mut mgr = OrmManager::new();
        let id = mgr.create("users".to_string(), 1, dummy_ident());
        let instance = mgr.get_mut(id).unwrap();
        instance.key_column = Some("id".to_string());

        assert_eq!(mgr.get(id).unwrap().key_column.as_deref(), Some("id"));
    }

    #[test]
    fn manager_destroy() {
        let mut mgr = OrmManager::new();
        let id = mgr.create("users".to_string(), 1, dummy_ident());
        assert!(mgr.destroy(id));
        assert!(mgr.get(id).is_none());
    }

    #[test]
    fn manager_destroy_nonexistent() {
        let mut mgr = OrmManager::new();
        assert!(!mgr.destroy(999));
    }

    #[test]
    fn manager_destroy_by_amx() {
        let mut mgr = OrmManager::new();
        let id1 = mgr.create("users".to_string(), 1, dummy_ident());
        let id2 = mgr.create("vehicles".to_string(), 1, dummy_ident_2());
        let id3 = mgr.create("houses".to_string(), 1, dummy_ident());

        mgr.destroy_by_amx(dummy_ident());

        assert!(mgr.get(id1).is_none()); // destroyed
        assert!(mgr.get(id2).is_some()); // kept (different AMX)
        assert!(mgr.get(id3).is_none()); // destroyed
    }

    #[test]
    fn manager_wrapping_id() {
        let mut mgr = OrmManager::new();
        mgr.next_id = i32::MAX;
        let id1 = mgr.create("t1".to_string(), 1, dummy_ident());
        assert_eq!(id1, i32::MAX);
        let id2 = mgr.create("t2".to_string(), 1, dummy_ident());
        assert!(id2 >= 1);
    }

    #[test]
    fn orm_instance_defaults() {
        let inst = OrmInstance::new("players".to_string(), 1, dummy_ident());
        assert_eq!(inst.table, "players");
        assert_eq!(inst.conn_id, 1);
        assert!(inst.key_column.is_none());
        assert!(inst.variables.is_empty());
        assert_eq!(inst.errno, OrmError::Ok);
    }
}
