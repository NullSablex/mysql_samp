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
}
