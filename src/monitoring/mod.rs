use crate::error::Result;
use crate::lifecycle::LifecycleManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Monitoring module implementation
#[derive(Clone)]
pub struct MonitoringModule {
    lifecycle_manager: Option<Arc<LifecycleManager>>,
}

impl MonitoringModule {
    /// Create a new monitoring module
    pub fn new() -> Self {
        Self {
            lifecycle_manager: None
        }
    }
    
    /// Create a new monitoring module with a specific lifecycle manager
    pub fn with_lifecycle(lifecycle: Arc<LifecycleManager>) -> Self {
        Self {
            lifecycle_manager: Some(lifecycle)
        }
    }
    
    /// Check if monitoring capabilities are available
    pub async fn check_available(&self) -> Result<bool> {
        // This is a placeholder
        Ok(true)
    }
}

/// Metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    /// Timestamp
    pub timestamp: String,
    /// Value
    pub value: f64,
}

/// Metric data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,
    /// Metric description
    pub description: Option<String>,
    /// Metric unit
    pub unit: Option<String>,
    /// Metric provider
    pub provider: String,
    /// Metric labels/tags
    pub labels: HashMap<String, String>,
    /// Metric data points
    pub points: Vec<MetricPoint>,
    /// Additional metric metadata
    pub metadata: Option<Value>,
}

/// Alert severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Critical severity
    Critical,
    /// High severity
    High,
    /// Medium severity
    Medium,
    /// Low severity
    Low,
    /// Info severity
    Info,
}

/// Alert status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertStatus {
    /// Alert is active
    Active,
    /// Alert is resolved
    Resolved,
    /// Alert is acknowledged
    Acknowledged,
    /// Alert is suppressed
    Suppressed,
}

/// Alert data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Alert name
    pub name: String,
    /// Alert description
    pub description: Option<String>,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert status
    pub status: AlertStatus,
    /// Alert provider
    pub provider: String,
    /// Alert labels/tags
    pub labels: HashMap<String, String>,
    /// Alert started at timestamp
    pub started_at: String,
    /// Alert ended at timestamp
    pub ended_at: Option<String>,
    /// Additional alert metadata
    pub metadata: Option<Value>,
} 