# Error handling

The plugin reports failures through three channels:

1. **Plugin-level errno** — `mysql_errno(connId)` / `mysql_error(connId, dest)`. Each connection has its own error slot; `connId == 0` reads the global state (set by connect failures, ORM build failures, etc.).
2. **Threaded-query forward** — `OnQueryError` is fired on every loaded AMX when a `mysql_query` / `mysql_pquery` fails.
3. **Console + `logs/mysql.log`** — short message in the console, full details in the file.

## mysql_errno

```pawn
native mysql_errno(connId = 0);
```

Returns one of the plugin error codes below. `connId == 0` returns the **global error**, useful for `mysql_connect` failures.

| Constant | Value | Set when |
|---|---|---|
| `MYSQL_OK` | 0 | No error since the last reset |
| `MYSQL_ERROR_CONNECTION_FAILED` | 1 | `mysql_connect` failed (global) |
| `MYSQL_ERROR_INVALID_OPTIONS` | 2 | `mysql_connect` received an unknown options handle (global) |
| `MYSQL_ERROR_INVALID_CONNECTION` | 3 | `mysql_query` / `mysql_pquery` / `orm_create` received an unknown `connId` (global) |
| `MYSQL_ERROR_PING_FAILED` | 4 | `mysql_status` could not fetch server stats (per-connection) |
| `MYSQL_ERROR_QUERY_FAILED` | 5 | A threaded query failed (per-connection); the detailed message is the MySQL server text |
| `MYSQL_ERROR_NO_CACHE_ACTIVE` | 6 | `orm_apply_cache` was called without an active cache (global) |
| `MYSQL_ERROR_INVALID_ORM` | 7 | An ORM native received an unknown `orm_id`, or ORM insert/save could not build a query (global) |
| `MYSQL_ERROR_ORM_KEY_NOT_SET` | 8 | ORM select/update/delete could not build a query because the key column is not set or no variables are bound (global) |

```pawn
g_mysql = mysql_connect("127.0.0.1", "root", "pw", "samp_db");
if (mysql_errno() != MYSQL_OK)
{
    new msg[256];
    mysql_error(0, msg);
    printf("connect failed (%d): %s", mysql_errno(), msg);
    return 1;
}
```

## mysql_error

```pawn
native bool:mysql_error(connId, dest[], max_len = sizeof(dest));
```

Writes the text of the last error into `dest`. Returns `true` on success.

When `connId == 0`, reads the global error message. When `connId` is unknown, also falls back to the global error.

## OnQueryError

```pawn
forward OnQueryError(errorid, const error[], const callback[], const query[], connId);
```

Fired on every loaded AMX when a threaded query fails. Parameters:

| Parameter | Type | Description |
|---|---|---|
| `errorid` | int | MySQL server error code (1062, 1045, 1064, …) or `0` for transport-level errors (TCP drop, IO error) |
| `error` | string | Full error text from the `mysql` crate |
| `callback` | string | Name of the public the query asked for (empty if fire-and-forget) |
| `query` | string | Exact SQL that was sent to the server |
| `connId` | int | Id of the connection that produced the error |

```pawn
public OnQueryError(errorid, const error[], const callback[], const query[], connId)
{
    printf("[mysql %d] %s", errorid, error);
    printf("  callback: %s", callback);
    printf("  query:    %s", query);
    printf("  conn:     %d", connId);
    return 1;
}
```

The plugin also sets the per-connection errno to `MYSQL_ERROR_QUERY_FAILED` after firing the forward — so `mysql_errno(connId)` / `mysql_error(connId, ...)` can be read afterwards if you want the message in a different context.

### Common MySQL error codes

| Code | Cause |
|---|---|
| 1045 | Access denied (wrong user/password) |
| 1049 | Unknown database |
| 1062 | Duplicate entry (UNIQUE / PRIMARY KEY violation) |
| 1064 | SQL syntax error |
| 1146 | Table does not exist |
| 1451 | Foreign-key constraint blocked the operation |
| 2002 | Could not connect to the server (host unreachable) |
| 2006 | "MySQL server has gone away" |
| 0 | Transport/IO error — the `mysql` crate uses `0` for non-MySQL failures such as TCP drops. When `MYSQL_OPT_AUTO_RECONNECT` is on, the plugin retries the query once before reporting it. |

## Logs

### Console

The console gets only generic messages, never query text or credentials:

```
[MySQL] Connection failed (error 1). See logs/mysql.log for details.
[MySQL] Query failed on connection 1 (error 1064). See logs/mysql.log for details.
[MySQL] Query result truncated to 100000 rows.
```

### logs/mysql.log

Detailed entries with timestamp:

```
[2026-05-18 14:30:15] [ERROR] Pool creation failed: Access denied for user 'root'@'localhost'
[2026-05-18 14:30:20] [ERROR] Query error: You have an error in your SQL syntax; check the manual ...
[2026-05-18 14:30:30] [WARNING] cache_save failed: maximum saved caches reached (1024).
```

If the directory or the file is not writable, the plugin emits **one** console error of the form `[MySQL] Failed to write logs/mysql.log: <io error>. Further file-write errors will be suppressed.` and then stops trying. This avoids flooding the console when the disk is full or permissions are wrong.

### Log level

`mysql_log` adjusts the minimum severity that the plugin emits:

```pawn
mysql_log(MYSQL_LOG_NONE);     // 0 — nothing
mysql_log(MYSQL_LOG_ERROR);    // 1 — errors only
mysql_log(MYSQL_LOG_WARNING);  // 2 — errors + warnings
mysql_log(MYSQL_LOG_INFO);     // 3 — errors + warnings + info
mysql_log(MYSQL_LOG_ALL);      // 4 — everything (default)
```

The setting is atomic and takes effect immediately.

## Practical checklist

1. **Always check `mysql_errno()` after `mysql_connect`** — a failed connect returns `0`. Without a check, the gamemode happily issues queries against connection `0`, which is invalid.
2. **Implement `OnQueryError`** — a missing implementation is not a compile error, but you will be flying blind when a query fails in production.
3. **Read `logs/mysql.log` for the long error text** — the console does not have the SQL or the original server message.
4. **Use `mysql_format` with `%s`** — it escapes by default and removes a whole class of SQL injection bugs.
5. **Guard cache reads with `cache_get_row_count()`** — avoids spurious `-1` / `0` / empty-string readings when the result is empty.
