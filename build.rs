use std::process::Command;

fn main() {
    // Get current timestamp for build
    let output = Command::new("date")
        .arg("+%Y-%m-%d %H:%M:%S UTC")
        .env("TZ", "UTC")
        .output()
        .expect("Failed to get date");
    
    let build_time = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8")
        .trim()
        .to_string();
    
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_time);
}
