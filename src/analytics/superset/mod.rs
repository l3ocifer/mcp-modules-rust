use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION}};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::path::PathBuf;

/// Superset dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    /// Dashboard ID
    pub id: i64,
    /// Dashboard title
    pub title: String,
    /// Dashboard slug
    pub slug: Option<String>,
    /// Owner ID
    pub owner_id: Option<i64>,
    /// Published status
    pub published: bool,
    /// JSON metadata
    pub json_metadata: Option<String>,
    /// Position JSON
    pub position_json: Option<String>,
    /// Created by name
    pub created_by_name: Option<String>,
    /// Changed by name
    pub changed_by_name: Option<String>,
    /// Created on
    pub created_on: Option<String>,
    /// Changed on
    pub changed_on: Option<String>,
}

/// Superset chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    /// Chart ID
    pub id: i64,
    /// Chart title
    pub slice_name: String,
    /// Visualization type
    pub viz_type: String,
    /// Dataset ID
    pub datasource_id: Option<i64>,
    /// Dataset type
    pub datasource_type: Option<String>,
    /// Dashboard ID (if part of a dashboard)
    pub dashboards: Option<Vec<i64>>,
    /// Description
    pub description: Option<String>,
    /// Owner IDs
    pub owners: Option<Vec<i64>>,
    /// JSON metadata
    pub params: Option<String>,
    /// Query context
    pub query_context: Option<String>,
    /// Created by name
    pub created_by_name: Option<String>,
    /// Changed by name
    pub changed_by_name: Option<String>,
    /// Created on
    pub created_on: Option<String>,
    /// Changed on
    pub changed_on: Option<String>,
}

/// Superset dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    /// Dataset ID
    pub id: i64,
    /// Dataset name
    pub table_name: String,
    /// Schema name
    pub schema: Option<String>,
    /// Database ID
    pub database_id: i64,
    /// SQL query (if virtual dataset)
    pub sql: Option<String>,
    /// Owner IDs
    pub owners: Option<Vec<i64>>,
    /// Description
    pub description: Option<String>,
    /// Main timestamp column
    pub main_dttm_col: Option<String>,
    /// Created by name
    pub created_by_name: Option<String>,
    /// Changed by name
    pub changed_by_name: Option<String>,
    /// Created on
    pub created_on: Option<String>,
    /// Changed on
    pub changed_on: Option<String>,
}

/// Superset database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    /// Database ID
    pub id: i64,
    /// Database name
    pub database_name: String,
    /// SQL Alchemy URI
    pub sqlalchemy_uri: String,
    /// Connection parameters (redacted)
    pub parameters: Option<Value>,
    /// Owner IDs
    pub owners: Option<Vec<i64>>,
    /// Allow CSV upload
    pub allow_csv_upload: bool,
    /// Allow CTAS (Create Table As Select)
    pub allow_ctas: bool,
    /// Allow DML (Data Manipulation Language)
    pub allow_dml: bool,
    /// Created by name
    pub created_by_name: Option<String>,
    /// Changed by name
    pub changed_by_name: Option<String>,
    /// Created on
    pub created_on: Option<String>,
    /// Changed on
    pub changed_on: Option<String>,
}

/// Query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResults {
    /// Query ID
    pub id: String,
    /// SQL statement
    pub sql: String,
    /// Status
    pub status: String,
    /// Database ID
    pub database_id: i64,
    /// Progress
    pub progress: Option<i32>,
    /// Error message (if any)
    pub error_message: Option<String>,
    /// Results
    pub results: Option<QueryResultData>,
}

/// Query result data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResultData {
    /// Column names
    pub columns: Vec<String>,
    /// Column types
    pub column_types: Vec<String>,
    /// Data rows
    pub data: Vec<Vec<Value>>,
    /// Row count
    pub row_count: i64,
    /// Expanded columns
    pub expanded_columns: Option<Vec<String>>,
}

/// Superset client authentication
#[derive(Debug, Clone)]
struct SupersetAuth {
    /// Base URL
    base_url: String,
    /// Username
    username: String,
    /// Password
    password: String,
    /// Access token
    access_token: Arc<RwLock<Option<String>>>,
    /// Token refresh endpoint
    refresh_endpoint: Option<String>,
    /// Refresh token
    refresh_token: Arc<RwLock<Option<String>>>,
    /// Token file path
    token_file: Option<PathBuf>,
}

/// Superset client for MCP
pub struct SupersetClient<'a> {
    /// Lifecycle manager
    lifecycle: &'a LifecycleManager,
    /// HTTP client
    client: Client,
    /// Authentication
    auth: SupersetAuth,
}

impl<'a> SupersetClient<'a> {
    /// Create a new Superset client
    pub fn new(lifecycle: &'a LifecycleManager, base_url: &str, username: &str, password: &str, token_file: Option<&str>) -> Result<Self> {
        if base_url.is_empty() {
            return Err(Error::config("Superset base URL is required".to_string()));
        }
        
        if username.is_empty() || password.is_empty() {
            return Err(Error::config("Superset username and password are required".to_string()));
        }
        
        let client = Client::builder()
            .build()
            .map_err(|e| Error::internal(format!("Failed to create HTTP client: {}", e)))?;
        
        let token_file_path = token_file.map(PathBuf::from);
        
        // Create authentication object
        let auth = SupersetAuth {
            base_url: base_url.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            access_token: Arc::new(RwLock::new(None)),
            refresh_endpoint: Some("/api/v1/security/refresh".to_string()),
            refresh_token: Arc::new(RwLock::new(None)),
            token_file: token_file_path,
        };
        
        Ok(Self {
            lifecycle,
            client,
            auth,
        })
    }
    
    /// Authenticate with Superset
    async fn authenticate(&self) -> Result<()> {
        let login_url = format!("{}/api/v1/security/login", self.auth.base_url);
        
        let credentials = json!({
            "username": self.auth.username,
            "password": self.auth.password,
            "provider": "db",
            "refresh": true
        });
        
        let response = self.client.post(&login_url)
            .header(CONTENT_TYPE, "application/json")
            .json(&credentials)
            .send()
            .await
            .map_err(|e| Error::auth(format!("Failed to authenticate with Superset: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(Error::auth(format!("Authentication failed: {}", response.status())));
        }
        
        let auth_response: Value = response.json().await
            .map_err(|e| Error::protocol(format!("Failed to parse authentication response: {}", e)))?;
        
        let access_token = auth_response.get("access_token")
            .and_then(|t| t.as_str())
            .ok_or_else(|| Error::auth("No access token in response".to_string()))?
            .to_string();
            
        let refresh_token = auth_response.get("refresh_token")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());
        
        // Store tokens
        {
            let mut token = self.auth.access_token.write().await;
            *token = Some(access_token);
        }
        
        if let Some(rt) = refresh_token {
            let mut refresh = self.auth.refresh_token.write().await;
            *refresh = Some(rt);
        }
        
        // Save to file if configured
        if let Some(path) = &self.auth.token_file {
            let token_data = json!({
                "access_token": *self.auth.access_token.read().await,
                "refresh_token": *self.auth.refresh_token.read().await
            });
            
            tokio::fs::write(path, token_data.to_string())
                .await
                .map_err(|e| Error::internal(format!("Failed to save tokens to file: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Get authentication headers
    async fn get_auth_headers(&self) -> Result<HeaderMap> {
        // Check if we have a token
        let token = {
            let token = self.auth.access_token.read().await;
            match &*token {
                Some(t) => t.clone(),
                None => {
                    // Try to authenticate
                    drop(token);
                    self.authenticate().await?;
                    self.auth.access_token.read().await.clone().unwrap()
                }
            }
        };
        
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token))
            .map_err(|_| Error::internal("Invalid token for header".to_string()))?);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
        Ok(headers)
    }
    
    /// List all dashboards
    pub async fn list_dashboards(&self) -> Result<Vec<Dashboard>> {
        let headers = self.get_auth_headers().await?;
        let url = format!("{}/api/v1/dashboard/", self.auth.base_url);
        
        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to get dashboards: {}", e)))?;
            
        if !response.status().is_success() {
            return Err(Error::service(format!("Failed to get dashboards: {}", response.status())));
        }
        
        let dashboard_data: Value = response.json().await
            .map_err(|e| Error::protocol(format!("Failed to parse dashboard response: {}", e)))?;
        
        let dashboards = dashboard_data.get("result")
            .and_then(|r| r.as_array())
            .ok_or_else(|| Error::protocol("Invalid dashboard result format".to_string()))?;
        
        let mut result = Vec::new();
        for dash in dashboards {
            if let Ok(dashboard) = serde_json::from_value::<Dashboard>(dash.clone()) {
                result.push(dashboard);
            }
        }
        
        Ok(result)
    }
    
    /// Get dashboard by ID
    pub async fn get_dashboard(&self, dashboard_id: i64) -> Result<Dashboard> {
        let headers = self.get_auth_headers().await?;
        let url = format!("{}/api/v1/dashboard/{}", self.auth.base_url, dashboard_id);
        
        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to get dashboard: {}", e)))?;
            
        if !response.status().is_success() {
            return Err(Error::service(format!("Failed to get dashboard: {}", response.status())));
        }
        
        let dashboard_data: Value = response.json().await
            .map_err(|e| Error::protocol(format!("Failed to parse dashboard response: {}", e)))?;
        
        let dashboard = dashboard_data.get("result")
            .ok_or_else(|| Error::protocol("Invalid dashboard result format".to_string()))?;
        
        let dashboard = serde_json::from_value::<Dashboard>(dashboard.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse dashboard data: {}", e)))?;
        
        Ok(dashboard)
    }
    
    /// List all charts
    pub async fn list_charts(&self) -> Result<Vec<Chart>> {
        let headers = self.get_auth_headers().await?;
        let url = format!("{}/api/v1/chart/", self.auth.base_url);
        
        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to get charts: {}", e)))?;
            
        if !response.status().is_success() {
            return Err(Error::service(format!("Failed to get charts: {}", response.status())));
        }
        
        let chart_data: Value = response.json().await
            .map_err(|e| Error::protocol(format!("Failed to parse chart response: {}", e)))?;
        
        let charts = chart_data.get("result")
            .and_then(|r| r.as_array())
            .ok_or_else(|| Error::protocol("Invalid chart result format".to_string()))?;
        
        let mut result = Vec::new();
        for chart in charts {
            if let Ok(c) = serde_json::from_value::<Chart>(chart.clone()) {
                result.push(c);
            }
        }
        
        Ok(result)
    }
    
    /// Get chart by ID
    pub async fn get_chart(&self, chart_id: i64) -> Result<Chart> {
        let headers = self.get_auth_headers().await?;
        let url = format!("{}/api/v1/chart/{}", self.auth.base_url, chart_id);
        
        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to get chart: {}", e)))?;
            
        if !response.status().is_success() {
            return Err(Error::service(format!("Failed to get chart: {}", response.status())));
        }
        
        let chart_data: Value = response.json().await
            .map_err(|e| Error::protocol(format!("Failed to parse chart response: {}", e)))?;
        
        let chart = chart_data.get("result")
            .ok_or_else(|| Error::protocol("Invalid chart result format".to_string()))?;
        
        let chart = serde_json::from_value::<Chart>(chart.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse chart data: {}", e)))?;
        
        Ok(chart)
    }
    
    /// List all databases
    pub async fn list_databases(&self) -> Result<Vec<Database>> {
        let headers = self.get_auth_headers().await?;
        let url = format!("{}/api/v1/database/", self.auth.base_url);
        
        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to get databases: {}", e)))?;
            
        if !response.status().is_success() {
            return Err(Error::service(format!("Failed to get databases: {}", response.status())));
        }
        
        let db_data: Value = response.json().await
            .map_err(|e| Error::protocol(format!("Failed to parse database response: {}", e)))?;
        
        let databases = db_data.get("result")
            .and_then(|r| r.as_array())
            .ok_or_else(|| Error::protocol("Invalid database result format".to_string()))?;
        
        let mut result = Vec::new();
        for db in databases {
            if let Ok(database) = serde_json::from_value::<Database>(db.clone()) {
                result.push(database);
            }
        }
        
        Ok(result)
    }
    
    /// Execute SQL query
    pub async fn execute_sql(&self, database_id: i64, sql: &str) -> Result<QueryResults> {
        let headers = self.get_auth_headers().await?;
        let url = format!("{}/api/v1/sqllab/execute/", self.auth.base_url);
        
        let request_data = json!({
            "database_id": database_id,
            "sql": sql,
            "client_id": format!("mcp-{}", uuid::Uuid::new_v4()),
            "runAsync": false
        });
        
        let response = self.client.post(&url)
            .headers(headers)
            .json(&request_data)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to execute SQL: {}", e)))?;
            
        if !response.status().is_success() {
            return Err(Error::service(format!("Failed to execute SQL: {}", response.status())));
        }
        
        let query_data: QueryResults = response.json().await
            .map_err(|e| Error::protocol(format!("Failed to parse query response: {}", e)))?;
        
        Ok(query_data)
    }
    
    /// Get current user info
    pub async fn get_current_user(&self) -> Result<Value> {
        let headers = self.get_auth_headers().await?;
        let url = format!("{}/api/v1/me/", self.auth.base_url);
        
        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to get user info: {}", e)))?;
            
        if !response.status().is_success() {
            return Err(Error::service(format!("Failed to get user info: {}", response.status())));
        }
        
        let user_data: Value = response.json().await
            .map_err(|e| Error::protocol(format!("Failed to parse user response: {}", e)))?;
        
        Ok(user_data)
    }
} 