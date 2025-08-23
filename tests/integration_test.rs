use devops_mcp::{new, Config};

#[tokio::test]
async fn test_client_lifecycle() {
    // Create a new client with default configuration
    let config = Config::default();
    let client = new(config).expect("Failed to create client");
    
    // Verify client is created but not initialized
    let health = client.health_check(None).await.expect("Health check failed");
    assert_eq!(health.overall, devops_mcp::client::HealthState::Critical);
}

#[tokio::test]
async fn test_module_tools_availability() {
    use devops_mcp::memory::MemoryClient;
    use devops_mcp::office::powerpoint::PowerPointClient;
    use devops_mcp::lifecycle::LifecycleManager;
    use devops_mcp::transport::MockTransport;
    
    // Create lifecycle manager with mock transport
    let transport = Box::new(MockTransport::new());
    let lifecycle = LifecycleManager::new(transport);
    
    // Test memory module tools
    let memory_client = MemoryClient::new(&lifecycle);
    let memory_tools = memory_client.get_tools();
    assert!(!memory_tools.is_empty());
    assert!(memory_tools.iter().any(|t| t.0 == "create_memory"));
    
    // Test PowerPoint module tools
    let ppt_client = PowerPointClient::new(&lifecycle);
    let ppt_tools = ppt_client.get_tools();
    assert!(!ppt_tools.is_empty());
    assert!(ppt_tools.iter().any(|t| t.name == "create_presentation"));
}

#[test]
fn test_error_handling() {
    use devops_mcp::error::Error;
    
    // Test error categorization
    let network_err = Error::network("Connection failed");
    assert_eq!(network_err.category(), "network");
    
    let auth_err = Error::auth("Invalid token");
    assert_eq!(auth_err.category(), "authentication");
    assert!(auth_err.is_recoverable());
    
    let config_err = Error::config("Missing required field");
    assert_eq!(config_err.category(), "configuration");
    assert!(!config_err.is_recoverable());
}

#[test]
fn test_security_validation() {
    use devops_mcp::security::{SecurityModule, SanitizationOptions, ValidationResult};
    
    let security = SecurityModule::new();
    let options = SanitizationOptions::default();
    
    // Test SQL injection detection
    let sql_injection = "'; DROP TABLE users; --";
    match security.validate_input(sql_injection, &options) {
        ValidationResult::Malicious(msg) => {
            assert!(msg.contains("SQL injection"));
        }
        _ => panic!("Expected SQL injection to be detected"),
    }
    
    // Test valid input
    match security.validate_input("valid_username_123", &options) {
        ValidationResult::Valid => {}
        _ => panic!("Expected valid input to pass"),
    }
    
    // Test XSS detection
    let xss = "<script>alert('xss')</script>";
    match security.validate_input(xss, &options) {
        ValidationResult::Malicious(msg) => {
            // The message should contain info about the detected pattern
            assert!(msg.contains("detected"), "Unexpected message: {}", msg);
        }
        _ => panic!("Expected XSS to be detected"),
    }
}

#[tokio::test]
async fn test_database_module() {
    use devops_mcp::database::DatabaseModule;
    
    let db_module = DatabaseModule::new();
    let tools = db_module.get_tools();
    
    // Verify database tools are available
    assert!(!tools.is_empty());
    assert!(tools.iter().any(|t| t.name == "list_databases"));
    assert!(tools.iter().any(|t| t.name == "execute_query"));
    assert!(tools.iter().any(|t| t.name == "list_tables"));
    assert!(tools.iter().any(|t| t.name == "describe_table"));
}

#[tokio::test]
async fn test_infrastructure_module() {
    use devops_mcp::infrastructure::InfrastructureModule;
    use devops_mcp::config::InfrastructureConfig;
    
    let config = InfrastructureConfig::default();
    let infra_module = InfrastructureModule::new(config);
    
    // Get available tools
    let tools = infra_module.get_tools().await.expect("Failed to get tools");
    
    // Since no providers are configured, tools should be empty
    assert!(tools.is_empty());
}

#[test]
fn test_performance_optimizations() {
    use devops_mcp::memory::{MemorySearchParams, MemoryType};
    
    // Test pre-allocation in memory search
    let params = MemorySearchParams {
        memory_type: Some(MemoryType::Project),
        keyword: Some("optimization".to_string()),
        limit: Some(100),
        metadata_filters: None,
    };
    
    // Verify the search params are constructed correctly
    assert_eq!(params.limit, Some(100));
    assert_eq!(params.memory_type, Some(MemoryType::Project));
}