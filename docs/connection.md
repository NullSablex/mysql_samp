# Connection

Each `mysql_connect` call creates a [`mysql::Pool`](https://docs.rs/mysql/latest/mysql/struct.Pool.html) that worker threads share. The plugin always issues `SET NAMES utf8mb4` on every new connection, so all string round-trips are UTF-8 safe by default.

## mysql_connect

```pawn
native mysql_connect(const host[], const user[], const password[], const database[], options = 0);
```

| Parameter | Type | Description |
|---|---|---|
| `host` | string | Hostname, IPv4, IPv6 in brackets (`[::1]`), **or** absolute path to a Unix socket (path starts with `/`) |
| `user` | string | MySQL user |
| `password` | string | MySQL password (may be empty) |
| `database` | string | Default schema for the connection |
| `options` | int | Options handle from `mysql_options_new`. `0` means "use defaults" |

**Returns:** the connection id (`>= 1`) on success, `0` on failure. On failure the global error state is populated and `mysql_errno(0)` / `mysql_error(0, …)` describe the cause.

### TCP example

```pawn
new g_mysql;

public OnGameModeInit()
{
    g_mysql = mysql_connect("127.0.0.1", "root", "password", "samp_db");
    if (g_mysql == 0)
    {
        new err[256];
        mysql_error(0, err);
        printf("[MySQL] connect failed: %s", err);
        return 1;
    }
    printf("[MySQL] connected (id=%d)", g_mysql);
    return 1;
}
```

### IPv6

IPv6 works, and the address goes in `host` like any other. **Put it in square brackets:**

```pawn
g_mysql = mysql_connect("[::1]", "root", "password", "samp_db");
g_mysql = mysql_connect("[2001:db8::3306:1]", "root", "password", "samp_db");
```

The brackets are not decoration. With them, the address is parsed and validated as an IPv6 address; without them it is taken as a *host name* and handed to the resolver, which happens to accept a literal — so it usually connects, but nothing checks the address and a typo surfaces as a name-resolution failure instead of a clear error.

The port is never part of `host`. It stays in `MYSQL_OPT_PORT`, exactly as with IPv4, so there is no `[::1]:3306` form:

```pawn
new opt = mysql_options_new();
mysql_options_set_int(opt, MYSQL_OPT_PORT, 3307);
g_mysql = mysql_connect("[::1]", "root", "password", "samp_db", opt);
```

Two things to know:

- **The server has to be listening on IPv6.** MySQL binds IPv4 only by default; it needs `bind-address = ::` (or a specific IPv6 address) in the server configuration. A refused connection to `[::1]` is usually this, not the plugin.
- **TLS to an IP literal needs a certificate issued for that IP** (an `iPAddress` entry in the certificate's SAN), which is uncommon — certificates are normally issued for names. If you use `MYSQL_OPT_SSL` with certificate verification on, connect by hostname, or expect verification to fail.

## mysql_tls_active

```pawn
native bool:mysql_tls_active(connId);
```

Whether the session is encrypted, as the handshake settled it — not whether
`MYSQL_OPT_SSL` was asked for. The option is the request; this is the answer.

## mysql_tls_cipher

```pawn
native bool:mysql_tls_cipher(connId, dest[], max_len = sizeof(dest));
```

Writes the cipher suite the session negotiated, for example
`TLS_AES_256_GCM_SHA384`. An unencrypted session writes an empty string and
returns `false` — that is the answer, not an error; the return value is what
separates it from an unknown connection id.

Both read a value captured once, on the connection the handshake opened, so
neither costs a query.

```pawn
new cipher[64];
if (mysql_tls_cipher(g_mysql, cipher))
    printf("[MySQL] encrypted with %s", cipher);
else
    print("[MySQL] NOT encrypted");
```

Worth checking at startup on a server that is supposed to require TLS: a
misconfiguration that leaves the session in plaintext looks exactly like a
working connection from every other angle.

### Unix socket

If `host` starts with `/`, the plugin connects via the local socket and the configured port is ignored.

```pawn
g_mysql = mysql_connect("/var/run/mysqld/mysqld.sock", "root", "", "samp_db");
```

### Multiple connections

A gamemode can hold any number of connections in parallel; each one has its own pool and its own error slot.

```pawn
new g_main, g_logs;

public OnGameModeInit()
{
    g_main = mysql_connect("127.0.0.1", "root", "pass", "samp_main");
    g_logs = mysql_connect("127.0.0.1", "root", "pass", "samp_logs");
    return 1;
}
```

### Customizing the connection

To use a non-default port, TLS or timeout, create an options handle first. See [Options](options.md).

```pawn
new opts = mysql_options_new();
mysql_options_set_int(opts, MYSQL_OPT_PORT, 3307);
mysql_options_set_int(opts, MYSQL_OPT_CONNECT_TIMEOUT, 10);

g_mysql = mysql_connect("db.example.com", "user", "pass", "samp_db", opts);
```

## mysql_connect_file

```pawn
native mysql_connect_file(const path[] = "mysql.ini", options = 0);
```

Reads the credentials from a file instead of the gamemode source. The `.pwn` is usually in version control; the config file is not, which is the point.

```ini
# mysql.ini — keep this out of your repository
host     = 127.0.0.1
user     = samp
password = s3cr3t
database = samp_server
```

```pawn
g_mysql = mysql_connect_file("mysql.ini");
```

Format:

- `key = value`, one per line. Keys are case-insensitive and whitespace is trimmed.
- `#` and `;` start a comment.
- Wrap a value in quotes to keep leading or trailing spaces: `password = "  spaced  "`.
- Only the first `=` splits the line, so a password may contain `=`.
- `host`, `user` and `database` are required. `password` may be absent or empty — a local socket account often has none.
- Unknown keys are ignored, so a file shared with another tool still works.
- `${NAME}` in a value is replaced by that environment variable.

### Keeping the secret out of the file as well

`mysql_connect_file` keeps the password out of the gamemode. `${NAME}` takes
the next step and keeps it out of the file, which is what makes a stolen copy
of the file worth nothing:

```ini
host     = 127.0.0.1
user     = samp
password = ${MYSQL_PASSWORD}
database = samp_server
```

The operator exports `MYSQL_PASSWORD` once — in the service unit, the shell
that starts the server, or the container's environment — and the file only
names the secret.

- **An unset name expands to nothing** and logs which name was missing. Sending
  the literal `${MYSQL_PASSWORD}` to the server instead would fail with a
  confusing "access denied"; an empty password fails for a reason you can read.
- **A bare `$NAME` is left alone**, so a password that contains a dollar sign
  keeps working. Only `${...}` is a reference.
- **It works in any value**, not just the password, and more than one fits in
  the same value.

This is worth more than encrypting the file. A key that the server must be able
to read at startup has to live somewhere the server can reach, so an encrypted
file plus its key is the same secret in two pieces. Naming an environment
variable removes the secret from the repository and from the deployed files
outright.

Connection **options** are not part of the file — they stay with `mysql_options_new` and the second parameter, so there is one place to look for tuning:

```pawn
new opts = mysql_options_new();
mysql_options_set_int(opts, MYSQL_OPT_SSL, 1);
g_mysql = mysql_connect_file("mysql.ini", opts);
```

On failure the plugin logs which **key** was missing, never a value, and sets `mysql_errno(0)` to `MYSQL_ERROR_INVALID_OPTIONS`. The file's contents are never written to the log.

## mysql_close

```pawn
native bool:mysql_close(connId);
```

Removes the connection entry, dropping the underlying `Pool`. Returns `true` if the connection existed, `false` otherwise. Any in-flight queries continue running to completion on their worker threads, but their results will be discarded when the dispatcher cannot find the pool anymore.

```pawn
public OnGameModeExit()
{
    mysql_close(g_mysql);
    return 1;
}
```

## mysql_status

```pawn
native bool:mysql_status(connId, dest[], max_len = sizeof(dest));
```

Runs `SHOW GLOBAL STATUS` and formats a fixed subset of metrics into `dest`, separated by two spaces. Returns `true` on success, `false` if the query failed or the connection id is invalid.

Reported metrics: `Uptime`, `Threads_connected`, `Questions`, `Slow_queries`, `Opens`, `Flush_tables`, `Open_tables`, `Queries_per_second_avg`.

```pawn
new status[256];
if (mysql_status(g_mysql, status))
{
    printf("Status: %s", status);
    // Status: Uptime: 12345  Threads_connected: 3  Questions: 567  ...
}
```

## Charset

The plugin forces `utf8mb4` on every fresh connection via the pool's `init` block. You can switch to another charset at runtime, but consider the security note in [Security](security.md#multi-byte-charsets) before doing so.

### mysql_set_charset

```pawn
native bool:mysql_set_charset(connId, const charset[]);
```

Runs `SET NAMES '<charset>'` on the next available pool connection. Returns `true` if the statement succeeded, `false` if the connection is invalid or the server rejected the charset.

```pawn
mysql_set_charset(g_mysql, "utf8mb4");
```

### mysql_get_charset

```pawn
native bool:mysql_get_charset(connId, dest[], max_len = sizeof(dest));
```

Runs `SELECT @@character_set_connection`. Returns `true` if the query succeeded and `dest` was populated.

```pawn
new charset[32];
mysql_get_charset(g_mysql, charset);
printf("charset = %s", charset);  // utf8mb4
```

## Connection pool

`mysql::Pool` is `Clone + Send + Sync`. The plugin gives each worker thread its own `Pool` clone so they fetch connections without explicit locking. Highlights:

- Connections are created lazily on the first `get_conn()`.
- The plugin does not expose the pool size — the `mysql` crate manages it internally.
- A query that fails with a connection-lost error (the mysql crate maps these to error code `0`) is retried once by `attempt_query` when `MYSQL_OPT_AUTO_RECONNECT` is enabled (the default).

## Pool size

```pawn
mysql_options_set_int(opts, MYSQL_OPT_POOL_SIZE, 16);
```

Caps how many connections the pool may open. Without it the driver's default applies (a maximum of 100). Lower it when the MySQL server has a tight `max_connections` and several game servers share it; the value must be at least 1.

## What happens on failure

When `mysql_connect` cannot build the pool or cannot open the validation connection:

1. A short message goes to the console: `[MySQL] Connection failed (error 1). See logs/mysql.log for details.`
2. The detailed reason (including the raw `mysql` crate error) goes to `logs/mysql.log`.
3. The plugin global error state is set: `mysql_errno(0)` returns `MYSQL_ERROR_CONNECTION_FAILED`, `mysql_error(0, …)` returns the full message.
4. The native returns `0`.

The Pawn side **must** check `mysql_errno()` after `mysql_connect` — a failed call cannot be detected from the return value alone if the gamemode does not store it.
