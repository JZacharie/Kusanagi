#[derive(Debug, thiserror::Error)]
pub enum KusanagiError {
    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Cache error: {message}")]
    Cache { message: String },
}

pub type Result<T> = std::result::Result<T, KusanagiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = KusanagiError::Config {
            message: "test".to_string(),
        };
        assert_eq!(err.to_string(), "Configuration error: test");

        let err = KusanagiError::Cache {
            message: "fail".to_string(),
        };
        assert_eq!(err.to_string(), "Cache error: fail");
    }
}
