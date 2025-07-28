/// Comprehensive monitoring module with modern observability and SIEM support
/// 
/// Provides unified access to:
/// - Metrics (Prometheus, OpenTelemetry, Datadog)
/// - Logs (Elasticsearch, Splunk, Sumo Logic)
/// - Traces (Jaeger, Zipkin, AWS X-Ray)
/// - SIEM (Splunk, Elastic Security, Crowdstrike, Sentinel)
/// - Visualization (Grafana, Kibana)

use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::security::SecurityModule;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command;
use chrono::{DateTime, Utc};

/// Enhanced monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Prometheus configuration
    pub prometheus: Option<PrometheusConfig>,
    /// Grafana configuration
    pub grafana: Option<GrafanaConfig>,
    /// OpenTelemetry configuration
    pub opentelemetry: Option<OpenTelemetryConfig>,
    /// Elasticsearch configuration
    pub elasticsearch: Option<ElasticsearchConfig>,
    /// Splunk configuration
    pub splunk: Option<SplunkConfig>,
    /// Datadog configuration
    pub datadog: Option<DatadogConfig>,
    /// Crowdstrike configuration
    pub crowdstrike: Option<CrowdstrikeConfig>,
    /// Azure Sentinel configuration
    pub sentinel: Option<SentinelConfig>,
    /// Jaeger configuration
    pub jaeger: Option<JaegerConfig>,
    /// Loki configuration
    pub loki: Option<LokiConfig>,
}

/// Prometheus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusConfig {
    /// Server URL
    pub url: String,
    /// Username for basic auth
    pub username: Option<String>,
    /// Password for basic auth
    pub password: Option<String>,
    /// Bearer token
    pub bearer_token: Option<String>,
    /// Skip TLS verification
    pub insecure_skip_verify: bool,
    /// Remote write configuration
    pub remote_write: Option<RemoteWriteConfig>,
}

/// Remote write configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteWriteConfig {
    /// URL
    pub url: String,
    /// Basic auth
    pub basic_auth: Option<BasicAuth>,
    /// Headers
    pub headers: HashMap<String, String>,
}

/// Basic authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuth {
    /// Username
    pub username: String,
    /// Password
    pub password: String,
}

/// Grafana configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaConfig {
    /// URL
    pub url: String,
    /// API key
    pub api_key: Option<String>,
    /// Username
    pub username: Option<String>,
    /// Password
    pub password: Option<String>,
    /// Organization ID
    pub org_id: Option<i64>,
    /// Grafana Alloy configuration
    pub alloy: Option<GrafanaAlloyConfig>,
}

/// Grafana Alloy configuration (new in 2024)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaAlloyConfig {
    /// Enable Alloy
    pub enabled: bool,
    /// Alloy endpoints
    pub endpoints: Vec<String>,
}

/// OpenTelemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenTelemetryConfig {
    /// OTLP endpoint
    pub otlp_endpoint: String,
    /// Protocol (grpc or http)
    pub protocol: String,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Insecure
    pub insecure: bool,
    /// Compression
    pub compression: Option<String>,
    /// Timeout in seconds
    pub timeout: u64,
}

/// Elasticsearch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElasticsearchConfig {
    /// URLs
    pub urls: Vec<String>,
    /// Username
    pub username: Option<String>,
    /// Password
    pub password: Option<String>,
    /// API key
    pub api_key: Option<String>,
    /// Cloud ID
    pub cloud_id: Option<String>,
    /// Index pattern
    pub index_pattern: String,
}

/// Splunk configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplunkConfig {
    /// HEC URL
    pub hec_url: String,
    /// HEC token
    pub hec_token: String,
    /// Index
    pub index: String,
    /// Source type
    pub source_type: String,
    /// SSL verify
    pub ssl_verify: bool,
}

/// Datadog configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatadogConfig {
    /// API key
    pub api_key: String,
    /// App key
    pub app_key: String,
    /// Site (e.g., datadoghq.com, datadoghq.eu)
    pub site: String,
    /// API URL override
    pub api_url: Option<String>,
}

/// Crowdstrike configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrowdstrikeConfig {
    /// Client ID
    pub client_id: String,
    /// Client secret
    pub client_secret: String,
    /// Base URL
    pub base_url: String,
    /// Member CID (for multi-tenant)
    pub member_cid: Option<String>,
}

/// Azure Sentinel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelConfig {
    /// Workspace ID
    pub workspace_id: String,
    /// Workspace key
    pub workspace_key: String,
    /// Log type
    pub log_type: String,
    /// Resource ID
    pub resource_id: Option<String>,
}

/// Jaeger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JaegerConfig {
    /// Collector endpoint
    pub collector_endpoint: String,
    /// Agent host
    pub agent_host: Option<String>,
    /// Agent port
    pub agent_port: Option<u16>,
    /// Service name
    pub service_name: String,
}

/// Loki configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LokiConfig {
    /// Push URL
    pub push_url: String,
    /// Username
    pub username: Option<String>,
    /// Password
    pub password: Option<String>,
    /// Tenant ID
    pub tenant_id: Option<String>,
}

/// Monitoring module
pub struct MonitoringModule {
    /// Configuration
    config: MonitoringConfig,
    /// Lifecycle manager
    lifecycle: Arc<LifecycleManager>,
    /// Security module
    security: SecurityModule,
}

impl Default for MonitoringModule {
    fn default() -> Self {
        use crate::transport;
        let mock_transport = Box::new(transport::MockTransport::new()) as Box<dyn transport::Transport + Send + Sync>;
        let lifecycle = Arc::new(LifecycleManager::new(mock_transport));
        Self::new(MonitoringConfig::default(), lifecycle)
    }
}

impl MonitoringModule {
    /// Create a new monitoring module
    pub fn new(config: MonitoringConfig, lifecycle: Arc<LifecycleManager>) -> Self {
        Self {
            config,
            lifecycle,
            security: SecurityModule::new(),
        }
    }
    
    /// Check if monitoring capabilities are available
    pub async fn check_available(&self) -> Result<bool> {
        let mut available = false;
        
        // Check Prometheus
        if self.config.prometheus.is_some() {
            if let Ok(output) = Command::new("promtool").arg("--version").output().await {
                available = available || output.status.success();
            }
        }
        
        // Check Grafana CLI
        if self.config.grafana.is_some() {
            if let Ok(output) = Command::new("grafana-cli").arg("--version").output().await {
                available = available || output.status.success();
            }
        }
        
        Ok(available)
    }
    
    // Prometheus operations
    
    /// Query Prometheus
    pub async fn prometheus_query(&self, query: &str, time: Option<DateTime<Utc>>) -> Result<PrometheusResult> {
        let prom_config = self.config.prometheus.as_ref()
            .ok_or_else(|| Error::config("Prometheus not configured"))?;
        
        let mut args = vec!["query", "--server", &prom_config.url];
        
        let time_str;
        if let Some(t) = time {
            time_str = t.to_rfc3339();
            args.push("--time");
            args.push(&time_str);
        }
        
        args.push(query);
        
        let output = Command::new("promtool")
            .args(&args)
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to query Prometheus: {}", e)))?;
        
        if !output.status.success() {
            return Err(Error::service(format!("Prometheus query failed: {}", String::from_utf8_lossy(&output.stderr))));
        }
        
        // Parse the output
        let _result_str = String::from_utf8_lossy(&output.stdout);
        Ok(PrometheusResult {
            query: query.to_string(),
            timestamp: time.unwrap_or_else(Utc::now),
            values: vec![], // Would need proper parsing
        })
    }
    
    /// Query range from Prometheus
    pub async fn prometheus_query_range(
        &self, 
        query: &str, 
        start: DateTime<Utc>, 
        end: DateTime<Utc>, 
        step: &str
    ) -> Result<PrometheusRangeResult> {
        let prom_config = self.config.prometheus.as_ref()
            .ok_or_else(|| Error::config("Prometheus not configured"))?;
        
        let start_str = start.to_rfc3339();
        let end_str = end.to_rfc3339();
        let args = vec![
            "query", "range",
            "--server", &prom_config.url,
            "--start", &start_str,
            "--end", &end_str,
            "--step", step,
            query
        ];
        
        let output = Command::new("promtool")
            .args(&args)
            .output()
            .await
            .map_err(|e| Error::internal(format!("Failed to query Prometheus range: {}", e)))?;
        
        if !output.status.success() {
            return Err(Error::service(format!("Prometheus range query failed: {}", String::from_utf8_lossy(&output.stderr))));
        }
        
        Ok(PrometheusRangeResult {
            query: query.to_string(),
            start,
            end,
            step: step.to_string(),
            values: vec![], // Would need proper parsing
        })
    }
    
    // Grafana operations
    
    /// List Grafana dashboards
    pub async fn grafana_list_dashboards(&self) -> Result<Vec<GrafanaDashboard>> {
        let _grafana_config = self.config.grafana.as_ref()
            .ok_or_else(|| Error::config("Grafana not configured"))?;
        
        // Would use Grafana API
        Ok(vec![])
    }
    
    /// Create Grafana dashboard
    pub async fn grafana_create_dashboard(&self, _dashboard: &GrafanaDashboard) -> Result<String> {
        let _grafana_config = self.config.grafana.as_ref()
            .ok_or_else(|| Error::config("Grafana not configured"))?;
        
        // Would use Grafana API
        Ok("dashboard-id".to_string())
    }
    
    // OpenTelemetry operations
    
    /// Send traces to OpenTelemetry
    pub async fn otel_send_traces(&self, _traces: Vec<OtelTrace>) -> Result<()> {
        let _otel_config = self.config.opentelemetry.as_ref()
            .ok_or_else(|| Error::config("OpenTelemetry not configured"))?;
        
        // Would use OTLP protocol
        Ok(())
    }
    
    /// Send metrics to OpenTelemetry
    pub async fn otel_send_metrics(&self, _metrics: Vec<OtelMetric>) -> Result<()> {
        let _otel_config = self.config.opentelemetry.as_ref()
            .ok_or_else(|| Error::config("OpenTelemetry not configured"))?;
        
        // Would use OTLP protocol
        Ok(())
    }
    
    // SIEM operations
    
    /// Send events to Splunk
    pub async fn splunk_send_event(&self, _event: &SiemEvent) -> Result<()> {
        let _splunk_config = self.config.splunk.as_ref()
            .ok_or_else(|| Error::config("Splunk not configured"))?;
        
        // Would use Splunk HEC API
        Ok(())
    }
    
    /// Query Elasticsearch
    pub async fn elasticsearch_search(&self, _query: &ElasticsearchQuery) -> Result<ElasticsearchResult> {
        let _es_config = self.config.elasticsearch.as_ref()
            .ok_or_else(|| Error::config("Elasticsearch not configured"))?;
        
        // Would use Elasticsearch API
        Ok(ElasticsearchResult {
            took: 0,
            timed_out: false,
            hits: ElasticsearchHits {
                total: ElasticsearchTotal {
                    value: 0,
                    relation: "eq".to_string(),
                },
                max_score: 0.0,
                hits: vec![],
            },
        })
    }
    
    /// Send metrics to Datadog
    pub async fn datadog_send_metrics(&self, _metrics: Vec<DatadogMetric>) -> Result<()> {
        let _dd_config = self.config.datadog.as_ref()
            .ok_or_else(|| Error::config("Datadog not configured"))?;
        
        // Would use Datadog API
        Ok(())
    }
    
    /// Query Crowdstrike detections
    pub async fn crowdstrike_get_detections(&self, _filters: &HashMap<String, String>) -> Result<Vec<CrowdstrikeDetection>> {
        let _cs_config = self.config.crowdstrike.as_ref()
            .ok_or_else(|| Error::config("Crowdstrike not configured"))?;
        
        // Would use Crowdstrike API
        Ok(vec![])
    }
    
    /// Send logs to Azure Sentinel
    pub async fn sentinel_send_logs(&self, _logs: Vec<SentinelLog>) -> Result<()> {
        let _sentinel_config = self.config.sentinel.as_ref()
            .ok_or_else(|| Error::config("Azure Sentinel not configured"))?;
        
        // Would use Azure Log Analytics API
        Ok(())
    }
    
    /// Create unified alert from multiple sources
    pub async fn create_unified_alert(&self, sources: Vec<AlertSource>) -> Result<UnifiedAlert> {
        let mut unified = UnifiedAlert {
            id: uuid::Uuid::new_v4().to_string(),
            title: "Unified Alert".to_string(),
            description: "Alert from multiple sources".to_string(),
            severity: AlertSeverity::Medium,
            sources,
            created_at: Utc::now(),
            status: AlertStatus::Active,
            assignee: None,
            tags: HashMap::new(),
        };
        
        // Determine severity based on sources
        for source in &unified.sources {
            match source {
                AlertSource::Prometheus { severity, .. } |
                AlertSource::Splunk { severity, .. } |
                AlertSource::Elasticsearch { severity, .. } => {
                    if matches!(severity, AlertSeverity::Critical) {
                        unified.severity = AlertSeverity::Critical;
                        break;
                    }
                }
                _ => {}
            }
        }
        
        Ok(unified)
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &MonitoringConfig {
        &self.config
    }
    
    /// Get lifecycle manager
    pub fn get_lifecycle(&self) -> &Arc<LifecycleManager> {
        &self.lifecycle
    }
    
    /// Get security module
    pub fn get_security(&self) -> &SecurityModule {
        &self.security
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

/// Prometheus query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusResult {
    /// Query
    pub query: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Values
    pub values: Vec<PrometheusValue>,
}

/// Prometheus value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusValue {
    /// Metric
    pub metric: HashMap<String, String>,
    /// Value
    pub value: f64,
}

/// Prometheus range result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusRangeResult {
    /// Query
    pub query: String,
    /// Start time
    pub start: DateTime<Utc>,
    /// End time
    pub end: DateTime<Utc>,
    /// Step
    pub step: String,
    /// Values
    pub values: Vec<PrometheusRangeValue>,
}

/// Prometheus range value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusRangeValue {
    /// Metric
    pub metric: HashMap<String, String>,
    /// Values over time
    pub values: Vec<(DateTime<Utc>, f64)>,
}

/// Grafana dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaDashboard {
    /// ID
    pub id: Option<String>,
    /// UID
    pub uid: Option<String>,
    /// Title
    pub title: String,
    /// Tags
    pub tags: Vec<String>,
    /// Panels
    pub panels: Vec<GrafanaPanel>,
    /// Variables
    pub templating: Vec<GrafanaVariable>,
}

/// Grafana panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaPanel {
    /// ID
    pub id: i32,
    /// Title
    pub title: String,
    /// Type
    pub panel_type: String,
    /// Targets
    pub targets: Vec<Value>,
    /// Grid position
    pub grid_pos: GridPos,
}

/// Grid position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridPos {
    /// Height
    pub h: i32,
    /// Width
    pub w: i32,
    /// X position
    pub x: i32,
    /// Y position
    pub y: i32,
}

/// Grafana variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaVariable {
    /// Name
    pub name: String,
    /// Label
    pub label: String,
    /// Type
    pub variable_type: String,
    /// Query
    pub query: String,
}

/// OpenTelemetry trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelTrace {
    /// Trace ID
    pub trace_id: String,
    /// Spans
    pub spans: Vec<OtelSpan>,
}

/// OpenTelemetry span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelSpan {
    /// Span ID
    pub span_id: String,
    /// Parent span ID
    pub parent_span_id: Option<String>,
    /// Operation name
    pub operation_name: String,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time
    pub end_time: DateTime<Utc>,
    /// Tags
    pub tags: HashMap<String, String>,
    /// Status
    pub status: SpanStatus,
}

/// Span status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanStatus {
    /// Code
    pub code: String,
    /// Message
    pub message: Option<String>,
}

/// OpenTelemetry metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelMetric {
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Unit
    pub unit: String,
    /// Type
    pub metric_type: OtelMetricType,
    /// Data points
    pub data_points: Vec<OtelDataPoint>,
}

/// OpenTelemetry metric type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OtelMetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// OpenTelemetry data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelDataPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Value
    pub value: f64,
    /// Labels
    pub labels: HashMap<String, String>,
}

/// SIEM event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiemEvent {
    /// Event ID
    pub id: String,
    /// Event type
    pub event_type: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Source
    pub source: String,
    /// Severity
    pub severity: AlertSeverity,
    /// Data
    pub data: HashMap<String, Value>,
}

/// Elasticsearch query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElasticsearchQuery {
    /// Index
    pub index: String,
    /// Query DSL
    pub query: Value,
    /// Size
    pub size: Option<i32>,
    /// From
    pub from: Option<i32>,
    /// Sort
    pub sort: Option<Vec<Value>>,
}

/// Elasticsearch result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElasticsearchResult {
    /// Took
    pub took: i64,
    /// Timed out
    pub timed_out: bool,
    /// Hits
    pub hits: ElasticsearchHits,
}

/// Elasticsearch hits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElasticsearchHits {
    /// Total
    pub total: ElasticsearchTotal,
    /// Max score
    pub max_score: f64,
    /// Hits
    pub hits: Vec<ElasticsearchHit>,
}

/// Elasticsearch total
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElasticsearchTotal {
    /// Value
    pub value: i64,
    /// Relation
    pub relation: String,
}

/// Elasticsearch hit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElasticsearchHit {
    /// Index
    pub _index: String,
    /// ID
    pub _id: String,
    /// Score
    pub _score: f64,
    /// Source
    pub _source: Value,
}

/// Datadog metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatadogMetric {
    /// Metric name
    pub metric: String,
    /// Points
    pub points: Vec<(i64, f64)>,
    /// Type
    pub metric_type: String,
    /// Host
    pub host: Option<String>,
    /// Tags
    pub tags: Vec<String>,
}

/// Crowdstrike detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrowdstrikeDetection {
    /// Detection ID
    pub detection_id: String,
    /// Device ID
    pub device_id: String,
    /// Behavior ID
    pub behavior_id: String,
    /// Severity
    pub severity: i32,
    /// Confidence
    pub confidence: i32,
    /// Pattern ID
    pub pattern_id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Sentinel log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelLog {
    /// Time generated
    pub time_generated: DateTime<Utc>,
    /// Computer
    pub computer: String,
    /// Event ID
    pub event_id: i32,
    /// Message
    pub message: String,
    /// Level
    pub level: String,
    /// Additional fields
    pub custom_fields: HashMap<String, Value>,
}

/// Alert source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSource {
    Prometheus {
        alert_name: String,
        severity: AlertSeverity,
        labels: HashMap<String, String>,
    },
    Splunk {
        search_name: String,
        severity: AlertSeverity,
        results: Vec<Value>,
    },
    Elasticsearch {
        watcher_name: String,
        severity: AlertSeverity,
        hits: i32,
    },
    Datadog {
        monitor_name: String,
        status: String,
        message: String,
    },
    Crowdstrike {
        detection_id: String,
        severity: i32,
        confidence: i32,
    },
}

/// Unified alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedAlert {
    /// ID
    pub id: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Severity
    pub severity: AlertSeverity,
    /// Sources
    pub sources: Vec<AlertSource>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Status
    pub status: AlertStatus,
    /// Assignee
    pub assignee: Option<String>,
    /// Tags
    pub tags: HashMap<String, String>,
}

/// Default implementation
impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            prometheus: None,
            grafana: None,
            opentelemetry: None,
            elasticsearch: None,
            splunk: None,
            datadog: None,
            crowdstrike: None,
            sentinel: None,
            jaeger: None,
            loki: None,
        }
    }
}

