# Security Policy — mysql_samp

## Reporting a vulnerability

Found a security vulnerability? Please do not open a public issue.

**Contact:** open a private [Security Advisory](https://github.com/NullSablex/mysql_samp/security/advisories/new)
on GitHub, or e-mail the maintainer directly.

Expected response within **7 business days**.

Useful things to include, as far as you have them: the plugin version
(`MYSQL_SAMP_VERSION`), the server (SA-MP or open.mp, native or legacy), the
MySQL or MariaDB version, what an attacker gains, and the smallest Pawn snippet
or SQL that shows it. A proof of concept helps, but do not hold a report back
for lack of one.

---

## Scope

This policy covers the plugin in `NullSablex/mysql_samp`: the Rust source, the
generated Pawn includes and the examples.

A plugin that sits between untrusted input (whatever a player typed) and a
database credential makes these the findings that matter most:

- **SQL injection** — any value that reaches the SQL text unescaped when the
  API promised otherwise: `%s` in `mysql_format`, an ORM column or table name,
  a `cache_*` value fed back into a query, or escaping that does not match the
  server's `sql_mode`. The plugin detects `NO_BACKSLASH_ESCAPES` per connection
  and escapes accordingly; a case where that detection is wrong, or where the
  mode changes under it, is in scope.
- **TLS** — a session that continues unencrypted when `MYSQL_OPT_SSL` asked for
  encryption, a certificate accepted while verification is on, or a hostname
  that is not checked.
- **Credential exposure** — the database password, or a value carrying it,
  reaching the console, `logs/mysql.log`, an error message or a panic. The same
  applies to a plaintext password handed to `mysql_hash_password`.
- **Memory safety and exhaustion** — a crash, a use-after-free or unbounded
  growth reachable from Pawn: cache entries, result rows, prepared statements,
  transactions or ORM instances.
- **Password storage** — an Argon2id hash that verifies when it should not, or
  parameters weaker than documented.

Out of scope: vulnerabilities in MySQL, MariaDB, SA-MP or open.mp themselves; a
server operator's own configuration, such as granting the database user more
privileges than the gamemode needs; and gamemode code built on top of the
plugin — including a gamemode that turns a protection off
(`MYSQL_OPT_SSL_VERIFY_CERT = 0`, `%r`), which the plugin documents as
dangerous and requires you to opt into.

A report showing that **the plugin's own documentation leads people into an
insecure pattern** is in scope, even when no code change is needed.

---

## Supported versions

Only the most recent [release](https://github.com/NullSablex/mysql_samp/releases)
receives security fixes.

---

## Dependencies

The MySQL protocol and TLS are compiled into the binary: the plugin loads no
`libmysqlclient` and no OpenSSL. TLS is
[rustls](https://github.com/rustls/rustls), verifying against the webpki root
bundle that ships inside the binary rather than the operating system's store —
which is why a server with an internal CA needs `MYSQL_OPT_SSL_CA`. The driver
is [mysql](https://github.com/blackbeam/rust-mysql-simple) with
`default-features = false`, and password hashing is
[argon2](https://github.com/RustCrypto/password-hashes), keeping the whole
dependency tree pure Rust.

Known advisories against dependencies are tracked in
[`CHANGELOG.md`](CHANGELOG.md) and audited in CI on every pull request and
weekly on a schedule.

---

## Hardening guidance

For using the plugin safely — prepared statements, escaping and `sql_mode`,
TLS and certificate pinning, Argon2id password storage, and what does and does
not reach the logs — see the
[security documentation](https://nullsablex.github.io/mysql_samp/security/).
