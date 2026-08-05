# mysql_samp examples

Runnable Pawn snippets showing how to use the plugin on **SA-MP** and **Open Multiplayer**. The plugin natives (`mysql_*`, `cache_*`, `orm_*`) and the `OnQueryError` forward are identical across both servers — every snippet here exercises only the plugin API.

> **Start with [`08_prepared_statements.pwn`](08_prepared_statements.pwn) for anything involving player input.** `mysql_format` (example 04) escapes values into the SQL text, which is correct only as long as the escaping matches the server's `sql_mode`. Prepared statements bind values server-side, so there is nothing to escape and nothing to get wrong.

| File | Topic |
|---|---|
| [`01_basic_connection.pwn`](01_basic_connection.pwn) | Connect / close, with and without `mysql_options_new()` |
| [`02_threaded_query.pwn`](02_threaded_query.pwn) | `mysql_query` + callback, read cache by column name / index |
| [`03_parallel_query.pwn`](03_parallel_query.pwn) | `mysql_pquery` for unordered, parallel work |
| [`04_escape_and_format.pwn`](04_escape_and_format.pwn) | `mysql_format` (`%s` auto-escape, `%e`, `%r`, `%d`, `%f`) |
| [`05_orm.pwn`](05_orm.pwn) | ORM: bind Pawn vars to columns, `orm_save` / `orm_select` |
| [`06_ssl.pwn`](06_ssl.pwn) | TLS: `MYSQL_OPT_SSL`, CA pinning, mutual TLS, verification (v1.2.0+) |
| [`07_error_handling.pwn`](07_error_handling.pwn) | `OnQueryError`, `mysql_errno`, `mysql_error` |
| [`08_prepared_statements.pwn`](08_prepared_statements.pwn) | `mysql_stmt_*` — **the safe way to pass player input** |
| [`09_transactions.pwn`](09_transactions.pwn) | `mysql_transaction_*` — all-or-nothing batches |
| [`10_password_hashing.pwn`](10_password_hashing.pwn) | Argon2id: `mysql_hash_password` / `mysql_verify_password` |
| [`11_config_and_scripts.pwn`](11_config_and_scripts.pwn) | `mysql_connect_file`, `mysql_query_file`, multiple result sets |

### Companion files

| File | Used by |
|---|---|
| [`mysql.ini.example`](mysql.ini.example) | Template for `mysql_connect_file`. Copy to `mysql.ini`, fill it in, and **add it to `.gitignore`** |
| [`schema.sql`](schema.sql) | Fixture for `mysql_query_file`. Deliberately contains semicolons inside comments and a string literal, so it also exercises the statement splitter |

## Compiling

The examples assume the include path is set up so that `<mysql_samp>` resolves to [`../include/mysql_samp.inc`](../include/mysql_samp.inc).

```bash
pawncc -i../include 01_basic_connection.pwn
```

Or copy `mysql_samp.inc` into your `pawno/include/` (SA-MP) or `qawno/include/` (open.mp) folder and compile from inside the gamemode tree.

## Installing the plugin

### SA-MP

Drop `mysql_samp.so` (Linux) or `mysql_samp.dll` (Windows) into `plugins/` and register it in `server.cfg`:

```
plugins mysql_samp.so
```

### Open Multiplayer — native component (recommended)

Drop the binary into the `components/` folder. open.mp auto-discovers it on start and loads it via `ComponentEntryPoint`. **No `config.json` entry is required** — listing the file in `components` is not a thing; the folder itself IS the registration.

### Open Multiplayer — legacy mode

Drop the binary into `plugins/` and declare it under `legacy_plugins` in `config.json` (legacy plugins must be listed explicitly, unlike native components):

```json
{
  "pawn": {
    "legacy_plugins": ["mysql_samp"]
  }
}
```

## Conventions used across the examples

- Connection credentials are placed at the top as `#define`s — replace with values from a config file in real code. [`11_config_and_scripts.pwn`](11_config_and_scripts.pwn) shows how, with `mysql_connect_file`.
- One global `g_MysqlConn` holds the connection id; `0` means "not connected".
- Threaded callbacks read from the implicit active cache; no need to call `cache_set_active` unless you persisted the cache with `cache_save`.
- All queries are non-blocking — the gamemode never waits on MySQL. That includes password hashing.
- Callbacks for `mysql_hash_password` / `mysql_verify_password` receive the **result first**, then the extras from the format string.
