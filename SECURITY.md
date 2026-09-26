# Security policy

## Supported versions

Only the latest release line receives security fixes. The version currently
shipped is the one in [`Cargo.toml`](Cargo.toml); releases are listed on the
[releases page](https://github.com/NullSablex/mysql_samp/releases).

| Version | Supported |
|---|---|
| 1.x (latest release) | Yes |
| Older 1.x releases | No — upgrade to the latest |
| Pre-1.0 | No |

## Reporting a vulnerability

**Report privately, not as a public issue.** Use GitHub's private reporting:

- **[Open a private security advisory](https://github.com/NullSablex/mysql_samp/security/advisories/new)**
  — the preferred route. It is visible only to you and the maintainer.
- If that page is unavailable to you, open a normal
  [issue](https://github.com/NullSablex/mysql_samp/issues) saying only that you
  have a security report and asking for a private channel. Do not put the
  details in it.

Useful things to include, as far as you have them:

- the plugin version (`MYSQL_SAMP_VERSION`, or the release you downloaded) and
  the server it runs on (SA-MP or open.mp, native or legacy);
- the MySQL or MariaDB version;
- what an attacker can do with it — reading other players' data, crashing the
  server, running SQL they should not be able to run;
- the smallest Pawn snippet or SQL that shows the problem.

A proof of concept helps, but do not hold a report back for lack of one.

## What to expect

This is an independent project maintained by one person, so the honest answer
is best effort rather than a contractual window: reports are acknowledged as
soon as they are seen, and a fix is prioritised over everything else once the
issue is confirmed. You will be told what was concluded either way, including
when the conclusion is that it is not a vulnerability.

Fixes ship in a normal release, with the advisory published once the fix is
available. Credit goes to the reporter unless you ask otherwise.

## Scope

In scope: anything in this repository — the plugin itself, the Pawn includes it
generates, the build scripts and the CI workflows.

Out of scope, because they are not this project's to fix:

- vulnerabilities in MySQL, MariaDB, SA-MP or open.mp themselves;
- gamemode code that misuses the API, for example building SQL with
  [`%r`](https://nullsablex.github.io/mysql_samp/security/) or with the standard
  `format` and then passing player input into it. The
  [security page](https://nullsablex.github.io/mysql_samp/security/) explains
  the safe path;
- a server operator's own configuration, such as granting the database user
  more privileges than the gamemode needs, or disabling certificate
  verification with `MYSQL_OPT_SSL_VERIFY_CERT`.

A report that shows the **plugin's own documentation leads people into an
insecure pattern** is in scope, even when no code change is needed.

## Hardening guidance

For using the plugin safely — prepared statements, escaping and `sql_mode`,
TLS and certificate pinning, Argon2id password storage, and what does and does
not reach the logs — see the
[security documentation](https://nullsablex.github.io/mysql_samp/security/).
