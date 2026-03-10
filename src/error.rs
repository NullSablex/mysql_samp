#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MysqlError {
    Ok = 0,
    ConnectionFailed = 1,
    InvalidOptions = 2,
    InvalidConnection = 3,
    PingFailed = 4,
    QueryFailed = 5,
    NoCacheActive = 6,
    InvalidOrm = 7,
    OrmKeyNotSet = 8,
}

impl MysqlError {
    pub fn code(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone)]
pub struct ErrorState {
    pub code: MysqlError,
    pub message: String,
}

impl ErrorState {
    pub fn ok() -> Self {
        Self {
            code: MysqlError::Ok,
            message: String::new(),
        }
    }

    pub fn new(code: MysqlError, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_match_repr() {
        assert_eq!(MysqlError::Ok.code(), 0);
        assert_eq!(MysqlError::ConnectionFailed.code(), 1);
        assert_eq!(MysqlError::InvalidOptions.code(), 2);
        assert_eq!(MysqlError::InvalidConnection.code(), 3);
        assert_eq!(MysqlError::PingFailed.code(), 4);
        assert_eq!(MysqlError::QueryFailed.code(), 5);
        assert_eq!(MysqlError::NoCacheActive.code(), 6);
        assert_eq!(MysqlError::InvalidOrm.code(), 7);
        assert_eq!(MysqlError::OrmKeyNotSet.code(), 8);
    }

    #[test]
    fn error_state_ok() {
        let state = ErrorState::ok();
        assert_eq!(state.code, MysqlError::Ok);
        assert!(state.message.is_empty());
    }

    #[test]
    fn error_state_new_with_str() {
        let state = ErrorState::new(MysqlError::ConnectionFailed, "connection refused");
        assert_eq!(state.code, MysqlError::ConnectionFailed);
        assert_eq!(state.message, "connection refused");
    }

    #[test]
    fn error_state_new_with_string() {
        let msg = String::from("timeout exceeded");
        let state = ErrorState::new(MysqlError::PingFailed, msg);
        assert_eq!(state.code, MysqlError::PingFailed);
        assert_eq!(state.message, "timeout exceeded");
    }

    #[test]
    fn error_state_clone() {
        let state = ErrorState::new(MysqlError::QueryFailed, "syntax error");
        let cloned = state.clone();
        assert_eq!(cloned.code, MysqlError::QueryFailed);
        assert_eq!(cloned.message, "syntax error");
    }

    #[test]
    fn mysql_error_equality() {
        assert_eq!(MysqlError::Ok, MysqlError::Ok);
        assert_ne!(MysqlError::Ok, MysqlError::ConnectionFailed);
    }
}
