#[derive(Debug, thiserror::Error)]
pub enum KusanagiError {
    #[error("Configuration error: {message}")]
    Config { message: String },
    
    #[error("Cache error: {message}")]
    Cache { message: String },
}

pub type Result<T> = std::result::Result<T, KusanagiError>;
