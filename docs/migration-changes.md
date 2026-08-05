# What changed (R41-4 → mysql_samp)

Flat reference of every difference between MySQL R41-4 and mysql_samp. See [migration.md](migration.md) for the explained guide and [migration-examples.md](migration-examples.md) for side-by-side snippets.

## Include

```pawn
// R41-4
#include <a_mysql>

// mysql_samp
#include <mysql_samp>
```

## Tags removed

mysql_samp does not use custom Pawn tags. Every handle is a plain `int`.

| R41-4 | mysql_samp |
|---|---|
| `new MySQL:g_mysql` | `new g_mysql` |
| `new Cache:c = cache_save();` | `new c = cache_save();` |
| `new ORM:o = orm_create(...)` | `new o = orm_create(...)` |
| `cache_delete(Cache:c);` | `cache_delete(c);` |
| `MySQLOpt:o = mysql_init_options()` | `new o = mysql_options_new()` |
| `MYSQL_INVALID_HANDLE` | plain `0` |

## Callback format

`"d"` and `"i"` are both accepted as int. If you already use `"i"`, no change is needed.

| R41-4 | mysql_samp | Meaning |
|---|---|---|
| `"i"` | `"d"` or `"i"` | int |
| `"f"` | `"f"` | float |
| `"s"` | `"s"` | string |

```pawn
// R41-4
mysql_tquery(g_mysql, query, "OnLoad", "i", playerid);

// mysql_samp (both work)
mysql_query(g_mysql, query, "OnLoad", "d", playerid);
mysql_query(g_mysql, query, "OnLoad", "i", playerid);
```

## mysql_format specifiers

| Specifier | R41-4 | mysql_samp |
|---|---|---|
| `%d`, `%i` | int | int |
| `%f` | float | float (4 decimals — `{:.4}`) |
| `%s` | **raw** (no escape) | **escaped** |
| `%e` | escaped | escaped (alias of `%s`) |
| `%r` | *does not exist* | **raw** (no escape) |
| `%%` | literal | literal |

**Rule of thumb:** where you used `%e`, use `%s` (both escape). Where you used `%s` to inject a table or column name, switch to `%r`.

## Queries

| R41-4 | mysql_samp | Behavior |
|---|---|---|
| `Cache:mysql_query(handle, query, use_cache)` | *removed* | The synchronous version blocked the server tick |
| `mysql_tquery(handle, query, cb, fmt, ...)` | `mysql_query(connId, query, cb, fmt, ...)` | Threaded, FIFO ordering (same as before) |
| `mysql_pquery(handle, query, cb, fmt, ...)` | `mysql_pquery(connId, query, cb, fmt, ...)` | Threaded, no order (same as before) |

Every query is now non-blocking. The cache is only valid inside the callback (or after `cache_set_active` on a saved id).

## Connection

| R41-4 | mysql_samp |
|---|---|
| `MySQL:mysql_connect(host, user, pass, db, MySQLOpt:opt)` | `mysql_connect(host, user, pass, db, options)` |
| `MySQLOpt:mysql_init_options()` | `mysql_options_new()` |
| `mysql_set_option(opt, type, ...)` | `mysql_options_set_int(opt, type, val)` / `mysql_options_set_str(opt, type, val[])` |
| `mysql_escape_string(src, dest, max_len, MySQL:handle)` | `mysql_escape_string(src, dest, max_len, connId)` |
| `mysql_stat(dest, max_len, MySQL:handle)` | `mysql_status(connId, dest, max_len)` |
| `mysql_error(dest, max_len, MySQL:handle)` | `mysql_error(connId, dest, max_len)` |
| `mysql_errno(MySQL:handle)` | `mysql_errno(connId)` |
| `mysql_set_charset(charset, MySQL:handle)` | `mysql_set_charset(connId, charset)` |
| `mysql_get_charset(dest, max_len, MySQL:handle)` | `mysql_get_charset(connId, dest, max_len)` |
| `mysql_close(MySQL:handle)` | `mysql_close(connId)` |
| `mysql_unprocessed_queries(MySQL:handle)` | `mysql_unprocessed_queries()` |
| *does not exist* | `mysql_log(level)` |

**Argument order changed.** In R41-4 the handle is the last (optional) parameter on most calls. In mysql_samp the `connId` is always the **first** parameter.

**Port option name changed.** In R41-4 you wrote `mysql_set_option(opt, SERVER_PORT, 3307)`. In mysql_samp it is `mysql_options_set_int(opt, MYSQL_OPT_PORT, 3307)`.

```pawn
// R41-4
new MySQLOpt:opt = mysql_init_options();
mysql_set_option(opt, SERVER_PORT, 3307);
new MySQL:g_mysql = mysql_connect("127.0.0.1", "root", "pass", "db", opt);

// mysql_samp
new opt = mysql_options_new();
mysql_options_set_int(opt, MYSQL_OPT_PORT, 3307);
new g_mysql = mysql_connect("127.0.0.1", "root", "pass", "db", opt);

// mysql_samp (default port 3306 — no options needed)
new g_mysql = mysql_connect("127.0.0.1", "root", "pass", "db");
```

## Cache — by-ref → return value

The biggest single change. R41-4 wrote the result through a pointer; mysql_samp returns the value directly.

### Row/field counts

| R41-4 | mysql_samp |
|---|---|
| `cache_get_row_count(&dest)` | `cache_get_row_count()` → `int` |
| `cache_get_field_count(&dest)` | `cache_get_field_count()` → `int` |
| `cache_num_rows()` (stock wrapper) | `cache_get_row_count()` |
| `cache_num_fields()` (stock wrapper) | `cache_get_field_count()` |

```pawn
// R41-4
new rows;
cache_get_row_count(rows);
// or
new rows2 = cache_num_rows();

// mysql_samp
new rows = cache_get_row_count();
```

### Values by index

| R41-4 | mysql_samp |
|---|---|
| `cache_get_value_index(row, col, dest[], max_len)` | unchanged |
| `cache_get_value_index_int(row, col, &dest)` | `cache_get_value_index_int(row, col)` → `int` |
| `cache_get_value_index_float(row, col, &Float:dest)` | `cache_get_value_index_float(row, col)` → `Float` |

```pawn
// R41-4
new score;
cache_get_value_index_int(0, 2, score);

// mysql_samp
new score = cache_get_value_index_int(0, 2);
```

### Values by name

| R41-4 | mysql_samp |
|---|---|
| `cache_get_value_name(row, col_name, dest[], max_len)` | unchanged |
| `cache_get_value_name_int(row, col_name, &dest)` | `cache_get_value_name_int(row, col_name)` → `int` |
| `cache_get_value_name_float(row, col_name, &Float:dest)` | `cache_get_value_name_float(row, col_name)` → `Float` |

Column name lookup is case-insensitive in both plugins.

```pawn
// R41-4
new score;
cache_get_value_name_int(0, "score", score);
new Float:pos_x;
cache_get_value_name_float(0, "pos_x", pos_x);

// mysql_samp
new score = cache_get_value_name_int(0, "score");
new Float:pos_x = cache_get_value_name_float(0, "pos_x");
```

### NULL checks

| R41-4 | mysql_samp |
|---|---|
| `cache_is_value_index_null(row, col, &bool:dest)` | `cache_is_value_index_null(row, col)` → `bool` |
| `cache_is_value_name_null(row, col_name, &bool:dest)` | `cache_is_value_name_null(row, col_name)` → `bool` |

```pawn
// R41-4
new bool:is_null;
cache_is_value_name_null(0, "email", is_null);

// mysql_samp
new bool:is_null = cache_is_value_name_null(0, "email");
```

## Cache — natives with unchanged signatures

- `cache_get_value_index(row, col, dest[], max_len)`
- `cache_get_value_name(row, col_name, dest[], max_len)`
- `cache_get_field_name(idx, dest[], max_len)`
- `cache_affected_rows()`
- `cache_insert_id()`
- `cache_warning_count()`
- `cache_save()`
- `cache_delete(cache_id)` (without the `Cache:` tag)
- `cache_set_active(cache_id)` (without the `Cache:` tag)
- `cache_unset_active()`
- `cache_is_any_active()`
- `cache_is_valid(cache_id)` (without the `Cache:` tag)
- `cache_get_query_string(dest[], max_len)`

## Cache — new natives

| Native | Description |
|---|---|
| `cache_get_field_type(idx)` | Raw MySQL `ColumnType` byte for the given column index |
| `cache_get_query_exec_time()` | Server-side execution time in milliseconds |

## Cache — removed natives

| R41-4 | Why |
|---|---|
| `cache_get_result_count(&dest)` | Multi-result sets — unused by SA-MP gamemodes |
| `cache_set_result(idx)` | Multi-result sets |
| `cache_get_query_exec_time(unit)` | Replaced by the no-argument version (always ms) |

## ORM

The API is almost identical, with these differences:

| R41-4 | mysql_samp | Difference |
|---|---|---|
| `ORM:orm_create(table, MySQL:handle)` | `orm_create(table, connId)` | No tags |
| `orm_destroy(ORM:id)` | `orm_destroy(orm_id)` → `bool` | Now returns `bool` |
| `E_ORM_ERROR:orm_errno(ORM:id)` | `orm_errno(orm_id)` → `int` | No enum tag |
| `orm_apply_cache(ORM:id, row, result_idx)` | `orm_apply_cache(orm_id, row)` | No `result_idx` (single result set) |
| `orm_load(...)` | *removed* | Was an alias of `orm_select` |
| `orm_addvar_string(orm, var, max_len, col)` | unchanged | `max_len` now capped at 4096 |

ORM error enum:

| R41-4 | mysql_samp |
|---|---|
| `ERROR_INVALID = 0` | *does not exist* |
| `ERROR_OK = 1` | `ORM_OK = 0` |
| `ERROR_NO_DATA = 2` | `ORM_NO_DATA = 1` |

The rest of the ORM natives (`orm_select`, `orm_update`, `orm_insert`, `orm_delete`, `orm_save`, `orm_addvar_*`, `orm_delvar`, `orm_clear_vars`, `orm_setkey`) keep the same signatures, only without the `ORM:` tag.

## Error

| R41-4 | mysql_samp | Difference |
|---|---|---|
| `mysql_errno(MySQL:handle)` | `mysql_errno(connId = 0)` | No tag, `connId` first |
| `mysql_error(dest, max_len, MySQL:handle)` | `mysql_error(connId, dest, max_len)` | `connId` first (was last) |

## Log

| R41-4 | mysql_samp |
|---|---|
| `mysql_log(E_LOGLEVEL:level)` — bitflags (DEBUG=1, INFO=2, WARNING=4, ERROR=8) | `mysql_log(level)` — sequential (NONE=0, ERROR=1, WARNING=2, INFO=3, ALL=4) |

```pawn
// R41-4
mysql_log(ERROR | WARNING);

// mysql_samp
mysql_log(MYSQL_LOG_WARNING);  // includes ERROR
```

## Forward

| R41-4 | mysql_samp |
|---|---|
| `OnQueryError(errorid, error[], callback[], query[], MySQL:handle)` | `OnQueryError(errorid, error[], callback[], query[], connId)` |

Same parameters, no `MySQL:` tag on the last one.

## Mandatory checklist

1. Replace `#include <a_mysql>` with `#include <mysql_samp>`.
2. Replace `mysql_tquery` with `mysql_query`.
3. Replace `%s` (raw) with `%r` in `mysql_format` wherever the old `%s` was intentionally unescaped.
4. Keep the connection on `mysql_escape_string` — it now selects the escaping rules for the server's `sql_mode`. Better still, move player input to prepared statements (`mysql_stmt_*`).
5. Replace `cache_num_rows()` with `cache_get_row_count()`.
6. Convert `cache_get_value_*_int` / `cache_get_value_*_float` from by-ref to return value.
7. Convert `cache_is_value_*_null` from by-ref to return value.
8. Convert `cache_get_row_count` / `cache_get_field_count` from by-ref to return value.
9. Strip every tag (`MySQL:`, `Cache:`, `ORM:`, `MySQLOpt:`, `E_ORM_ERROR:`).
10. Replace `mysql_init_options()` with `mysql_options_new()`.
11. Replace `mysql_set_option(opt, type, ...)` with `mysql_options_set_int` / `mysql_options_set_str`.
12. Reverse the parameter order of `mysql_error` (connId is now first).
13. Replace `mysql_stat` with `mysql_status` (connId first).
14. Move any code that read the cache after a synchronous `mysql_query` into a proper callback.

**Optional:**

- `"i"` → `"d"` in callback formats (both work in mysql_samp).
