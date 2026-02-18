#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MysqlError {
    Ok = 0,
    ConnectionFailed = 1,
    InvalidOptions = 2,
    InvalidConnection = 3,
    PingFailed = 4,
    Unknown = 5,
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
