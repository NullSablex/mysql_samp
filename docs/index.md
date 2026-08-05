# mysql_samp

MySQL plugin for SA-MP and Open Multiplayer, written entirely in Rust. Non-blocking queries with FIFO ordering, a result cache, an ORM, zero external runtime dependencies.

!!! info "Not affiliated"
    This is an independent, community-maintained project. It is **not**
    affiliated with, endorsed by, sponsored by, or otherwise connected to
    SA-MP, the open.mp (Open Multiplayer) project, or the MySQL plugin by
    BlueG / maddinat0r that this one is compared against. It has no
    relationship with any of them. "SA-MP", "open.mp" and "MySQL" belong to
    their respective owners and are referenced here solely to describe what
    this plugin is compatible with.

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
| [Connection](connection.md) | `mysql_connect`, `mysql_connect_file`, `mysql_close`, `mysql_status`, charset, pool size |
| [Options](options.md) | All `MYSQL_OPT_*` values, defaults, TLS and mutual TLS |
| [Queries](queries.md) | `mysql_query`, `mysql_pquery`, `mysql_format`, `mysql_escape_string`, `mysql_stmt_*`, `mysql_transaction_*` |
| [Cache](cache.md) | `cache_*` natives, active stack, persistent caches, multiple result sets |
| [ORM](orm.md) | Bind Pawn variables to columns, CRUD without writing SQL |
| [Errors](errors.md) | `mysql_errno`, `mysql_error`, `OnQueryError`, MySQL error codes |
| [Security](security.md) | Prepared statements vs escaping, password storage, TLS, resource limits |
| [API reference](api-reference.md) | One-line table of every native and forward (75 total) |

## Plugin facts

- **rust-samp**: built on top of [rust-samp v3.4.0](https://github.com/NullSablex/rust-samp).
- **MySQL crate**: `mysql` 28.0 with `default-rust` + `rustls-tls-ring`. The MySQL protocol itself is pure Rust; the TLS backend (`ring`) carries a C/assembly crypto core that is compiled **into** the binary, so the shipped artifact still needs no `libmysqlclient` and no system OpenSSL.
- **TLS**: rustls is compiled into the binary (`rustls-tls-ring`); `MYSQL_OPT_SSL` enables TLS, `MYSQL_OPT_SSL_CA` sets the root certificate, and `MYSQL_OPT_SSL_CERT`/`_KEY` do mutual TLS. Without `MYSQL_OPT_SSL_CA` only the **bundled webpki roots** are trusted — not the OS trust store — so a self-signed or internal-CA server needs it. See [Options](options.md#ssl).
- **Injection safety**: prefer [prepared statements](queries.md) (`mysql_stmt_*`) over `mysql_format` for player input — values are bound server-side and never enter the SQL text.
- **Password hashing**: `mysql_hash_password` / `mysql_verify_password` run Argon2id on a worker thread.
- **Tick dispatch**: the unified `on_tick` from rust-samp v3 fires on both SA-MP (via `ProcessTick`) and Open Multiplayer native mode (via `ITimersComponent`). No Pawn timer is required.
- **Server compatibility**: MySQL 5.7, 8.x and 9.x, plus MariaDB. The driver implements `caching_sha2_password` (the default since MySQL 8.0.4, and the only option since 9.0 removed `mysql_native_password`), including the RSA public-key exchange used for first authentication over a plaintext connection.
- **Threading**: each `mysql_query` spawns a worker thread that pulls a connection from a `mysql::Pool`; results travel back over an `mpsc` channel and are dispatched on the next tick.
