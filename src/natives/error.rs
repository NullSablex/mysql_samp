use samp::native;
use samp::prelude::*;

use crate::plugin::MysqlPlugin;

impl MysqlPlugin {
    #[native(name = "mysql_errno")]
    pub fn mysql_errno(&mut self, _amx: &Amx, conn_id: i32) -> i32 {
        self.connections.get_error(conn_id).code.code()
    }

    #[native(name = "mysql_error")]
    pub fn mysql_error(
        &mut self,
        _amx: &Amx,
        conn_id: i32,
        dest: UnsizedBuffer,
        dest_len: usize,
    ) -> AmxResult<bool> {
        let error = self.connections.get_error(conn_id);
        let msg = error.message.clone();
        dest.write_str(dest_len, &msg)?;
        Ok(true)
    }
}
