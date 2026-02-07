#[cfg(test)]
mod tests {
    use kusanagi::error::KusanagiError;

    #[test]
    fn test_config_error() {
        let err = KusanagiError::Config {
            message: "test config".to_string(),
        };
        assert_eq!(err.to_string(), "Configuration error: test config");
    }

    #[test]
    fn test_cache_error() {
        let err = KusanagiError::Cache {
            message: "cache failed".to_string(),
        };
        assert_eq!(err.to_string(), "Cache error: cache failed");
    }
}
