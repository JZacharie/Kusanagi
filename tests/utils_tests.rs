//! Tests for utility functions

use chrono::{DateTime, Utc, Duration};

/// Calculate age from timestamp
fn calculate_age_from_timestamp(timestamp: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*timestamp);

    if duration.num_days() > 0 {
        format!("{}d", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{}m", duration.num_minutes())
    } else {
        format!("{}s", duration.num_seconds())
    }
}

/// Format bytes to human readable string
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }

    let exp = (bytes as f64).log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
    let value = bytes as f64 / 1024_f64.powi(exp as i32);

    if exp == 0 {
        format!("{} {}", bytes, UNITS[exp])
    } else {
        format!("{:.2} {}", value, UNITS[exp])
    }
}

/// Parse memory string to bytes
fn parse_memory_to_bytes(memory: &str) -> Result<u64, String> {
    let memory = memory.trim().to_uppercase();
    
    if memory.ends_with("GI") || memory.ends_with("GIB") {
        let num: f64 = memory
            .trim_end_matches("GIB")
            .trim_end_matches("GI")
            .trim()
            .parse()
            .map_err(|_| "Invalid number".to_string())?;
        Ok((num * 1024.0 * 1024.0 * 1024.0) as u64)
    } else if memory.ends_with("MI") || memory.ends_with("MIB") {
        let num: f64 = memory
            .trim_end_matches("MIB")
            .trim_end_matches("MI")
            .trim()
            .parse()
            .map_err(|_| "Invalid number".to_string())?;
        Ok((num * 1024.0 * 1024.0) as u64)
    } else if memory.ends_with("KI") || memory.ends_with("KIB") {
        let num: f64 = memory
            .trim_end_matches("KIB")
            .trim_end_matches("KI")
            .trim()
            .parse()
            .map_err(|_| "Invalid number".to_string())?;
        Ok((num * 1024.0) as u64)
    } else if memory.ends_with('B') {
        let num: u64 = memory
            .trim_end_matches('B')
            .trim()
            .parse()
            .map_err(|_| "Invalid number".to_string())?;
        Ok(num)
    } else {
        Err("Unknown unit".to_string())
    }
}

/// Truncate string with ellipsis
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[test]
fn test_calculate_age_from_timestamp_days() {
    let timestamp = Utc::now() - Duration::days(5);
    let age = calculate_age_from_timestamp(&timestamp);
    assert_eq!(age, "5d");
}

#[test]
fn test_calculate_age_from_timestamp_hours() {
    let timestamp = Utc::now() - Duration::hours(3);
    let age = calculate_age_from_timestamp(&timestamp);
    assert_eq!(age, "3h");
}

#[test]
fn test_calculate_age_from_timestamp_minutes() {
    let timestamp = Utc::now() - Duration::minutes(45);
    let age = calculate_age_from_timestamp(&timestamp);
    assert_eq!(age, "45m");
}

#[test]
fn test_calculate_age_from_timestamp_seconds() {
    let timestamp = Utc::now() - Duration::seconds(30);
    let age = calculate_age_from_timestamp(&timestamp);
    assert!(age.ends_with('s'));
}

#[test]
fn test_format_bytes_bytes() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(500), "500 B");
    assert_eq!(format_bytes(1023), "1023 B");
}

#[test]
fn test_format_bytes_kib() {
    assert_eq!(format_bytes(1024), "1.00 KiB");
    assert_eq!(format_bytes(1536), "1.50 KiB");
    assert_eq!(format_bytes(10240), "10.00 KiB");
}

#[test]
fn test_format_bytes_mib() {
    assert_eq!(format_bytes(1024 * 1024), "1.00 MiB");
    assert_eq!(format_bytes(1024 * 1024 * 512), "512.00 MiB");
}

#[test]
fn test_format_bytes_gib() {
    assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GiB");
    assert_eq!(format_bytes(1024 * 1024 * 1024 * 16), "16.00 GiB");
}

#[test]
fn test_format_bytes_tib() {
    let tib = 1024_u64.pow(4);
    assert_eq!(format_bytes(tib), "1.00 TiB");
}

#[test]
fn test_parse_memory_to_bytes_gib() {
    assert_eq!(parse_memory_to_bytes("1Gi").unwrap(), 1024 * 1024 * 1024);
    assert_eq!(parse_memory_to_bytes("2GiB").unwrap(), 2 * 1024 * 1024 * 1024);
    assert_eq!(parse_memory_to_bytes("0.5 Gi").unwrap(), 512 * 1024 * 1024);
}

#[test]
fn test_parse_memory_to_bytes_mib() {
    assert_eq!(parse_memory_to_bytes("512Mi").unwrap(), 512 * 1024 * 1024);
    assert_eq!(parse_memory_to_bytes("256 MiB").unwrap(), 256 * 1024 * 1024);
}

#[test]
fn test_parse_memory_to_bytes_kib() {
    assert_eq!(parse_memory_to_bytes("64Ki").unwrap(), 64 * 1024);
    assert_eq!(parse_memory_to_bytes("128 KiB").unwrap(), 128 * 1024);
}

#[test]
fn test_parse_memory_to_bytes_bytes() {
    assert_eq!(parse_memory_to_bytes("1024B").unwrap(), 1024);
    assert_eq!(parse_memory_to_bytes("512 B").unwrap(), 512);
}

#[test]
fn test_parse_memory_to_bytes_invalid() {
    assert!(parse_memory_to_bytes("invalid").is_err());
    assert!(parse_memory_to_bytes("10XB").is_err());
    assert!(parse_memory_to_bytes("").is_err());
}

#[test]
fn test_truncate_string_short() {
    assert_eq!(truncate_string("hello", 10), "hello");
    assert_eq!(truncate_string("test", 4), "test");
}

#[test]
fn test_truncate_string_long() {
    assert_eq!(truncate_string("hello world", 8), "hello...");
    assert_eq!(truncate_string("very long string here", 10), "very lo...");
}

#[test]
fn test_truncate_string_exact() {
    assert_eq!(truncate_string("exact", 5), "exact");
    assert_eq!(truncate_string("four", 3), "...");
}
