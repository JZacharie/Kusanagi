use thiserror::Error;

pub type Result<T> = std::result::Result<T, KusanagiError>;

#[derive(Error, Debug)]
pub enum KusanagiError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Kubernetes API error: {0}")]
    Kubernetes(String),
    
    #[error("HTTP error: {0}")]
    Http(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl KusanagiError {
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }
    
    pub fn k8s(msg: impl Into<String>) -> Self {
        Self::Kubernetes(msg.into())
    }
    
    pub fn http(msg: impl Into<String>) -> Self {
        Self::Http(msg.into())
    }
    
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

impl From<std::io::Error> for KusanagiError {
    fn from(err: std::io::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<std::num::ParseIntError> for KusanagiError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::Config(format!("Invalid number: {}", err))
    }
}
