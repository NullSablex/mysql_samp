## What this changes

<!-- What was happening, and why this fix and not another. -->

## How it was verified

<!-- Which of these ran, and anything you tested by hand on a server. -->

- [ ] `cargo test --target i686-unknown-linux-gnu`
- [ ] `cargo clippy --target i686-unknown-linux-gnu --all-targets -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] `python3 .github/scripts/check_pawn_encoding.py`
- [ ] The examples compile with no diagnostics

## Checklist

- [ ] A test that would have caught the bug, or that covers the new behaviour
- [ ] Documentation updated in the same change
- [ ] A new native is declared in `include/mysql_samp.inc.in` (never in the
      generated includes), listed in `src/lib.rs` and used by an example
- [ ] Nothing added blocks the server tick
