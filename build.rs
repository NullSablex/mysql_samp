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
    // without manual intervention. The write below is idempotent — it only
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
}
