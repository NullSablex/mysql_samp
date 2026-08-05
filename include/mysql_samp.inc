/*
 * mysql_samp v1.2.0 - MySQL plugin for SA-MP and Open Multiplayer.
 * Author:     NullSablex
 * Repository: https://github.com/NullSablex/mysql_samp
 * License:    GPL-3.0
 */

#if defined _mysql_samp_included
    #endinput
#endif
#define _mysql_samp_included

#define MYSQL_SAMP_VERSION "1.2.0"

// Connection options
enum {
    MYSQL_OPT_PORT = 0,
    MYSQL_OPT_SSL,
    MYSQL_OPT_SSL_CA,
    MYSQL_OPT_CONNECT_TIMEOUT,
    MYSQL_OPT_AUTO_RECONNECT,
    // Client certificate chain (PEM/DER) for mutual TLS. Needs MYSQL_OPT_SSL_KEY.
    MYSQL_OPT_SSL_CERT,
    // Client private key (PEM/DER) for mutual TLS. Needs MYSQL_OPT_SSL_CERT.
    MYSQL_OPT_SSL_KEY,
    // 1 (default) verifies the server certificate and hostname. Setting it to 0
    // keeps the traffic encrypted but accepts ANY certificate, which defeats the
    // point of TLS against an active attacker. Prefer MYSQL_OPT_SSL_CA.
    MYSQL_OPT_SSL_VERIFY_CERT,
}

// Error codes
enum {
    MYSQL_OK = 0,
    MYSQL_ERROR_CONNECTION_FAILED,
    MYSQL_ERROR_INVALID_OPTIONS,
    MYSQL_ERROR_INVALID_CONNECTION,
    MYSQL_ERROR_PING_FAILED,
    MYSQL_ERROR_QUERY_FAILED,
    MYSQL_ERROR_NO_CACHE_ACTIVE,
    MYSQL_ERROR_INVALID_ORM,
    MYSQL_ERROR_ORM_KEY_NOT_SET,
}

// Log levels
enum {
    MYSQL_LOG_NONE = 0,
    MYSQL_LOG_ERROR,
    MYSQL_LOG_WARNING,
    MYSQL_LOG_INFO,
    MYSQL_LOG_ALL,
}

// ORM error codes
enum {
    ORM_OK = 0,
    ORM_NO_DATA,
}

// Forward: invoked when a threaded query fails.
forward OnQueryError(errorid, const error[], const callback[], const query[], connId);

// Options
native mysql_options_new();
native bool:mysql_options_set_int(handle, option, value);
native bool:mysql_options_set_str(handle, option, const value[]);

// Connection
native mysql_connect(const host[], const user[], const password[], const database[], options = 0);
native bool:mysql_close(connId);
native bool:mysql_status(connId, dest[], max_len = sizeof(dest));

// Error
native mysql_errno(connId = 0);
native bool:mysql_error(connId, dest[], max_len = sizeof(dest));

// Charset
native bool:mysql_set_charset(connId, const charset[]);
native bool:mysql_get_charset(connId, dest[], max_len = sizeof(dest));

// Utility
native mysql_unprocessed_queries();
native bool:mysql_log(log_level);

// Query (non-blocking)
native bool:mysql_query(connId, const query[], const callback[] = "", const format[] = "", {Float,_}:...);
native bool:mysql_pquery(connId, const query[], const callback[] = "", const format[] = "", {Float,_}:...);
// Kept for backwards compatibility only: since rust-samp v3.0.0 the unified
// on_tick already dispatches callbacks automatically on SA-MP and native open.mp.
native bool:mysql_tick();
// connId selects the escaping rules, which depend on the server's sql_mode.
// Pass the real connection whenever the server may run with NO_BACKSLASH_ESCAPES;
// 0 assumes the MySQL default.
native bool:mysql_escape_string(const src[], dest[], max_len = sizeof(dest), connId = 0);
native mysql_format(connId, dest[], max_len, const format[], {Float,_}:...);

// Prepared statements (non-blocking, FIFO like mysql_query)
//
// Values are bound server-side over the binary protocol instead of being pasted
// into the SQL text, so no escaping is involved and a value can never be
// reinterpreted as syntax. Prefer this over mysql_format for anything that
// carries player input.
//
//   new stmt = mysql_stmt_new(connId, "SELECT id FROM users WHERE name = ? AND age > ?");
//   mysql_stmt_bind_str(stmt, inputName);
//   mysql_stmt_bind_int(stmt, 18);
//   mysql_stmt_execute(stmt, "OnUsersFound", "d", playerid);
//
// The statement is reusable: call mysql_stmt_reset to drop the bound values and
// bind a fresh set. mysql_stmt_execute fails if the number of bound values does
// not match the number of ? placeholders.
native mysql_stmt_new(connId, const query[]);
native bool:mysql_stmt_close(stmtId);
native bool:mysql_stmt_reset(stmtId);
native bool:mysql_stmt_bind_int(stmtId, value);
native bool:mysql_stmt_bind_float(stmtId, Float:value);
native bool:mysql_stmt_bind_str(stmtId, const value[]);
native bool:mysql_stmt_bind_null(stmtId);
native bool:mysql_stmt_execute(stmtId, const callback[] = "", const format[] = "", {Float,_}:...);

// Cache
native cache_get_row_count();
native cache_get_field_count();
native bool:cache_get_field_name(field_idx, dest[], max_len = sizeof(dest));
native cache_get_field_type(field_idx);
native bool:cache_get_value_index(row, col, dest[], max_len = sizeof(dest));
native cache_get_value_index_int(row, col);
native Float:cache_get_value_index_float(row, col);
native bool:cache_get_value_name(row, const field_name[], dest[], max_len = sizeof(dest));
native cache_get_value_name_int(row, const field_name[]);
native Float:cache_get_value_name_float(row, const field_name[]);
native bool:cache_is_value_index_null(row, col);
native bool:cache_is_value_name_null(row, const field_name[]);
native cache_affected_rows();
native cache_insert_id();
native cache_warning_count();
native cache_get_query_exec_time();
native bool:cache_get_query_string(dest[], max_len = sizeof(dest));
native cache_save();
native bool:cache_delete(cache_id);
native bool:cache_set_active(cache_id);
native bool:cache_unset_active();
native bool:cache_is_any_active();
native bool:cache_is_valid(cache_id);

// ORM
native orm_create(const table[], connId);
native bool:orm_destroy(orm_id);
native orm_errno(orm_id);
native bool:orm_select(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:orm_update(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:orm_insert(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:orm_delete(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:orm_save(orm_id, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:orm_apply_cache(orm_id, row = 0);
native bool:orm_addvar_int(orm_id, &var, const column_name[]);
native bool:orm_addvar_float(orm_id, &Float:var, const column_name[]);
native bool:orm_addvar_string(orm_id, var[], var_max_len, const column_name[]);
native bool:orm_delvar(orm_id, const column_name[]);
native bool:orm_clear_vars(orm_id);
native bool:orm_setkey(orm_id, const column_name[]);

// Password hashing (Argon2id)
//
// Both natives run on a worker thread — Argon2id is deliberately slow, so
// hashing on the server thread would stall it. The result is delivered as the
// FIRST argument of the callback, followed by the extras described by format.
//
//   forward OnHashed(const hash[], playerid);
//   mysql_hash_password("secret", "OnHashed", "d", playerid);
//
//   forward OnChecked(bool:success, playerid);
//   mysql_verify_password("secret", stored_hash, "OnChecked", "d", playerid);
//
// The hash is a PHC string (`$argon2id$v=19$m=...`) around 100 characters long;
// size the storage column and any Pawn buffer accordingly (VARCHAR(255) is the
// usual choice). It already embeds its own random salt — do not add one.
native bool:mysql_hash_password(const password[], const callback[], const format[] = "", {Float,_}:...);
native bool:mysql_verify_password(const password[], const hash[], const callback[], const format[] = "", {Float,_}:...);

// Transactions (non-blocking, all-or-nothing)
//
// Steps are collected first and then executed as one unit on a single
// connection: START TRANSACTION, every step, COMMIT. Any failing step rolls the
// whole batch back and fires OnQueryError. There is no interactive
// begin/commit API on purpose — holding a pooled connection between ticks would
// leak it whenever a gamemode never reached the commit.
//
//   new tx = mysql_transaction_new(connId);
//   mysql_transaction_add(tx, "UPDATE accounts SET balance = balance - 100 WHERE id = 1");
//   mysql_transaction_add(tx, "UPDATE accounts SET balance = balance + 100 WHERE id = 2");
//   mysql_transaction_execute(tx, "OnTransferDone", "d", playerid);
//
// mysql_transaction_add_stmt appends a prepared statement with its bound values,
// which is the safe way to put player input inside a transaction.
// mysql_transaction_execute consumes the handle: the ID is invalid afterwards.
// The callback receives the cache of the LAST step.
native mysql_transaction_new(connId);
native bool:mysql_transaction_destroy(txId);
native bool:mysql_transaction_add(txId, const query[]);
native bool:mysql_transaction_add_stmt(txId, stmtId);
native bool:mysql_transaction_execute(txId, const callback[] = "", const format[] = "", {Float,_}:...);
