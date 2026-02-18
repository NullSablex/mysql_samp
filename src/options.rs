use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MysqlOptions {
    pub port: u16,
    pub ssl: bool,
    pub ssl_ca: Option<String>,
    pub connect_timeout: Option<u32>,
}

impl Default for MysqlOptions {
    fn default() -> Self {
        Self {
            port: 3306,
            ssl: false,
            ssl_ca: None,
            connect_timeout: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MysqlOptionKind {
    Port = 0,
    Ssl = 1,
    SslCa = 2,
    ConnectTimeout = 3,
}

impl MysqlOptionKind {
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(Self::Port),
            1 => Some(Self::Ssl),
            2 => Some(Self::SslCa),
            3 => Some(Self::ConnectTimeout),
            _ => None,
        }
    }
}

pub struct OptionsManager {
    options: HashMap<i32, MysqlOptions>,
    next_id: i32,
}

impl OptionsManager {
    pub fn new() -> Self {
        Self {
            options: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create(&mut self) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        self.options.insert(id, MysqlOptions::default());
        id
    }

    pub fn get(&self, id: i32) -> Option<&MysqlOptions> {
        self.options.get(&id)
    }

    pub fn set_int(&mut self, id: i32, option: MysqlOptionKind, value: i32) -> bool {
        let Some(opts) = self.options.get_mut(&id) else {
            return false;
        };

        match option {
            MysqlOptionKind::Port => opts.port = value as u16,
            MysqlOptionKind::Ssl => opts.ssl = value != 0,
            MysqlOptionKind::ConnectTimeout => opts.connect_timeout = Some(value as u32),
            _ => return false,
        }

        true
    }

    pub fn set_str(&mut self, id: i32, option: MysqlOptionKind, value: String) -> bool {
        let Some(opts) = self.options.get_mut(&id) else {
            return false;
        };

        match option {
            MysqlOptionKind::SslCa => opts.ssl_ca = Some(value),
            _ => return false,
        }

        true
    }
}
