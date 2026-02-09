#[derive(Debug, thiserror::Error)]
pub enum KusanagiError {
    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Cache error: {message}")]
    Cache { message: String },
    
    #[error("External service error: {0}")]
    ExternalService(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

// Type alias for backward compatibility
pub type Result<T> = std::result::Result<T, KusanagiError>;

// AppError alias for hexagonal architecture
pub use KusanagiError as AppError;

// Helper constructors
impl KusanagiError {
    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Config { message: msg.into() }
    }
    
    pub fn external_service(msg: impl Into<String>) -> Self {
        Self::ExternalService(msg.into())
    }
    
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }
}

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
