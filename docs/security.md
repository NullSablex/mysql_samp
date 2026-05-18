# Security

This page describes the security-relevant defaults and the rules you should follow to keep them effective.

## SQL injection

The plugin protects against SQL injection through three layers:

1. **`mysql_format` with `%s` or `%e`** — the formatted string is escaped via the rules below before being substituted.
2. **`mysql_escape_string`** — pure escape function, no connection needed.
3. **ORM** — every bound string is escaped through the same rules; table and column names are sanitized via `escape_identifier`.

### Safe pattern

```pawn
new query[256];
mysql_format(g_mysql, query, sizeof(query),
    "SELECT * FROM players WHERE name = '%s'", player_name);
mysql_query(g_mysql, query, "OnPlayerFound");
```

### Unsafe pattern — do not do this

```pawn
new query[256];
format(query, sizeof(query),
    "SELECT * FROM players WHERE name = '%s'", player_name);  // standard a_samp format, no escape
mysql_query(g_mysql, query);
// If player_name == "'; DROP TABLE players; --" → SQL injection.
```

### `%s` vs `%r`

| Specifier | Escaped? | Use for |
|---|---|---|
| `%s`, `%e` | yes | Any value originating outside your code: player input, file contents, network data |
| `%r` | **no** | Compile-time constants only: table names, column names, fixed SQL fragments |

> **Rule of thumb:** default to `%s`. Use `%r` only when the value is a string literal in your source.

### Escape rules

`mysql_escape_string` and `mysql_format %s` use the same backslash-escape rules over UTF-8 input. Bytes escaped:

| Input | Output |
|---|---|
| `\0` (NUL) | `\0` |
| `\n` | `\n` |
| `\r` | `\r` |
| `\` (backslash) | `\\` |
| `'` | `\'` |
| `"` | `\"` |
| `\x1a` (Ctrl-Z) | `\Z` |
| every other byte | unchanged |

The escape function is **not idempotent**: feeding its output back through itself produces a deeper-escaped string. Escape **once**, right before the value is interpolated into the SQL.

## Multi-byte charsets

The plugin forces `SET NAMES utf8mb4` on every new pool connection. This blocks a class of escape-bypass attacks where multi-byte sequences in legacy charsets (such as `gbk`) can "swallow" the backslash that the escape function added.

`mysql_set_charset(connId, "...")` lets you change the charset at runtime. Avoid switching to a non-ASCII-safe charset such as `gbk`, `big5` or `sjis` unless you have a specific need — the escape rules above assume an ASCII-safe encoding.

## Resource limits

| Resource | Limit | Why |
|---|---|---|
| Saved caches | 1 024 | Prevents memory growth from misused `cache_save` (CWE-770) |
| Rows per single result | 100 000 | Caps the worst-case allocation for a single query (CWE-770) |
| `orm_addvar_string` `max_len` | 1..=4 096 | Bounds the size of writes into the AMX heap when `orm_apply_cache` copies a column (CWE-787) |

When a limit is hit:

- the native returns its failure sentinel (`false`, `0`),
- a warning is written to `logs/mysql.log`,
- the server keeps running.

The 4096 cap on string bindings means a single ORM-managed string column cannot overflow a Pawn array even if a hostile `orm_addvar_string(orm, var, max_len, col)` were attempted with `max_len = INT_MAX`.

## Integer-conversion safety

Every cross-width or sign-changing integer conversion in the plugin uses explicit `TryFrom`/`From`, not the silent `as` cast:

- `i32 → usize` (Pawn row/col indices into Rust container indices): rejected when negative.
- `usize / u64 / u128 → i32` (counts returned to Pawn): saturated at `i32::MAX` instead of wrapping to negative.
- `i32 → u16` (`MYSQL_OPT_PORT`): rejected when negative or `> 65535`.
- `i32 → u32` (`MYSQL_OPT_CONNECT_TIMEOUT`): rejected when negative.

This is stricter than the old MySQL R41-4 plugin, which silently wrapped values. Callers that pass garbage now get a `false` return instead of an obscure misbehavior.

## Callback dispatch

The callback dispatcher checks every step:

- `find_public` is required to succeed in the first AMX that has the callback; AMXes that do not are silently skipped.
- Every `push` of a parameter checks the result; one failed push aborts the call and logs an error naming the callback.
- A failed string allocation also aborts and logs an error.

The server cannot crash because of a malformed callback format string — at worst the callback is skipped and a warning is logged.

## Console hygiene

`logs/mysql.log` gets full detail. The server console gets a short, sanitized message with the error code only. The console never prints:

- SQL query text
- Credentials or hostnames
- Row data

If the log file cannot be written, the plugin emits one console error (`Failed to write logs/mysql.log: …`) and then suppresses further file-write attempts to avoid flooding the console.

## Authority and ABI

The plugin uses Rust's `unsafe_op_in_unsafe_fn = "deny"` policy: every `unsafe` block at the call site is annotated, not inherited from a containing `unsafe fn`. The only `unsafe` blocks in the codebase are around the AMX pointer arithmetic in `orm.rs` (reading/writing AMX heap cells), which is bounded by `safe_max.saturating_sub(1)` and a NUL terminator slot.

The FFI layer (the `samp` crate from rust-samp v3) wraps every native invocation in `catch_unwind`. A panic inside a native logs an error and returns `0` to the AMX caller; the server stays up.

## Best practices

1. **Always use `mysql_format` with `%s`** for user input.
2. **Implement `OnQueryError`** — undetected query failures hide bugs.
3. **Verify `cache_get_row_count()`** before reading rows.
4. **Release resources** when you are done with them:
   ```pawn
   orm_destroy(orm_id);
   cache_delete(cache_id);
   mysql_close(conn_id);
   ```
   The plugin auto-cleans ORMs when their AMX unloads, but explicit destruction is cheap and clear.
5. **Never feed user input through `%r`.**
6. **Avoid `mysql_set_charset` to legacy multi-byte charsets** — stay on `utf8mb4`.

## Threat model summary

| CWE | Mitigation |
|---|---|
| CWE-89 (SQL injection) | Automatic escape on `%s` / `%e`, ORM string columns and identifiers |
| CWE-770 (resource exhaustion) | 1024-cache cap, 100k-row cap, 4096-byte ORM string cap |
| CWE-787 (out-of-bounds write) | `orm_addvar_string` `max_len` clamped at 4096; `orm_apply_cache` writes up to `safe_max - 1` bytes plus NUL |
| CWE-252 (unchecked error) | Callback dispatcher checks every AMX operation; failed pushes log and abort |
| CWE-190 (integer overflow) | `wrapping_add(...).max(1)` on every id counter; `TryFrom` on every cross-width conversion |
| Memory safety (general) | Rust borrow checker; no manual memory management; `unsafe` is opt-in per block |
