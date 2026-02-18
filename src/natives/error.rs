use samp::native;
use samp::prelude::*;

use crate::plugin::MysqlPlugin;

impl MysqlPlugin {
    #[native(name = "mysql_errno")]
    pub fn mysql_errno(&mut self, _amx: &Amx, conn_id: i32) -> AmxResult<i32> {
        let error = self.connections.get_error(conn_id);
        Ok(error.code.code())
    }
}
