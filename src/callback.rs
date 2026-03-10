use samp::amx::{AmxIdent, get as get_amx};
use samp::exec_public;

use crate::logger::Logger;
use crate::query::{CallbackInfo, CallbackParam};

/// Invokes a Pawn callback with the given parameters.
/// Searches through the AMX list to find the public function.
pub fn invoke_callback(amx_list: &[AmxIdent], info: &CallbackInfo) {
    if info.name.is_empty() {
        return;
    }

    for ident in amx_list {
        let amx = match get_amx(*ident) {
            Some(a) => a,
            None => continue,
        };

        let idx = match amx.find_public(&info.name) {
            Ok(i) => i,
            Err(_) => continue,
        };

        // Push parameters in reverse order (AMX stack convention)
        let allocator = amx.allocator();
        let format_chars: Vec<char> = info.format.chars().collect();
        let mut push_ok = true;

        for (i, ch) in format_chars.iter().enumerate().rev() {
            let param = match info.params.get(i) {
                Some(p) => p,
                None => continue,
            };

            match (ch, param) {
                ('d' | 'i', CallbackParam::Int(v)) => {
                    if amx.push(*v).is_err() {
                        push_ok = false;
                        break;
                    }
                }
                ('f', CallbackParam::Float(v)) => {
                    if amx.push(*v).is_err() {
                        push_ok = false;
                        break;
                    }
                }
                ('s', CallbackParam::String(v)) => {
                    match allocator.allot_string(v) {
                        Ok(s) => {
                            if amx.push(s).is_err() {
                                push_ok = false;
                                break;
                            }
                        }
                        Err(_) => {
                            push_ok = false;
                            Logger::error(&format!(
                                "Failed to allocate string for callback '{}'.",
                                info.name
                            ));
                            break;
                        }
                    }
                }
                _ => {}
            }
        }

        if push_ok {
            let _ = amx.exec(idx);
        } else {
            Logger::error(&format!(
                "Callback '{}' aborted: failed to push parameters to AMX stack.",
                info.name
            ));
        }
        break; // Only invoke in the first AMX that has the public
    }
}

/// Fires the OnQueryError forward on all AMX instances.
pub fn fire_on_query_error(
    amx_list: &[AmxIdent],
    error_id: i32,
    error_msg: &str,
    callback: &str,
    query: &str,
    conn_id: i32,
) {
    for ident in amx_list {
        let amx = match get_amx(*ident) {
            Some(a) => a,
            None => continue,
        };

        let _ = exec_public!(
            amx,
            "OnQueryError",
            conn_id,
            query => string,
            callback => string,
            error_msg => string,
            error_id
        );
    }
}
