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


#define MYSQL_SAMP_VERSION "1.3.0"

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

/**
 * Fired on every loaded script when a threaded query fails.
 *
 * @param errorid   MySQL error code (1062, 1045, ...), or 0 for transport errors
 * @param error     server error message
 * @param callback  the callback the query asked for, or "" if none
 * @param query     the SQL that failed
 * @param connId    connection that produced the error
 */
forward OnQueryError(errorid, const error[], const callback[], const query[], connId);

// Options

/**
 * Creates an options handle for a connection.
 *
 * @return the options handle (>= 1), or 0 on failure
 */
native MySQL_OptionsNew() = mysql_options_new;

/**
 * Sets an integer option (MYSQL_OPT_PORT, MYSQL_OPT_SSL, ...).
 *
 * @param handle  an options handle
 * @param option  a MYSQL_OPT_* value that takes an int
 * @param value   the value
 * @return true on success, false on a bad handle or type mismatch
 */
native bool:MySQL_OptionsSetInt(handle, option, value) = mysql_options_set_int;

/**
 * Sets a string option (MYSQL_OPT_SSL_CA, MYSQL_OPT_SSL_CERT, MYSQL_OPT_SSL_KEY).
 *
 * @param handle  an options handle
 * @param option  a MYSQL_OPT_* value that takes a string
 * @param value   the value
 * @return true on success, false on a bad handle or type mismatch
 */
native bool:MySQL_OptionsSetStr(handle, option, const value[]) = mysql_options_set_str;

// Connection

/**
 * Opens a connection pool.
 *
 * @param host      hostname/IP, or a path starting with / for a unix socket
 * @param user      MySQL user
 * @param password  MySQL password
 * @param database  default schema
 * @param options   an options handle, or 0 for defaults
 * @return connection id (>= 1), or 0 on failure
 */
native MySQL_Connect(const host[], const user[], const password[], const database[], options = 0) = mysql_connect;

/**
 * Opens a connection reading host/user/password/database from a
 * `key = value` file, keeping credentials out of the gamemode source.
 *
 * @param path     path to the config file
 * @param options  an options handle, or 0 for defaults
 * @return connection id (>= 1), or 0 on failure
 */
native MySQL_ConnectFile(const path[] = "mysql.ini", options = 0) = mysql_connect_file;

/**
 * Closes a connection and frees its statements and transactions.
 *
 * @param connId  connection id
 * @return true on success, false if the id is unknown
 */
native bool:MySQL_Close(connId) = mysql_close;

/**
 * Writes a short server-status line into dest.
 *
 * @param connId   connection id
 * @param dest     destination buffer
 * @param max_len  size of dest
 * @return true on success, false on failure
 */
native bool:MySQL_Status(connId, dest[], max_len = sizeof(dest)) = mysql_status;

// Error

/**
 * Last error code for a connection.
 *
 * @param connId  connection id, or 0 for the global error
 * @return the MySQL error code, a MYSQL_ERROR_* value, or 0 for none
 */
native MySQL_Errno(connId = 0) = mysql_errno;

/**
 * Writes the last error message into dest.
 *
 * @param connId   connection id, or 0 for the global error
 * @param dest     destination buffer
 * @param max_len  size of dest
 * @return true on success, false on failure
 */
native bool:MySQL_Error(connId, dest[], max_len = sizeof(dest)) = mysql_error;

// Charset

/**
 * Runs SET NAMES to change the connection charset.
 *
 * @param connId   connection id
 * @param charset  charset name (e.g. "utf8mb4")
 * @return true on success, false on failure
 */
native bool:MySQL_SetCharset(connId, const charset[]) = mysql_set_charset;

/**
 * Reads the connection's current charset into dest.
 *
 * @param connId   connection id
 * @param dest     destination buffer
 * @param max_len  size of dest
 * @return true on success, false on failure
 */
native bool:MySQL_GetCharset(connId, dest[], max_len = sizeof(dest)) = mysql_get_charset;

// Utility

/**
 * Number of queries submitted but not yet dispatched to a callback.
 *
 * @return the pending count
 */
native MySQL_UnprocessedQueries() = mysql_unprocessed_queries;

/**
 * Sets the minimum log level at runtime.
 *
 * @param log_level  a MYSQL_LOG_* value
 * @return true
 */
native bool:MySQL_Log(log_level) = mysql_log;

// Query (non-blocking)

/**
 * Runs a query on a worker thread, FIFO-ordered.
 *
 * @param connId    connection id
 * @param query     the SQL
 * @param callback  callback to fire with the result, or "" for none
 * @param format    types of the extra callback args ("d" int, "f" float, "s" string)
 * @return true if submitted, false on a bad connection
 */
native bool:MySQL_Query(connId, const query[], const callback[] = "", const format[] = "", {Float,_}:...) = mysql_query;

/**
 * Runs a query on a worker thread, parallel: no ordering guarantee.
 *
 * @param connId    connection id
 * @param query     the SQL
 * @param callback  callback to fire with the result, or "" for none
 * @param format    types of the extra callback args
 * @return true if submitted, false on a bad connection
 */
native bool:MySQL_PQuery(connId, const query[], const callback[] = "", const format[] = "", {Float,_}:...) = mysql_pquery;

/**
 * Drains the dispatch queue manually. You do not need to call this: since
 * rust-samp v3.0.0 the queue is pumped automatically every tick. Kept for
 * backwards compatibility.
 *
 * @return true
 */
native bool:MySQL_Tick() = mysql_tick;

/**
 * Escapes a string for use inside a single-quoted SQL literal.
 *
 * @param src      the raw string
 * @param dest     destination buffer
 * @param max_len  size of dest
 * @param connId   connection whose sql_mode picks the escaping rules; pass the
 *                 real one when the server may run NO_BACKSLASH_ESCAPES (0 assumes
 *                 the MySQL default)
 * @return true on success, false on failure
 */
native bool:MySQL_EscapeString(const src[], dest[], max_len = sizeof(dest), connId = 0) = mysql_escape_string;

/**
 * Builds a query into dest with printf-style specifiers (%d, %f, %s escaped,
 * %e escaped, %r raw). Truncates at the buffer boundary.
 *
 * @param connId   connection whose sql_mode picks the escaping rules
 * @param dest     destination buffer
 * @param max_len  size of dest
 * @param format   the format string
 * @return bytes written into dest
 */
native MySQL_Format(connId, dest[], max_len, const format[], {Float,_}:...) = mysql_format;

/**
 * Runs the statements of a .sql file in order on one connection, non-blocking.
 * Split on `;` outside literals and comments. NOT transactional: DDL commits
 * implicitly in MySQL, and on failure the statements before it stay applied.
 *
 * @param connId    connection id
 * @param path      path to the .sql file
 * @param callback  callback to fire with the last statement's result, or ""
 * @param format    types of the extra callback args
 * @return true if submitted, false on a bad connection or unreadable file
 */
native bool:MySQL_QueryFile(connId, const path[], const callback[] = "", const format[] = "", {Float,_}:...) = mysql_query_file;

// Prepared statements (non-blocking, FIFO)

/**
 * Creates a prepared statement. Use `?` for each bound value.
 *
 * @param connId  connection id
 * @param query   the SQL with `?` placeholders
 * @return statement id (>= 1), or 0 on a bad connection
 */
native MySQL_StmtNew(connId, const query[]) = mysql_stmt_new;

/**
 * Frees a statement.
 *
 * @param stmtId  statement id
 * @return true on success, false if the id is unknown
 */
native bool:MySQL_StmtClose(stmtId) = mysql_stmt_close;

/**
 * Clears the bound values, keeping the statement for reuse.
 *
 * @param stmtId  statement id
 * @return true on success, false if the id is unknown
 */
native bool:MySQL_StmtReset(stmtId) = mysql_stmt_reset;

/**
 * Binds an integer to the next placeholder.
 *
 * @param stmtId  statement id
 * @param value   the value
 * @return true on success, false if the id is unknown or the limit is reached
 */
native bool:MySQL_StmtBindInt(stmtId, value) = mysql_stmt_bind_int;

/**
 * Binds a float to the next placeholder.
 *
 * @param stmtId  statement id
 * @param value   the value
 * @return true on success, false if the id is unknown or the limit is reached
 */
native bool:MySQL_StmtBindFloat(stmtId, Float:value) = mysql_stmt_bind_float;

/**
 * Binds a string to the next placeholder.
 *
 * @param stmtId  statement id
 * @param value   the value
 * @return true on success, false if the id is unknown or the limit is reached
 */
native bool:MySQL_StmtBindStr(stmtId, const value[]) = mysql_stmt_bind_str;

/**
 * Binds SQL NULL to the next placeholder.
 *
 * @param stmtId  statement id
 * @return true on success, false if the id is unknown or the limit is reached
 */
native bool:MySQL_StmtBindNull(stmtId) = mysql_stmt_bind_null;

/**
 * Executes a statement, FIFO-ordered. Fails if the number of bound values does
 * not match the number of `?` placeholders.
 *
 * @param stmtId    statement id
 * @param callback  callback to fire with the result, or "" for none
 * @param format    types of the extra callback args
 * @return true if submitted, false on a bad id or arity mismatch
 */
native bool:MySQL_StmtExecute(stmtId, const callback[] = "", const format[] = "", {Float,_}:...) = mysql_stmt_execute;

/**
 * Executes a statement on a worker thread, parallel: no ordering guarantee.
 *
 * @param stmtId    statement id
 * @param callback  callback to fire with the result, or "" for none
 * @param format    types of the extra callback args
 * @return true if submitted, false on a bad id or arity mismatch
 */
native bool:MySQL_StmtPExecute(stmtId, const callback[] = "", const format[] = "", {Float,_}:...) = mysql_stmt_pexecute;

// Cache (read inside a query callback)

/**
 * Rows in the active result set.
 *
 * @return the row count
 */
native Cache_GetRowCount() = cache_get_row_count;

/**
 * Columns in the active result set.
 *
 * @return the field count
 */
native Cache_GetFieldCount() = cache_get_field_count;

/**
 * Writes a column's name into dest.
 *
 * @param field_idx  column index
 * @param dest       destination buffer
 * @param max_len    size of dest
 * @return true on success, false if the index is out of range
 */
native bool:Cache_GetFieldName(field_idx, dest[], max_len = sizeof(dest)) = cache_get_field_name;

/**
 * A column's MySQL type (a raw ColumnType byte).
 *
 * @param field_idx  column index
 * @return the type byte, or 0 if out of range
 */
native Cache_GetFieldType(field_idx) = cache_get_field_type;

/**
 * Reads a cell as a string, by column index.
 *
 * @param row      row index
 * @param col      column index
 * @param dest     destination buffer
 * @param max_len  size of dest
 * @return true on success, false if out of range
 */
native bool:Cache_GetValueIndex(row, col, dest[], max_len = sizeof(dest)) = cache_get_value_index;

/**
 * Reads a cell as an int, by column index.
 *
 * @param row  row index
 * @param col  column index
 * @return the value, or 0 if out of range or NULL
 */
native Cache_GetValueIndexInt(row, col) = cache_get_value_index_int;

/**
 * Reads a cell as a float, by column index.
 *
 * @param row  row index
 * @param col  column index
 * @return the value, or 0.0 if out of range or NULL
 */
native Float:Cache_GetValueIndexFloat(row, col) = cache_get_value_index_float;

/**
 * Reads a cell as a string, by column name (case-insensitive).
 *
 * @param row         row index
 * @param field_name  column name
 * @param dest        destination buffer
 * @param max_len     size of dest
 * @return true on success, false if the row or name is unknown
 */
native bool:Cache_GetValueName(row, const field_name[], dest[], max_len = sizeof(dest)) = cache_get_value_name;

/**
 * Reads a cell as an int, by column name.
 *
 * @param row         row index
 * @param field_name  column name
 * @return the value, or 0 if unknown or NULL
 */
native Cache_GetValueNameInt(row, const field_name[]) = cache_get_value_name_int;

/**
 * Reads a cell as a float, by column name.
 *
 * @param row         row index
 * @param field_name  column name
 * @return the value, or 0.0 if unknown or NULL
 */
native Float:Cache_GetValueNameFloat(row, const field_name[]) = cache_get_value_name_float;

/**
 * Whether a cell is SQL NULL, by column index.
 *
 * @param row  row index
 * @param col  column index
 * @return true if NULL
 */
native bool:Cache_IsValueIndexNull(row, col) = cache_is_value_index_null;

/**
 * Whether a cell is SQL NULL, by column name.
 *
 * @param row         row index
 * @param field_name  column name
 * @return true if NULL
 */
native bool:Cache_IsValueNameNull(row, const field_name[]) = cache_is_value_name_null;

/**
 * Rows affected by the last INSERT/UPDATE/DELETE.
 *
 * @return the affected-row count
 */
native Cache_AffectedRows() = cache_affected_rows;

/**
 * AUTO_INCREMENT id generated by the last INSERT.
 *
 * @return the insert id
 */
native Cache_InsertId() = cache_insert_id;

/**
 * Warnings the server raised for the query.
 *
 * @return the warning count
 */
native Cache_WarningCount() = cache_warning_count;

/**
 * How long the query took to execute.
 *
 * @return the time in milliseconds
 */
native Cache_GetQueryExecTime() = cache_get_query_exec_time;

/**
 * Writes the query text into dest.
 *
 * @param dest     destination buffer
 * @param max_len  size of dest
 * @return true on success, false on failure
 */
native bool:Cache_GetQueryString(dest[], max_len = sizeof(dest)) = cache_get_query_string;

/**
 * Result sets in the active cache. A plain query yields 1; a stored-procedure
 * CALL or a script can yield several.
 *
 * @return the result-set count
 */
native Cache_GetResultCount() = cache_get_result_count;

/**
 * Selects which result set the other cache_* natives read (0 until changed).
 *
 * @param result_index  result-set index
 * @return true on success, false if the index is out of range
 */
native bool:Cache_SetResult(result_index) = cache_set_result;

/**
 * Persists the active cache so it can be read after the callback returns.
 *
 * @return a saved-cache id (>= 1), or 0 on failure
 */
native Cache_Save() = cache_save;

/**
 * Frees a saved cache.
 *
 * @param cache_id  id from cache_save
 * @return true on success, false if the id is unknown
 */
native bool:Cache_Delete(cache_id) = cache_delete;

/**
 * Makes a saved cache the active one.
 *
 * @param cache_id  id from cache_save
 * @return true on success, false if the id is unknown
 */
native bool:Cache_SetActive(cache_id) = cache_set_active;

/**
 * Clears the manually set active cache.
 *
 * @return true on success, false if none was set
 */
native bool:Cache_UnsetActive() = cache_unset_active;

/**
 * Whether any cache is currently active.
 *
 * @return true if one is active
 */
native bool:Cache_IsAnyActive() = cache_is_any_active;

/**
 * Whether a saved-cache id is still valid.
 *
 * @param cache_id  id from cache_save
 * @return true if valid
 */
native bool:Cache_IsValid(cache_id) = cache_is_valid;

// ORM (bind Pawn variables to columns)

/**
 * Creates an ORM instance bound to a table.
 *
 * @param table   table name
 * @param connId  connection id
 * @return an ORM id (>= 1), or 0 on a bad connection
 */
native ORM_Create(const table[], connId) = orm_create;

/**
 * Destroys an ORM instance.
 *
 * @param orm_id  ORM id
 * @return true on success, false if the id is unknown
 */
native bool:ORM_Destroy(orm_id) = orm_destroy;

/**
 * Last ORM error. Only the apply-cache operation sets it.
 *
 * @param orm_id  ORM id
 * @return ORM_OK or ORM_NO_DATA
 */
native ORM_Errno(orm_id) = orm_errno;

/**
 * SELECTs the row matching the key and fills the bound variables.
 *
 * @param orm_id    ORM id
 * @param callback  callback to fire when done, or ""
 * @param format    types of the extra callback args
 * @return true if submitted, false on failure
 */
native bool:ORM_Select(orm_id, const callback[] = "", const format[] = "", {Float,_}:...) = orm_select;

/**
 * UPDATEs the row matching the key with the bound variables.
 *
 * @param orm_id    ORM id
 * @param callback  callback to fire when done, or ""
 * @param format    types of the extra callback args
 * @return true if submitted, false on failure
 */
native bool:ORM_Update(orm_id, const callback[] = "", const format[] = "", {Float,_}:...) = orm_update;

/**
 * INSERTs a row from the bound variables.
 *
 * @param orm_id    ORM id
 * @param callback  callback to fire when done, or ""
 * @param format    types of the extra callback args
 * @return true if submitted, false on failure
 */
native bool:ORM_Insert(orm_id, const callback[] = "", const format[] = "", {Float,_}:...) = orm_insert;

/**
 * DELETEs the row matching the key.
 *
 * @param orm_id    ORM id
 * @param callback  callback to fire when done, or ""
 * @param format    types of the extra callback args
 * @return true if submitted, false on failure
 */
native bool:ORM_Delete(orm_id, const callback[] = "", const format[] = "", {Float,_}:...) = orm_delete;

/**
 * INSERTs when the key is empty, UPDATEs otherwise.
 *
 * @param orm_id    ORM id
 * @param callback  callback to fire when done, or ""
 * @param format    types of the extra callback args
 * @return true if submitted, false on failure
 */
native bool:ORM_Save(orm_id, const callback[] = "", const format[] = "", {Float,_}:...) = orm_save;

/**
 * Copies a row from the active cache into the bound variables.
 *
 * @param orm_id  ORM id
 * @param row     row index in the active cache
 * @return true on success, false if there is no such row
 */
native bool:ORM_ApplyCache(orm_id, row = 0) = orm_apply_cache;

/**
 * Binds an int variable to a column.
 *
 * @param orm_id       ORM id
 * @param var          the variable (read on write, written on load)
 * @param column_name  column name
 * @return true on success, false if the id is unknown
 */
native bool:ORM_AddVarInt(orm_id, &var, const column_name[]) = orm_addvar_int;

/**
 * Binds a float variable to a column.
 *
 * @param orm_id       ORM id
 * @param var          the variable
 * @param column_name  column name
 * @return true on success, false if the id is unknown
 */
native bool:ORM_AddVarFloat(orm_id, &Float:var, const column_name[]) = orm_addvar_float;

/**
 * Binds a string variable to a column.
 *
 * @param orm_id       ORM id
 * @param var          the buffer
 * @param var_max_len  size of the buffer (capped at 4096)
 * @param column_name  column name
 * @return true on success, false on a bad id or length
 */
native bool:ORM_AddVarString(orm_id, var[], var_max_len, const column_name[]) = orm_addvar_string;

/**
 * Unbinds a column.
 *
 * @param orm_id       ORM id
 * @param column_name  column name
 * @return true if a binding was removed
 */
native bool:ORM_DelVar(orm_id, const column_name[]) = orm_delvar;

/**
 * Removes all bindings.
 *
 * @param orm_id  ORM id
 * @return true on success, false if the id is unknown
 */
native bool:ORM_ClearVars(orm_id) = orm_clear_vars;

/**
 * Sets which bound column is the key for select/update/delete/save.
 *
 * @param orm_id       ORM id
 * @param column_name  column name
 * @return true on success, false if the id is unknown
 */
native bool:ORM_SetKey(orm_id, const column_name[]) = orm_setkey;

// Password hashing (Argon2id)

/**
 * Hashes a password with Argon2id, off the server thread.
 *
 * @param password  the plaintext
 * @param callback  callback receiving the PHC hash as its first argument
 * @param format    types of the extra callback args
 * @return true if queued, false if the callback is empty, the password exceeds
 *         1 KiB, or the work queue is full
 */
native bool:MySQL_HashPassword(const password[], const callback[], const format[] = "", {Float,_}:...) = mysql_hash_password;

/**
 * Verifies a password against a stored PHC hash, off the server thread.
 *
 * @param password  the plaintext
 * @param hash      the stored PHC string
 * @param callback  callback receiving the bool result as its first argument
 * @param format    types of the extra callback args
 * @return true if queued, false if the callback is empty, the password exceeds
 *         1 KiB, or the work queue is full
 */
native bool:MySQL_VerifyPassword(const password[], const hash[], const callback[], const format[] = "", {Float,_}:...) = mysql_verify_password;

// Transactions (non-blocking, all-or-nothing)

/**
 * Creates a transaction.
 *
 * @param connId  connection id
 * @return a transaction id (>= 1), or 0 on a bad connection
 */
native MySQL_TransactionNew(connId) = mysql_transaction_new;

/**
 * Discards a transaction that was never executed.
 *
 * @param txId  transaction id
 * @return true on success, false if the id is unknown
 */
native bool:MySQL_TransactionDestroy(txId) = mysql_transaction_destroy;

/**
 * Appends a plain SQL step.
 *
 * @param txId   transaction id
 * @param query  the SQL
 * @return true on success, false if the id is unknown
 */
native bool:MySQL_TransactionAdd(txId, const query[]) = mysql_transaction_add;

/**
 * Appends a prepared statement with its bound values - the safe way to put
 * player input in a transaction. The statement itself is untouched.
 *
 * @param txId    transaction id
 * @param stmtId  statement id
 * @return true on success, false on a bad id or arity mismatch
 */
native bool:MySQL_TransactionAddStmt(txId, stmtId) = mysql_transaction_add_stmt;

/**
 * Runs every step atomically, then destroys the transaction (the id is invalid
 * afterwards). The callback receives the cache of the LAST step.
 *
 * @param txId      transaction id
 * @param callback  callback to fire on commit, or ""
 * @param format    types of the extra callback args
 * @return true if submitted, false on a bad id or empty batch
 */
native bool:MySQL_TransactionExecute(txId, const callback[] = "", const format[] = "", {Float,_}:...) = mysql_transaction_execute;
