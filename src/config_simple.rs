use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub prometheus_url: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            host: env::var("KUSANAGI_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("KUSANAGI_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            prometheus_url: env::var("PROMETHEUS_URL")
                .unwrap_or_else(|_| "http://prometheus:9090".to_string()),
        }
    }
}
