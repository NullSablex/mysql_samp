# Security

This page describes the security-relevant defaults and the rules you should follow to keep them effective.

## SQL injection

There are two fundamentally different approaches here, and they are not equally strong.

**Prepared statements (`mysql_stmt_*`) are the safe one.** Values are sent to the server separately from the statement, over the binary protocol. There is no SQL text for a value to break out of and no escaping to get right. Use them for anything carrying player input.

```pawn
new stmt = mysql_stmt_new(g_mysql, "SELECT * FROM players WHERE name = ?");
mysql_stmt_bind_str(stmt, player_name);
mysql_stmt_execute(stmt, "OnPlayerFound");
mysql_stmt_close(stmt);
```

**Escaping (`mysql_format`, `mysql_escape_string`, the ORM) is the fallback.** It works, but its correctness depends on matching the server's `sql_mode` — see [Escape rules](#escape-rules). It is fine for values you control and for query shapes a placeholder cannot express (identifiers, `ORDER BY` direction), but it is one configuration change away from being wrong.

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

Escaping is **connection-dependent**, because the rules change with the server's `sql_mode`. The plugin reads `sql_mode` when the connection is opened and picks the matching rules automatically — which is why `mysql_format` and `mysql_escape_string` take a `connId`. Pass the real connection; the default (`0`) assumes standard MySQL rules and is wrong on a server running `NO_BACKSLASH_ESCAPES`.

#### Default `sql_mode`

`mysql_escape_string` and `mysql_format %s` use backslash-escape rules over UTF-8 input. Bytes escaped:

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

#### Under `NO_BACKSLASH_ESCAPES`

When the server runs with `sql_mode=NO_BACKSLASH_ESCAPES`, the backslash stops being an escape character. `\'` is then a literal backslash followed by a **live** quote, so backslash escaping does not merely fail to help — it lets a crafted value terminate the string literal. In that mode the only valid escape is doubling the quote (`'` → `''`), and nothing else is escaped. The plugin switches to those rules automatically.

Two consequences worth knowing:

- Results are only safe inside **single-quoted** literals. A double-quoted literal cannot be escaped safely in this mode; MySQL's own `mysql_real_escape_string` has the same limitation.
- The driver always enables `CLIENT_MULTI_STATEMENTS` and the underlying crate offers no way to turn it off. So an escaping mistake is not limited to leaking data — it can append a whole second statement. This is the main reason to prefer prepared statements.

#### Escape once

The escape function is **not idempotent**: feeding its output back through itself produces a deeper-escaped string. Escape **once**, right before the value is interpolated into the SQL.

## Multi-byte charsets

The plugin forces `SET NAMES utf8mb4` on every new pool connection. This blocks a class of escape-bypass attacks where multi-byte sequences in legacy charsets (such as `gbk`) can "swallow" the backslash that the escape function added.

`mysql_set_charset(connId, "...")` lets you change the charset at runtime. Avoid switching to a non-ASCII-safe charset such as `gbk`, `big5` or `sjis` unless you have a specific need — the escape rules above assume an ASCII-safe encoding.

## Password storage

`mysql_hash_password` / `mysql_verify_password` run **Argon2id** with the OWASP-recommended defaults (19 MiB memory, 2 iterations, 1 lane).

- The output is a PHC string (`$argon2id$v=19$m=19456,t=2,p=1$...`), roughly 100 characters. Store it as-is in a `VARCHAR(255)`.
- It **already contains a random per-hash salt**. Do not add a salt column, and do not reuse a salt across players — two accounts with the same password produce different hashes precisely so the table does not leak that fact.
- Verification reads the cost parameters back out of the stored hash, so hashes written with older settings keep verifying if the defaults ever change.
- Never store a password with `MD5`, `SHA1`, or MySQL's `PASSWORD()` / `SHA2()` functions. Those are fast by design, which is the opposite of what password storage needs, and a leaked table falls to commodity GPU cracking.
- Hashing a password through a SQL function would also put the plaintext in the query — and therefore in `logs/mysql.log` and any server-side query log. `mysql_hash_password` never puts it in SQL.

Both natives are non-blocking, and both return `false` if the work queue is saturated — see [Resource limits](#resource-limits).

## TLS

`MYSQL_OPT_SSL` enables TLS via rustls, compiled into the binary.

- Without `MYSQL_OPT_SSL_CA` only the **bundled webpki roots** (the Mozilla list) are trusted — *not* the operating system's trust store. A server with a self-signed certificate or an internal CA needs `MYSQL_OPT_SSL_CA` pointing at that CA.
- `MYSQL_OPT_SSL_VERIFY_CERT = 0` disables certificate and hostname verification. Traffic stays encrypted, but any machine-in-the-middle can present its own certificate and read or rewrite every query. It warns on every connect. Use `MYSQL_OPT_SSL_CA` instead.
- `MYSQL_OPT_SSL_CERT` + `MYSQL_OPT_SSL_KEY` provide a client certificate when the server requires mutual TLS.

#### Verified end to end

TLS was checked against a real MariaDB 11.8, not just by reading code: the server's completed-handshake counter advances for each connection, and with verification at its default a self-signed certificate is rejected outright (`invalid peer certificate: UnknownIssuer`) instead of quietly falling back to plaintext.

One limitation worth knowing: **TLS is not available over a unix socket.** A host starting with `/` connects over a socket, where there is nothing to encrypt and `MYSQL_OPT_SSL` has no effect. Use a TCP host if you need encryption.

> Plugin versions before 1.2.0 accepted `MYSQL_OPT_SSL` but shipped no TLS backend at all, so connections were never encrypted. If you relied on it, treat those credentials as exposed.

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

### Password hashing limits

| Limit | Value | Rationale |
|---|---|---|
| Worker threads | `min(cpus, 4)` | Argon2id costs ~19 MiB per concurrent hash; a thread per request would let a login flood allocate gigabytes |
| Queued jobs | 512 | Beyond this, submission is refused and the native returns `false` rather than growing the queue without bound |
| Password length | 1024 bytes | Cost grows with input and the input is remote; far above any real passphrase |

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

## What reaches the logs

Worth knowing precisely, because "the query text ends up in the log" is a common assumption and it is not what happens here.

### The plugin never logs query text

`logs/mysql.log` gets full detail, the server console gets a short sanitized line with the error code only. Neither receives the SQL. On a failed query the plugin logs the **server's error message**, not the statement that produced it.

The console never prints:

- SQL query text
- Credentials or hostnames
- Row data

If the log file cannot be written, the plugin emits one console error and then suppresses further file-write attempts to avoid flooding the console.

### The two places a query can still surface

1. **MySQL's own error text.** For a syntax error the server echoes a fragment of the statement back (`… check the manual … near '…' at line 1`), and that message *is* logged. It is a fragment, from the server, only on a query that failed — not the full statement.

2. **`OnQueryError` hands your gamemode the complete query.** The `query` parameter carries the full text. If your handler prints it:

    ```pawn
    public OnQueryError(errorid, const error[], const callback[], const query[], connId)
    {
        printf("  query: %s", query);   // this goes to the console and server_log.txt
    }
    ```

    then the whole statement lands in the server log — including any value interpolated by `mysql_format`. That is your gamemode's choice, not the plugin's. Log the `callback` name and `errorid` instead when the statement may carry sensitive data.

### Do not use the log level for privacy

`mysql_log(MYSQL_LOG_NONE)` does suppress the lines above, because they are emitted at `ERROR`. It is a bad trade: you lose the diagnosis of every failed query to hide a fragment. Use the tools below instead.

### Prepared statements remove the problem at the source

A prepared statement stores only the template with its `?` placeholders. The bound values never enter the query text, so they appear in neither the log, nor MySQL's error message, nor the `query` parameter of `OnQueryError`, nor `cache_get_query_string()`.

```pawn
// The log, OnQueryError and cache_get_query_string all see only:
//   SELECT id FROM accounts WHERE name = ? AND token = ?
```

The same applies to passwords: `mysql_hash_password` never puts the plaintext in SQL, which is the concrete reason to prefer it over hashing through a MySQL function.

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
| CWE-89 (SQL injection) | Prepared statements (`mysql_stmt_*`, values never in SQL text); `sql_mode`-aware escape on `%s` / `%e`, ORM string columns and identifiers |
| CWE-770 (resource exhaustion) | 1024-cache cap, 100k-row cap, 4096-byte ORM string cap |
| CWE-787 (out-of-bounds write) | `orm_addvar_string` `max_len` clamped at 4096; `orm_apply_cache` writes up to `safe_max - 1` bytes plus NUL |
| CWE-252 (unchecked error) | Callback dispatcher checks every AMX operation; failed pushes log and abort |
| CWE-190 (integer overflow) | `wrapping_add(...).max(1)` on every id counter; `TryFrom` on every cross-width conversion |
| Memory safety (general) | Rust borrow checker; no manual memory management; `unsafe` is opt-in per block |
