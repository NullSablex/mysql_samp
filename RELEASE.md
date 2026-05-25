## mysql_samp v1.1.1

Patch release. One fix: TLS connections actually work now.

### Fixed

- **`MYSQL_OPT_SSL` and `MYSQL_OPT_SSL_CA` are wired through to the connection builder.** Up to v1.1.0 both options were accepted by `mysql_options_set_int` / `mysql_options_set_str` but ignored on connect — every connection was plaintext, even when SSL was requested. From v1.1.1 on:
  - With `MYSQL_OPT_SSL` set to `true`, `mysql_connect` configures `SslOpts::default()` on the pool (rustls, via the `default-rust` feature of the `mysql` crate).
  - With `MYSQL_OPT_SSL_CA` also set, its path is passed to `with_root_cert_path`. Both `.pem` and `.der` are accepted.
  - Without `MYSQL_OPT_SSL_CA`, the platform's default trust store is used.

No Pawn API change. Existing scripts that already called `mysql_options_set_int(opts, MYSQL_OPT_SSL, 1)` start getting real TLS without any code edit — verify your CA path and server certificate before deploying.

### Artifacts

| File | Platform |
|---|---|
| `mysql_samp.so` | Linux i686 (`i686-unknown-linux-gnu`) |
| `mysql_samp.dll` | Windows i686 (`i686-pc-windows-msvc`) |
| `mysql_samp.inc` | Pawn include — identical file for SA-MP and Open Multiplayer |

The Pawn version constant in `mysql_samp.inc` is regenerated automatically from `Cargo.toml` on every build.

### Full changelog

See [`CHANGELOG.md`](https://github.com/NullSablex/mysql_samp/blob/master/CHANGELOG.md). Diff vs the previous release: [v1.1.0 → v1.1.1](https://github.com/NullSablex/mysql_samp/compare/v1.1.0...v1.1.1).
