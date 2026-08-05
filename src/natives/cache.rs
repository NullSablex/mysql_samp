use samp::native;
use samp::prelude::*;

use crate::logger::Logger;
use crate::plugin::MysqlPlugin;

impl MysqlPlugin {
    #[native(name = "cache_get_row_count")]
    pub fn cache_get_row_count(&mut self, _amx: &Amx) -> i32 {
        self.cache.get_active().map_or(-1, |entry| {
            i32::try_from(entry.row_count()).unwrap_or(i32::MAX)
        })
    }

    #[native(name = "cache_get_field_count")]
    pub fn cache_get_field_count(&mut self, _amx: &Amx) -> i32 {
        self.cache.get_active().map_or(-1, |entry| {
            i32::try_from(entry.field_count()).unwrap_or(i32::MAX)
        })
    }

    #[native(name = "cache_get_field_name")]
    pub fn cache_get_field_name(
        &mut self,
        _amx: &Amx,
        field_idx: i32,
        dest: UnsizedBuffer,
        dest_len: usize,
    ) -> AmxResult<bool> {
        let Ok(idx) = usize::try_from(field_idx) else {
            return Ok(false);
        };
        let Some(entry) = self.cache.get_active() else {
            return Ok(false);
        };

        match entry.field_name(idx) {
            Some(name) => {
                dest.write_str(dest_len, name)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    #[native(name = "cache_get_value_index")]
    pub fn cache_get_value_index(
        &mut self,
        _amx: &Amx,
        row: i32,
        col: i32,
        dest: UnsizedBuffer,
        dest_len: usize,
    ) -> AmxResult<bool> {
        let (Ok(row_idx), Ok(col_idx)) = (usize::try_from(row), usize::try_from(col)) else {
            return Ok(false);
        };
        let Some(entry) = self.cache.get_active() else {
            return Ok(false);
        };

        match entry.get_value(row_idx, col_idx) {
            Some(Some(val)) => {
                dest.write_str(dest_len, val)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    #[native(name = "cache_get_value_index_int")]
    pub fn cache_get_value_index_int(&mut self, _amx: &Amx, row: i32, col: i32) -> i32 {
        let (Ok(row_idx), Ok(col_idx)) = (usize::try_from(row), usize::try_from(col)) else {
            return 0;
        };
        let Some(entry) = self.cache.get_active() else {
            return 0;
        };

        match entry.get_value(row_idx, col_idx) {
            Some(Some(val)) => val.parse::<i32>().unwrap_or(0),
            _ => 0,
        }
    }

    #[native(name = "cache_get_value_index_float")]
    pub fn cache_get_value_index_float(&mut self, _amx: &Amx, row: i32, col: i32) -> f32 {
        let (Ok(row_idx), Ok(col_idx)) = (usize::try_from(row), usize::try_from(col)) else {
            return 0.0;
        };
        let Some(entry) = self.cache.get_active() else {
            return 0.0;
        };

        match entry.get_value(row_idx, col_idx) {
            Some(Some(val)) => val.parse::<f32>().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    #[native(name = "cache_get_value_name")]
    pub fn cache_get_value_name(
        &mut self,
        _amx: &Amx,
        row: i32,
        field_name: &AmxString,
        dest: UnsizedBuffer,
        dest_len: usize,
    ) -> AmxResult<bool> {
        let Ok(row_idx) = usize::try_from(row) else {
            return Ok(false);
        };
        let Some(entry) = self.cache.get_active() else {
            return Ok(false);
        };
        let Some(col) = entry.field_index(field_name) else {
            return Ok(false);
        };

        match entry.get_value(row_idx, col) {
            Some(Some(val)) => {
                dest.write_str(dest_len, val)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    #[native(name = "cache_get_value_name_int")]
    pub fn cache_get_value_name_int(
        &mut self,
        _amx: &Amx,
        row: i32,
        field_name: &AmxString,
    ) -> i32 {
        let Ok(row_idx) = usize::try_from(row) else {
            return 0;
        };
        let Some(entry) = self.cache.get_active() else {
            return 0;
        };
        let Some(col) = entry.field_index(field_name) else {
            return 0;
        };

        match entry.get_value(row_idx, col) {
            Some(Some(val)) => val.parse::<i32>().unwrap_or(0),
            _ => 0,
        }
    }

    #[native(name = "cache_get_value_name_float")]
    pub fn cache_get_value_name_float(
        &mut self,
        _amx: &Amx,
        row: i32,
        field_name: &AmxString,
    ) -> f32 {
        let Ok(row_idx) = usize::try_from(row) else {
            return 0.0;
        };
        let Some(entry) = self.cache.get_active() else {
            return 0.0;
        };
        let Some(col) = entry.field_index(field_name) else {
            return 0.0;
        };

        match entry.get_value(row_idx, col) {
            Some(Some(val)) => val.parse::<f32>().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    #[native(name = "cache_is_value_index_null")]
    pub fn cache_is_value_index_null(&mut self, _amx: &Amx, row: i32, col: i32) -> bool {
        let (Ok(row_idx), Ok(col_idx)) = (usize::try_from(row), usize::try_from(col)) else {
            return true;
        };
        let Some(entry) = self.cache.get_active() else {
            return true;
        };

        match entry.get_value(row_idx, col_idx) {
            Some(None) | None => true,
            Some(Some(_)) => false,
        }
    }

    #[native(name = "cache_is_value_name_null")]
    pub fn cache_is_value_name_null(
        &mut self,
        _amx: &Amx,
        row: i32,
        field_name: &AmxString,
    ) -> bool {
        let Ok(row_idx) = usize::try_from(row) else {
            return true;
        };
        let Some(entry) = self.cache.get_active() else {
            return true;
        };
        let Some(col) = entry.field_index(field_name) else {
            return true;
        };

        match entry.get_value(row_idx, col) {
            Some(None) | None => true,
            Some(Some(_)) => false,
        }
    }

    #[native(name = "cache_affected_rows")]
    pub fn cache_affected_rows(&mut self, _amx: &Amx) -> i32 {
        self.cache.get_active().map_or(-1, |entry| {
            i32::try_from(entry.affected_rows()).unwrap_or(i32::MAX)
        })
    }

    #[native(name = "cache_insert_id")]
    pub fn cache_insert_id(&mut self, _amx: &Amx) -> i32 {
        self.cache.get_active().map_or(-1, |entry| {
            i32::try_from(entry.insert_id()).unwrap_or(i32::MAX)
        })
    }

    #[native(name = "cache_get_query_exec_time")]
    pub fn cache_get_query_exec_time(&mut self, _amx: &Amx) -> i32 {
        self.cache.get_active().map_or(-1, |entry| {
            i32::try_from(entry.exec_time_ms()).unwrap_or(i32::MAX)
        })
    }

    #[native(name = "cache_get_query_string")]
    pub fn cache_get_query_string(
        &mut self,
        _amx: &Amx,
        dest: UnsizedBuffer,
        dest_len: usize,
    ) -> AmxResult<bool> {
        let Some(entry) = self.cache.get_active() else {
            return Ok(false);
        };

        let query = entry.query_string().to_string();
        dest.write_str(dest_len, &query)?;
        Ok(true)
    }

    /// cache_get_result_count()
    ///
    /// Number of result sets in the active cache. A plain query yields 1; a
    /// script or a stored-procedure `CALL` can yield several.
    #[native(name = "cache_get_result_count")]
    pub fn cache_get_result_count(&mut self, _amx: &Amx) -> i32 {
        let Some(cache) = self.cache.get_active() else {
            return 0;
        };
        i32::try_from(cache.result_count()).unwrap_or(i32::MAX)
    }

    /// cache_set_result(result_index)
    ///
    /// Selects which result set the `cache_*` readers report on. The first set
    /// (index 0) is selected until this is called.
    #[native(name = "cache_set_result")]
    pub fn cache_set_result(&mut self, _amx: &Amx, result_index: i32) -> bool {
        let Ok(index) = usize::try_from(result_index) else {
            return false;
        };
        let Some(cache) = self.cache.get_active_mut() else {
            Logger::warn("cache_set_result failed: no active cache.");
            return false;
        };
        cache.set_result(index)
    }

    #[native(name = "cache_save")]
    pub fn cache_save(&mut self, _amx: &Amx) -> i32 {
        self.cache.save()
    }

    #[native(name = "cache_delete")]
    pub fn cache_delete(&mut self, _amx: &Amx, cache_id: i32) -> bool {
        self.cache.delete(cache_id)
    }

    #[native(name = "cache_set_active")]
    pub fn cache_set_active(&mut self, _amx: &Amx, cache_id: i32) -> bool {
        self.cache.set_active(cache_id)
    }

    #[native(name = "cache_unset_active")]
    pub fn cache_unset_active(&mut self, _amx: &Amx) -> bool {
        self.cache.unset_active()
    }

    #[native(name = "cache_is_any_active")]
    pub fn cache_is_any_active(&mut self, _amx: &Amx) -> bool {
        self.cache.is_any_active()
    }

    #[native(name = "cache_is_valid")]
    pub fn cache_is_valid(&mut self, _amx: &Amx, cache_id: i32) -> bool {
        self.cache.is_valid(cache_id)
    }

    #[native(name = "cache_warning_count")]
    pub fn cache_warning_count(&mut self, _amx: &Amx) -> i32 {
        self.cache
            .get_active()
            .map_or(-1, |entry| i32::from(entry.warning_count()))
    }

    #[native(name = "cache_get_field_type")]
    pub fn cache_get_field_type(&mut self, _amx: &Amx, field_idx: i32) -> i32 {
        let Ok(idx) = usize::try_from(field_idx) else {
            return -1;
        };
        let Some(entry) = self.cache.get_active() else {
            return -1;
        };

        match entry.field_type(idx) {
            Some(t) => i32::from(t),
            None => -1,
        }
    }
}
