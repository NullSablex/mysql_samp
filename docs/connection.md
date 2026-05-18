# Connection

Each `mysql_connect` call creates a [`mysql::Pool`](https://docs.rs/mysql/latest/mysql/struct.Pool.html) that worker threads share. The plugin always issues `SET NAMES utf8mb4` on every new connection, so all string round-trips are UTF-8 safe by default.

## mysql_connect

```pawn
native mysql_connect(const host[], const user[], const password[], const database[], options = 0);
```

| Parameter | Type | Description |
|---|---|---|
| `host` | string | IPv4/hostname **or** absolute path to a Unix socket (path starts with `/`) |
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

## What happens on failure

When `mysql_connect` cannot build the pool or cannot open the validation connection:

1. A short message goes to the console: `[MySQL] Connection failed (error 1). See logs/mysql.log for details.`
2. The detailed reason (including the raw `mysql` crate error) goes to `logs/mysql.log`.
3. The plugin global error state is set: `mysql_errno(0)` returns `MYSQL_ERROR_CONNECTION_FAILED`, `mysql_error(0, …)` returns the full message.
4. The native returns `0`.

The Pawn side **must** check `mysql_errno()` after `mysql_connect` — a failed call cannot be detected from the return value alone if the gamemode does not store it.
