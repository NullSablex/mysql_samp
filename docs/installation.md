# Installation

## Requirements

- SA-MP 0.3.7+ **or** Open Multiplayer
- MySQL 5.7+ / MariaDB 10.3+ (8.0 / 10.6+ recommended)
- 32-bit binary host: both servers load `i686` plugins
- No system libraries — the plugin is statically self-contained

## Download

Grab the latest release from [Releases](https://github.com/NullSablex/mysql_samp/releases/latest):

| File | Platform |
|---|---|
| `mysql_samp.so` | Linux i686 (`i686-unknown-linux-gnu`) |
| `mysql_samp.dll` | Windows i686 (`i686-pc-windows-msvc`) |
| `mysql_samp.inc` | Pawn include — shared between SA-MP and Open Multiplayer |

The same `.so` / `.dll` runs on SA-MP and on Open Multiplayer.

## Register the plugin

### SA-MP

Drop the binary into `plugins/` and add the file name to `server.cfg`:

```
plugins mysql_samp.so
```

(`mysql_samp.dll` on Windows.)

### Open Multiplayer — native mode (recommended)

Drop the binary into the `components/` folder. open.mp auto-discovers and loads it via `ComponentEntryPoint` on start, with access to `ICore`, `ITimersComponent` and the other native APIs. No `config.json` entry is required — native components are discovered by scanning the folder, not by declaration.

### Open Multiplayer — legacy mode

The same binary still ships the SA-MP plugin ABI (`Supports`, `Load`, `Unload`, `AmxLoad`, `AmxUnload`, `ProcessTick`). Drop it into `plugins/` and register it under `legacy_plugins` in `config.json` if you prefer the SA-MP compatibility path over the native component. No rebuild, no extra flag.

## Place the include

| Compiler | Path |
|---|---|
| Pawno (Windows) | `pawno/include/mysql_samp.inc` |
| Qawno (open.mp) | `qawno/include/mysql_samp.inc` |
| Linux | `include/mysql_samp.inc` (at the server root) |

Then in your gamemode:

```pawn
#include <mysql_samp>
```

The include exposes the plugin version as a Pawn constant:

```pawn
printf("mysql_samp version: %s", MYSQL_SAMP_VERSION);
```

## Directory layout

**Linux (SA-MP):**

```
server/
├── gamemodes/
│   └── your_gm.amx
├── include/
│   └── mysql_samp.inc
├── plugins/
│   └── mysql_samp.so
├── logs/
│   └── mysql.log         ← auto-created on first error
└── server.cfg
```

**Windows (SA-MP):**

```
server/
├── gamemodes/
│   └── your_gm.amx
├── pawno/include/
│   └── mysql_samp.inc
├── plugins/
│   └── mysql_samp.dll
├── logs/
│   └── mysql.log         ← auto-created on first error
└── server.cfg
```

## Verifying the install

When the server starts, the plugin prints a banner:

```
  | mysql_samp 1.0.0 | 2026
  |-------------------------------
  | Author and maintainer: NullSablex
  |
  | Compiled: Feb 23 2026 at 14:30:00
  |-------------------------------
  | Repository: https://github.com/NullSablex/mysql_samp
```

If the banner is missing:

- Check the binary is in the right folder (`plugins/`, `components/`, or `legacy_plugins` per the registration path).
- Confirm the architecture (the binary is 32-bit; the server must be too — both SA-MP and open.mp run 32-bit on Linux).
- Make sure `server.cfg` / `config.json` references the exact file name.

## Logs

The plugin keeps a separate log file for sensitive details: `logs/mysql.log`. The directory is created automatically; if the working directory is not writable the plugin emits one console error and suppresses further file-write attempts.

| Destination | Contents |
|---|---|
| Server console | Generic messages with error codes — `[MySQL] Query failed on connection 1 (error 1064). See logs/mysql.log for details.` |
| `logs/mysql.log` | Full details with timestamp — full error messages, including raw text reported by the MySQL server |

The console never prints query text, credentials or row data.

### Log level

`mysql_log(level)` adjusts the minimum severity that reaches both sinks at runtime. The default is `MYSQL_LOG_ALL`.

```pawn
mysql_log(MYSQL_LOG_NONE);     // 0 — disable every category
mysql_log(MYSQL_LOG_ERROR);    // 1 — errors only
mysql_log(MYSQL_LOG_WARNING);  // 2 — errors + warnings
mysql_log(MYSQL_LOG_INFO);     // 3 — errors + warnings + info
mysql_log(MYSQL_LOG_ALL);      // 4 — everything (default)
```
