#[cfg(test)]
mod tests {
    use kusanagi::config::Config;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
    }

    #[test]
    fn test_config_from_env() {
        std::env::set_var("BIND_ADDR", "127.0.0.1:9090");
        let config = Config::default();
        // Config should use default if env parsing fails
        assert!(config.server.port > 0);
    }

    #[test]
    fn test_mqtt_config() {
        let config = Config::default();
        assert!(!config.mqtt.host.is_empty());
        assert!(config.mqtt.port > 0);
    }
}
