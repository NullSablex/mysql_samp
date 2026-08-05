# Checklist: mysql_samp vs MySQL R41-4

Coverage of the MySQL R41-4 (BlueG / maddinat0r) Pawn API by **mysql_samp**. Source of truth: [`include/mysql_samp.inc.in`](include/mysql_samp.inc.in) and [`src/lib.rs`](src/lib.rs). Current plugin version lives in [`Cargo.toml`](Cargo.toml).

## Connection

| Feature | R41-4 | mysql_samp | Notes |
|---|---|---|---|
| `mysql_connect` | Yes | Yes | No custom Pawn tags |
| `mysql_connect_file` | Yes | Yes | `key = value` file with host / user / password / database |
| `mysql_close` | Yes | Yes | |
| `mysql_errno` | Yes | Yes | Returns the MySQL error code (1062, 1045, …) or `0` for no error / `MYSQL_ERROR_*` (1..=8) for plugin-side errors |
| `mysql_error` | Yes | Yes | Writes the last error message into the destination buffer |
| `mysql_escape_string` | Yes | Yes | Optional trailing `connId` selects the escaping rules for the server's `sql_mode` |
| `mysql_format` | Yes | Yes | `printf`-like with `%d`, `%i`, `%f`, `%s`, `%e`, `%r`, `%%` |
| `mysql_set_charset` | Yes | Yes | Runs `SET NAMES '<charset>'` on the connection |
| `mysql_get_charset` | Yes | Yes | Reads `@@character_set_connection` |
| `mysql_stat` / `mysql_status` | Yes | Yes | We export `mysql_status` |
| `mysql_unprocessed_queries` | Yes | Yes | In-flight + buffered count |
| `mysql_log` | Yes | Yes | Runtime level switch (`MYSQL_LOG_*`) |
| `mysql_tick` | — | Yes | Manual drain. Optional — `on_tick` (rust-samp v3) already pumps the queue automatically |
| Unix socket | — | Yes | Auto-detected when `host` starts with `/` |

## Options

| Feature | R41-4 | mysql_samp | Notes |
|---|---|---|---|
| Create handle | `mysql_init_options` | `mysql_options_new` | |
| Set value | `mysql_set_option` (variadic) | `mysql_options_set_int` / `_set_str` | int and string setters are split |
| `mysql_global_options` | Yes | — | Global option pool (not supported) |
| AUTO_RECONNECT | Yes | Yes | `MYSQL_OPT_AUTO_RECONNECT`; one-shot retry on connection-loss errors |
| MULTI_STATEMENTS | Yes | — | Always on in the `mysql` crate and not disableable through its API |
| POOL_SIZE | Yes | Yes | `MYSQL_OPT_POOL_SIZE` caps the pool maximum |
| SERVER_PORT | Yes | Yes | `MYSQL_OPT_PORT` (`u16`; negative or `> 65535` rejected) |
| SSL_ENABLE | Yes | Yes | `MYSQL_OPT_SSL` — rustls compiled in, no system library needed |
| SSL_KEY_FILE | Yes | Yes | `MYSQL_OPT_SSL_KEY` (mutual TLS; needs `MYSQL_OPT_SSL_CERT`) |
| SSL_CERT_FILE | Yes | Yes | `MYSQL_OPT_SSL_CERT` (mutual TLS; needs `MYSQL_OPT_SSL_KEY`) |
| SSL_CA_FILE | Yes | Yes | `MYSQL_OPT_SSL_CA`. Without it only the bundled webpki roots are trusted, **not** the OS trust store |
| SSL_CA_PATH | Yes | — | The driver's `SslOpts` takes a CA **file**, not a directory. Use `MYSQL_OPT_SSL_CA` |
| SSL_CIPHER | Yes | — | Not expressible: rustls fixes the cipher suites per crypto provider and the driver exposes no cipher setting |
| CONNECT_TIMEOUT | — | Yes | Exclusive — `MYSQL_OPT_CONNECT_TIMEOUT` (`u32`; negative rejected) |

## Queries

| Feature | R41-4 | mysql_samp | Notes |
|---|---|---|---|
| `mysql_query` | Yes (sync) | Yes (non-blocking, FIFO) | Always threaded — replaces `tquery` |
| `mysql_tquery` | Yes | — | Subsumed by `mysql_query` (which is already non-blocking) |
| `mysql_pquery` | Yes | Yes | Parallel, no ordering guarantee |
| `mysql_query_file` | Yes | Yes | Non-blocking, like every query here. Not transactional — see the note in [Queries](docs/queries.md) |
| `mysql_tquery_file` | Yes | — | Subsumed by `mysql_query_file`, which is already non-blocking |

## Cache

| Feature | R41-4 | mysql_samp | Notes |
|---|---|---|---|
| `cache_get_row_count` | Yes | Yes | |
| `cache_get_field_count` | Yes | Yes | |
| `cache_get_result_count` | Yes | Yes | Number of result sets in the active cache |
| `cache_get_field_name` | Yes | Yes | |
| `cache_get_field_type` | Yes | Yes | Returns the raw `mysql::consts::ColumnType` byte |
| `cache_set_result` | Yes | Yes | Selects which set the other `cache_*` natives report on |
| `cache_get_value_index` | Yes | Yes | String by index |
| `cache_get_value_index_int` | Yes | Yes | Int by index |
| `cache_get_value_index_float` | Yes | Yes | Float by index |
| `cache_is_value_index_null` | Yes | Yes | |
| `cache_get_value_name` | Yes | Yes | String by name (case-insensitive) |
| `cache_get_value_name_int` | Yes | Yes | Int by name |
| `cache_get_value_name_float` | Yes | Yes | Float by name |
| `cache_is_value_name_null` | Yes | Yes | |
| `cache_save` | Yes | Yes | Persists the active entry for later reuse |
| `cache_delete` | Yes | Yes | |
| `cache_set_active` | Yes | Yes | |
| `cache_unset_active` | Yes | Yes | |
| `cache_is_any_active` | Yes | Yes | |
| `cache_is_valid` | Yes | Yes | |
| `cache_affected_rows` | Yes | Yes | |
| `cache_insert_id` | Yes | Yes | |
| `cache_warning_count` | Yes | Yes | Reported by the server after each query |
| `cache_get_query_exec_time` | Yes | Yes | Always in milliseconds |
| `cache_get_query_string` | Yes | Yes | |

## ORM

| Feature | R41-4 | mysql_samp | Notes |
|---|---|---|---|
| `orm_create` | Yes | Yes | |
| `orm_destroy` | Yes | Yes | |
| `orm_errno` | Yes | Yes | `ORM_OK` / `ORM_NO_DATA` |
| `orm_apply_cache` | Yes | Yes | |
| `orm_select` / `orm_load` | Yes | Yes | Non-blocking |
| `orm_update` | Yes | Yes | Non-blocking |
| `orm_insert` | Yes | Yes | Non-blocking |
| `orm_delete` | Yes | Yes | Non-blocking |
| `orm_save` | Yes | Yes | INSERT when the key is empty, UPDATE otherwise |
| `orm_addvar_int` | Yes | Yes | |
| `orm_addvar_float` | Yes | Yes | |
| `orm_addvar_string` | Yes | Yes | `var_max_len` capped at `MAX_ORM_STRING_LEN` (4096) |
| `orm_clear_vars` | Yes | Yes | |
| `orm_delvar` | Yes | Yes | |
| `orm_setkey` | Yes | Yes | |

## Prepared statements (mysql_samp only)

| Feature | Notes |
|---|---|
| `mysql_stmt_new` | `?` placeholders; bound to a connection |
| `mysql_stmt_bind_int` / `_float` / `_str` / `_null` | Values travel over the binary protocol, never through SQL text |
| `mysql_stmt_reset` | Drops bound values, keeps the statement for reuse |
| `mysql_stmt_execute` | Non-blocking, FIFO; rejects a placeholder/value count mismatch |
| `mysql_stmt_pexecute` | Parallel counterpart, mirroring `mysql_pquery` |
| `mysql_stmt_close` | |

## Transactions (mysql_samp only)

| Feature | Notes |
|---|---|
| `mysql_transaction_new` | |
| `mysql_transaction_add` | Plain SQL step |
| `mysql_transaction_add_stmt` | Copies a prepared statement with its bound values |
| `mysql_transaction_execute` | Atomic batch on one connection; consumes the handle; rolls back on any failure |
| `mysql_transaction_destroy` | Discards a batch that was never executed |

## Password hashing (mysql_samp only)

| Feature | Notes |
|---|---|
| `mysql_hash_password` | Argon2id on a worker thread; PHC output with an embedded random salt |
| `mysql_verify_password` | Reads parameters back from the stored hash, so old hashes keep verifying |

## Forwards

| Feature | R41-4 | mysql_samp | Notes |
|---|---|---|---|
| `OnQueryError` | Yes | Yes | Fired on every loaded AMX when a threaded query fails |

## Extras (mysql_samp only)

| Feature | Notes |
|---|---|
| Zero external runtime dependencies | No `libmysqlclient`, no OpenSSL — `mysql` crate with `default-rust` + `rustls-tls-ring` |
| `MYSQL_OPT_CONNECT_TIMEOUT` | Configurable TCP connect timeout |
| `MYSQL_SAMP_VERSION` in the include | Version constant available to Pawn for introspection |
| Prepared statements | `mysql_stmt_*` — values bound server-side over the binary protocol, immune to injection |
| Transactions | `mysql_transaction_*` — atomic batch, rolls back on any failing step |
| Argon2id password hashing | `mysql_hash_password` / `mysql_verify_password`, threaded; PHC output with embedded salt |
| TLS | rustls compiled in (`rustls-tls-ring`), CA / mutual TLS / verification toggle |
| Detailed file logs | `logs/mysql.log` with timestamp, 50 MB rotation into gzipped `logs/archive/`, and `MYSQL_SAMP_LOG_*` env overrides |
| Build banner | Date/time stamped by `build.rs` via `BUILD_DATE` / `BUILD_TIME` / `BUILD_YEAR` |
| Connection pool | `mysql::Pool` (`Clone + Send + Sync`) for safe multi-threaded access |
| Fully non-blocking queries | Both `mysql_query` (FIFO) and `mysql_pquery` (parallel) run on worker threads |
| ORM auto-cleanup | `OrmManager::destroy_by_amx` frees instances when their AMX is unloaded |
| Universal SA-MP + Open Multiplayer binary | The same `.so` / `.dll` runs natively (component) or in legacy mode (`legacy_plugins`) |
| Unified `on_tick` | Dispatches callbacks via `ProcessTick` (SA-MP) and `ITimersComponent` (Open Multiplayer native), no Pawn `SetTimer` required |
| `mysql_format` safe truncation | Truncates at the destination buffer boundary respecting UTF-8 char boundaries; warns once per call |
| Strict integer conversions | Every cross-width / sign-changing conversion goes through `TryFrom` / `From`; no silent wrap from `as` |
| 167 unit tests | Cover the entire pure surface (parser, renderer, escape modes, placeholder scanner, cache, ORM, statements, transactions, Argon2id) |

## Totals

| Category | R41-4 | mysql_samp |
|---|---|---|
| Pawn natives | — | **75** |
| Pawn forwards | — | **1** |
| Plugin error codes (`MYSQL_ERROR_*`) | — | 9 (`MYSQL_OK` + 8) |
| Connection options (`MYSQL_OPT_*`) | many | 9 (all wired) |
| Log levels (`MYSQL_LOG_*`) | bitflags | 5 sequential |
| ORM error codes (`ORM_*`) | 3 | 2 |
