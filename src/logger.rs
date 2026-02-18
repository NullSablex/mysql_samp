use std::fs::{self, OpenOptions};
use std::io::Write;

const LOG_DIR: &str = "logs";
const LOG_FILE: &str = "logs/mysql.log";
const PREFIX: &str = "[MySQL]";

pub struct Logger;

impl Logger {
    pub fn init() {
        let _ = fs::create_dir_all(LOG_DIR);
        Self::print_banner();
    }

    pub fn info(msg: &str) {
        log::info!("{} {}", PREFIX, msg);
    }

    pub fn warn(msg: &str) {
        log::warn!("{} {}", PREFIX, msg);
    }

    pub fn error(msg: &str) {
        log::error!("{} {}", PREFIX, msg);
        Self::write_file("ERROR", msg);
    }

    pub fn error_detail(console_msg: &str, detail: &str) {
        log::error!("{} {}", PREFIX, console_msg);
        Self::write_file("ERROR", detail);
    }

    fn print_banner() {
        let name = env!("CARGO_PKG_NAME");
        let version = env!("CARGO_PKG_VERSION");
        let author = env!("CARGO_PKG_AUTHORS");
        let repository = env!("CARGO_PKG_REPOSITORY");
        let build_date = env!("BUILD_DATE");
        let build_time = env!("BUILD_TIME");
        let build_year = env!("BUILD_YEAR");

        log::info!("");
        log::info!("  | {} {} | {}", name, version, build_year);
        log::info!("  |-------------------------------");
        log::info!("  | Author and maintainer: {}", value_or(author, "Unknown"));
        log::info!("");
        log::info!("  | Compiled: {} at {}", build_date, build_time);
        log::info!("  |-------------------------------");
        log::info!("  | Repository: {}", value_or(repository, "N/A"));
        log::info!("");
    }

    fn write_file(level: &str, message: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("[{}] [{}] {}\n", timestamp, level, message);

        let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(LOG_FILE)
        else {
            return;
        };

        let _ = file.write_all(line.as_bytes());
    }
}

fn value_or<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value
    }
}
