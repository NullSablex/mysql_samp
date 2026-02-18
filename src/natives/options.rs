use samp::native;
use samp::prelude::*;

use crate::options::MysqlOptionKind;
use crate::plugin::MysqlPlugin;

impl MysqlPlugin {
    #[native(name = "mysql_options_new")]
    pub fn mysql_options_new(&mut self, _amx: &Amx) -> AmxResult<i32> {
        let id = self.options.create();
        Ok(id)
    }

    #[native(name = "mysql_options_set_int")]
    pub fn mysql_options_set_int(
        &mut self,
        _amx: &Amx,
        handle: i32,
        option: i32,
        value: i32,
    ) -> AmxResult<bool> {
        match MysqlOptionKind::from_i32(option) {
            Some(kind) => Ok(self.options.set_int(handle, kind, value)),
            None => Ok(false),
        }
    }

    #[native(name = "mysql_options_set_str")]
    pub fn mysql_options_set_str(
        &mut self,
        _amx: &Amx,
        handle: i32,
        option: i32,
        value: AmxString,
    ) -> AmxResult<bool> {
        match MysqlOptionKind::from_i32(option) {
            Some(kind) => Ok(self.options.set_str(handle, kind, value.to_string())),
            None => Ok(false),
        }
    }
}
