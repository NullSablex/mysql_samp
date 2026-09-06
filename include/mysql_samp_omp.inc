/*
 * mysql_samp - open.mp naming style (standalone).
 *
 * GENERATED from mysql_samp.inc by build.rs - do not edit by hand.
 *
 * Include THIS instead of <mysql_samp> to write the API in
 * open.mp's Prefix_PascalCase style (MySQL_Connect, Cache_GetRowCount,
 * ORM_Create). Each native aliases the real one, so there is no
 * runtime cost and nothing changes on the plugin side. This file is
 * self-contained - do not include it together with <mysql_samp>.
 */

#if defined _mysql_samp_omp_included
    #endinput
#endif
#define _mysql_samp_omp_included


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
    // Maximum connections the pool may open. Default is the driver's (max 100).
    MYSQL_OPT_POOL_SIZE,
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
native MySQL_OptionsNew() = mysql_options_new;
native bool:MySQL_OptionsSetInt(handle, option, value) = mysql_options_set_int;
native bool:MySQL_OptionsSetStr(handle, option, const value[]) = mysql_options_set_str;

// Connection
native MySQL_Connect(const host[], const user[], const password[], const database[], options = 0) = mysql_connect;
// Reads host / user / password / database from a `key = value` file, keeping
// credentials out of the gamemode source. Options stay with mysql_options_new.
native MySQL_ConnectFile(const path[] = "mysql.ini", options = 0) = mysql_connect_file;
native bool:MySQL_Close(connId) = mysql_close;
native bool:MySQL_Status(connId, dest[], max_len = sizeof(dest)) = mysql_status;

// Error
native MySQL_Errno(connId = 0) = mysql_errno;
native bool:MySQL_Error(connId, dest[], max_len = sizeof(dest)) = mysql_error;

// Charset
native bool:MySQL_SetCharset(connId, const charset[]) = mysql_set_charset;
native bool:MySQL_GetCharset(connId, dest[], max_len = sizeof(dest)) = mysql_get_charset;

// Utility
native MySQL_UnprocessedQueries() = mysql_unprocessed_queries;
native bool:MySQL_Log(log_level) = mysql_log;

// Query (non-blocking)
native bool:MySQL_Query(connId, const query[], const callback[] = "", const format[] = "", {Float,_}:...) = mysql_query;
native bool:MySQL_PQuery(connId, const query[], const callback[] = "", const format[] = "", {Float,_}:...) = mysql_pquery;
// Kept for backwards compatibility only: since rust-samp v3.0.0 the unified
// on_tick already dispatches callbacks automatically on SA-MP and native open.mp.
native bool:MySQL_Tick() = mysql_tick;
// connId selects the escaping rules, which depend on the server's sql_mode.
// Pass the real connection whenever the server may run with NO_BACKSLASH_ESCAPES;
// 0 assumes the MySQL default.
native bool:MySQL_EscapeString(const src[], dest[], max_len = sizeof(dest), connId = 0) = mysql_escape_string;
native MySQL_Format(connId, dest[], max_len, const format[], {Float,_}:...) = mysql_format;
// Runs the statements of a .sql file in order on one connection, non-blocking.
// Split on `;` outside literals and comments. NOT transactional: DDL commits
// implicitly in MySQL. Stops at the first failure, naming which statement.
native bool:MySQL_QueryFile(connId, const path[], const callback[] = "", const format[] = "", {Float,_}:...) = mysql_query_file;

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
native MySQL_StmtNew(connId, const query[]) = mysql_stmt_new;
native bool:MySQL_StmtClose(stmtId) = mysql_stmt_close;
native bool:MySQL_StmtReset(stmtId) = mysql_stmt_reset;
native bool:MySQL_StmtBindInt(stmtId, value) = mysql_stmt_bind_int;
native bool:MySQL_StmtBindFloat(stmtId, Float:value) = mysql_stmt_bind_float;
native bool:MySQL_StmtBindStr(stmtId, const value[]) = mysql_stmt_bind_str;
native bool:MySQL_StmtBindNull(stmtId) = mysql_stmt_bind_null;
native bool:MySQL_StmtExecute(stmtId, const callback[] = "", const format[] = "", {Float,_}:...) = mysql_stmt_execute;
// Parallel counterpart, mirroring mysql_pquery: no ordering guarantee.
native bool:MySQL_StmtPExecute(stmtId, const callback[] = "", const format[] = "", {Float,_}:...) = mysql_stmt_pexecute;

// Cache
native Cache_GetRowCount() = cache_get_row_count;
native Cache_GetFieldCount() = cache_get_field_count;
native bool:Cache_GetFieldName(field_idx, dest[], max_len = sizeof(dest)) = cache_get_field_name;
native Cache_GetFieldType(field_idx) = cache_get_field_type;
native bool:Cache_GetValueIndex(row, col, dest[], max_len = sizeof(dest)) = cache_get_value_index;
native Cache_GetValueIndexInt(row, col) = cache_get_value_index_int;
native Float:Cache_GetValueIndexFloat(row, col) = cache_get_value_index_float;
native bool:Cache_GetValueName(row, const field_name[], dest[], max_len = sizeof(dest)) = cache_get_value_name;
native Cache_GetValueNameInt(row, const field_name[]) = cache_get_value_name_int;
native Float:Cache_GetValueNameFloat(row, const field_name[]) = cache_get_value_name_float;
native bool:Cache_IsValueIndexNull(row, col) = cache_is_value_index_null;
native bool:Cache_IsValueNameNull(row, const field_name[]) = cache_is_value_name_null;
native Cache_AffectedRows() = cache_affected_rows;
native Cache_InsertId() = cache_insert_id;
native Cache_WarningCount() = cache_warning_count;
native Cache_GetQueryExecTime() = cache_get_query_exec_time;
native bool:Cache_GetQueryString(dest[], max_len = sizeof(dest)) = cache_get_query_string;
// Result sets in the active cache. A plain query yields 1; a script or a
// stored-procedure CALL can yield several. cache_set_result picks which one
// every other cache_* native reports on (index 0 until changed).
native Cache_GetResultCount() = cache_get_result_count;
native bool:Cache_SetResult(result_index) = cache_set_result;

native Cache_Save() = cache_save;
native bool:Cache_Delete(cache_id) = cache_delete;
native bool:Cache_SetActive(cache_id) = cache_set_active;
native bool:Cache_UnsetActive() = cache_unset_active;
native bool:Cache_IsAnyActive() = cache_is_any_active;
native bool:Cache_IsValid(cache_id) = cache_is_valid;

// ORM
native ORM_Create(const table[], connId) = orm_create;
native bool:ORM_Destroy(orm_id) = orm_destroy;
native ORM_Errno(orm_id) = orm_errno;
native bool:ORM_Select(orm_id, const callback[] = "", const format[] = "", {Float,_}:...) = orm_select;
native bool:ORM_Update(orm_id, const callback[] = "", const format[] = "", {Float,_}:...) = orm_update;
native bool:ORM_Insert(orm_id, const callback[] = "", const format[] = "", {Float,_}:...) = orm_insert;
native bool:ORM_Delete(orm_id, const callback[] = "", const format[] = "", {Float,_}:...) = orm_delete;
native bool:ORM_Save(orm_id, const callback[] = "", const format[] = "", {Float,_}:...) = orm_save;
native bool:ORM_ApplyCache(orm_id, row = 0) = orm_apply_cache;
native bool:ORM_AddVarInt(orm_id, &var, const column_name[]) = orm_addvar_int;
native bool:ORM_AddVarFloat(orm_id, &Float:var, const column_name[]) = orm_addvar_float;
native bool:ORM_AddVarString(orm_id, var[], var_max_len, const column_name[]) = orm_addvar_string;
native bool:ORM_DelVar(orm_id, const column_name[]) = orm_delvar;
native bool:ORM_ClearVars(orm_id) = orm_clear_vars;
native bool:ORM_SetKey(orm_id, const column_name[]) = orm_setkey;

// Password hashing (Argon2id)
//
// Both natives run on a worker thread - Argon2id is deliberately slow, so
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
// usual choice). It already embeds its own random salt - do not add one.
native bool:MySQL_HashPassword(const password[], const callback[], const format[] = "", {Float,_}:...) = mysql_hash_password;
native bool:MySQL_VerifyPassword(const password[], const hash[], const callback[], const format[] = "", {Float,_}:...) = mysql_verify_password;

// Transactions (non-blocking, all-or-nothing)
//
// Steps are collected first and then executed as one unit on a single
// connection: START TRANSACTION, every step, COMMIT. Any failing step rolls the
// whole batch back and fires OnQueryError. There is no interactive
// begin/commit API on purpose - holding a pooled connection between ticks would
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
native MySQL_TransactionNew(connId) = mysql_transaction_new;
native bool:MySQL_TransactionDestroy(txId) = mysql_transaction_destroy;
native bool:MySQL_TransactionAdd(txId, const query[]) = mysql_transaction_add;
native bool:MySQL_TransactionAddStmt(txId, stmtId) = mysql_transaction_add_stmt;
native bool:MySQL_TransactionExecute(txId, const callback[] = "", const format[] = "", {Float,_}:...) = mysql_transaction_execute;
