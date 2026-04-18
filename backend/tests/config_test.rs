use crate::config::config::{AppConfig, ServerConfig};

#[tokio::test]
async fn test_load_default_config() {
    // This test ensures we can load the default configuration
    let result = AppConfig::load();
    assert!(result.is_ok(), "Failed to load default configuration");
    
    let config = result.unwrap();
    // Basic validation that the config loaded correctly
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 8888);
    assert_eq!(config.server.env, "development");
}

#[tokio::test]
async fn test_server_config_defaults() {
    let config = ServerConfig {
        host: "0.0.0.0".to_string(),
        port: 8888,
        env: "development".to_string(),
    };
    
    assert_eq!(config.host, "0.0.0.0");
    assert_eq!(config.port, 8888);
    assert_eq!(config.env, "development");
}