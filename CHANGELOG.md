# Changelog

All notable changes to this project are documented in this file.

Format inspired by [Keep a Changelog](https://keepachangelog.com/). Versioning follows [Semantic Versioning](https://semver.org/). Older entries live under [`changelog/`](changelog/).

## [1.2.0] — 2026/08/05

Built on top of [rust-samp v3.4.0](https://github.com/NullSablex/rust-samp). No Pawn-visible API was removed or renamed.

### Security

Three issues, all present since the feature they affect was introduced. **Read the first two if you use `MYSQL_OPT_SSL` or interpolate player input into queries.**

- **`MYSQL_OPT_SSL` never encrypted anything.** The `mysql` crate was built with the `default-rust` feature, which — despite the name — enables no TLS backend at all (it only switches `flate2` to its Rust backend). Without a backend the crate's `make_secure` is a `panic!` stub, so enabling SSL aborted the connection rather than securing it. v1.1.1 announced these options as wired through "(rustls)"; that was wrong. The `rustls-tls-ring` feature is now enabled and TLS genuinely works. **If you enabled `MYSQL_OPT_SSL` and your server appeared to work, your traffic was never encrypted** — credentials and query contents crossed the network in clear text. Treat those credentials as exposed.
- **SQL injection under `sql_mode=NO_BACKSLASH_ESCAPES`.** `mysql_escape_string`, `mysql_format` and the ORM escaped `'` as `\'`. Under that mode the backslash is not an escape character, so `\'` is a literal backslash followed by a *live* quote and a crafted value breaks out of the literal. Because the driver always enables `CLIENT_MULTI_STATEMENTS` (the crate forbids disabling it), that escape is enough to append a second statement. The escaping mode is now detected per connection from the server's `sql_mode` and switches to quote doubling, matching `mysql_real_escape_string`.
- **ORM string values were double-encoded.** `read_string` mapped each AMX byte to a `char`, i.e. decoded the value as Latin-1 and re-encoded it as UTF-8. The two bytes of `é` became the four of `Ã©`, silently corrupting every non-ASCII value written through the ORM. It now decodes as UTF-8, matching the `SET NAMES utf8mb4` the connection already uses. **Rows already written with the old build stay corrupted** and need fixing at the database level.
- **Statement and transaction handles leaked.** Neither manager released anything on script unload or connection close, so a gamemode restart — routine during development — left every handle ever created in memory, growing without bound across restarts. Both are now reclaimed in `on_amx_unload` (like the ORM already was) and on `mysql_close`, which also drops handles that could never execute again.
- **`mysql_format` ignored its `connId`.** The argument was parsed and discarded, so per-connection escaping rules could not be applied even in principle. It is now used.

### Added

- **Prepared statements** — `mysql_stmt_new`, `mysql_stmt_bind_int` / `_float` / `_str` / `_null`, `mysql_stmt_reset`, `mysql_stmt_execute`, `mysql_stmt_close`. Values are bound server-side over the binary protocol instead of being pasted into SQL text, so no escaping is involved and a value can never be reinterpreted as syntax. This is the structural fix for injection — prefer it over `mysql_format` for anything carrying player input. `mysql_stmt_execute` rejects a mismatch between bound values and `?` placeholders, naming both counts.
- **Transactions** — `mysql_transaction_new`, `mysql_transaction_add`, `mysql_transaction_add_stmt`, `mysql_transaction_execute`, `mysql_transaction_destroy`. Steps are collected and then run as one unit on a single connection (`START TRANSACTION` … `COMMIT`); any failing step rolls the batch back. Deliberately batch-based rather than an interactive begin/commit: holding a pooled connection across ticks would leak it whenever a gamemode never reached the commit. `mysql_transaction_execute` consumes the handle so a batch cannot run twice.
- **Argon2id password hashing** — `mysql_hash_password` and `mysql_verify_password`, both non-blocking on a bounded worker pool (`min(cpus, 4)` threads, 512-job queue). The pool is bounded on purpose: Argon2id costs ~19 MiB per concurrent hash, so a thread per request would let a login flood allocate gigabytes. Both natives return `false` when the queue is saturated. Argon2id is deliberately slow and memory-hard, which is exactly why it cannot run on the server thread. Output is a PHC string (`$argon2id$v=19$m=...`, ~100 chars) with its own embedded random salt — store it as-is in a `VARCHAR(255)`, do not add a salt of your own. Verification reads the parameters back from the stored hash, so old hashes keep working when the defaults change.
- **Mutual TLS** — `MYSQL_OPT_SSL_CERT` and `MYSQL_OPT_SSL_KEY`. Supplying only one of the two logs a warning instead of silently ignoring it.
- **`MYSQL_OPT_SSL_VERIFY_CERT`** (default on) — turning it off accepts any certificate and skips the hostname check. Encrypted but trivially machine-in-the-middled; it warns on every connect. `MYSQL_OPT_SSL_CA` is the correct fix for a self-signed server.
- `mysql_escape_string` gained an optional trailing `connId` so it can apply the right escaping rules. Existing calls keep compiling.
- **Not-affiliated disclaimer** in the README and the docs home, making explicit that this is an independent project with no connection to SA-MP, open.mp, or the MySQL plugin by BlueG / maddinat0r that the checklist compares against.
- **OpenSSF Scorecard badge** in the README.
- **`examples/` folder** — 11 self-contained `.pwn` scripts plus a README covering how to compile them and where to drop the binary on each server. Connection, threaded and parallel queries, `mysql_format`, ORM, TLS, `OnQueryError`, and the four new ones for this release: [`08_prepared_statements.pwn`](examples/08_prepared_statements.pwn), [`09_transactions.pwn`](examples/09_transactions.pwn) and [`10_password_hashing.pwn`](examples/10_password_hashing.pwn). [`11_config_and_scripts.pwn`](examples/11_config_and_scripts.pwn) covers `mysql_connect_file`, `mysql_query_file` and multiple result sets, with the companion `mysql.ini.example` and `schema.sql`. `06_ssl.pwn` was rewritten for the TLS options above, including the warning about older builds.
- **`CODEOWNERS`** for review routing.
- **`mysql_stmt_pexecute`** — parallel counterpart of `mysql_stmt_execute`, mirroring `mysql_pquery`. Without it, moving a `mysql_pquery` call to prepared statements silently gave up parallelism.
- **`mysql_connect_file(const path[] = "mysql.ini", options = 0)`** — reads host / user / password / database from a `key = value` file, keeping credentials out of a gamemode source that is usually in version control. Options stay with `mysql_options_new`, so there is one place to look for tuning. Parse errors name the offending *key*, never a value.
- **`mysql_query_file`** — runs a `.sql` file's statements in order on one connection, non-blocking like everything else. Split on `;` outside string literals and comments, so a semicolon inside `'...'` or `/* ... */` does not break a statement in two. Deliberately **not** transactional: DDL commits implicitly in MySQL, so a transaction would imply an atomicity the server does not provide. A failure names which statement failed and stops there.
- **Multi-result sets** — `cache_get_result_count` and `cache_set_result`. A stored-procedure `CALL` or a script produces several result sets; previously only the first was reachable. Every `cache_*` reader reports on the selected set, which stays at index 0 unless changed, so single-set queries behave exactly as before. `cache_save` copies every set, not just the selected one.
- **`MYSQL_OPT_POOL_SIZE`** — caps how many connections the pool may open (the driver defaults to a maximum of 100).
- **Log rotation.** `logs/mysql.log` now rotates into `logs/archive/mysql.log.{N}.gz` once it passes 50 MB, gzipping each archive. Archives are never deleted by the plugin — cleanup stays with the operator, or `MYSQL_SAMP_LOG_ROTATION_KEEP` prunes automatically. Compression reuses the `flate2` the MySQL driver already pulls in, so it costs no new dependency.
- **Environment overrides for the log pipeline.** `MYSQL_SAMP_LOG_LEVEL`, `_DIR`, `_FILE`, `_ROTATION_MB`, `_ROTATION_KEEP`, `_NO_ROTATION`, `_NO_BANNER` and `_COMPRESS` retune logging without rebuilding the plugin. `mysql_log()` still overrides the level at runtime.
- The startup banner is now written to `logs/mysql.log` as well as the console, marking where each restart begins in a rotated log.
- The log file is flushed explicitly on plugin unload, so the tail is not lost if the process exits before the OS syncs the handle.

### Changed

- **Upgraded to rust-samp v3.4.0.** Brings SDK-side hardening that applies to this plugin directly: `Allocator` now rewinds the VM stack as well as the heap on drop, so a partially-pushed callback argument list in `callback.rs` can no longer leave the AMX stack unbalanced; `UnsizedBuffer::into_sized_buffer` clamps requested sizes to the VM data region; and `AmxString::to_bytes` no longer panics on a corrupted length.
- **File logging now uses the SDK's turnkey logger** instead of the hand-rolled writer. The console/file split is preserved — short, sanitized lines on the server console, full detail only in the file — by keeping the two channels separate: the SDK logger owns the file (`also_to_server(false)`), while the console is driven by `samp::plugin::logger()` converted into a standalone sink.
- Levels in `logs/mysql.log` are now written as `WARN` rather than `WARNING`.
- **Corrected the open.mp native-component registration instructions.** Native components are auto-discovered from the `components/` folder — the folder *is* the registration and no `config.json` entry is needed. Only legacy plugins must be declared under `pawn.legacy_plugins`. The old instructions told everyone to add an entry that does nothing. Fixed in the README, `docs/installation.md`, the examples README and the 1.1.0 changelog entry.
- **Release notes are now assembled by the workflow** from the matching `## [X.Y.Z]` section of this file, plus the `New Contributors` / `Full Changelog` appendix from the GitHub API and an artifacts footer. `RELEASE.md` was dropped as a drift hazard, as was the duplicated root `CNAME` (only `docs/CNAME` is deployed). Version literals were stripped from `CHECKLIST.md` and `docs/api-reference.md` so `Cargo.toml` stays the single source of truth.
- **Build: new toolchain requirements, because the TLS backend compiles C.** Until now the Linux build was pure Rust; `ring` changes that.
  - The **Linux i686 target needs 32-bit C support** (`gcc-multilib` / `glibc-devel.i686` / `lib32-glibc`). Both build scripts now check `gcc -m32` up front and name the package, since cc-rs otherwise fails with an opaque missing-header error. `scripts/build-windows.sh` checks this inside WSL as well and points at `--docker` as the alternative.
  - **Cross-compiling the `.dll` from Linux needs `llvm-lib`**, which cargo-xwin does not ship. `scripts/build-linux.sh` locates it under `/usr/lib/llvm-*/bin` automatically.
  - Building **on** Windows needs neither: MSVC's `lib.exe` does the archiving and `ring` ships pre-assembled objects, so NASM is not required.

### Removed

- `chrono` dependency — log timestamps come from the SDK logger.

### Security & governance

Supply-chain hardening of the repository itself. None of this changes the shipped plugin, but it constrains what a compromised dependency or workflow step can do.

- **Every GitHub Action is pinned by commit SHA**, with a trailing `# vX` comment so the intent stays readable. A mutable tag can be repointed at new code by whoever controls the action; a SHA cannot. Two actions take their parameter *from the ref* (`dtolnay/rust-toolchain`, `taiki-e/install-action`), so they are pinned by SHA **and** pass `toolchain:` / `tool:` explicitly — pinning alone would have broken them.
- **`docs/requirements.txt` is now a hash-pinned lockfile**, compiled from the new `docs/requirements.in` with `pip-compile --generate-hashes` and installed with `pip install --require-hashes`. Every MkDocs dependency is verified against a known digest before it runs in CI.
- **`.github/dependabot.yml`** — weekly updates for `github-actions`, `cargo` and the `pip` requirements under `/docs`. Pinning trades automatic updates for reproducibility; this pays that back. Note Dependabot does not bump git dependencies, so the `samp` revision stays a manual step.
- **OpenSSF Scorecard workflow** — analyses the repository's security posture and publishes the result.
- **Least-privilege tokens** — `permissions: {}` at the top of the Scorecard and Release Drafter workflows, with jobs opting into exactly what they need, and `persist-credentials: false` on every checkout so a compromised step cannot reuse the git credentials.

## [1.1.1] — 2026/05/18

### Fixed

- `MYSQL_OPT_SSL` and `MYSQL_OPT_SSL_CA` are now wired through to the connection builder. When `MYSQL_OPT_SSL` is enabled, `mysql_connect` configures `SslOpts::default()` on the pool (rustls). When `MYSQL_OPT_SSL_CA` is also set, its path is passed to `with_root_cert_path`; otherwise the platform trust store is used. Previously both options were accepted but silently ignored, so connections were always plaintext.

## [1.1.0] — 2026/05/18

Built on top of [rust-samp v3.0.0](https://github.com/NullSablex/rust-samp/releases/tag/v3.0.0). The same `.so` / `.dll` now loads on SA-MP and on Open Multiplayer (native component or legacy mode). No Pawn-visible API was removed or renamed.

### Added

- **Universal binary.** A single artifact runs on SA-MP and on Open Multiplayer. Open Multiplayer auto-loads it as a native component when dropped into the `components/` folder (no `config.json` entry needed — the folder itself is the registration), or in legacy mode when dropped into `plugins/` and declared under `pawn.legacy_plugins` in `config.json`.
- `mysql_tick()` — drains the dispatch queue manually. Kept for backwards compatibility only; with rust-samp v3 the unified `on_tick` callback already pumps the queue on both SA-MP (`ProcessTick`) and Open Multiplayer (`ITimersComponent`, 5 ms by default).
- `MYSQL_SAMP_VERSION` constant in `mysql_samp.inc` — string with the plugin version, auto-generated from `CARGO_PKG_VERSION` by `build.rs`.
- `on_component_free` lifecycle hook — emits a single informational log line when any neighbouring Open Multiplayer component is released. Useful when correlating "mysql_samp misbehaved after plugin X was unloaded" reports.
- CI: new `fmt` job (`cargo fmt --all -- --check`), `audit` job (`rustsec/audit-check`) and `coverage` job (`cargo llvm-cov`, LCOV uploaded as a workflow artifact).
- Release workflow: tag-vs-`Cargo.toml` sanity check. A `v1.2.3` release tag whose `Cargo.toml` still declares `1.2.2` fails before any artifact is built.
- Release workflow: the relevant `## [X.Y.Z]` section of `CHANGELOG.md` is now extracted and appended to the GitHub release body automatically — no copy-paste required.

### Changed

- **Tick dispatch is automatic.** `mysql_query` / `mysql_pquery` callbacks no longer require `SetTimer` + `mysql_tick()` in Open Multiplayer. The unified `on_tick` from rust-samp v3 fires on both servers and the plugin processes pending results inside it.
- `mysql_format` now truncates the rendered string at the destination buffer boundary (respecting UTF-8 char boundaries) and logs a warning, instead of aborting the native and returning `0`. The returned value is always the number of bytes written into `dest`.
- `mysql_format` and the callback format string now log a warning when they find an unknown specifier (`%z`, `%q`, …). Previously these were silently passed through or dropped.
- `mysql_options_set_int` rejects out-of-range integers explicitly:
  - `MYSQL_OPT_PORT` requires `0..=65535`. Negative or oversized values return `false` (the old code silently wrapped to `u16`).
  - `MYSQL_OPT_CONNECT_TIMEOUT` requires `>= 0`. Negative values return `false` (the old code silently wrapped to `u32`).
- The `logs/mysql.log` writer reports failures: when the file or the `logs/` directory cannot be written, the plugin emits exactly one console error (`[MySQL] Failed to write logs/mysql.log: <io error>. Further file-write errors will be suppressed.`) and then stops trying, instead of silently dropping every log line.
- Build scripts replaced. `scripts/build.sh` is gone; the project now ships `scripts/build-linux.sh` (Linux + Windows from Linux via `cargo-xwin`) and `scripts/build-windows.sh` (Windows + Linux via WSL or Docker/cross), matching the rust-samp upstream layout.
- Windows target moved from `i686-pc-windows-gnu` to `i686-pc-windows-msvc`. The MSVC toolchain is required so the binary can implement the Open Multiplayer `ComponentEntryPoint` ABI (Itanium on Linux, MSVC on Windows). `cargo-xwin` performs the cross-compile from Linux.
- Release workflow ships **raw artifacts** instead of zips: `mysql_samp.so`, `mysql_samp.dll`, `mysql_samp.inc`. Release notes are auto-generated from `Cargo.toml`, pinning the rust-samp SDK version.
- CI workflows declare per-job permissions (least privilege). `contents: read` at the top level; jobs opt in to `contents: write`, `issues: write` or `pull-requests: write` only when needed.
- Documentation site upgraded to a full MkDocs Material configuration with strict build (`mkdocs build --strict`). Documentation language switched to en-US; doc files renamed to English filenames (`installation.md`, `connection.md`, `errors.md`, `api-reference.md`, `security.md`, `migration.md`, `migration-changes.md`, `migration-examples.md`).
- README rewritten in en-US. Project links and table of contents updated to the new filenames.
- Internal: every cross-width or sign-changing integer conversion uses explicit `TryFrom` / `From` instead of the silent `as` cast. Negative Pawn cell values used as container indices are rejected explicitly instead of wrapping to `usize::MAX`.
- Internal: ORM raw natives (`orm_select` / `orm_update` / `orm_insert` / `orm_delete` / `orm_save`) deduplicated through a shared `run_orm_op` helper and an `OrmOp` enum — about 120 lines of repetition removed, error semantics preserved (same `MYSQL_ERROR_*` codes, same messages).
- Internal: `mysql_format` refactored into pure helpers (`parse_format`, `collect_format_values`, `render_format`, `truncate_to_buffer`) that are testable in isolation.
- Internal: Pawn native return types simplified to bare `T` (`bool`, `i32`, `f32`) where the native cannot fail; the macros from rust-samp v3 accept this without an `AmxResult` wrapper.

### Fixed

- Open Multiplayer log levels: with rust-samp v3 the default routing maps `log::Level` to `samp_sdk::omp::LogLevel` (`Error → Error`, `Warn → Warning`, `Info → Message`, `Debug → Debug`). Prior to v3 every line landed as `Message` regardless of severity.
- Documentation corrected: prior pages referenced a `cache_next_row()` native that does not exist, miscounted the natives (51 vs the actual 55), and omitted `MYSQL_OPT_AUTO_RECONNECT`, `cache_get_field_type`, `cache_is_any_active`, `cache_is_valid`, `cache_warning_count`. All fixed against the source of truth (`include/mysql_samp.inc.in` and `src/lib.rs`).
- `build.rs` no longer caches its `rerun-if-changed` directive on the `.inc.in` template only. The `MYSQL_SAMP_VERSION` literal in `include/mysql_samp.inc` is regenerated on every `cargo build`, so a `Cargo.toml` version bump propagates without `touch build.rs` or `cargo clean`. The write itself is idempotent — no spurious timestamp churn when nothing changed.

### Documentation

- New honest SSL caveat: `MYSQL_OPT_SSL` and `MYSQL_OPT_SSL_CA` are accepted by `mysql_options_set_int` / `mysql_options_set_str`, but the connection builder does not wire them through to the `mysql` crate yet (`TODO` in `src/connection.rs:67`). The connection is always plaintext today. This was true in 1.0.0 as well; only the documentation was honest about it in 1.1.0.
- Documented the universal-binary registration paths (`plugins=` for SA-MP, `components` for Open Multiplayer native, `legacy_plugins` for Open Multiplayer legacy).

### Tests

- Unit tests: 93 → 113 (+20). New coverage for `parse_format`, `render_format`, `truncate_to_buffer` and additional `escape_string` edge cases (consecutive quotes, double-escape invariant, all special characters at once, low-ASCII passthrough).

### Removed

- `samp-only` Cargo feature — never actually used, dropped to reduce surface area.
- `.cargo/config.toml` — was forcing every `cargo` invocation to `i686-unknown-linux-gnu`. The release scripts and CI workflows pass `--target` explicitly, so the override only slowed local `cargo check`/`test`/`clippy` without benefit.

### Dependencies

- `samp` switched from a local path dependency to git (`tag = "v3.0.0"`).
- `log` removed as a direct dependency (re-exported by `samp`).
- GitHub Actions bumped to the current major: `actions/checkout` v4 → v6, `actions/setup-python` v5 → v6, `actions/cache` v4 → v5, `actions/upload-artifact` v4 → v7, `softprops/action-gh-release` v2 → v3, `rustsec/audit-check` v2.0.0 → v2 (floats within the major for patches).

## [1.0.0] — 2026/03/09

First stable release. Added ORM, cache, threaded query pipeline, full safety net.

### Added

#### Options
- `MYSQL_OPT_AUTO_RECONNECT` — turns on the one-shot retry when a query fails because the server dropped the connection (enabled by default).

#### Queries (non-blocking)
- `mysql_query` — threaded query with callback and FIFO ordering (replaces `mysql_tquery` from R41-4).
- `mysql_pquery` — parallel query with no ordering guarantee.
- `mysql_escape_string` — pure SQL escape function.
- `mysql_format` — `printf`-style query builder (`%d`, `%f`, `%s` / `%e` with automatic escape, `%r` raw, `%%` literal).

#### Cache
- `cache_get_row_count` / `cache_get_field_count` — result dimensions.
- `cache_get_field_name` / `cache_get_field_type` — column metadata.
- `cache_get_value_index` / `cache_get_value_index_int` / `cache_get_value_index_float` — value by index.
- `cache_get_value_name` / `cache_get_value_name_int` / `cache_get_value_name_float` — value by name (case-insensitive).
- `cache_is_value_index_null` / `cache_is_value_name_null` — NULL checks.
- `cache_affected_rows` / `cache_insert_id` — write metadata.
- `cache_warning_count` — server warning count.
- `cache_get_query_exec_time` / `cache_get_query_string` — query debugging.
- `cache_save` / `cache_delete` — persist a cache across callbacks.
- `cache_set_active` / `cache_unset_active` — manually activate a saved cache.
- `cache_is_any_active` / `cache_is_valid` — state checks.

#### ORM
- `orm_create` / `orm_destroy` — ORM lifecycle.
- `orm_errno` — last error code for the instance.
- `orm_select` / `orm_update` / `orm_insert` / `orm_delete` — non-blocking CRUD.
- `orm_save` — INSERT or UPDATE depending on the key value.
- `orm_apply_cache` — copy a cache row into the bound Pawn variables.
- `orm_addvar_int` / `orm_addvar_float` / `orm_addvar_string` — bind a Pawn variable to a column.
- `orm_delvar` / `orm_clear_vars` — remove bindings.
- `orm_setkey` — declare the primary-key column.

#### Forward
- `OnQueryError(errorid, error[], callback[], query[], connId)` — fired when a threaded query fails.

#### Utility
- `mysql_error` — fetch the last error message into a buffer.
- `mysql_set_charset` / `mysql_get_charset` — read/write the connection charset.
- `mysql_unprocessed_queries` — count of queries currently in flight + buffered.
- `mysql_log` — runtime log-level switch.

#### Infrastructure
- Connection pool (`mysql::Pool`) shared between worker threads.
- Cache stack (push/pop around callbacks) plus persistent slots.
- `QueryManager` with `mpsc` channel and FIFO reordering.
- Callback dispatcher with variadic-parameter support (int, float, string).
- ORM auto-cleanup when the owning AMX unloads.

#### Tests
- 93 unit tests covering every pure path: `error.rs`, `options.rs`, `cache.rs`, `connection.rs` (`escape_string`/`escape_identifier`/`ConnectionManager`), `query.rs` (FIFO ordering, partial dispatch, parallel, mixed), `orm.rs`.

#### Documentation
- Banner in `include/mysql_samp.inc` with project metadata.
- `docs/options.md`, `docs/benchmark.md`, `docs/mudancas.md`, `docs/exemplos-migracao.md`, `docs/migracao.md`, `docs/api.md`, `docs/queries.md`, `docs/cache.md`, `docs/orm.md`, `docs/erros.md`, `docs/conexao.md`, `docs/seguranca.md`, `docs/instalacao.md`.

### Changed

- `ConnectionEntry` migrated from `mysql::Conn` to `mysql::Pool` (`Send + Sync + Clone`).
- `MysqlPlugin` now implements `on_amx_load`, `on_amx_unload`, and `process_tick`.
- `enable_process_tick()` enabled in the plugin constructor.

### Removed

- `MysqlError::Unknown` variant — never used; remaining variants renumbered (`QueryFailed = 5`, `NoCacheActive = 6`, `InvalidOrm = 7`, `OrmKeyNotSet = 8`).
- `MAX_CONCURRENT_QUERIES` artificial cap of 128 — `mysql::Pool` already applies backpressure; the cap was silently rejecting queries above the ceiling.

### Security

- **CWE-89** — `%s` in `mysql_format` now escapes by default (was raw). New `%r` for raw strings.
- **CWE-89** — SQL identifiers in the ORM are sanitised via `escape_identifier()`.
- **CWE-787** — ORM string writes capped at 4096 bytes.
- **CWE-770** — 1024 saved caches and 100 000 rows per result, both enforced.
- **CWE-252** — every push to the AMX stack is checked.
- **CWE-190** — id counters use `wrapping_add(...).max(1)` in every manager.
- UTF-8 forced via `SET NAMES utf8mb4` on every pool connection — blocks multi-byte escape-bypass attacks.
- Auto reconnect retries dropped connections once before reporting through `OnQueryError`.

## Historical releases

- [v0.x](changelog/v0.x.md) — 0.1.0
