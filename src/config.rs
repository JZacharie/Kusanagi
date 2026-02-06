#[derive(Debug, Clone, Default)]
pub struct Config {
    pub server: ServerConfig,
    pub mqtt: MqttConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(), // Default local broker
            port: 1883,
        }
    }
}
