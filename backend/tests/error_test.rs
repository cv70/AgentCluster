use crate::error::{AppError, AppResult};

#[test]
fn test_app_error_display() {
    let config_err = AppError::ConfigurationError("Test config error".to_string());
    assert_eq!(
        config_err.to_string(),
        "Configuration error: Test config error"
    );

    let db_err = AppError::DatabaseError("Test database error".to_string());
    assert_eq!(db_err.to_string(), "Database error: Test database error");

    let infra_err = AppError::InfrastructureError("Test infra error".to_string());
    assert_eq!(
        infra_err.to_string(),
        "Infrastructure error: Test infra error"
    );

    let domain_err = AppError::DomainError("Test domain error".to_string());
    assert_eq!(domain_err.to_string(), "Domain error: Test domain error");

    let io_err = AppError::IOError("Test IO error".to_string());
    assert_eq!(io_err.to_string(), "IO error: Test IO error");
}

#[test]
fn test_app_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let app_err: AppError = io_err.into();
    assert!(matches!(app_err, AppError::IOError(_)));
    assert_eq!(app_err.to_string(), "IO error: file not found");
}
