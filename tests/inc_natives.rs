//! The includes declare exactly the natives the plugin registers.
//!
//! Adding a native touches four places - the file under `src/natives/`, the
//! list in `src/lib.rs`, the declaration in `include/mysql_samp.inc.in` and an
//! example. Forgetting the third is the easy one: everything builds, and the
//! native is simply unreachable from Pawn, which only shows up when someone
//! tries to call it.
//!
//! rust-samp 3.5.0 can emit the authoritative list at runtime - start the
//! server with `SAMP_PAWN_INCLUDE=/tmp/generated.inc` and the SDK writes the
//! declarations of every registered native. That needs a live server, so it is
//! a tool for checking by hand rather than something CI can run. This test
//! reproduces the comparison from the two files a build already produces.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let path = root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

/// Every native registered in the `initialize_plugin!` list.
fn registered_natives() -> BTreeSet<String> {
    let src = read("src/lib.rs");
    let list = src
        .split_once("natives: [")
        .expect("no `natives: [` in src/lib.rs")
        .1
        .split_once("],")
        .expect("unterminated natives list in src/lib.rs")
        .0;

    list.lines()
        .filter_map(|line| {
            let line = line.trim();
            // Skip the section comments that group the list.
            let rest = line.strip_prefix("MysqlPlugin::")?;
            Some(rest.trim_end_matches(',').to_string())
        })
        .collect()
}

/// Every native declared in an include, tags and return types stripped.
fn declared_natives(rel: &str) -> BTreeSet<String> {
    read(rel)
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("native ")?;
            let name = rest.split('(').next()?;
            // `bool:mysql_close` and `Float:cache_...` carry a Pawn tag.
            Some(name.rsplit(':').next()?.trim().to_string())
        })
        .collect()
}

#[test]
fn every_registered_native_is_declared_in_the_include() {
    let registered = registered_natives();
    let declared = declared_natives("include/mysql_samp.inc");

    assert!(
        !registered.is_empty(),
        "parsed no natives from src/lib.rs - the parser broke, not the include"
    );

    let missing: Vec<_> = registered.difference(&declared).collect();
    assert!(
        missing.is_empty(),
        "registered in src/lib.rs but not declared in include/mysql_samp.inc.in: {missing:?}"
    );

    let extra: Vec<_> = declared.difference(&registered).collect();
    assert!(
        extra.is_empty(),
        "declared in the include but not registered in src/lib.rs, so calling \
         them fails at runtime: {extra:?}"
    );
}

#[test]
fn the_omp_include_aliases_every_native() {
    // The styled include is generated from the base one, so a mismatch here
    // means `to_omp_name()` in build.rs dropped or renamed something.
    let base = declared_natives("include/mysql_samp.inc");
    let omp = read("include/mysql_samp_omp.inc");

    let aliased: BTreeSet<String> = omp
        .lines()
        .filter_map(|line| {
            // `native MySQL_Connect(...) = mysql_connect;`
            let target = line.strip_prefix("native ")?.rsplit_once('=')?.1;
            Some(target.trim().trim_end_matches(';').to_string())
        })
        .collect();

    let missing: Vec<_> = base.difference(&aliased).collect();
    assert!(
        missing.is_empty(),
        "no open.mp alias generated for: {missing:?}"
    );
}
