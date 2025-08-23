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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use reqwest::{Client, header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE}};
use base64::Engine;
use std::time::Duration;

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

/// Monitoring module with direct API integrations
pub struct MonitoringModule {
    /// Configuration
    config: MonitoringConfig,
    /// Lifecycle manager
    lifecycle: Arc<LifecycleManager>,
    /// Security module
    security: SecurityModule,
    /// HTTP client for API calls
    http_client: Client,
}

impl Default for MonitoringModule {
    fn default() -> Self {
        use crate::transport;
        let mock_transport = Box::new(transport::MockTransport::new())
            as Box<dyn transport::Transport + Send + Sync>;
        let lifecycle = Arc::new(LifecycleManager::new(mock_transport));
        Self::new(MonitoringConfig::default(), lifecycle)
    }
}

impl MonitoringModule {
    /// Create a new monitoring module
    pub fn new(config: MonitoringConfig, lifecycle: Arc<LifecycleManager>) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            config,
            lifecycle,
            security: SecurityModule::new(),
            http_client,
        }
    }

    /// Check if monitoring services are available via API
    pub async fn check_available(&self) -> Result<bool> {
        let mut available = false;

        // Check Prometheus API
        if let Some(prom_config) = &self.config.prometheus {
            available = available || self.prometheus_health_check(prom_config).await.unwrap_or(false);
        }

        // Check Grafana API
        if let Some(grafana_config) = &self.config.grafana {
            available = available || self.grafana_health_check(grafana_config).await.unwrap_or(false);
        }

        // Check Elasticsearch API
        if let Some(es_config) = &self.config.elasticsearch {
            available = available || self.elasticsearch_health_check(es_config).await.unwrap_or(false);
        }

        // Check Datadog API
        if let Some(dd_config) = &self.config.datadog {
            available = available || self.datadog_health_check(dd_config).await.unwrap_or(false);
        }

        Ok(available)
    }

    // Prometheus operations

    /// Query Prometheus via API
    pub async fn prometheus_query(
        &self,
        query: &str,
        time: Option<DateTime<Utc>>,
    ) -> Result<PrometheusResult> {
        let prom_config = self
            .config
            .prometheus
            .as_ref()
            .ok_or_else(|| Error::config("Prometheus not configured"))?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));

        // Add authentication
        if let Some(token) = &prom_config.bearer_token {
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|e| Error::config(&format!("Invalid bearer token: {}", e)))?);
        } else if let (Some(username), Some(password)) = (&prom_config.username, &prom_config.password) {
            let credentials = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", username, password));
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Basic {}", credentials))
                .map_err(|e| Error::config(&format!("Invalid credentials: {}", e)))?);
        }

        let url = format!("{}/api/v1/query", prom_config.url);
        let mut params = vec![("query", query.to_string())];
        
        if let Some(t) = time {
            params.push(("time", t.timestamp().to_string()));
        }

        let response = self.http_client
            .post(&url)
            .headers(headers)
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::service(&format!("Failed to query Prometheus: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::service(&format!("Prometheus query failed: {}", error_text)));
        }

        let response_data: serde_json::Value = response.json().await
            .map_err(|e| Error::service(&format!("Failed to parse Prometheus response: {}", e)))?;

        // Parse Prometheus response format
        let values = if let Some(data) = response_data.get("data").and_then(|d| d.get("result")).and_then(|r| r.as_array()) {
            data.iter().filter_map(|item| {
                let metric = item.get("metric")?.as_object()?
                    .iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect();
                let value = item.get("value")?.as_array()?.get(1)?.as_str()?.parse().ok()?;
                Some(PrometheusValue { metric, value })
            }).collect()
        } else {
            vec![]
        };

        Ok(PrometheusResult {
            query: query.to_string(),
            timestamp: time.unwrap_or_else(Utc::now),
            values,
        })
    }

    /// Query range from Prometheus via API
    pub async fn prometheus_query_range(
        &self,
        query: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        step: &str,
    ) -> Result<PrometheusRangeResult> {
        let prom_config = self
            .config
            .prometheus
            .as_ref()
            .ok_or_else(|| Error::config("Prometheus not configured"))?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));

        // Add authentication
        if let Some(token) = &prom_config.bearer_token {
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|e| Error::config(&format!("Invalid bearer token: {}", e)))?);
        } else if let (Some(username), Some(password)) = (&prom_config.username, &prom_config.password) {
            let credentials = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", username, password));
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Basic {}", credentials))
                .map_err(|e| Error::config(&format!("Invalid credentials: {}", e)))?);
        }

        let url = format!("{}/api/v1/query_range", prom_config.url);
        let params = vec![
            ("query", query.to_string()),
            ("start", start.timestamp().to_string()),
            ("end", end.timestamp().to_string()),
            ("step", step.to_string()),
        ];

        let response = self.http_client
            .post(&url)
            .headers(headers)
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::service(&format!("Failed to query Prometheus range: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::service(&format!("Prometheus range query failed: {}", error_text)));
        }

        let response_data: serde_json::Value = response.json().await
            .map_err(|e| Error::service(&format!("Failed to parse Prometheus response: {}", e)))?;

        // Parse Prometheus range response format
        let values = if let Some(data) = response_data.get("data").and_then(|d| d.get("result")).and_then(|r| r.as_array()) {
            data.iter().filter_map(|item| {
                let metric = item.get("metric")?.as_object()?
                    .iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect();
                
                let values = item.get("values")?.as_array()?
                    .iter()
                    .filter_map(|val| {
                        let arr = val.as_array()?;
                        let timestamp = arr.get(0)?.as_f64()? as i64;
                        let value = arr.get(1)?.as_str()?.parse().ok()?;
                        Some((DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now), value))
                    })
                    .collect();
                
                Some(PrometheusRangeValue { metric, values })
            }).collect()
        } else {
            vec![]
        };

        Ok(PrometheusRangeResult {
            query: query.to_string(),
            start,
            end,
            step: step.to_string(),
            values,
        })
    }

    // Grafana operations

    /// List Grafana dashboards via API
    pub async fn grafana_list_dashboards(&self) -> Result<Vec<GrafanaDashboard>> {
        let grafana_config = self
            .config
            .grafana
            .as_ref()
            .ok_or_else(|| Error::config("Grafana not configured"))?;

        let mut headers = HeaderMap::new();
        
        // Add authentication
        if let Some(api_key) = &grafana_config.api_key {
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| Error::config(&format!("Invalid API key: {}", e)))?);
        } else if let (Some(username), Some(password)) = (&grafana_config.username, &grafana_config.password) {
            let credentials = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", username, password));
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Basic {}", credentials))
                .map_err(|e| Error::config(&format!("Invalid credentials: {}", e)))?);
        }

        let url = format!("{}/api/search?type=dash-db", grafana_config.url);
        
        let response = self.http_client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| Error::service(&format!("Failed to list Grafana dashboards: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::service(&format!("Grafana dashboard listing failed: {}", error_text)));
        }

        let dashboards_data: serde_json::Value = response.json().await
            .map_err(|e| Error::service(&format!("Failed to parse Grafana response: {}", e)))?;

        let dashboards = if let Some(arr) = dashboards_data.as_array() {
            arr.iter().filter_map(|item| {
                Some(GrafanaDashboard {
                    id: item.get("id")?.as_i64().map(|i| i.to_string()),
                    uid: item.get("uid")?.as_str().map(|s| s.to_string()),
                    title: item.get("title")?.as_str()?.to_string(),
                    tags: item.get("tags")?.as_array()?.iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_string()))
                        .collect(),
                    panels: vec![], // Would need separate API call to get full dashboard
                    templating: vec![],
                })
            }).collect()
        } else {
            vec![]
        };

        Ok(dashboards)
    }

    /// Create Grafana dashboard via API
    pub async fn grafana_create_dashboard(&self, dashboard: &GrafanaDashboard) -> Result<String> {
        let grafana_config = self
            .config
            .grafana
            .as_ref()
            .ok_or_else(|| Error::config("Grafana not configured"))?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
        // Add authentication
        if let Some(api_key) = &grafana_config.api_key {
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| Error::config(&format!("Invalid API key: {}", e)))?);
        } else if let (Some(username), Some(password)) = (&grafana_config.username, &grafana_config.password) {
            let credentials = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", username, password));
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Basic {}", credentials))
                .map_err(|e| Error::config(&format!("Invalid credentials: {}", e)))?);
        }

        let url = format!("{}/api/dashboards/db", grafana_config.url);
        
        let dashboard_json = serde_json::json!({
            "dashboard": {
                "id": dashboard.id,
                "uid": dashboard.uid,
                "title": dashboard.title,
                "tags": dashboard.tags,
                "panels": dashboard.panels,
                "templating": { "list": dashboard.templating }
            },
            "overwrite": true
        });

        let response = self.http_client
            .post(&url)
            .headers(headers)
            .json(&dashboard_json)
            .send()
            .await
            .map_err(|e| Error::service(&format!("Failed to create Grafana dashboard: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::service(&format!("Grafana dashboard creation failed: {}", error_text)));
        }

        let response_data: serde_json::Value = response.json().await
            .map_err(|e| Error::service(&format!("Failed to parse Grafana response: {}", e)))?;

        let dashboard_id = response_data.get("id")
            .and_then(|id| id.as_i64())
            .map(|id| id.to_string())
            .or_else(|| response_data.get("uid").and_then(|uid| uid.as_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        Ok(dashboard_id)
    }

    // OpenTelemetry operations

    /// Send traces to OpenTelemetry via OTLP
    pub async fn otel_send_traces(&self, traces: Vec<OtelTrace>) -> Result<()> {
        let otel_config = self
            .config
            .opentelemetry
            .as_ref()
            .ok_or_else(|| Error::config("OpenTelemetry not configured"))?;

        let mut headers = HeaderMap::new();
        
        // Set content type based on protocol
        if otel_config.protocol == "grpc" {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-protobuf"));
        } else {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        }

        // Add custom headers
        for (key, value) in &otel_config.headers {
            if let (Ok(header_name), Ok(header_value)) = (key.parse::<HeaderName>(), value.parse::<HeaderValue>()) {
                headers.insert(header_name, header_value);
            }
        }

        let endpoint = if otel_config.protocol == "grpc" {
            format!("{}/v1/traces", otel_config.otlp_endpoint)
        } else {
            format!("{}/v1/traces", otel_config.otlp_endpoint)
        };

        // Convert traces to OTLP format (simplified JSON representation)
        let otlp_data = serde_json::json!({
            "resource_spans": traces.iter().map(|trace| {
                serde_json::json!({
                    "resource": {
                        "attributes": []
                    },
                    "scope_spans": [{
                        "scope": {
                            "name": "mcp-monitoring",
                            "version": "1.0.0"
                        },
                        "spans": trace.spans.iter().map(|span| {
                            serde_json::json!({
                                "trace_id": trace.trace_id,
                                "span_id": span.span_id,
                                "parent_span_id": span.parent_span_id,
                                "name": span.operation_name,
                                "start_time_unix_nano": span.start_time.timestamp_nanos_opt().unwrap_or(0),
                                "end_time_unix_nano": span.end_time.timestamp_nanos_opt().unwrap_or(0),
                                "attributes": span.tags.iter().map(|(k, v)| {
                                    serde_json::json!({
                                        "key": k,
                                        "value": { "string_value": v }
                                    })
                                }).collect::<Vec<_>>(),
                                "status": {
                                    "code": span.status.code,
                                    "message": span.status.message
                                }
                            })
                        }).collect::<Vec<_>>()
                    }]
                })
            }).collect::<Vec<_>>()
        });

        let response = self.http_client
            .post(&endpoint)
            .headers(headers)
            .json(&otlp_data)
            .timeout(Duration::from_secs(otel_config.timeout))
            .send()
            .await
            .map_err(|e| Error::service(&format!("Failed to send traces to OpenTelemetry: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::service(&format!("OpenTelemetry trace sending failed: {}", error_text)));
        }

        Ok(())
    }

    /// Send metrics to OpenTelemetry via OTLP
    pub async fn otel_send_metrics(&self, metrics: Vec<OtelMetric>) -> Result<()> {
        let otel_config = self
            .config
            .opentelemetry
            .as_ref()
            .ok_or_else(|| Error::config("OpenTelemetry not configured"))?;

        let mut headers = HeaderMap::new();
        
        // Set content type based on protocol
        if otel_config.protocol == "grpc" {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-protobuf"));
        } else {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        }

        // Add custom headers
        for (key, value) in &otel_config.headers {
            if let (Ok(header_name), Ok(header_value)) = (key.parse::<HeaderName>(), value.parse::<HeaderValue>()) {
                headers.insert(header_name, header_value);
            }
        }

        let endpoint = if otel_config.protocol == "grpc" {
            format!("{}/v1/metrics", otel_config.otlp_endpoint)
        } else {
            format!("{}/v1/metrics", otel_config.otlp_endpoint)
        };

        // Convert metrics to OTLP format (simplified JSON representation)
        let otlp_data = serde_json::json!({
            "resource_metrics": [{
                "resource": {
                    "attributes": []
                },
                "scope_metrics": [{
                    "scope": {
                        "name": "mcp-monitoring",
                        "version": "1.0.0"
                    },
                    "metrics": metrics.iter().map(|metric| {
                        let metric_data = match metric.metric_type {
                            OtelMetricType::Counter => serde_json::json!({
                                "sum": {
                                    "data_points": metric.data_points.iter().map(|dp| {
                                        serde_json::json!({
                                            "attributes": dp.labels.iter().map(|(k, v)| {
                                                serde_json::json!({
                                                    "key": k,
                                                    "value": { "string_value": v }
                                                })
                                            }).collect::<Vec<_>>(),
                                            "start_time_unix_nano": dp.timestamp.timestamp_nanos_opt().unwrap_or(0),
                                            "time_unix_nano": dp.timestamp.timestamp_nanos_opt().unwrap_or(0),
                                            "as_double": dp.value
                                        })
                                    }).collect::<Vec<_>>(),
                                    "aggregation_temporality": 2,
                                    "is_monotonic": true
                                }
                            }),
                            OtelMetricType::Gauge => serde_json::json!({
                                "gauge": {
                                    "data_points": metric.data_points.iter().map(|dp| {
                                        serde_json::json!({
                                            "attributes": dp.labels.iter().map(|(k, v)| {
                                                serde_json::json!({
                                                    "key": k,
                                                    "value": { "string_value": v }
                                                })
                                            }).collect::<Vec<_>>(),
                                            "time_unix_nano": dp.timestamp.timestamp_nanos_opt().unwrap_or(0),
                                            "as_double": dp.value
                                        })
                                    }).collect::<Vec<_>>()
                                }
                            }),
                            _ => serde_json::json!({
                                "gauge": {
                                    "data_points": metric.data_points.iter().map(|dp| {
                                        serde_json::json!({
                                            "attributes": [],
                                            "time_unix_nano": dp.timestamp.timestamp_nanos_opt().unwrap_or(0),
                                            "as_double": dp.value
                                        })
                                    }).collect::<Vec<_>>()
                                }
                            })
                        };

                        let mut result = serde_json::json!({
                            "name": metric.name,
                            "description": metric.description,
                            "unit": metric.unit
                        });
                        
                        if let serde_json::Value::Object(ref mut obj) = result {
                            if let serde_json::Value::Object(data_obj) = metric_data {
                                obj.extend(data_obj);
                            }
                        }
                        
                        result
                    }).collect::<Vec<_>>()
                }]
            }]
        });

        let response = self.http_client
            .post(&endpoint)
            .headers(headers)
            .json(&otlp_data)
            .timeout(Duration::from_secs(otel_config.timeout))
            .send()
            .await
            .map_err(|e| Error::service(&format!("Failed to send metrics to OpenTelemetry: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::service(&format!("OpenTelemetry metrics sending failed: {}", error_text)));
        }

        Ok(())
    }

    // SIEM operations

    /// Send events to Splunk via HEC API
    pub async fn splunk_send_event(&self, event: &SiemEvent) -> Result<()> {
        let splunk_config = self
            .config
            .splunk
            .as_ref()
            .ok_or_else(|| Error::config("Splunk not configured"))?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Splunk {}", splunk_config.hec_token))
            .map_err(|e| Error::config(&format!("Invalid HEC token: {}", e)))?);

        let hec_event = serde_json::json!({
            "time": event.timestamp.timestamp(),
            "host": "mcp-monitoring",
            "source": event.source,
            "sourcetype": splunk_config.source_type,
            "index": splunk_config.index,
            "event": {
                "id": event.id,
                "event_type": event.event_type,
                "severity": format!("{:?}", event.severity),
                "data": event.data
            }
        });

        let response = self.http_client
            .post(&splunk_config.hec_url)
            .headers(headers)
            .json(&hec_event)
            .send()
            .await
            .map_err(|e| Error::service(&format!("Failed to send event to Splunk: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::service(&format!("Splunk event sending failed: {}", error_text)));
        }

        Ok(())
    }

    /// Query Elasticsearch via API
    pub async fn elasticsearch_search(
        &self,
        query: &ElasticsearchQuery,
    ) -> Result<ElasticsearchResult> {
        let es_config = self
            .config
            .elasticsearch
            .as_ref()
            .ok_or_else(|| Error::config("Elasticsearch not configured"))?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Add authentication
        if let Some(api_key) = &es_config.api_key {
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("ApiKey {}", api_key))
                .map_err(|e| Error::config(&format!("Invalid API key: {}", e)))?);
        } else if let (Some(username), Some(password)) = (&es_config.username, &es_config.password) {
            let credentials = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", username, password));
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Basic {}", credentials))
                .map_err(|e| Error::config(&format!("Invalid credentials: {}", e)))?);
        }

        // Use first URL from the list
        let base_url = es_config.urls.first().ok_or_else(|| Error::config("No Elasticsearch URLs configured"))?;
        let url = format!("{}/{}/_search", base_url, query.index);

        let mut search_body = serde_json::json!({
            "query": query.query
        });

        if let Some(size) = query.size {
            search_body["size"] = serde_json::json!(size);
        }
        if let Some(from) = query.from {
            search_body["from"] = serde_json::json!(from);
        }
        if let Some(ref sort) = query.sort {
            search_body["sort"] = serde_json::json!(sort);
        }

        let response = self.http_client
            .post(&url)
            .headers(headers)
            .json(&search_body)
            .send()
            .await
            .map_err(|e| Error::service(&format!("Failed to search Elasticsearch: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::service(&format!("Elasticsearch search failed: {}", error_text)));
        }

        let search_result: serde_json::Value = response.json().await
            .map_err(|e| Error::service(&format!("Failed to parse Elasticsearch response: {}", e)))?;

        // Parse Elasticsearch response
        let result = ElasticsearchResult {
            took: search_result.get("took").and_then(|v| v.as_i64()).unwrap_or(0),
            timed_out: search_result.get("timed_out").and_then(|v| v.as_bool()).unwrap_or(false),
            hits: ElasticsearchHits {
                total: ElasticsearchTotal {
                    value: search_result.get("hits")
                        .and_then(|h| h.get("total"))
                        .and_then(|t| t.get("value"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                    relation: search_result.get("hits")
                        .and_then(|h| h.get("total"))
                        .and_then(|t| t.get("relation"))
                        .and_then(|r| r.as_str())
                        .unwrap_or("eq")
                        .to_string(),
                },
                max_score: search_result.get("hits")
                    .and_then(|h| h.get("max_score"))
                    .and_then(|s| s.as_f64())
                    .unwrap_or(0.0),
                hits: search_result.get("hits")
                    .and_then(|h| h.get("hits"))
                    .and_then(|h| h.as_array())
                    .map(|hits| {
                        hits.iter().filter_map(|hit| {
                            Some(ElasticsearchHit {
                                _index: hit.get("_index")?.as_str()?.to_string(),
                                _id: hit.get("_id")?.as_str()?.to_string(),
                                _score: hit.get("_score")?.as_f64().unwrap_or(0.0),
                                _source: hit.get("_source")?.clone(),
                            })
                        }).collect()
                    })
                    .unwrap_or_default(),
            },
        };

        Ok(result)
    }

    /// Send metrics to Datadog via API
    pub async fn datadog_send_metrics(&self, metrics: Vec<DatadogMetric>) -> Result<()> {
        let dd_config = self
            .config
            .datadog
            .as_ref()
            .ok_or_else(|| Error::config("Datadog not configured"))?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("DD-API-KEY", HeaderValue::from_str(&dd_config.api_key)
            .map_err(|e| Error::config(&format!("Invalid API key: {}", e)))?);
        headers.insert("DD-APPLICATION-KEY", HeaderValue::from_str(&dd_config.app_key)
            .map_err(|e| Error::config(&format!("Invalid application key: {}", e)))?);

        let api_url = dd_config.api_url.as_ref()
            .map(|u| u.clone())
            .unwrap_or_else(|| format!("https://api.{}", dd_config.site));
        let url = format!("{}/api/v1/series", api_url);

        let series_data = serde_json::json!({
            "series": metrics.iter().map(|metric| {
                serde_json::json!({
                    "metric": metric.metric,
                    "points": metric.points,
                    "type": metric.metric_type,
                    "host": metric.host,
                    "tags": metric.tags
                })
            }).collect::<Vec<_>>()
        });

        let response = self.http_client
            .post(&url)
            .headers(headers)
            .json(&series_data)
            .send()
            .await
            .map_err(|e| Error::service(&format!("Failed to send metrics to Datadog: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::service(&format!("Datadog metrics sending failed: {}", error_text)));
        }

        Ok(())
    }

    /// Query Crowdstrike detections via API
    pub async fn crowdstrike_get_detections(
        &self,
        filters: &HashMap<String, String>,
    ) -> Result<Vec<CrowdstrikeDetection>> {
        let cs_config = self
            .config
            .crowdstrike
            .as_ref()
            .ok_or_else(|| Error::config("Crowdstrike not configured"))?;

        // First, get an OAuth token
        let token = self.crowdstrike_get_token(cs_config).await?;

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token))
            .map_err(|e| Error::config(&format!("Invalid token: {}", e)))?);

        // Build query parameters
        let mut query_params = Vec::new();
        for (key, value) in filters {
            query_params.push(format!("{}={}", key, value));
        }
        let query_string = if query_params.is_empty() {
            String::new()
        } else {
            format!("?{}", query_params.join("&"))
        };

        let url = format!("{}/detects/queries/detects/v1{}", cs_config.base_url, query_string);

        let response = self.http_client
            .get(&url)
            .headers(headers.clone())
            .send()
            .await
            .map_err(|e| Error::service(&format!("Failed to query Crowdstrike detections: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::service(&format!("Crowdstrike detection query failed: {}", error_text)));
        }

        let detection_ids: serde_json::Value = response.json().await
            .map_err(|e| Error::service(&format!("Failed to parse Crowdstrike response: {}", e)))?;

        // Extract detection IDs and fetch detailed information
        if let Some(ids) = detection_ids.get("resources").and_then(|r| r.as_array()) {
            if ids.is_empty() {
                return Ok(vec![]);
            }

            // Get detailed detection information
            let ids_str: Vec<String> = ids.iter()
                .filter_map(|id| id.as_str().map(|s| s.to_string()))
                .collect();
            
            let details_url = format!("{}/detects/entities/summaries/GET/v1", cs_config.base_url);
            let body = serde_json::json!({
                "ids": ids_str
            });

            let details_response = self.http_client
                .post(&details_url)
                .headers(headers)
                .json(&body)
                .send()
                .await
                .map_err(|e| Error::service(&format!("Failed to get Crowdstrike detection details: {}", e)))?;

            if !details_response.status().is_success() {
                let error_text = details_response.text().await.unwrap_or_default();
                return Err(Error::service(&format!("Crowdstrike detection details failed: {}", error_text)));
            }

            let details_data: serde_json::Value = details_response.json().await
                .map_err(|e| Error::service(&format!("Failed to parse Crowdstrike details response: {}", e)))?;

            let detections = if let Some(resources) = details_data.get("resources").and_then(|r| r.as_array()) {
                resources.iter().filter_map(|detection| {
                    Some(CrowdstrikeDetection {
                        detection_id: detection.get("detection_id")?.as_str()?.to_string(),
                        device_id: detection.get("device").and_then(|d| d.get("device_id"))?.as_str()?.to_string(),
                        behavior_id: detection.get("behaviors").and_then(|b| b.as_array())?
                            .first().and_then(|first| first.get("behavior_id"))?.as_str()?.to_string(),
                        severity: detection.get("max_severity")?.as_i64()? as i32,
                        confidence: detection.get("max_confidence")?.as_i64()? as i32,
                        pattern_id: detection.get("behaviors").and_then(|b| b.as_array())?
                            .first().and_then(|first| first.get("pattern_disposition"))?.as_str()?.to_string(),
                        timestamp: detection.get("first_behavior")?.as_str()
                            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(Utc::now),
                    })
                }).collect()
            } else {
                vec![]
            };

            Ok(detections)
        } else {
            Ok(vec![])
        }
    }

    /// Send logs to Azure Sentinel via Log Analytics API
    pub async fn sentinel_send_logs(&self, logs: Vec<SentinelLog>) -> Result<()> {
        let sentinel_config = self
            .config
            .sentinel
            .as_ref()
            .ok_or_else(|| Error::config("Azure Sentinel not configured"))?;

        // Generate date and authorization signature
        let date = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        let json_data = serde_json::to_string(&logs)
            .map_err(|e| Error::internal(&format!("Failed to serialize logs: {}", e)))?;
        
        let signature = self.build_azure_signature(
            &date,
            json_data.len(),
            "POST",
            "application/json",
            "/api/logs",
            &sentinel_config.workspace_key,
        )?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("Log-Type", HeaderValue::from_str(&sentinel_config.log_type)
            .map_err(|e| Error::config(&format!("Invalid log type: {}", e)))?);
        headers.insert("x-ms-date", HeaderValue::from_str(&date)
            .map_err(|e| Error::config(&format!("Invalid date: {}", e)))?);
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&signature)
            .map_err(|e| Error::config(&format!("Invalid signature: {}", e)))?);
        
        if let Some(resource_id) = &sentinel_config.resource_id {
            headers.insert("x-ms-AzureResourceId", HeaderValue::from_str(resource_id)
                .map_err(|e| Error::config(&format!("Invalid resource ID: {}", e)))?);
        }

        let url = format!(
            "https://{}.ods.opinsights.azure.com/api/logs?api-version=2016-04-01",
            sentinel_config.workspace_id
        );

        let response = self.http_client
            .post(&url)
            .headers(headers)
            .body(json_data)
            .send()
            .await
            .map_err(|e| Error::service(&format!("Failed to send logs to Azure Sentinel: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::service(&format!("Azure Sentinel log sending failed: {}", error_text)));
        }

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
                AlertSource::Prometheus { severity, .. }
                | AlertSource::Splunk { severity, .. }
                | AlertSource::Elasticsearch { severity, .. } => {
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

    // Helper methods for authentication and health checks

    /// Get Crowdstrike OAuth token
    async fn crowdstrike_get_token(&self, config: &CrowdstrikeConfig) -> Result<String> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));

        let body = format!(
            "client_id={}&client_secret={}&grant_type=client_credentials",
            config.client_id, config.client_secret
        );

        let url = format!("{}/oauth2/token", config.base_url);

        let response = self.http_client
            .post(&url)
            .headers(headers)
            .body(body)
            .send()
            .await
            .map_err(|e| Error::service(&format!("Failed to get Crowdstrike token: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::service(&format!("Crowdstrike token request failed: {}", error_text)));
        }

        let token_data: serde_json::Value = response.json().await
            .map_err(|e| Error::service(&format!("Failed to parse token response: {}", e)))?;

        token_data.get("access_token")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| Error::service("No access token in response"))
    }

    /// Build Azure Log Analytics signature
    fn build_azure_signature(
        &self,
        date: &str,
        content_length: usize,
        method: &str,
        content_type: &str,
        resource: &str,
        workspace_key: &str,
    ) -> Result<String> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        type HmacSha256 = Hmac<Sha256>;
        
        let string_to_hash = format!(
            "{}\n{}\n{}\nx-ms-date:{}\n{}",
            method, content_length, content_type, date, resource
        );
        
        let decoded_key = base64::engine::general_purpose::STANDARD.decode(workspace_key)
            .map_err(|e| Error::config(&format!("Invalid workspace key: {}", e)))?;
        
        let mut mac = HmacSha256::new_from_slice(&decoded_key)
            .map_err(|e| Error::internal(&format!("Failed to create HMAC: {}", e)))?;
        mac.update(string_to_hash.as_bytes());
        
        let result = mac.finalize();
        let encoded_hash = base64::engine::general_purpose::STANDARD.encode(result.into_bytes());
        
        Ok(format!("SharedKey {}:{}", "workspace_id", encoded_hash))
    }

    /// Health check for Prometheus
    async fn prometheus_health_check(&self, config: &PrometheusConfig) -> Result<bool> {
        let url = format!("{}/api/v1/query", config.url);
        
        let mut headers = HeaderMap::new();
        if let Some(token) = &config.bearer_token {
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|_| Error::config("Invalid bearer token"))?);
        }
        
        match self.http_client
            .get(&url)
            .headers(headers)
            .timeout(Duration::from_secs(5))
            .send()
            .await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Health check for Grafana
    async fn grafana_health_check(&self, config: &GrafanaConfig) -> Result<bool> {
        let url = format!("{}/api/health", config.url);
        
        let mut headers = HeaderMap::new();
        if let Some(api_key) = &config.api_key {
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|_| Error::config("Invalid API key"))?);
        }
        
        match self.http_client
            .get(&url)
            .headers(headers)
            .timeout(Duration::from_secs(5))
            .send()
            .await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Health check for Elasticsearch
    async fn elasticsearch_health_check(&self, config: &ElasticsearchConfig) -> Result<bool> {
        if let Some(base_url) = config.urls.first() {
            let url = format!("{}/_cluster/health", base_url);
            
            let mut headers = HeaderMap::new();
            if let Some(api_key) = &config.api_key {
                headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("ApiKey {}", api_key))
                    .map_err(|_| Error::config("Invalid API key"))?);
            }
            
            match self.http_client
                .get(&url)
                .headers(headers)
                .timeout(Duration::from_secs(5))
                .send()
                .await {
                Ok(response) => Ok(response.status().is_success()),
                Err(_) => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    /// Health check for Datadog
    async fn datadog_health_check(&self, config: &DatadogConfig) -> Result<bool> {
        let api_url = config.api_url.as_ref()
            .map(|u| u.clone())
            .unwrap_or_else(|| format!("https://api.{}", config.site));
        let url = format!("{}/api/v1/validate", api_url);
        
        let mut headers = HeaderMap::new();
        headers.insert("DD-API-KEY", HeaderValue::from_str(&config.api_key)
            .map_err(|_| Error::config("Invalid API key"))?);
        
        match self.http_client
            .get(&url)
            .headers(headers)
            .timeout(Duration::from_secs(5))
            .send()
            .await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
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
