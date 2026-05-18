use samp::native;
use samp::prelude::*;

use crate::options::MysqlOptionKind;
use crate::plugin::MysqlPlugin;

impl MysqlPlugin {
    #[native(name = "mysql_options_new")]
    pub fn mysql_options_new(&mut self, _amx: &Amx) -> i32 {
        self.options.create()
    }

    #[native(name = "mysql_options_set_int")]
    pub fn mysql_options_set_int(
        &mut self,
        _amx: &Amx,
        handle: i32,
        option: i32,
        value: i32,
    ) -> bool {
        match MysqlOptionKind::from_i32(option) {
            Some(kind) => self.options.set_int(handle, kind, value),
            None => false,
        }
    }

    #[native(name = "mysql_options_set_str")]
    pub fn mysql_options_set_str(
        &mut self,
        _amx: &Amx,
        handle: i32,
        option: i32,
        value: &AmxString,
    ) -> bool {
        match MysqlOptionKind::from_i32(option) {
            Some(kind) => self.options.set_str(handle, kind, value.to_string()),
            None => false,
        }
    }
}
