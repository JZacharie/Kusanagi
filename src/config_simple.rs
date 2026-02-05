use crate::error::{Result, KusanagiError};
use std::env;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub dev_mode: bool,
}

impl Config {
    pub fn load() -> Result<Self> {
        Ok(Config {
            server: ServerConfig {
                host: env::var("KUSANAGI_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("KUSANAGI_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .map_err(|_| KusanagiError::config("Invalid port number"))?,
            },
            dev_mode: env::var("KUSANAGI_DEV_MODE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        })
    }
}
