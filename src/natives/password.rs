use samp::args::Args;
use samp::native;
use samp::prelude::*;

use crate::logger::Logger;
use crate::natives::query::parse_variadic_params;
use crate::plugin::MysqlPlugin;
use crate::query::CallbackInfo;

/// Upper bound on the password handed to Argon2id.
///
/// Argon2 itself has no meaningful length limit, but an unbounded password is
/// a free denial-of-service: cost grows with input, and the caller is remote.
/// 1 KiB is far above any legitimate passphrase.
const MAX_PASSWORD_LEN: usize = 1024;

impl MysqlPlugin {
    /// mysql_hash_password(const password[], const callback[], const format[] = "", {Float,_}:...)
    ///
    /// The callback receives the PHC hash string as its **first** argument,
    /// followed by the extra values described by `format`.
    #[native(name = "mysql_hash_password", raw)]
    pub fn mysql_hash_password(&mut self, _amx: &Amx, mut args: Args) -> bool {
        let Some(password) = args.next_arg::<AmxString>() else {
            return false;
        };
        let Some(callback) = args.next_arg::<AmxString>() else {
            return false;
        };

        let password = password.to_string();
        let callback = callback.to_string();

        if callback.is_empty() {
            Logger::warn(
                "mysql_hash_password: a callback is required — the hash is delivered through it.",
            );
            return false;
        }
        if !password_len_ok(&password, "mysql_hash_password") {
            return false;
        }

        let format = args
            .next_arg::<AmxString>()
            .map(|s| s.to_string())
            .unwrap_or_default();

        // The hash is prepended, so the callback signature starts with a string.
        let params = parse_variadic_params(&mut args, &format, 3);
        let queued = self.passwords.submit_hash(
            password,
            CallbackInfo {
                name: callback,
                format: format!("s{format}"),
                params,
            },
        );
        if !queued {
            Logger::warn(
                "mysql_hash_password: hashing queue is full, request refused. \
                 The server is receiving password operations faster than it can process them.",
            );
        }
        queued
    }

    /// mysql_verify_password(const password[], const hash[], const callback[], const format[] = "", {Float,_}:...)
    ///
    /// The callback receives the boolean result as its **first** argument,
    /// followed by the extra values described by `format`.
    #[native(name = "mysql_verify_password", raw)]
    pub fn mysql_verify_password(&mut self, _amx: &Amx, mut args: Args) -> bool {
        let Some(password) = args.next_arg::<AmxString>() else {
            return false;
        };
        let Some(hash) = args.next_arg::<AmxString>() else {
            return false;
        };
        let Some(callback) = args.next_arg::<AmxString>() else {
            return false;
        };

        let password = password.to_string();
        let hash = hash.to_string();
        let callback = callback.to_string();

        if callback.is_empty() {
            Logger::warn(
                "mysql_verify_password: a callback is required — the result is delivered through it.",
            );
            return false;
        }
        if !password_len_ok(&password, "mysql_verify_password") {
            return false;
        }

        let format = args
            .next_arg::<AmxString>()
            .map(|s| s.to_string())
            .unwrap_or_default();

        let params = parse_variadic_params(&mut args, &format, 4);
        let queued = self.passwords.submit_verify(
            password,
            hash,
            CallbackInfo {
                name: callback,
                format: format!("d{format}"),
                params,
            },
        );
        if !queued {
            Logger::warn(
                "mysql_verify_password: hashing queue is full, request refused. \
                 The server is receiving password operations faster than it can process them.",
            );
        }
        queued
    }
}

fn password_len_ok(password: &str, native: &str) -> bool {
    if password.len() > MAX_PASSWORD_LEN {
        Logger::warn(&format!(
            "{native}: password rejected, {} bytes exceeds the {MAX_PASSWORD_LEN}-byte limit.",
            password.len()
        ));
        return false;
    }
    true
}
