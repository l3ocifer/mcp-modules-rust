use super::*;
use crate::transport::mock::MockTransport;
use std::sync::Arc;

#[tokio::test]
async fn test_home_assistant_client_lifecycle_handling() {
    // Create a mock transport
    let mock_transport = Arc::new(MockTransport::new());
    
    // Create a lifecycle manager
    let lifecycle_manager = LifecycleManager::new(mock_transport);
    
    // Create a HomeAssistantClient
    let ha_client = HomeAssistantClient::new(&lifecycle_manager);
    
    // Get the light service
    let light_service = ha_client.light_service();
    
    // Create a climate service
    let climate_service = ha_client.climate_service();
    
    // Create lock service
    let lock_service = ha_client.lock_service();
    
    // Create alarm service
    let alarm_service = ha_client.alarm_control_panel_service();
    
    // Create humidifier service
    let humidifier_service = ha_client.humidifier_service();
    
    // Test passes if we can create these services without lifetime errors
    assert!(true);
} 