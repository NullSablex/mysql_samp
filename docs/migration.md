# Migration from MySQL R41-4

This page lists every signature and semantic difference between the **MySQL R41-4** plugin (BlueG / maddinat0r) and **mysql_samp**. For side-by-side gamemode snippets see [migration-examples.md](migration-examples.md). For a flat list of "what changed", see [migration-changes.md](migration-changes.md).

## Include

```pawn
// R41-4
#include <a_mysql>

// mysql_samp
#include <mysql_samp>
```

## Pawn tags

mysql_samp does **not** use custom Pawn tags. Every handle is a plain `int`.

| R41-4 | mysql_samp |
|---|---|
| `new MySQL:g_mysql` | `new g_mysql` |
| `new Cache:c = cache_save()` | `new c = cache_save()` |
| `new ORM:o = orm_create(...)` | `new o = orm_create(...)` |
| `new MySQLOpt:o = mysql_init_options()` | `new o = mysql_options_new()` |
| `MYSQL_INVALID_HANDLE` | plain `0` |

Remove every `MySQL:`, `Cache:`, `ORM:`, `MySQLOpt:`, `E_ORM_ERROR:` and `E_MYSQL_OPTION:` tag.

## Queries

| R41-4 | mysql_samp | Notes |
|---|---|---|
| `Cache:mysql_query(handle, query, use_cache)` | *removed* | The sync version blocked the server tick — removed by design |
| `mysql_tquery(handle, query, cb, fmt, ...)` | `mysql_query(connId, query, cb, fmt, ...)` | Same behavior (threaded, FIFO) |
| `mysql_pquery(handle, query, cb, fmt, ...)` | `mysql_pquery(connId, query, cb, fmt, ...)` | Same behavior (threaded, no order) |
| `mysql_query_file(file)` / `mysql_tquery_file(file)` | *removed* | Not supported |

> In R41-4, `mysql_query` was synchronous. In mysql_samp the same name is always threaded — equivalent to the old `mysql_tquery`.

## mysql_format: `%s` now escapes

In R41-4, `%s` inserted the raw string; you had to remember to use `%e` for escaping. In mysql_samp, **`%s` escapes by default** — the safe direction is now the default.

| Specifier | R41-4 | mysql_samp |
|---|---|---|
| `%d`, `%i` | int | int (unchanged) |
| `%f` | float | float (4 decimals — `{:.4}`) |
| `%s` | **raw** (no escape) | **escaped** |
| `%e` | escaped | escaped (alias of `%s`) |
| `%r` | *does not exist* | **raw**, no escape |
| `%%` | literal `%` | literal `%` |

How to migrate:

- `%e` → keep as `%e` or change to `%s`. Both escape.
- `%s` on user input → keep as `%s`. Now safe automatically.
- `%s` used to inject trusted constants (table names, fixed fragments) → change to `%r`.

```pawn
// R41-4
mysql_format(g_mysql, query, sizeof(query),
    "SELECT * FROM %s WHERE name = '%e'", tableName, playerName);

// mysql_samp
mysql_format(g_mysql, query, sizeof(query),
    "SELECT * FROM %r WHERE name = '%s'", tableName, playerName);
```

## mysql_escape_string

| R41-4 | mysql_samp |
|---|---|
| `mysql_escape_string(src, dest, max_len, MySQL:handle)` | `mysql_escape_string(src, dest, max_len)` |

`mysql_samp` does not take a handle: the escape is a pure function. The charset is always UTF-8 (the plugin forces `SET NAMES utf8mb4` on every new connection).

## Cache — by-ref → return value

The largest signature change. R41-4 used out-parameters; mysql_samp returns the value directly.

| R41-4 | mysql_samp | Change |
|---|---|---|
| `cache_get_row_count(&dest)` | `cache_get_row_count()` → `int` | by-ref → return |
| `cache_get_field_count(&dest)` | `cache_get_field_count()` → `int` | by-ref → return |
| `cache_get_result_count(&dest)` | *removed* | Multi-result sets |
| `cache_get_value_index(row, col, dest[], max_len)` | unchanged | — |
| `cache_get_value_index_int(row, col, &dest)` | `cache_get_value_index_int(row, col)` → `int` | by-ref → return |
| `cache_get_value_index_float(row, col, &Float:dest)` | `cache_get_value_index_float(row, col)` → `Float` | by-ref → return |
| `cache_get_value_name(row, name, dest[], max_len)` | unchanged | — |
| `cache_get_value_name_int(row, name, &dest)` | `cache_get_value_name_int(row, name)` → `int` | by-ref → return |
| `cache_get_value_name_float(row, name, &Float:dest)` | `cache_get_value_name_float(row, name)` → `Float` | by-ref → return |
| `cache_is_value_index_null(row, col, &bool:dest)` | `cache_is_value_index_null(row, col)` → `bool` | by-ref → return |
| `cache_is_value_name_null(row, name, &bool:dest)` | `cache_is_value_name_null(row, name)` → `bool` | by-ref → return |
| `cache_set_result(idx)` | *removed* | Multi-result sets |
| `cache_save()` | `cache_save()` | No `Cache:` tag |
| `cache_delete(Cache:id)` | `cache_delete(id)` | No tag |
| `cache_set_active(Cache:id)` | `cache_set_active(id)` | No tag |
| `cache_unset_active()` | unchanged | — |
| `cache_affected_rows()` | unchanged | — |
| `cache_insert_id()` | unchanged | — |
| `cache_warning_count()` | unchanged | — |
| `cache_get_query_exec_time(unit)` | `cache_get_query_exec_time()` | No `unit` parameter — value is always milliseconds |
| `cache_get_query_string(dest, max_len)` | unchanged | — |
| `cache_is_any_active()` | unchanged | — |
| `cache_is_valid(Cache:id)` | `cache_is_valid(id)` | No tag |

R41-4 stocks `cache_num_rows()` / `cache_num_fields()` map directly to `cache_get_row_count()` / `cache_get_field_count()`.

## ORM

| R41-4 | mysql_samp | Difference |
|---|---|---|
| `ORM:orm_create(table, MySQL:handle)` | `orm_create(table, connId)` | No tags |
| `orm_destroy(ORM:id)` | `orm_destroy(orm_id)` → `bool` | Now returns `bool` |
| `E_ORM_ERROR:orm_errno(ORM:id)` | `orm_errno(orm_id)` → `int` | No tag |
| `orm_select` / `orm_update` / `orm_insert` / `orm_delete` / `orm_save` | same names, no `ORM:` tag | — |
| `orm_load(...)` | *removed* | Was an alias of `orm_select` |
| `orm_apply_cache(ORM:id, row, result_idx)` | `orm_apply_cache(orm_id, row)` | No `result_idx` (single result set) |
| `orm_addvar_int` / `orm_addvar_float` / `orm_addvar_string` | same names, no tag | `max_len` now capped at 4096 |
| `orm_delvar(ORM:id, col)` | `orm_delvar(orm_id, col)` | No tag |
| `orm_clear_vars(ORM:id)` | `orm_clear_vars(orm_id)` | No tag |
| `orm_setkey(ORM:id, col)` | `orm_setkey(orm_id, col)` | No tag |

ORM error enum:

| R41-4 | mysql_samp |
|---|---|
| `ERROR_INVALID = 0` | *does not exist* |
| `ERROR_OK = 1` | `ORM_OK = 0` |
| `ERROR_NO_DATA = 2` | `ORM_NO_DATA = 1` |

## Connection

| R41-4 | mysql_samp |
|---|---|
| `MySQL:mysql_connect(host, user, pass, db, MySQLOpt:opt)` | `mysql_connect(host, user, pass, db, options)` |
| `MySQLOpt:mysql_init_options()` | `mysql_options_new()` |
| `mysql_set_option(opt, type, ...)` | `mysql_options_set_int(opt, type, val)` / `mysql_options_set_str(opt, type, val[])` |
| `mysql_close(MySQL:handle)` | `mysql_close(connId)` |
| `mysql_stat(dest, max_len, MySQL:handle)` | `mysql_status(connId, dest, max_len)` |
| `mysql_connect_file(file)` | *removed* |
| `mysql_global_options(type, val)` | *removed* |

**Argument order changed:** in R41-4, the `MySQL:handle` is the last (optional) parameter on most calls. In mysql_samp, the `connId` is always the **first** parameter.

## Error

| R41-4 | mysql_samp | Difference |
|---|---|---|
| `mysql_errno(MySQL:handle)` | `mysql_errno(connId = 0)` | No tag, `connId` first |
| `mysql_error(dest, max_len, MySQL:handle)` | `mysql_error(connId, dest, max_len)` | `connId` first (was last) |

## Charset

| R41-4 | mysql_samp |
|---|---|
| `mysql_set_charset(charset, MySQL:handle)` | `mysql_set_charset(connId, charset)` |
| `mysql_get_charset(dest, max_len, MySQL:handle)` | `mysql_get_charset(connId, dest, max_len)` |

`connId` moved to the first position in both.

## Log

| R41-4 | mysql_samp |
|---|---|
| `mysql_log(E_LOGLEVEL:level)` — bitflags (`DEBUG = 1`, `INFO = 2`, `WARNING = 4`, `ERROR = 8`) | `mysql_log(level)` — sequential (`NONE = 0`, `ERROR = 1`, `WARNING = 2`, `INFO = 3`, `ALL = 4`) |

## Callback format

The mysql_samp callback format accepts both `"d"` and `"i"` as int. No change needed if you already use `"i"`.

| Char | R41-4 | mysql_samp |
|---|---|---|
| `i` | int | int |
| `d` | *not accepted* | int |
| `f` | float | float |
| `s` | string | string |

## Forward

| R41-4 | mysql_samp |
|---|---|
| `OnQueryError(errorid, error[], callback[], query[], MySQL:handle)` | `OnQueryError(errorid, error[], callback[], query[], connId)` |

Same parameters, no `MySQL:` tag on the last one.

## Removed natives

| R41-4 | Why |
|---|---|
| `Cache:mysql_query(handle, query, use_cache)` | The synchronous version blocked the server tick |
| `mysql_tquery_file(...)` / `mysql_query_file(...)` | Not supported |
| `mysql_connect_file(file)` | Not supported |
| `mysql_global_options(type, val)` | Not applicable |
| `cache_get_result_count(&dest)` | Multi-result sets — not used by SA-MP gamemodes |
| `cache_set_result(idx)` | Multi-result sets |
| `orm_load(...)` | Was an alias of `orm_select` |

## Step-by-step

### 1. Swap the include

```pawn
// before
#include <a_mysql>

// after
#include <mysql_samp>
```

### 2. Rename query calls

```pawn
// before
mysql_tquery(g_mysql, query, "OnData", "i", playerid);

// after (both forms work)
mysql_query(g_mysql, query, "OnData", "d", playerid);
mysql_query(g_mysql, query, "OnData", "i", playerid);
```

### 3. Update mysql_format placeholders

```pawn
// before (R41-4: %s = raw, %e = escaped)
mysql_format(g_mysql, query, sizeof(query),
    "SELECT * FROM %s WHERE name = '%e'", table, name);

// after (mysql_samp: %r = raw, %s = escaped)
mysql_format(g_mysql, query, sizeof(query),
    "SELECT * FROM %r WHERE name = '%s'", table, name);
```

### 4. Update connect + options

```pawn
// before
new MySQLOpt:opt = mysql_init_options();
mysql_set_option(opt, SERVER_PORT, 3307);
new MySQL:g_mysql = mysql_connect("127.0.0.1", "root", "pass", "db", opt);

// after
new opt = mysql_options_new();
mysql_options_set_int(opt, MYSQL_OPT_PORT, 3307);
new g_mysql = mysql_connect("127.0.0.1", "root", "pass", "db", opt);

// or, if port 3306 is fine, drop the options entirely:
new g_mysql = mysql_connect("127.0.0.1", "root", "pass", "db");
```

### 5. Update mysql_escape_string

```pawn
// before
mysql_escape_string(input, escaped, sizeof(escaped), g_mysql);

// after
mysql_escape_string(input, escaped);
```

### 6. Convert cache reads to return-value form

```pawn
// before
new score;
cache_get_value_name_int(0, "score", score);

// after
new score = cache_get_value_name_int(0, "score");
```

### 7. Convert row/field counts

```pawn
// before
new rows;
cache_get_row_count(rows);          // by-ref
new rows2 = cache_num_rows();       // stock wrapper

// after
new rows = cache_get_row_count();
```

### 8. Strip every Pawn tag

```pawn
// before
new MySQL:g_mysql;
new Cache:c = cache_save();
new ORM:o   = orm_create("table", g_mysql);

// after
new g_mysql;
new c = cache_save();
new o = orm_create("table", g_mysql);
```

### 9. Swap `mysql_error` argument order

```pawn
// before
new err[256];
mysql_error(err, sizeof(err), g_mysql);

// after
new err[256];
mysql_error(g_mysql, err);
```

### 10. Compile and test

Compile the gamemode against the new include and run it. Watch `logs/mysql.log` for errors that the console summarized.
