use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

const LOG_DIR: &str = "logs";
const LOG_FILE: &str = "logs/mysql.log";
const PREFIX: &str = "[MySQL]";

/// Log level: 0=none, 1=error, 2=warning, 3=info, 4=all (default)
static LOG_LEVEL: AtomicI32 = AtomicI32::new(4);

/// Set once if writing to `logs/mysql.log` fails — used to emit a single
/// console error instead of silently dropping every subsequent log line.
static FILE_WRITE_REPORTED: AtomicBool = AtomicBool::new(false);

pub struct Logger;

impl Logger {
    pub fn init() {
        let _ = fs::create_dir_all(LOG_DIR);
        Self::print_banner();
    }

    pub fn set_log_level(level: i32) {
        LOG_LEVEL.store(level.clamp(0, 4), Ordering::Relaxed);
    }

    pub fn info(msg: &str) {
        if LOG_LEVEL.load(Ordering::Relaxed) >= 3 {
            samp::log::info!("{} {}", PREFIX, msg);
            Self::write_file("INFO", msg);
        }
    }

    pub fn warn(msg: &str) {
        if LOG_LEVEL.load(Ordering::Relaxed) >= 2 {
            samp::log::warn!("{} {}", PREFIX, msg);
            Self::write_file("WARNING", msg);
        }
    }

    pub fn error(msg: &str) {
        if LOG_LEVEL.load(Ordering::Relaxed) >= 1 {
            samp::log::error!("{} {}", PREFIX, msg);
            Self::write_file("ERROR", msg);
        }
    }

    pub fn error_detail(console_msg: &str, detail: &str) {
        if LOG_LEVEL.load(Ordering::Relaxed) >= 1 {
            samp::log::error!("{} {}", PREFIX, console_msg);
            Self::write_file("ERROR", detail);
        }
    }

    fn print_banner() {
        let name = env!("CARGO_PKG_NAME");
        let version = env!("CARGO_PKG_VERSION");
        let author = env!("CARGO_PKG_AUTHORS");
        let repository = env!("CARGO_PKG_REPOSITORY");
        let build_date = env!("BUILD_DATE");
        let build_time = env!("BUILD_TIME");
        let build_year = env!("BUILD_YEAR");

        samp::log::info!("");
        samp::log::info!("  | {} {} | {}", name, version, build_year);
        samp::log::info!("  |-------------------------------");
        samp::log::info!("  | Author and maintainer: {}", value_or(author, "Unknown"));
        samp::log::info!("");
        samp::log::info!("  | Compiled: {} at {}", build_date, build_time);
        samp::log::info!("  |-------------------------------");
        samp::log::info!("  | Repository: {}", value_or(repository, "N/A"));
        samp::log::info!("");
    }

    fn write_file(level: &str, message: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("[{}] [{}] {}\n", timestamp, level, message);

        let result = OpenOptions::new()
            .create(true)
            .append(true)
            .open(LOG_FILE)
            .and_then(|mut file| file.write_all(line.as_bytes()));

        if let Err(err) = result
            && !FILE_WRITE_REPORTED.swap(true, Ordering::Relaxed)
        {
            samp::log::error!(
                "{} Failed to write {}: {}. Further file-write errors will be suppressed.",
                PREFIX,
                LOG_FILE,
                err
            );
        }
    }
}

fn value_or<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value
    }
}
