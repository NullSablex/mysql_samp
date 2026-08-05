use std::fmt;
use std::sync::OnceLock;

use samp::log::{Level, LevelFilter, Log, Record};
use samp::logger::{BannerMetadata, LoggerConfig};

const LOG_DIR: &str = "logs";
const LOG_FILE: &str = "mysql.log";
const PREFIX: &str = "[MySQL]";

/// Console-only sink: `samp::plugin::logger()` is a `fern::Dispatch` already
/// chained to the server's log output (SA-MP `logprintf` / open.mp
/// `ICore::logLnU8`). Turning it into a `Box<dyn Log>` instead of applying it
/// globally keeps it addressable on its own, so a message can go to the
/// console without also going to `logs/mysql.log` — and vice versa. That split
/// is what lets `error_detail` keep sensitive detail out of the console.
///
/// Calling `samp::plugin::logger()` also disables the SDK's default routing,
/// leaving the global `log` facade free for the file pipeline below.
static CONSOLE: OnceLock<Box<dyn Log>> = OnceLock::new();

pub struct Logger;

impl Logger {
    pub fn init() {
        let (_, console) = samp::plugin::logger()
            .format(|out, message, _record| out.finish(format_args!("{PREFIX} {message}")))
            .into_log();
        let _ = CONSOLE.set(console);

        // The SDK logger owns the global `log` facade and writes only to
        // `logs/mysql.log` — size-based rotation into `logs/archive/`, its own
        // timestamps, and a single console report on I/O failure, all of which
        // used to be hand-rolled here. `from_env` lets operators override the
        // level, directory, filename, rotation and banner via
        // `MYSQL_SAMP_LOG_*` without a rebuild.
        let installed = samp::enable_logger_with!(
            LoggerConfig::new(env!("CARGO_PKG_NAME"))
                .directory(LOG_DIR)
                .filename(LOG_FILE)
                .prefix(PREFIX)
                .level(LevelFilter::Trace)
                // The SDK drives the banner: it captures `CARGO_PKG_*` in the
                // macro above and calls this at the end of `install`. We only
                // add the build stamp and mirror the lines to the console —
                // the returned lines go to the file through the SDK's own
                // `log::info!`, which `also_to_server(false)` keeps off the
                // console. A banner in the file is useful on its own: it marks
                // where each restart begins in a rotated log.
                .banner_with(|meta| {
                    let lines = banner_lines(meta);
                    for line in &lines {
                        Logger::to_console(Level::Info, format_args!("{line}"));
                    }
                    lines
                })
                // Rotated archives are gzipped: log text compresses ~10x and a
                // busy server produces a lot of it. Placed before `from_env` so
                // an operator can turn it off with MYSQL_SAMP_LOG_COMPRESS=0.
                .compress_archives(true)
                .from_env()
                // After `from_env` on purpose: `MYSQL_SAMP_LOG_SERVER=1` would
                // echo every record to the console — duplicating the lines we
                // already print and leaking the detail that `error_detail`
                // deliberately keeps in the file.
                .also_to_server(false)
        );

        if let Err(err) = installed {
            Self::to_console(
                Level::Error,
                format_args!("Failed to install the file logger: {err}. Logging to console only."),
            );
        }
    }

    /// Flushes the log file. Called on unload so nothing is lost if the
    /// process goes away before the OS syncs the handle.
    pub fn flush() {
        samp::logger::flush();
    }

    pub fn set_log_level(level: i32) {
        samp::logger::set_level(i32_to_level(level.clamp(0, 4)));
    }

    pub fn info(msg: &str) {
        Self::to_console(Level::Info, format_args!("{msg}"));
        samp::log::info!("{msg}");
    }

    pub fn warn(msg: &str) {
        Self::to_console(Level::Warn, format_args!("{msg}"));
        samp::log::warn!("{msg}");
    }

    pub fn error(msg: &str) {
        Self::to_console(Level::Error, format_args!("{msg}"));
        samp::log::error!("{msg}");
    }

    /// Short, sanitized line on the server console; full detail in the file.
    pub fn error_detail(console_msg: &str, detail: &str) {
        Self::to_console(Level::Error, format_args!("{console_msg}"));
        samp::log::error!("{detail}");
    }

    /// Both channels share the SDK logger's level — there is no second gate to
    /// keep in sync. `install` applies it before emitting the banner, so the
    /// banner honours `MYSQL_SAMP_LOG_LEVEL` too.
    fn to_console(level: Level, args: fmt::Arguments<'_>) {
        if level.to_level_filter() > samp::logger::level() {
            return;
        }

        if let Some(console) = CONSOLE.get() {
            console.log(&Record::builder().level(level).args(args).build());
        }
    }
}

/// Banner lines, built from the `CARGO_PKG_*` metadata the SDK captured plus
/// the build stamp `build.rs` injects.
fn banner_lines(meta: &BannerMetadata) -> Vec<String> {
    let build_date = env!("BUILD_DATE");
    let build_time = env!("BUILD_TIME");
    let build_year = env!("BUILD_YEAR");

    vec![
        String::new(),
        format!("  | {} {} | {}", meta.name, meta.version, build_year),
        String::from("  |-------------------------------"),
        format!(
            "  | Author and maintainer: {}",
            value_or(meta.authors, "Unknown")
        ),
        String::new(),
        format!("  | Compiled: {build_date} at {build_time}"),
        String::from("  |-------------------------------"),
        format!("  | Repository: {}", value_or(meta.repository, "N/A")),
        String::new(),
    ]
}

/// Maps the Pawn-facing level (0=none .. 4=all) onto the `log` facade.
fn i32_to_level(level: i32) -> LevelFilter {
    match level {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        _ => LevelFilter::Trace,
    }
}

fn value_or<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value
    }
}
