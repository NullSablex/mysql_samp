# mysql_samp

MySQL plugin for SA-MP and Open Multiplayer, written entirely in Rust. Non-blocking queries with FIFO ordering, a result cache, an ORM, zero external runtime dependencies.

The same `.so` / `.dll` runs on SA-MP and on Open Multiplayer — natively as a component (recommended) or via legacy mode. See [Installation](installation.md) for both registration paths.

## Where to start

| Goal | Path |
|---|---|
| First time here | [Installation](installation.md) → [Connection](connection.md) → [Queries](queries.md) |
| Coming from MySQL R41-4 | [Migration guide](migration.md) → [Migration changes](migration-changes.md) → [Migration examples](migration-examples.md) |
| Quick lookup | [API reference](api-reference.md) |
| Performance numbers | [Benchmark](benchmark.md) |

## Minimal example

Connect, fire a threaded query, read the result inside the callback:

```pawn
#include <a_samp>
#include <mysql_samp>

new g_mysql;

public OnGameModeInit()
{
    g_mysql = mysql_connect("127.0.0.1", "root", "password", "samp_db");

    if (mysql_errno() != MYSQL_OK)
    {
        printf("[MySQL] connect failed: errno=%d", mysql_errno());
        return 1;
    }

    // Non-blocking, FIFO-ordered query. Callback receives playerid via "d" format.
    mysql_query(g_mysql, "SELECT id, name FROM players LIMIT 5", "OnPlayersLoaded", "d", 0);
    return 1;
}

forward OnPlayersLoaded(playerid);
public OnPlayersLoaded(playerid)
{
    new rows = cache_get_row_count();
    for (new i = 0; i < rows; i++)
    {
        new id   = cache_get_value_name_int(i, "id");
        new name[MAX_PLAYER_NAME];
        cache_get_value_name(i, "name", name);
        printf("Player #%d: %s", id, name);
    }
}

public OnGameModeExit()
{
    mysql_close(g_mysql);
    return 1;
}
```

## Topics

| Topic | Contents |
|---|---|
| [Installation](installation.md) | Download, register on SA-MP and Open Multiplayer, log files |
| [Connection](connection.md) | `mysql_connect`, `mysql_close`, `mysql_status`, charset |
| [Options](options.md) | All `MYSQL_OPT_*` values, defaults, SSL caveat |
| [Queries](queries.md) | `mysql_query`, `mysql_pquery`, `mysql_format`, `mysql_escape_string` |
| [Cache](cache.md) | `cache_*` natives, active stack, persistent caches |
| [ORM](orm.md) | Bind Pawn variables to columns, CRUD without writing SQL |
| [Errors](errors.md) | `mysql_errno`, `mysql_error`, `OnQueryError`, MySQL error codes |
| [Security](security.md) | SQL injection, escaping rules, resource limits |
| [API reference](api-reference.md) | One-line table of every native and forward (55 total) |

## Plugin facts

- **rust-samp**: built on top of [rust-samp v3.0.0](https://github.com/NullSablex/rust-samp).
- **MySQL crate**: `mysql` 28.0 with the `default-rust` feature (pure-Rust driver, no `libmysqlclient`).
- **TLS**: rustls is compiled into the binary, but the SSL options (`MYSQL_OPT_SSL`, `MYSQL_OPT_SSL_CA`) are currently **not wired through** to the connection layer — see the caveat in [Options](options.md#ssl).
- **Tick dispatch**: the unified `on_tick` from rust-samp v3 fires on both SA-MP (via `ProcessTick`) and Open Multiplayer native mode (via `ITimersComponent`). No Pawn timer is required.
- **Threading**: each `mysql_query` spawns a worker thread that pulls a connection from a `mysql::Pool`; results travel back over an `mpsc` channel and are dispatched on the next tick.
