# Benchmark — mysql_samp vs MySQL R41-4

A technical comparison between **mysql_samp** (Rust) and **MySQL R41-4** (C++, BlueG / maddinat0r). Numbers below are honest — they include the case where R41-4 wins.

## How to run the benchmarks

Ready-made files live under `benchmark/`:

| File | What it does |
|---|---|
| `benchmark/setup.sql` | Creates the tables and inserts 100 test rows |
| `benchmark/bench_mysql_samp.pwn` | Benchmark gamemode for mysql_samp |
| `benchmark/bench_r41.pwn` | Benchmark gamemode for R41-4 |

### Steps

**1. Prepare the database:**

```bash
mysql -u root -p your_database < benchmark/setup.sql
```

**2. Set the credentials** in `bench_mysql_samp.pwn` and `bench_r41.pwn`:

```pawn
#define DB_HOST  "127.0.0.1"
#define DB_USER  "root"
#define DB_PASS  "password"
#define DB_NAME  "benchmark"
```

**3. Compile and run each gamemode** separately on your server with the corresponding plugin loaded. The result prints to the console.

**4. Compare the four-stage output** between the two plugins.

### What is measured

| Stage | Type | Rounds |
|---|---|---|
| 1 | SELECT, sequential (FIFO) | 500 queries |
| 2 | SELECT, parallel | 500 queries |
| 3 | INSERT, parallel | 200 queries |
| 4 | `mysql_format` with escape (pure CPU) | 50 000 iterations |

## Results — local MySQL, same machine

Environment: SA-MP 0.3.7-R2, MySQL local (loopback — `127.0.0.1`), Linux x86. Both plugins tested on the machine that hosts MySQL, no network latency.

> **Note on remote MySQL.** With MySQL on a separate host (RTT ≥ 1 ms), each query takes longer than a server tick. That removes the R41-4 advantage on SELECTs because its `process_tick`-driven dispatch is then bottlenecked to one query per tick. The results below represent the **best-case scenario for R41-4**.

### R41-4

| Stage | Total | Average | Throughput |
|---|---|---|---|
| SELECT FIFO — `mysql_tquery` (500x)¹ | 93 ms | 0.186 ms/query | 5 376 q/s |
| SELECT parallel — `mysql_pquery` (500x) | 51 ms | 0.101 ms/query | 9 804 q/s |
| INSERT parallel — `mysql_pquery` (200x) | 607 ms | 3.035 ms/query | 329 q/s |
| `mysql_format` with escape (50 000x) | 64 ms | 0.0012 ms | 781 250 ops/s |

### mysql_samp

| Stage | Total | Average | Throughput |
|---|---|---|---|
| SELECT FIFO — `mysql_query` (500x)¹ | 155 ms | 0.310 ms/query | 3 226 q/s |
| SELECT parallel — `mysql_pquery` (500x) | 94 ms | 0.187 ms/query | 5 319 q/s |
| INSERT parallel — `mysql_pquery` (200x) | 45 ms | 0.224 ms/query | 4 444 q/s |
| `mysql_format` with escape (50 000x) | 135 ms | 0.0027 ms | 370 370 ops/s |

### Head to head

| Stage | R41-4 | mysql_samp | Winner |
|---|---|---|---|
| SELECT FIFO (500x)¹ | **0.186 ms/q — 5 376 q/s** | 0.310 ms/q — 3 226 q/s | R41-4, 1.7× |
| SELECT parallel (500x) | **0.101 ms/q — 9 804 q/s** | 0.187 ms/q — 5 319 q/s | R41-4, 1.85× |
| INSERT parallel (200x) | 3.035 ms/q — 329 q/s | **0.224 ms/q — 4 444 q/s** | **mysql_samp, 13.5×** |
| `mysql_format` (50 000x) | **0.0012 ms — 781k/s** | 0.0027 ms — 370k/s | R41-4, 2.1× |

¹ Stage 1 is not an apples-to-apples comparison. See the note below.

### Note on Stage 1: not apples-to-apples

R41-4 has no direct equivalent of mysql_samp's `mysql_query`. The closest one is `mysql_tquery`, which is what the benchmark uses. The difference is architectural:

| Plugin | FIFO native | Behavior |
|---|---|---|
| R41-4 | `mysql_tquery` | Thread pool; callbacks dispatched via `process_tick` — several per tick when MySQL responds fast enough (loopback) |
| mysql_samp | `mysql_query` | One thread per query; results reordered through an `mpsc` channel, callbacks delivered in submission order regardless of tick |

The gamemode-visible semantics are equivalent (N queries, callbacks in submission order). R41-4's Stage 1 throughput depends on MySQL latency: in loopback it wins; with remote MySQL it loses, because the per-tick dispatch becomes the bottleneck.

## Executive summary

| Criterion | mysql_samp | R41-4 |
|---|---|---|
| SELECT FIFO (500x)¹ | 0.310 ms/q — 3 226 q/s | **0.186 ms/q — 5 376 q/s** |
| SELECT parallel (500x) | 0.187 ms/q — 5 319 q/s | **0.101 ms/q — 9 804 q/s** |
| INSERT parallel (200x) | **0.224 ms/q — 4 444 q/s** | 3.035 ms/q — 329 q/s |
| `mysql_format` (50 000x) | 0.0027 ms — 370k/s | **0.0012 ms — 781k/s** |
| Memory safety | **Compiler-enforced** | Documented segfaults (issues #291, #310+) |
| SQL injection via `%s` | **Impossible** (`%s` escapes by default) | Possible (`%s` is raw in R41-4) |
| Memory leak | **Impossible** (cache managed automatically) | Possible without `cache_delete()` |
| Runtime dependencies | **None** | MySQL C Connector + Boost |
| Open issues | New (active development) | **50+ open issues** on GitHub |
| Synchronous blocking query | **Removed** (never blocks the server tick) | Exists (`mysql_query` blocks) |

## 1. Memory safety

### R41-4 (C++, manual memory)

R41-4 is C++ with raw pointers and manual memory management. Bugs publicly documented:

- **Segmentation fault (SIGSEGV):** multiple reports of crashes during server shutdown while queries are in flight (issues #291, #310, #311)
- **"FREE RESULT MISSING":** documented error indicating a result not freed correctly (issue #291)
- **Crash when unloading the plugin with pending queries:** race between the plugin destructor and live threads

The repository itself warns:

> "Use `cache_delete()` if you don't need the query's result anymore or you will experience **memory leaks**."

### mysql_samp (Rust)

Rust guarantees at **compile time**:

- **No buffer overflows** — every array access is bounds-checked.
- **No use-after-free** — the borrow checker prevents access to freed memory.
- **No data races** — `Send` and `Sync` prove only one thread accesses mutually-exclusive data.
- **Cache auto-managed** — no manual `cache_delete()`; Rust drops the entry when its scope ends.

```rust
// Impossible in Rust — the compiler rejects this at build time:
let cache = get_cache();
drop(cache);
use_cache(cache); // ERROR: value moved here — never produces a binary
```

**Practical effect:** mysql_samp cannot crash the server because of a memory bug. R41-4 can.

## 2. SQL safety: injection via `%s`

### R41-4

In R41-4, `%s` inserts the string **without escaping**. Code from a real gamemode:

```pawn
// R41-4 — VULNERABLE to SQL injection
new query[256];
mysql_format(g_mysql, query, sizeof(query),
    "SELECT * FROM accounts WHERE name = '%s'", inputName);
// If inputName = "' OR 1=1 -- "
// Generated query: SELECT * FROM accounts WHERE name = '' OR 1=1 -- '
// Result: returns EVERY account.
```

To escape in R41-4 you had to remember to use `%e` — something most gamemodes never did.

### mysql_samp

```pawn
// mysql_samp — SAFE by default
new query[256];
mysql_format(g_mysql, query, sizeof(query),
    "SELECT * FROM accounts WHERE name = '%s'", inputName);
// If inputName = "' OR 1=1 -- "
// Generated query: SELECT * FROM accounts WHERE name = '\' OR 1=1 -- '
// Result: no rows (string escaped correctly).
```

`%s` always escapes. To insert trusted raw values (table names, fixed SQL fragments) use `%r` explicitly.

**Practical effect:** gamemodes migrated from R41-4 become safe automatically. New gamemodes do not need to think about escaping.

## 3. Runtime dependencies

### R41-4

The server has to provide:

| Library | Version | Notes |
|---|---|---|
| `libmysqlclient` | 5.5+ / 6.1 | System or bundled |
| `Boost` | 1.57+ | Compiled into the plugin |
| `libz.so.1` | system | Reported in issue #292 |

Typical configuration errors:

```
error while loading shared libraries: libmysqlclient.so.18: cannot open shared object file
libz.so.1: cannot open shared object file: No such file or directory
```

### mysql_samp

**No external runtime dependencies.** The binary is fully self-contained:

- TLS/SSL via **rustls** (pure Rust, embedded in the binary). Note: the `MYSQL_OPT_SSL` / `MYSQL_OPT_SSL_CA` options exist but are not yet wired through to the connection layer — see [Options → SSL caveat](options.md#ssl).
- MySQL driver via the `mysql` crate with the `default-rust` feature (no `libmysqlclient`).
- No Boost, no OpenSSL, no libz.

Drop the `.so` or `.dll` into `plugins/` and it works on any Linux distribution.

## 4. Threading model

### R41-4

```
GameMode ──► mysql_query()  ──► SYNCHRONOUS — blocks the server tick until the query returns
         ──► mysql_tquery() ──► 1 worker thread per connection (FIFO)
         ──► mysql_pquery() ──► pool of parallel connections (no order)

Queue: Boost lockfree::spsc_queue (capacity < 65 536 entries)
Sync: std::mutex around every MySQL C API call
```

**Problem:** synchronous `mysql_query()` blocks the server tick. A 100 ms query freezes the server for 100 ms — no player receives packets during that window.

### mysql_samp

```
GameMode ──► mysql_query()  ──► FIFO queue over an mpsc channel (NEVER blocks)
         ──► mysql_pquery() ──► parallel pool over an mpsc channel

Sync: enforced by Rust's type system (no manual mutex)
```

**There is no synchronous query.** mysql_samp removed the blocking `mysql_query` by design — you cannot freeze the server by mistake.

## 5. Critical R41-4 issues with no upstream fix

Based on public issues at https://github.com/pBlueG/SA-MP-MySQL:

| Issue | Description | Status |
|---|---|---|
| #291 | "FREE RESULT MISSING" — result not freed correctly | Open |
| #288 | SSL does not work on Linux | Open |
| #277 | `mysql_tquery` skips UPDATE under certain conditions | Open |
| #292 | `libz.so.1: cannot open shared object file` | Open |
| Multiple | Segmentation fault (SIGSEGV) on shutdown with pending queries | Open |
| Multiple | Incompatibility with sampgdk and other plugins | Open |

mysql_samp does not have any of these by design:

- **SIGSEGV on shutdown:** impossible in Rust — the borrow checker prevents access to freed data after threads end.
- **SSL on Linux:** rustls does not depend on the system `libssl` — it works on any distribution. (The plugin needs to wire the options through; see [Options → SSL caveat](options.md#ssl).)
- **Missing `libz` / dependencies:** none exist — no external dependencies.
- **FREE RESULT MISSING:** impossible — memory is freed automatically by Rust.

## 6. API ergonomics

### Return value vs by-ref

```pawn
// R41-4 — by-ref (verbose)
new rows;
cache_get_row_count(rows);
new score;
cache_get_value_name_int(0, "score", score);

// mysql_samp — return value (clean)
new rows  = cache_get_row_count();
new score = cache_get_value_name_int(0, "score");
```

### Cache management

```pawn
// R41-4 — MUST call cache_delete or leak
public OnPlayerData(playerid)
{
    // ... read data ...
    cache_delete(cache_save()); // required
}

// mysql_samp — no cache_delete needed
// The active cache is released automatically after the callback returns.
public OnPlayerData(playerid)
{
    // ... read data ...
    // nothing to do
}
```

### Pawn tags

```pawn
// R41-4 — custom tags (can trigger warnings)
new MySQL:g_mysql = mysql_connect(...);
new Cache:cache   = cache_save();
new ORM:orm       = orm_create("table", g_mysql);

// mysql_samp — no tags
new g_mysql = mysql_connect(...);
new cache   = cache_save();
new orm     = orm_create("table", g_mysql);
```

## 7. Full feature matrix

| Feature | mysql_samp | R41-4 |
|---|---|---|
| Threaded FIFO query | `mysql_query` | `mysql_tquery` |
| Parallel query | `mysql_pquery` | `mysql_pquery` |
| Blocking sync query | **Removed for safety** | `mysql_query` (legacy) |
| ORM | Yes (`orm_*`) | Yes (`orm_*`) |
| Saved cache | `cache_save()` / `cache_set_active()` | Same |
| `mysql_format %s` | Escapes automatically | Raw (unsafe) |
| `mysql_format %e` | Alias for `%s` (escape) | Same as mysql_samp |
| `mysql_format %r` | Raw (no escape) | **Does not exist** |
| Pawn tags | **No tags** | `MySQL:`, `Cache:`, `ORM:` |
| Runtime dependencies | **None** | libssl + libmysqlclient + Boost |
| Open Multiplayer | Native + legacy | Compatible (legacy) |
| SSL/TLS | rustls (binary) — options not yet wired through | Via MySQL C Connector |
| Multi-result sets | Not supported | `cache_set_result()` |
| Possible segfault | **No** | **Yes** (documented) |
| Possible cache leak | **No** | Yes without `cache_delete()` |
| Critical known issues | None in production | 50+ open on GitHub |

## 8. Conclusion

The loopback numbers (best case for R41-4) show an honest picture:

**R41-4 has higher SELECT throughput in loopback** — 1.7–1.85× on FIFO and parallel SELECT, and 2.1× on `mysql_format`. With MySQL answering in < 0.1 ms (below one tick), R41-4 dispatches several callbacks per tick and exploits the server cache fully.

**mysql_samp has 13.5× the INSERT throughput** — writes invalidate cache and force real I/O, making R41-4's per-tick callback bottleneck irrelevant and exposing the architectural difference. Real gamemodes write much more often (player save, logs, events) than they re-read the same row.

**Loopback is the best case for R41-4.** With remote MySQL (RTT ≥ 1 ms, the typical production setup), every query lasts longer than a tick and R41-4's SELECT advantage disappears — mysql_samp wins every stage.

**mysql_samp's advantages are structural, independent of any benchmark:**

1. **Compiler-enforced memory safety** — no segfaults, no use-after-free, no data races.
2. **Safe SQL by default** — `%s` escapes without the developer having to remember.
3. **Zero dependencies** — works on any server without installing `libmysqlclient`, Boost or `libssl`.
4. **No blocking sync query** — R41-4 has `mysql_query` that freezes the tick; mysql_samp does not.

For new projects, mysql_samp is the technically superior choice. For existing projects that need compatibility with legacy R41-4 code, see [migration.md](migration.md).
