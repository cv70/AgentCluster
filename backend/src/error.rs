use std::error::Error;
use std::fmt;

/// Application-specific error type for better error handling
#[derive(Debug)]
pub enum AppError {
    ConfigurationError(String),
    DatabaseError(String),
    InfrastructureError(String),
    DomainError(String),
    IOError(String),
}

impl Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::InfrastructureError(msg) => write!(f, "Infrastructure error: {}", msg),
            AppError::DomainError(msg) => write!(f, "Domain error: {}", msg),
            AppError::IOError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

/// Result type alias for application functions
pub type AppResult<T> = Result<T, AppError>;

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IOError(err.to_string())
    }
}

impl From<serde_yaml::Error> for AppError {
    fn from(err: serde_yaml::Error) -> Self {
        AppError::ConfigurationError(err.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}

impl From<etcd_client::Error> for AppError {
    fn from(err: etcd_client::Error) -> Self {
        AppError::InfrastructureError(err.to_string())
    }
}

impl From<Box<dyn Error + Send + Sync>> for AppError {
    fn from(err: Box<dyn Error + Send + Sync>) -> Self {
        AppError::InfrastructureError(err.to_string())
    }
}
