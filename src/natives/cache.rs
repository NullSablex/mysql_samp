use samp::native;
use samp::prelude::*;

use crate::plugin::MysqlPlugin;

impl MysqlPlugin {
    #[native(name = "cache_get_row_count")]
    pub fn cache_get_row_count(&mut self, _amx: &Amx) -> AmxResult<i32> {
        match self.cache.get_active() {
            Some(entry) => Ok(entry.row_count() as i32),
            None => Ok(-1),
        }
    }

    #[native(name = "cache_get_field_count")]
    pub fn cache_get_field_count(&mut self, _amx: &Amx) -> AmxResult<i32> {
        match self.cache.get_active() {
            Some(entry) => Ok(entry.field_count() as i32),
            None => Ok(-1),
        }
    }

    #[native(name = "cache_get_field_name")]
    pub fn cache_get_field_name(
        &mut self,
        _amx: &Amx,
        field_idx: i32,
        dest: UnsizedBuffer,
        dest_len: usize,
    ) -> AmxResult<bool> {
        let entry = match self.cache.get_active() {
            Some(e) => e,
            None => return Ok(false),
        };

        match entry.field_name(field_idx as usize) {
            Some(name) => {
                let mut buf = dest.into_sized_buffer(dest_len);
                let _ = samp::cell::string::put_in_buffer(&mut buf, name);
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
        let entry = match self.cache.get_active() {
            Some(e) => e,
            None => return Ok(false),
        };

        match entry.get_value(row as usize, col as usize) {
            Some(Some(val)) => {
                let mut buf = dest.into_sized_buffer(dest_len);
                let _ = samp::cell::string::put_in_buffer(&mut buf, val);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    #[native(name = "cache_get_value_index_int")]
    pub fn cache_get_value_index_int(
        &mut self,
        _amx: &Amx,
        row: i32,
        col: i32,
    ) -> AmxResult<i32> {
        let entry = match self.cache.get_active() {
            Some(e) => e,
            None => return Ok(0),
        };

        match entry.get_value(row as usize, col as usize) {
            Some(Some(val)) => Ok(val.parse::<i32>().unwrap_or(0)),
            _ => Ok(0),
        }
    }

    #[native(name = "cache_get_value_index_float")]
    pub fn cache_get_value_index_float(
        &mut self,
        _amx: &Amx,
        row: i32,
        col: i32,
    ) -> AmxResult<f32> {
        let entry = match self.cache.get_active() {
            Some(e) => e,
            None => return Ok(0.0),
        };

        match entry.get_value(row as usize, col as usize) {
            Some(Some(val)) => Ok(val.parse::<f32>().unwrap_or(0.0)),
            _ => Ok(0.0),
        }
    }

    #[native(name = "cache_get_value_name")]
    pub fn cache_get_value_name(
        &mut self,
        _amx: &Amx,
        row: i32,
        field_name: AmxString,
        dest: UnsizedBuffer,
        dest_len: usize,
    ) -> AmxResult<bool> {
        let name = field_name.to_string();
        let entry = match self.cache.get_active() {
            Some(e) => e,
            None => return Ok(false),
        };

        let col = match entry.field_index(&name) {
            Some(i) => i,
            None => return Ok(false),
        };

        match entry.get_value(row as usize, col) {
            Some(Some(val)) => {
                let mut buf = dest.into_sized_buffer(dest_len);
                let _ = samp::cell::string::put_in_buffer(&mut buf, val);
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
        field_name: AmxString,
    ) -> AmxResult<i32> {
        let name = field_name.to_string();
        let entry = match self.cache.get_active() {
            Some(e) => e,
            None => return Ok(0),
        };

        let col = match entry.field_index(&name) {
            Some(i) => i,
            None => return Ok(0),
        };

        match entry.get_value(row as usize, col) {
            Some(Some(val)) => Ok(val.parse::<i32>().unwrap_or(0)),
            _ => Ok(0),
        }
    }

    #[native(name = "cache_get_value_name_float")]
    pub fn cache_get_value_name_float(
        &mut self,
        _amx: &Amx,
        row: i32,
        field_name: AmxString,
    ) -> AmxResult<f32> {
        let name = field_name.to_string();
        let entry = match self.cache.get_active() {
            Some(e) => e,
            None => return Ok(0.0),
        };

        let col = match entry.field_index(&name) {
            Some(i) => i,
            None => return Ok(0.0),
        };

        match entry.get_value(row as usize, col) {
            Some(Some(val)) => Ok(val.parse::<f32>().unwrap_or(0.0)),
            _ => Ok(0.0),
        }
    }

    #[native(name = "cache_is_value_index_null")]
    pub fn cache_is_value_index_null(
        &mut self,
        _amx: &Amx,
        row: i32,
        col: i32,
    ) -> AmxResult<bool> {
        let entry = match self.cache.get_active() {
            Some(e) => e,
            None => return Ok(true),
        };

        match entry.get_value(row as usize, col as usize) {
            Some(None) => Ok(true),
            Some(Some(_)) => Ok(false),
            None => Ok(true),
        }
    }

    #[native(name = "cache_is_value_name_null")]
    pub fn cache_is_value_name_null(
        &mut self,
        _amx: &Amx,
        row: i32,
        field_name: AmxString,
    ) -> AmxResult<bool> {
        let name = field_name.to_string();
        let entry = match self.cache.get_active() {
            Some(e) => e,
            None => return Ok(true),
        };

        let col = match entry.field_index(&name) {
            Some(i) => i,
            None => return Ok(true),
        };

        match entry.get_value(row as usize, col) {
            Some(None) => Ok(true),
            Some(Some(_)) => Ok(false),
            None => Ok(true),
        }
    }

    #[native(name = "cache_affected_rows")]
    pub fn cache_affected_rows(&mut self, _amx: &Amx) -> AmxResult<i32> {
        match self.cache.get_active() {
            Some(entry) => Ok(entry.affected_rows() as i32),
            None => Ok(-1),
        }
    }

    #[native(name = "cache_insert_id")]
    pub fn cache_insert_id(&mut self, _amx: &Amx) -> AmxResult<i32> {
        match self.cache.get_active() {
            Some(entry) => Ok(entry.insert_id() as i32),
            None => Ok(-1),
        }
    }

    #[native(name = "cache_get_query_exec_time")]
    pub fn cache_get_query_exec_time(&mut self, _amx: &Amx) -> AmxResult<i32> {
        match self.cache.get_active() {
            Some(entry) => Ok(entry.exec_time_ms() as i32),
            None => Ok(-1),
        }
    }

    #[native(name = "cache_get_query_string")]
    pub fn cache_get_query_string(
        &mut self,
        _amx: &Amx,
        dest: UnsizedBuffer,
        dest_len: usize,
    ) -> AmxResult<bool> {
        let entry = match self.cache.get_active() {
            Some(e) => e,
            None => return Ok(false),
        };

        let query = entry.query_string().to_string();
        let mut buf = dest.into_sized_buffer(dest_len);
        let _ = samp::cell::string::put_in_buffer(&mut buf, &query);
        Ok(true)
    }

    #[native(name = "cache_save")]
    pub fn cache_save(&mut self, _amx: &Amx) -> AmxResult<i32> {
        Ok(self.cache.save())
    }

    #[native(name = "cache_delete")]
    pub fn cache_delete(&mut self, _amx: &Amx, cache_id: i32) -> AmxResult<bool> {
        Ok(self.cache.delete(cache_id))
    }

    #[native(name = "cache_set_active")]
    pub fn cache_set_active(&mut self, _amx: &Amx, cache_id: i32) -> AmxResult<bool> {
        Ok(self.cache.set_active(cache_id))
    }

    #[native(name = "cache_unset_active")]
    pub fn cache_unset_active(&mut self, _amx: &Amx) -> AmxResult<bool> {
        Ok(self.cache.unset_active())
    }

    #[native(name = "cache_is_any_active")]
    pub fn cache_is_any_active(&mut self, _amx: &Amx) -> AmxResult<bool> {
        Ok(self.cache.is_any_active())
    }

    #[native(name = "cache_is_valid")]
    pub fn cache_is_valid(&mut self, _amx: &Amx, cache_id: i32) -> AmxResult<bool> {
        Ok(self.cache.is_valid(cache_id))
    }

    #[native(name = "cache_warning_count")]
    pub fn cache_warning_count(&mut self, _amx: &Amx) -> AmxResult<i32> {
        match self.cache.get_active() {
            Some(entry) => Ok(entry.warning_count() as i32),
            None => Ok(-1),
        }
    }

    #[native(name = "cache_get_field_type")]
    pub fn cache_get_field_type(&mut self, _amx: &Amx, field_idx: i32) -> AmxResult<i32> {
        let entry = match self.cache.get_active() {
            Some(e) => e,
            None => return Ok(-1),
        };

        match entry.field_type(field_idx as usize) {
            Some(t) => Ok(t as i32),
            None => Ok(-1),
        }
    }
}
