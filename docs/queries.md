# Queries

Every query in mysql_samp is **non-blocking**: the statement runs on a worker thread and the callback is invoked on a later tick. The server never freezes waiting for the database.

## mysql_query — FIFO-ordered

```pawn
native bool:mysql_query(connId, const query[], const callback[] = "", const format[] = "", {Float,_}:...);
```

| Parameter | Type | Description |
|---|---|---|
| `connId` | int | Connection id returned by `mysql_connect` |
| `query` | string | Raw SQL statement (already escaped — use `mysql_format` to build it safely) |
| `callback` | string | Pawn public to invoke when the result is ready. Empty (`""`) means fire-and-forget |
| `format` | string | One char per variadic parameter (`d`/`i` = int, `f` = float, `s` = string) |
| `...` | variadic | Values forwarded to the callback in the order they appear |

**Returns:** `true` if the query was queued, `false` if `connId` is unknown.

**Ordering guarantee:** callbacks are delivered in **submission order**, even when the underlying queries finish out of order. A slow query blocks the dispatch of later callbacks until it completes.

```pawn
mysql_query(g_mysql, "SELECT * FROM players WHERE level > 5", "OnHighLevelPlayers");

forward OnHighLevelPlayers();
public OnHighLevelPlayers()
{
    new rows = cache_get_row_count();
    printf("high-level players: %d", rows);
}
```

### Passing parameters to the callback

The `format` string declares the types; the variadics carry the actual values. The callback sees the parameters in left-to-right order.

```pawn
mysql_format(g_mysql, query, sizeof(query),
    "SELECT * FROM players WHERE id = %d", target_id);
mysql_query(g_mysql, query, "OnPlayerLoaded", "d", playerid);

forward OnPlayerLoaded(playerid);
public OnPlayerLoaded(playerid)
{
    if (cache_get_row_count() > 0)
    {
        new name[MAX_PLAYER_NAME];
        cache_get_value_name(0, "name", name);
        printf("player %d: %s", playerid, name);
    }
}
```

Callback format characters:

| Char | Pawn type | Notes |
|---|---|---|
| `d`, `i` | int | Both aliases work |
| `f` | float | |
| `s` | string | Pushed as an AMX string allocation |

Any other character logs a warning at submission time and is skipped.

### Fire and forget

Omit the callback to discard the result:

```pawn
mysql_query(g_mysql, "UPDATE players SET last_login = NOW() WHERE id = 1");
```

A failure still fires `OnQueryError`.

## mysql_pquery — parallel, no ordering

```pawn
native bool:mysql_pquery(connId, const query[], const callback[] = "", const format[] = "", {Float,_}:...);
```

Same signature as `mysql_query`, same callback semantics, but **no ordering guarantee**. Callbacks fire as soon as each query finishes; the dispatcher places parallel results behind ordered ones in the same tick.

Use `mysql_pquery` when the gamemode does not care which callback runs first.

```pawn
mysql_pquery(g_mysql, "UPDATE stats SET kills = kills + 1 WHERE id = 1");
mysql_pquery(g_mysql, "INSERT INTO logs (action) VALUES ('kill')");
mysql_pquery(g_mysql, "SELECT * FROM rewards WHERE id = 1", "OnRewards");
```

## Choosing between mysql_query and mysql_pquery

| Concern | `mysql_query` | `mysql_pquery` |
|---|---|---|
| Threading | One worker thread per query | One worker thread per query |
| Callback order | FIFO (submission order) | First done, first dispatched |
| Typical use | SELECT chains that depend on order | UPDATE/INSERT and independent reads |
| Reordering cost | Yes — results buffer until the next sequence is available | None |

Both natives create one OS thread per query — the cost of a thread spawn is roughly the cost of one TCP round-trip, well below the cost of a real MySQL query.

## mysql_tick — backwards compatibility

```pawn
native bool:mysql_tick();
```

Drains the threaded-query queue manually. **You do not need to call this.** Since rust-samp v3.0.0, the unified `on_tick` hook is driven by `ProcessTick` on SA-MP and by an `ITimersComponent` 5 ms timer on Open Multiplayer native mode; the plugin processes pending results automatically. `mysql_tick` is kept only so old gamemodes that called it explicitly still compile and behave.

## mysql_format

```pawn
native mysql_format(connId, dest[], max_len, const format[], {Float,_}:...);
```

`printf`-style query builder with **automatic SQL escaping** on `%s` / `%e`. Returns the byte length of the rendered string (after truncation, if any). The `connId` parameter is currently unused but kept for API compatibility — pass the connection you intend to run the query on.

| Specifier | Type | Behavior |
|---|---|---|
| `%d`, `%i` | int | Decimal integer |
| `%f` | float | 4 decimal places (`{:.4}`) |
| `%s`, `%e` | string | SQL-escaped (safe to drop into a quoted string literal) |
| `%r` | string | Raw, **not** escaped — only for trusted constants |
| `%%` | literal | Single `%` |

Any other `%x` is left as-is in the output and a single warning is logged for the whole call.

### Truncation

If the rendered string is longer than `max_len - 1` (the `-1` reserves a slot for the AMX NUL terminator), the plugin truncates at the nearest UTF-8 char boundary, logs a warning and still returns the length actually written. Old behavior aborted the native; the new behavior is deterministic and survives oversized inputs gracefully.

### Example

```pawn
new query[256];
new name[24] = "O'Brien";

mysql_format(g_mysql, query, sizeof(query),
    "SELECT * FROM players WHERE name = '%s'", name);
// Result: SELECT * FROM players WHERE name = 'O\'Brien'

mysql_query(g_mysql, query, "OnPlayerFound");
```

Combining types:

```pawn
new query[256];
mysql_format(g_mysql, query, sizeof(query),
    "INSERT INTO scores (player_id, score, name) VALUES (%d, %f, '%s')",
    playerid, 99.5, "Player1");
```

> **Rule of thumb.** Use `%s` for every string that originated outside your code. Use `%r` only for compile-time constants such as table names. Never pass user input through `%r`.

## mysql_escape_string

```pawn
native bool:mysql_escape_string(const src[], dest[], max_len = sizeof(dest), connId = 0);
```

Returns `true` on success, `false` only if writing into `dest` fails.

`connId` selects the escaping rules, which depend on the server's `sql_mode`. **Pass the real connection.** The parameter is optional purely so older code keeps compiling; the default assumes standard MySQL rules, which are wrong — and unsafe — on a server running `NO_BACKSLASH_ESCAPES`. See [Security](security.md#escape-rules).

Under the default `sql_mode`, the characters escaped are `\0`, `\n`, `\r`, `\\`, `'`, `"`, `\x1a` (Ctrl-Z). Every other byte (including `\t`, control bytes, and multi-byte UTF-8) passes through unchanged.

```pawn
new escaped[128];
new input[]  = "It's a \"test\"";
mysql_escape_string(input, escaped);
// escaped = "It\'s a \"test\""
```

**Never call `mysql_escape_string` on a string and then pass the result through `%s`** — `%s` escapes again and you end up with `\\'` instead of `\'`.

## Prepared statements

The safe alternative to escaping. Values are bound server-side over the binary protocol, so they never enter the SQL text — there is nothing to escape and nothing to get wrong. Use these for anything carrying player input.

```pawn
native mysql_stmt_new(connId, const query[]);
native bool:mysql_stmt_bind_int(stmtId, value);
native bool:mysql_stmt_bind_float(stmtId, Float:value);
native bool:mysql_stmt_bind_str(stmtId, const value[]);
native bool:mysql_stmt_bind_null(stmtId);
native bool:mysql_stmt_reset(stmtId);
native bool:mysql_stmt_execute(stmtId, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:mysql_stmt_pexecute(stmtId, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:mysql_stmt_close(stmtId);
```

```pawn
new stmt = mysql_stmt_new(g_mysql, "SELECT id FROM players WHERE name = ? AND score > ?");
mysql_stmt_bind_str(stmt, playerName);
mysql_stmt_bind_int(stmt, 100);
mysql_stmt_execute(stmt, "OnPlayersFound", "d", playerid);
mysql_stmt_close(stmt);
```

`mysql_stmt_execute` is non-blocking and FIFO-ordered, exactly like `mysql_query`, and the result reaches the callback through the same cache stack. `mysql_stmt_pexecute` is the parallel counterpart, matching `mysql_pquery`: no ordering guarantee, dispatched as soon as it completes. Choose between them on the same grounds as [`mysql_query` vs `mysql_pquery`](#choosing-between-mysql_query-and-mysql_pquery).

Notes:

- **Bind order matters** — values are positional, matching the `?` placeholders left to right.
- **Counts must match.** If the number of bound values differs from the number of placeholders, `mysql_stmt_execute` returns `false` and logs both numbers instead of sending a broken query.
- **`mysql_stmt_reset` keeps the statement, drops the values** — useful for bulk inserts.
- **A `?` stands for a value, never an identifier.** `SELECT * FROM ?` is not valid SQL. Build identifiers from a whitelist you control.
- **Values stay out of the logs.** Only the template with its placeholders reaches `logs/mysql.log` and `cache_get_query_string()`.
- Placeholders are counted outside of quoted strings and comments, so a literal `?` inside `'...'` or `/* ... */` is not mistaken for one.

## Transactions

Runs a group of statements atomically: `START TRANSACTION`, every step, `COMMIT`. Any failing step rolls the whole batch back and fires `OnQueryError`.

```pawn
native mysql_transaction_new(connId);
native bool:mysql_transaction_add(txId, const query[]);
native bool:mysql_transaction_add_stmt(txId, stmtId);
native bool:mysql_transaction_execute(txId, const callback[] = "", const format[] = "", {Float,_}:...);
native bool:mysql_transaction_destroy(txId);
```

```pawn
new tx = mysql_transaction_new(g_mysql);
mysql_transaction_add(tx, "UPDATE accounts SET balance = balance - 100 WHERE id = 1");
mysql_transaction_add(tx, "UPDATE accounts SET balance = balance + 100 WHERE id = 2");
mysql_transaction_execute(tx, "OnTransferDone", "d", playerid);
```

Notes:

- **There is no interactive begin/commit.** Holding a pooled connection between server ticks would leak it whenever a gamemode never reached the commit — an early `return`, a runtime error, a disconnect. Collecting the steps first avoids that entirely.
- **`mysql_transaction_execute` consumes the handle.** The ID is invalid afterwards, so a batch cannot run twice by accident.
- **The callback receives the cache of the last step**, which is where `cache_affected_rows` and `cache_insert_id` are usually wanted.
- **Use `mysql_transaction_add_stmt` for player input** — it copies a prepared statement together with its bound values.
- **No auto-reconnect retry.** Replaying a transaction after a mid-flight connection loss could re-apply steps the server already committed, so a dropped connection aborts and reports instead.
- Build a batch and change your mind? Call `mysql_transaction_destroy`.

## Running a .sql file

```pawn
native bool:mysql_query_file(connId, const path[], const callback[] = "", const format[] = "", {Float,_}:...);
```

Reads a file and runs its statements in order on one connection, non-blocking like every other query here. Useful for schema setup and migrations.

```pawn
mysql_query_file(g_mysql, "scripts/schema.sql", "OnSchemaReady");
```

The script is split on `;` **outside** string literals and comments, so a semicolon inside `'...'`, `"..."`, a backtick identifier, `-- …`, `# …` or `/* … */` does not break a statement in two. Comment-only fragments and a trailing semicolon are dropped rather than sent as empty statements.

Notes:

- **It is not a transaction.** These files are usually schema work, and DDL commits implicitly in MySQL, so wrapping them would imply an atomicity the server does not provide. If a statement fails, execution stops there and **everything before it stays applied**.
- **The error names the position** — `statement 7 of 23: …` — which matters in a file with dozens of statements.
- **The callback receives the cache of the last statement**, and `cache_get_result_count()` reports how many result sets the script produced.
- **The path is a file your gamemode opens.** Treat it as such: do not build it from player input.
- The plugin logs how many statements it is about to run, at `INFO`.

## Inspecting the queue

```pawn
native mysql_unprocessed_queries();
```

Returns `in_flight + pending_ordered.len()` — queries currently spawned on a worker thread plus queries whose results arrived but whose callbacks are blocked behind an earlier sequence.

```pawn
printf("pending queries: %d", mysql_unprocessed_queries());
```

Useful for graceful shutdown loops: keep ticking until `mysql_unprocessed_queries() == 0`.

## Result limits

| Limit | Value | Behavior on overflow |
|---|---|---|
| Rows per result | 100 000 | Drained but discarded; a warning is logged once per oversized query |
| Saved caches | 1 024 | `cache_save()` returns `0` and logs a warning |
| ORM string buffer | 4 096 chars | `orm_addvar_string` returns `false` |
