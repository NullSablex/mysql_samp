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
native bool:mysql_escape_string(const src[], dest[], max_len = sizeof(dest));
```

Pure escape function — no connection required because the escape rules are connection-independent under UTF-8 (which the plugin forces on every pool connection). Returns `true` on success, `false` only if writing into `dest` fails.

Characters escaped: `\0`, `\n`, `\r`, `\\`, `'`, `"`, `\x1a` (Ctrl-Z). Every other byte (including `\t`, control bytes, and multi-byte UTF-8) passes through unchanged.

```pawn
new escaped[128];
new input[]  = "It's a \"test\"";
mysql_escape_string(input, escaped);
// escaped = "It\'s a \"test\""
```

**Never call `mysql_escape_string` on a string and then pass the result through `%s`** — `%s` escapes again and you end up with `\\'` instead of `\'`.

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
