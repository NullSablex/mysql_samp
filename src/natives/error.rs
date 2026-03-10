use samp::native;
use samp::prelude::*;

use crate::plugin::MysqlPlugin;

impl MysqlPlugin {
    #[native(name = "mysql_errno")]
    pub fn mysql_errno(&mut self, _amx: &Amx, conn_id: i32) -> AmxResult<i32> {
        let error = self.connections.get_error(conn_id);
        Ok(error.code.code())
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
        let mut buf = dest.into_sized_buffer(dest_len);
        let _ = samp::cell::string::put_in_buffer(&mut buf, &msg);
        Ok(true)
    }
}
