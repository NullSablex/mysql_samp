# Options

Options are **optional**. A bare `mysql_connect("host", "user", "pass", "db")` uses the defaults below. Build an options handle only when you need a non-default port, a connection timeout or to opt out of auto-reconnect.

## API

```pawn
native mysql_options_new();
native bool:mysql_options_set_int(handle, option, value);
native bool:mysql_options_set_str(handle, option, const value[]);
```

- `mysql_options_new()` creates a fresh handle (`>= 1`) populated with the defaults.
- `mysql_options_set_int(handle, option, value)` returns `true` on success, `false` if the handle is invalid, the option does not exist, the option is a string-only option, or the integer is out of range for the option type (for example a negative port).
- `mysql_options_set_str(handle, option, value)` returns `true` on success, `false` if the handle is invalid, the option does not exist, or the option is an integer-only option.

Handles are not destroyed automatically. There is no explicit destroy native — once you pass the handle to `mysql_connect`, you can drop the reference. Memory used by the handle stays alive for the duration of the plugin (handles are stored in a `HashMap` keyed by id) and the cost is negligible (a small struct per handle).

## Usage

```pawn
new opts = mysql_options_new();
mysql_options_set_int(opts, MYSQL_OPT_PORT, 3307);
mysql_options_set_int(opts, MYSQL_OPT_CONNECT_TIMEOUT, 10);
mysql_options_set_int(opts, MYSQL_OPT_AUTO_RECONNECT, 0);

new g_mysql = mysql_connect("db.example.com", "user", "pass", "samp_db", opts);
```

The same handle can be reused across multiple `mysql_connect` calls.

## Available options

| Constant | Integer / String | Default | Description |
|---|---|---|---|
| `MYSQL_OPT_PORT` | int | `3306` | TCP port. Must fit in `u16` (`0..=65535`). |
| `MYSQL_OPT_SSL` | int (bool) | `0` (off) | SSL toggle — **currently a no-op**, see [SSL caveat](#ssl) below. |
| `MYSQL_OPT_SSL_CA` | string | empty | Path to a PEM CA file — **currently a no-op**, see [SSL caveat](#ssl) below. |
| `MYSQL_OPT_CONNECT_TIMEOUT` | int (seconds) | none | TCP connect timeout. Must fit in `u32`. |
| `MYSQL_OPT_AUTO_RECONNECT` | int (bool) | `1` (on) | Retry a query once when the server drops the connection (see [MYSQL_OPT_AUTO_RECONNECT](#mysql_opt_auto_reconnect)). |

### MYSQL_OPT_PORT

```pawn
mysql_options_set_int(opts, MYSQL_OPT_PORT, 3307);
```

Ignored when the host is a Unix socket (path starts with `/`).

### MYSQL_OPT_CONNECT_TIMEOUT

Time in seconds the plugin will wait for the initial TCP connection. Without this option the plugin waits indefinitely (the default of the `mysql` crate).

```pawn
mysql_options_set_int(opts, MYSQL_OPT_CONNECT_TIMEOUT, 10);
```

If the timeout expires, `mysql_connect` returns `0` and `mysql_errno(0)` returns `MYSQL_ERROR_CONNECTION_FAILED`.

### MYSQL_OPT_AUTO_RECONNECT

```pawn
mysql_options_set_int(opts, MYSQL_OPT_AUTO_RECONNECT, 1);  // default, retry once
mysql_options_set_int(opts, MYSQL_OPT_AUTO_RECONNECT, 0);  // do not retry, report immediately
```

**Behavior when enabled:** if a threaded query (`mysql_query` / `mysql_pquery`) fails with a connection-lost error (the `mysql` crate reports `code == 0` for IO and TCP errors), the plugin drops the current connection, fetches a new one from the pool and re-runs the same query once before surfacing the error to `OnQueryError`.

**Behavior when disabled:** the very first failure is reported. No retry.

When to disable: if your gamemode needs to know exactly when a reconnect happened — for instance, to reapply session-scoped state such as `SET @user_id` — turn this off and handle the recovery yourself in `OnQueryError`.

This option only affects in-flight queries. The TCP handshake inside `mysql_connect` is governed by `MYSQL_OPT_CONNECT_TIMEOUT`.

### SSL

> **Important caveat.** `MYSQL_OPT_SSL` and `MYSQL_OPT_SSL_CA` are accepted by `mysql_options_set_int` / `mysql_options_set_str` and the values are stored in the options struct, but the connection builder in [`src/connection.rs`](https://github.com/NullSablex/mysql_samp/blob/master/src/connection.rs) does **not** wire them through to the `mysql` crate yet — there is a `TODO` waiting for the `mysql` crate to expose rustls-friendly options. Setting these today has no effect on the connection: the plugin always connects in clear text.

The `mysql` crate is compiled with the `default-rust` feature (rustls is in the binary), so the SSL plumbing exists in the dependency tree — it just is not exposed by the plugin yet. Until that lands, plan around it (use a TLS-terminating proxy, or accept the limitation in your threat model).

## Setting a string option on an int-only option (or vice-versa)

The plugin rejects mismatched setters:

```pawn
mysql_options_set_str(opts, MYSQL_OPT_PORT, "3307");  // returns false, port is int
mysql_options_set_int(opts, MYSQL_OPT_SSL_CA, 1);     // returns false, ssl_ca is string
```

The only string option today is `MYSQL_OPT_SSL_CA`. Everything else is int.

## Out-of-range integers

`MYSQL_OPT_PORT` must fit in `u16`. `MYSQL_OPT_CONNECT_TIMEOUT` must fit in `u32`. Negative or oversized values are rejected:

```pawn
mysql_options_set_int(opts, MYSQL_OPT_PORT, -1);     // false
mysql_options_set_int(opts, MYSQL_OPT_PORT, 70000);  // false (over u16::MAX)
mysql_options_set_int(opts, MYSQL_OPT_CONNECT_TIMEOUT, -5);  // false
```

This is stricter than the old MySQL R41-4 plugin, which silently wrapped negative ports to large positive values. Catch the `false` return and fix the input.
