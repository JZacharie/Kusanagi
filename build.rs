fn main() {
    // Get current timestamp using Rust std
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    
    // Format as human-readable UTC timestamp
    let secs = now.as_secs();
    let datetime = chrono::NaiveDateTime::from_timestamp_opt(secs as i64, 0)
        .expect("Invalid timestamp");
    
    let build_time = datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string();
    
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_time);
    println!("cargo:rerun-if-changed=build.rs");
}
