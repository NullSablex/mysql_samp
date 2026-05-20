# mysql_samp

> MySQL plugin for SA-MP and Open Multiplayer, written in Rust — by [NullSablex](https://github.com/NullSablex)

![License](https://img.shields.io/badge/license-GPL--3.0-blue)
![SA-MP](https://img.shields.io/badge/SA--MP-0.3.7+-orange)
![Open Multiplayer](https://img.shields.io/badge/Open%20Multiplayer-native%20%26%20legacy-orange)
![Build](https://img.shields.io/badge/build-Linux%20%7C%20Windows-green)
![Architecture](https://img.shields.io/badge/arch-x86%20(32--bit)-lightgrey)
[![Release](https://img.shields.io/github/v/release/NullSablex/mysql_samp?label=download)](https://github.com/NullSablex/mysql_samp/releases/latest)

## Overview

**mysql_samp** is a modern MySQL plugin for SA-MP (San Andreas Multiplayer) and [Open Multiplayer](https://open.mp) (open.mp), written entirely in Rust. It provides a complete API for database connectivity, non-blocking queries, a cache system and an ORM, with zero external runtime dependencies.

The same binary loads on SA-MP and on Open Multiplayer — natively as a component (recommended) or via legacy mode.

### Highlights

- **Zero external dependencies** — no `libmysqlclient`, no OpenSSL. The MySQL protocol and TLS (via rustls) are compiled directly into the binary.
- **All queries are non-blocking** — `mysql_query` runs on background threads with FIFO ordering. The server never stalls.
- **Connection pool** — automatic reuse through `mysql::Pool`, thread-safe by design.
- **Built-in ORM** — maps Pawn variables to columns with CRUD helpers.
- **Cache system** — results accessible through an automatic stack or persisted manually with `cache_save`.
- **Safe by default** — string escaping, forced UTF-8, protection against SQL injection and memory exhaustion.
- **Universal binary** — built on top of [rust-samp](https://github.com/NullSablex/rust-samp) v3.0.0; one `.so`/`.dll` runs on SA-MP and on Open Multiplayer (native component or legacy).
- **Simple deploy** — drop the `.so` or `.dll` in and you are done. No system libraries to install.

## Installation

1. Download the latest release for your platform:
   - `mysql_samp.so` (Linux i686)
   - `mysql_samp.dll` (Windows i686, MSVC ABI)
   - `mysql_samp.inc` (Pawn include, shared between SA-MP and Open Multiplayer)
2. Place the binary in the server's `plugins/` directory.
3. Copy `mysql_samp.inc` to your compiler's include folder:
   - **Windows:** `pawno/include/` or `qawno/include/`
   - **Linux:** `include/` (at the server root)
4. Register the plugin:
   - **SA-MP** — add to `server.cfg`:
     ```
     plugins mysql_samp.so
     ```
     (or `mysql_samp.dll` on Windows)
   - **Open Multiplayer (native, recommended)** — drop the binary into the `components/` folder. open.mp auto-discovers it on start and loads it via `ComponentEntryPoint`, with access to `ICore`, `ITimersComponent` and the other native APIs. No `config.json` entry required.
   - **Open Multiplayer (legacy)** — same binary works as a legacy plugin. Drop it into `plugins/` and add it to `legacy_plugins` in `config.json` (this one DOES need to be declared, otherwise open.mp skips legacy plugins).

> [!IMPORTANT]
> No `libmysqlclient` or other system library is required. The plugin is self-contained.

## Quick start

```pawn
#include <a_samp>
#include <mysql_samp>

new gMysql;

public OnGameModeInit() {
    gMysql = mysql_connect("127.0.0.1", "root", "password", "samp_db");

    if (mysql_errno()) {
        return 1;
    }

    // Non-blocking query with callback
    mysql_query(gMysql, "SELECT * FROM players LIMIT 10", "OnPlayersLoaded");
    return 1;
}

forward OnPlayersLoaded();
public OnPlayersLoaded() {
    new rows = cache_get_row_count();
    printf("Players found: %d", rows);

    new name[MAX_PLAYER_NAME];
    for (new i = 0; i < rows; i++) {
        cache_get_value_name(i, "name", name);
        printf("  - %s", name);
    }
}

public OnGameModeExit() {
    mysql_close(gMysql);
    return 1;
}
```

Browse the [examples/](examples/) folder for self-contained `.pwn` scripts covering connection setup, threaded queries, ORM, TLS and error handling. The plugin natives (`mysql_*`, `cache_*`, `orm_*`) and the `OnQueryError` forward are identical across SA-MP and Open Multiplayer, so every example builds and runs on both — the only thing that differs between servers is the installation path documented above.

## Documentation

The full plugin documentation lives in [docs/](docs/):

| Document | Contents |
|---|---|
| [Installation and setup](docs/installation.md) | Setup, server.cfg / config.json, requirements |
| [Connection](docs/connection.md) | mysql_connect, mysql_close, mysql_status, charset |
| [Options](docs/options.md) | All `MYSQL_OPT_*` values, defaults, SSL caveat |
| [Queries](docs/queries.md) | mysql_query, mysql_pquery, mysql_format, mysql_escape_string |
| [Cache](docs/cache.md) | All cache_* functions, save/restore, lifecycle |
| [ORM](docs/orm.md) | Object-relational mapping, CRUD, bindings |
| [Errors](docs/errors.md) | mysql_errno, mysql_error, OnQueryError, error codes |
| [Security](docs/security.md) | Escaping, UTF-8, resource limits, best practices |
| [API reference](docs/api-reference.md) | Full table of every native and forward |
| [Migration from R41-4](docs/migration.md) | Differences and migration steps from mysql R41-4 |

## Building from source

### Requirements

- Rust stable toolchain with the targets `i686-unknown-linux-gnu` and `i686-pc-windows-msvc`
- `cargo-xwin` for cross-compiling the Windows `.dll` from Linux (installed automatically by the script)
- No system libraries — the build is 100% Rust

### Development build

```bash
cargo build --target i686-unknown-linux-gnu
```

### Release build (Linux + Windows)

From Linux:

```bash
./scripts/build-linux.sh
```

From Windows (Git Bash):

```bash
./scripts/build-windows.sh
```

Both scripts produce `dist/mysql_samp.so` and `dist/mysql_samp.dll`, each with full SA-MP + Open Multiplayer native support.

> [!CAUTION]
> This plugin is distributed under the GPL v3. Any derivative work must keep the source code open under the same license.

## License

Copyright (c) 2026 NullSablex

This project is licensed under the [GNU General Public License v3.0](LICENSE).
