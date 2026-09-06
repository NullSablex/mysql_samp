fn main() {
    let output = std::process::Command::new("date")
        .arg("+%b %d %Y|%H:%M:%S|%Y")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok());

    let (date, time, year) = match output {
        Some(s) => {
            let s = s.trim().to_string();
            let parts: Vec<&str> = s.splitn(3, '|').collect();
            (
                parts.first().unwrap_or(&"Unknown").to_string(),
                parts.get(1).unwrap_or(&"Unknown").to_string(),
                parts.get(2).unwrap_or(&"Unknown").to_string(),
            )
        }
        None => (
            "Unknown".to_string(),
            "Unknown".to_string(),
            "Unknown".to_string(),
        ),
    };

    println!("cargo:rustc-env=BUILD_DATE={date}");
    println!("cargo:rustc-env=BUILD_TIME={time}");
    println!("cargo:rustc-env=BUILD_YEAR={year}");

    generate_inc();
}

fn generate_inc() {
    use std::fs;

    let template_path = "include/mysql_samp.inc.in";
    let output_path = "include/mysql_samp.inc";

    // No `cargo:rerun-if-changed` directives: build.rs runs on every build
    // so the .inc tracks the current Cargo version, build date and template
    // without manual intervention. The write below is idempotent - it only
    // touches disk when the rendered output actually differs from the file
    // on disk, so the always-run policy does not churn timestamps.

    let template = fs::read_to_string(template_path)
        .unwrap_or_else(|e| panic!("failed to read {template_path}: {e}"));

    let version = env!("CARGO_PKG_VERSION");
    let rendered = template.replace("{{VERSION}}", version);

    if fs::read_to_string(output_path).ok().as_deref() != Some(rendered.as_str()) {
        fs::write(output_path, &rendered)
            .unwrap_or_else(|e| panic!("failed to write {output_path}: {e}"));
    }

    generate_omp_inc(&rendered);
}

/// Generates `mysql_samp_omp.inc` - a **standalone** include exposing the API
/// under open.mp-style `Prefix_PascalCase` names.
///
/// Self-contained on purpose: the two includes are alternatives (you write one
/// style or the other, never both in the same script), so this carries its own
/// copy of the enums, the `MYSQL_SAMP_VERSION` define and the `OnQueryError`
/// forward rather than pulling in the base. Only the natives differ - each
/// becomes `native Styled(...) = real;`, a Pawn alias to the real snake_case
/// native, which resolves at runtime with no cost and no plugin-side change
/// (the target need not even be declared, so the snake_case names are absent).
///
/// Derived from the base `.inc` on every build, so the two never drift: a
/// native, enum or constant added to the template appears here automatically.
fn generate_omp_inc(base_rendered: &str) {
    use std::fmt::Write;
    use std::fs;

    let output_path = "include/mysql_samp_omp.inc";

    let mut out = String::from(
        "/*\n\
         \x20* mysql_samp - open.mp naming style (standalone).\n\
         \x20*\n\
         \x20* GENERATED from mysql_samp.inc by build.rs - do not edit by hand.\n\
         \x20*\n\
         \x20* Include THIS instead of <mysql_samp> to write the API in\n\
         \x20* open.mp's Prefix_PascalCase style (MySQL_Connect, Cache_GetRowCount,\n\
         \x20* ORM_Create). Each native aliases the real one, so there is no\n\
         \x20* runtime cost and nothing changes on the plugin side. This file is\n\
         \x20* self-contained - do not include it together with <mysql_samp>.\n\
         \x20*/\n\n\
         #if defined _mysql_samp_omp_included\n    #endinput\n#endif\n\
         #define _mysql_samp_omp_included\n",
    );

    // Everything after the base guard: the version define, the enums, the
    // forward and the natives. The natives are rewritten as aliases; the rest
    // is copied verbatim, so enums/defines/forward stay a single source.
    let marker = "#define _mysql_samp_included";
    let body = match base_rendered.find(marker) {
        Some(i) => &base_rendered[i + marker.len()..],
        None => base_rendered,
    };

    for line in body.lines() {
        let trimmed = line.trim();

        let Some(rest) = trimmed.strip_prefix("native ") else {
            // Not a native declaration - copy the line as-is (enum, forward,
            // define, comment, blank).
            out.push_str(line);
            out.push('\n');
            continue;
        };

        let rest = rest.trim().trim_end_matches(';').trim();
        let (Some(open), Some(close)) = (rest.find('('), rest.rfind(')')) else {
            out.push_str(line);
            out.push('\n');
            continue;
        };
        let head = rest[..open].trim(); // optional `tag:` plus the native name
        let params = &rest[open + 1..close];

        let (tag, name) = match head.rfind(':') {
            Some(i) => (&head[..=i], head[i + 1..].trim()),
            None => ("", head),
        };

        let styled = to_omp_name(name);
        // The right-hand side is the real native the plugin registered; the
        // left-hand side is the alias the gamemode writes.
        let _ = writeln!(out, "native {tag}{styled}({params}) = {name};");
    }

    if fs::read_to_string(output_path).ok().as_deref() != Some(out.as_str()) {
        fs::write(output_path, &out)
            .unwrap_or_else(|e| panic!("failed to write {output_path}: {e}"));
    }
}

/// `mysql_stmt_new` -> `MySQL_StmtNew`, `cache_get_row_count` ->
/// `Cache_GetRowCount`, `orm_create` -> `ORM_Create`.
///
/// The first segment is the group prefix, capitalised the way open.mp writes
/// these acronyms; the remaining segments become one PascalCase word.
fn to_omp_name(name: &str) -> String {
    // A handful of native names glue two words together in one snake_case
    // segment (`pquery`, `addvar`, ...), which the mechanical split cannot break
    // apart. Map those explicitly so the styled name reads the way open.mp
    // would write it.
    let full = match name {
        "mysql_pquery" => Some("MySQL_PQuery"),
        "mysql_stmt_pexecute" => Some("MySQL_StmtPExecute"),
        "orm_addvar_int" => Some("ORM_AddVarInt"),
        "orm_addvar_float" => Some("ORM_AddVarFloat"),
        "orm_addvar_string" => Some("ORM_AddVarString"),
        "orm_delvar" => Some("ORM_DelVar"),
        "orm_setkey" => Some("ORM_SetKey"),
        _ => None,
    };
    if let Some(name) = full {
        return name.to_string();
    }

    let mut segments = name.split('_');

    let prefix = match segments.next().unwrap_or("") {
        "mysql" => "MySQL".to_string(),
        "cache" => "Cache".to_string(),
        "orm" => "ORM".to_string(),
        other => capitalize(other),
    };

    let rest: String = segments.map(capitalize).collect();
    if rest.is_empty() {
        prefix
    } else {
        format!("{prefix}_{rest}")
    }
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}
