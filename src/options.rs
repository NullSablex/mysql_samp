use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MysqlOptions {
    pub port: u16,
    pub ssl: bool,
    pub ssl_ca: Option<String>,
    /// Client certificate chain for mutual TLS. Requires [`Self::ssl_key`].
    pub ssl_cert: Option<String>,
    /// Client private key for mutual TLS. Requires [`Self::ssl_cert`].
    pub ssl_key: Option<String>,
    /// When false, the server certificate is accepted even if expired or
    /// untrusted. Dangerous — see `MYSQL_OPT_SSL_VERIFY_CERT`.
    pub ssl_verify_cert: bool,
    pub connect_timeout: Option<u32>,
    pub auto_reconnect: bool,
}

impl Default for MysqlOptions {
    fn default() -> Self {
        Self {
            port: 3306,
            ssl: false,
            ssl_ca: None,
            ssl_cert: None,
            ssl_key: None,
            ssl_verify_cert: true,
            connect_timeout: None,
            auto_reconnect: true,
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
    AutoReconnect = 4,
    SslCert = 5,
    SslKey = 6,
    SslVerifyCert = 7,
}

impl MysqlOptionKind {
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(Self::Port),
            1 => Some(Self::Ssl),
            2 => Some(Self::SslCa),
            3 => Some(Self::ConnectTimeout),
            4 => Some(Self::AutoReconnect),
            5 => Some(Self::SslCert),
            6 => Some(Self::SslKey),
            7 => Some(Self::SslVerifyCert),
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
        self.next_id = self.next_id.wrapping_add(1).max(1);
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
            MysqlOptionKind::Port => {
                let Ok(port) = u16::try_from(value) else {
                    return false;
                };
                opts.port = port;
            }
            MysqlOptionKind::Ssl => opts.ssl = value != 0,
            MysqlOptionKind::ConnectTimeout => {
                let Ok(timeout) = u32::try_from(value) else {
                    return false;
                };
                opts.connect_timeout = Some(timeout);
            }
            MysqlOptionKind::AutoReconnect => opts.auto_reconnect = value != 0,
            MysqlOptionKind::SslVerifyCert => opts.ssl_verify_cert = value != 0,
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
            MysqlOptionKind::SslCert => opts.ssl_cert = Some(value),
            MysqlOptionKind::SslKey => opts.ssl_key = Some(value),
            _ => return false,
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // MysqlOptionKind tests

    #[test]
    fn option_kind_from_valid_values() {
        assert_eq!(MysqlOptionKind::from_i32(0), Some(MysqlOptionKind::Port));
        assert_eq!(MysqlOptionKind::from_i32(1), Some(MysqlOptionKind::Ssl));
        assert_eq!(MysqlOptionKind::from_i32(2), Some(MysqlOptionKind::SslCa));
        assert_eq!(
            MysqlOptionKind::from_i32(3),
            Some(MysqlOptionKind::ConnectTimeout)
        );
        assert_eq!(
            MysqlOptionKind::from_i32(4),
            Some(MysqlOptionKind::AutoReconnect)
        );
    }

    #[test]
    fn option_kind_from_invalid_values() {
        assert_eq!(MysqlOptionKind::from_i32(-1), None);
        assert_eq!(MysqlOptionKind::from_i32(8), None);
        assert_eq!(MysqlOptionKind::from_i32(100), None);
    }

    #[test]
    fn option_kind_covers_tls_options() {
        assert_eq!(MysqlOptionKind::from_i32(5), Some(MysqlOptionKind::SslCert));
        assert_eq!(MysqlOptionKind::from_i32(6), Some(MysqlOptionKind::SslKey));
        assert_eq!(
            MysqlOptionKind::from_i32(7),
            Some(MysqlOptionKind::SslVerifyCert)
        );
    }

    #[test]
    fn tls_options_use_the_right_setter() {
        let mut mgr = OptionsManager::new();
        let id = mgr.create();

        assert!(mgr.set_str(id, MysqlOptionKind::SslCert, "cert.pem".into()));
        assert!(mgr.set_str(id, MysqlOptionKind::SslKey, "key.pem".into()));
        assert!(mgr.set_int(id, MysqlOptionKind::SslVerifyCert, 0));

        // Mismatched setters are rejected rather than silently ignored.
        assert!(!mgr.set_int(id, MysqlOptionKind::SslCert, 1));
        assert!(!mgr.set_str(id, MysqlOptionKind::SslVerifyCert, "yes".into()));

        let opts = mgr.get(id).expect("options exist");
        assert_eq!(opts.ssl_cert.as_deref(), Some("cert.pem"));
        assert_eq!(opts.ssl_key.as_deref(), Some("key.pem"));
        assert!(!opts.ssl_verify_cert);
    }

    #[test]
    fn tls_verification_is_on_by_default() {
        assert!(MysqlOptions::default().ssl_verify_cert);
    }

    // MysqlOptions tests

    #[test]
    fn default_options() {
        let opts = MysqlOptions::default();
        assert_eq!(opts.port, 3306);
        assert!(!opts.ssl);
        assert!(opts.ssl_ca.is_none());
        assert!(opts.connect_timeout.is_none());
        assert!(opts.auto_reconnect);
    }

    // OptionsManager tests

    #[test]
    fn create_returns_incremental_ids() {
        let mut mgr = OptionsManager::new();
        assert_eq!(mgr.create(), 1);
        assert_eq!(mgr.create(), 2);
        assert_eq!(mgr.create(), 3);
    }

    #[test]
    fn get_returns_created_options() {
        let mut mgr = OptionsManager::new();
        let id = mgr.create();
        let opts = mgr.get(id).unwrap();
        assert_eq!(opts.port, 3306);
    }

    #[test]
    fn get_returns_none_for_invalid_id() {
        let mgr = OptionsManager::new();
        assert!(mgr.get(999).is_none());
        assert!(mgr.get(0).is_none());
    }

    #[test]
    fn set_int_port() {
        let mut mgr = OptionsManager::new();
        let id = mgr.create();
        assert!(mgr.set_int(id, MysqlOptionKind::Port, 3307));
        assert_eq!(mgr.get(id).unwrap().port, 3307);
    }

    #[test]
    fn set_int_ssl() {
        let mut mgr = OptionsManager::new();
        let id = mgr.create();
        assert!(mgr.set_int(id, MysqlOptionKind::Ssl, 1));
        assert!(mgr.get(id).unwrap().ssl);

        assert!(mgr.set_int(id, MysqlOptionKind::Ssl, 0));
        assert!(!mgr.get(id).unwrap().ssl);
    }

    #[test]
    fn set_int_connect_timeout() {
        let mut mgr = OptionsManager::new();
        let id = mgr.create();
        assert!(mgr.set_int(id, MysqlOptionKind::ConnectTimeout, 10));
        assert_eq!(mgr.get(id).unwrap().connect_timeout, Some(10));
    }

    #[test]
    fn set_int_auto_reconnect() {
        let mut mgr = OptionsManager::new();
        let id = mgr.create();
        assert!(mgr.set_int(id, MysqlOptionKind::AutoReconnect, 0));
        assert!(!mgr.get(id).unwrap().auto_reconnect);
        assert!(mgr.set_int(id, MysqlOptionKind::AutoReconnect, 1));
        assert!(mgr.get(id).unwrap().auto_reconnect);
    }

    #[test]
    fn set_int_rejects_ssl_ca() {
        let mut mgr = OptionsManager::new();
        let id = mgr.create();
        assert!(!mgr.set_int(id, MysqlOptionKind::SslCa, 1));
    }

    #[test]
    fn set_int_invalid_id() {
        let mut mgr = OptionsManager::new();
        assert!(!mgr.set_int(999, MysqlOptionKind::Port, 3307));
    }

    #[test]
    fn set_str_ssl_ca() {
        let mut mgr = OptionsManager::new();
        let id = mgr.create();
        assert!(mgr.set_str(id, MysqlOptionKind::SslCa, "/path/to/ca.pem".to_string()));
        assert_eq!(
            mgr.get(id).unwrap().ssl_ca.as_deref(),
            Some("/path/to/ca.pem")
        );
    }

    #[test]
    fn set_str_rejects_port() {
        let mut mgr = OptionsManager::new();
        let id = mgr.create();
        assert!(!mgr.set_str(id, MysqlOptionKind::Port, "3307".to_string()));
    }

    #[test]
    fn set_str_rejects_ssl() {
        let mut mgr = OptionsManager::new();
        let id = mgr.create();
        assert!(!mgr.set_str(id, MysqlOptionKind::Ssl, "true".to_string()));
    }

    #[test]
    fn set_str_rejects_connect_timeout() {
        let mut mgr = OptionsManager::new();
        let id = mgr.create();
        assert!(!mgr.set_str(id, MysqlOptionKind::ConnectTimeout, "10".to_string()));
    }

    #[test]
    fn set_str_invalid_id() {
        let mut mgr = OptionsManager::new();
        assert!(!mgr.set_str(999, MysqlOptionKind::SslCa, "ca.pem".to_string()));
    }

    #[test]
    fn wrapping_id_overflow() {
        let mut mgr = OptionsManager::new();
        mgr.next_id = i32::MAX;
        let id1 = mgr.create();
        assert_eq!(id1, i32::MAX);
        let id2 = mgr.create();
        assert!(id2 >= 1); // wraps to positive value, never 0
    }
}
