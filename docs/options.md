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
| `MYSQL_OPT_SSL` | int (bool) | `0` (off) | Enables TLS for the connection. See [SSL](#ssl). |
| `MYSQL_OPT_SSL_CA` | string | empty | Path to a PEM/DER CA file used to verify the server. See [SSL](#ssl). |
| `MYSQL_OPT_SSL_CERT` | string | empty | Client certificate chain for mutual TLS. Requires `MYSQL_OPT_SSL_KEY`. |
| `MYSQL_OPT_SSL_KEY` | string | empty | Client private key for mutual TLS. Requires `MYSQL_OPT_SSL_CERT`. |
| `MYSQL_OPT_SSL_VERIFY_CERT` | int (bool) | `1` (on) | Verifies the server certificate and hostname. Disabling it is dangerous. |
| `MYSQL_OPT_POOL_SIZE` | int | driver default (100) | Maximum connections the pool may open. Must be ≥ 1. |
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

`MYSQL_OPT_SSL` turns on TLS for the connection. The TLS backend (rustls) is compiled into the binary — no OpenSSL, no system library to install.

```pawn
new opts = mysql_options_new();
mysql_options_set_int(opts, MYSQL_OPT_SSL, 1);
mysql_options_set_str(opts, MYSQL_OPT_SSL_CA, "certs/ca.pem");
```

#### Which certificates are trusted

Without `MYSQL_OPT_SSL_CA`, the driver trusts the **webpki root bundle compiled into the plugin** (the Mozilla root list) — *not* your operating system's trust store. That distinction matters in practice: a MySQL server using a self-signed certificate or an internal/company CA — the usual setup for a game server — is **not** in that bundle, so the connection fails until you point `MYSQL_OPT_SSL_CA` at the CA that signed it.

Use the public roots only when connecting to a managed provider whose certificate chains to a public CA (RDS, Cloud SQL, PlanetScale, …).

#### Mutual TLS

When the server requires a client certificate, supply both halves:

```pawn
mysql_options_set_str(opts, MYSQL_OPT_SSL_CERT, "certs/client-cert.pem");
mysql_options_set_str(opts, MYSQL_OPT_SSL_KEY,  "certs/client-key.pem");
```

Setting only one of the two logs a warning and the client certificate is ignored — TLS still proceeds without it. Keys may be PKCS#1, PKCS#8 or SEC1, in PEM or DER.

#### TLS needs a TCP host

`MYSQL_OPT_SSL` has no effect when the host is a unix socket (a path starting with `/`) — there is nothing to encrypt on a local socket. Use a TCP host such as `127.0.0.1` or a hostname.

#### Disabling verification

```pawn
mysql_options_set_int(opts, MYSQL_OPT_SSL_VERIFY_CERT, 0);  // dangerous
```

This accepts **any** certificate and skips the hostname check. Traffic stays encrypted, but an attacker who can intercept the connection can present their own certificate and read or alter every query — which is exactly what TLS is supposed to prevent. It logs a warning on every connect.

Reach for `MYSQL_OPT_SSL_CA` instead: pointing at the CA file is the correct fix for a self-signed server, and it keeps verification intact.

## Setting a string option on an int-only option (or vice-versa)

The plugin rejects mismatched setters:

```pawn
mysql_options_set_str(opts, MYSQL_OPT_PORT, "3307");  // returns false, port is int
mysql_options_set_int(opts, MYSQL_OPT_SSL_CA, 1);     // returns false, ssl_ca is string
```

The string options are `MYSQL_OPT_SSL_CA`, `MYSQL_OPT_SSL_CERT` and `MYSQL_OPT_SSL_KEY`. Everything else is int.

## Out-of-range integers

`MYSQL_OPT_PORT` must fit in `u16`. `MYSQL_OPT_CONNECT_TIMEOUT` must fit in `u32`. Negative or oversized values are rejected:

```pawn
mysql_options_set_int(opts, MYSQL_OPT_PORT, -1);     // false
mysql_options_set_int(opts, MYSQL_OPT_PORT, 70000);  // false (over u16::MAX)
mysql_options_set_int(opts, MYSQL_OPT_CONNECT_TIMEOUT, -5);  // false
```

This is stricter than the old MySQL R41-4 plugin, which silently wrapped negative ports to large positive values. Catch the `false` return and fix the input.
