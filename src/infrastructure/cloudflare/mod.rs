use reqwest::Client as ReqwestClient;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CloudflareConfig {
    pub api_token: String,
    pub zone_id: String,
    pub account_id: String,
}

/// Cloudflare API client for MCP
pub struct CloudflareClient {
    /// HTTP client
    client: ReqwestClient,
    /// API key
    #[allow(dead_code)]
    api_key: String,
    /// Account ID
    account_id: String,
}

/// Cloudflare DNS record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    /// Record ID
    pub id: String,
    /// Record type
    #[serde(rename = "type")]
    pub record_type: String,
    /// Record name
    pub name: String,
    /// Record content
    pub content: String,
    /// TTL (Time to live)
    pub ttl: i32,
    /// Proxied status
    pub proxied: bool,
}

/// Cloudflare DNS record creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDnsRecordParams {
    /// Record type
    #[serde(rename = "type")]
    pub record_type: String,
    /// Record name
    pub name: String,
    /// Record content
    pub content: String,
    /// TTL (Time to live)
    pub ttl: i32,
    /// Proxied status
    pub proxied: bool,
}

/// Cloudflare DNS record update parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDnsRecordParams {
    /// Record type
    #[serde(rename = "type")]
    pub record_type: Option<String>,
    /// Record name
    pub name: Option<String>,
    /// Record content
    pub content: Option<String>,
    /// TTL (Time to live)
    pub ttl: Option<i32>,
    /// Proxied status
    pub proxied: Option<bool>,
}

/// Cloudflare Zone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    /// Zone ID
    pub id: String,
    /// Zone name
    pub name: String,
    /// Zone status
    pub status: String,
    /// Zone paused status
    pub paused: bool,
}

/// Cloudflare website deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteDeployment {
    /// Deployment ID
    pub id: String,
    /// Deployment URL
    pub url: String,
    /// Deployment status
    pub status: String,
}

impl CloudflareClient {
    /// Create a new Cloudflare client
    pub fn new(config: CloudflareConfig) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", &config.api_token))
            .map_err(|e| Error::config(format!("Invalid API key: {}", e)))?);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
        let client = ReqwestClient::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| Error::network(format!("Failed to create HTTP client: {}", e)))?;
            
        Ok(Self {
            client,
            api_key: config.api_token,
            account_id: config.account_id,
        })
    }
    
    /// List DNS records for a zone
    pub async fn list_dns_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>> {
        let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", zone_id);
        
        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to list DNS records: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Failed to get response text".to_string());
            return Err(Error::api(format!("Failed to list DNS records: HTTP {} - {}", status, text)));
        }
        
        let response_json: serde_json::Value = response.json()
            .await
            .map_err(|e| Error::protocol(format!("Failed to parse response: {}", e)))?;
            
        let success = response_json.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if !success {
            let errors = response_json.get("errors")
                .and_then(|v| serde_json::to_string_pretty(v).ok())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(Error::api(format!("Cloudflare API error: {}", errors)));
        }
        
        let records = response_json.get("result")
            .ok_or_else(|| Error::protocol("Missing 'result' field in response".to_string()))?;
            
        let records: Vec<DnsRecord> = serde_json::from_value(records.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse DNS records: {}", e)))?;
            
        Ok(records)
    }
    
    /// Create a DNS record
    pub async fn create_dns_record(&self, zone_id: &str, params: CreateDnsRecordParams) -> Result<DnsRecord> {
        let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", zone_id);
        
        let response = self.client.post(&url)
            .json(&params)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to create DNS record: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Failed to get response text".to_string());
            return Err(Error::api(format!("Failed to create DNS record: HTTP {} - {}", status, text)));
        }
        
        let response_json: serde_json::Value = response.json()
            .await
            .map_err(|e| Error::protocol(format!("Failed to parse response: {}", e)))?;
            
        let success = response_json.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if !success {
            let errors = response_json.get("errors")
                .and_then(|v| serde_json::to_string_pretty(v).ok())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(Error::api(format!("Cloudflare API error: {}", errors)));
        }
        
        let record = response_json.get("result")
            .ok_or_else(|| Error::protocol("Missing 'result' field in response".to_string()))?;
            
        let record: DnsRecord = serde_json::from_value(record.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse DNS record: {}", e)))?;
            
        Ok(record)
    }
    
    /// Update a DNS record
    pub async fn update_dns_record(&self, zone_id: &str, record_id: &str, params: UpdateDnsRecordParams) -> Result<DnsRecord> {
        let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}", zone_id, record_id);
        
        let response = self.client.patch(&url)
            .json(&params)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to update DNS record: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Failed to get response text".to_string());
            return Err(Error::api(format!("Failed to update DNS record: HTTP {} - {}", status, text)));
        }
        
        let response_json: serde_json::Value = response.json()
            .await
            .map_err(|e| Error::protocol(format!("Failed to parse response: {}", e)))?;
            
        let success = response_json.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if !success {
            let errors = response_json.get("errors")
                .and_then(|v| serde_json::to_string_pretty(v).ok())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(Error::api(format!("Cloudflare API error: {}", errors)));
        }
        
        let record = response_json.get("result")
            .ok_or_else(|| Error::protocol("Missing 'result' field in response".to_string()))?;
            
        let record: DnsRecord = serde_json::from_value(record.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse DNS record: {}", e)))?;
            
        Ok(record)
    }
    
    /// Delete a DNS record
    pub async fn delete_dns_record(&self, zone_id: &str, record_id: &str) -> Result<()> {
        let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}", zone_id, record_id);
        
        let response = self.client.delete(&url)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to delete DNS record: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Failed to get response text".to_string());
            return Err(Error::api(format!("Failed to delete DNS record: HTTP {} - {}", status, text)));
        }
        
        let response_json: serde_json::Value = response.json()
            .await
            .map_err(|e| Error::protocol(format!("Failed to parse response: {}", e)))?;
            
        let success = response_json.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if !success {
            let errors = response_json.get("errors")
                .and_then(|v| serde_json::to_string_pretty(v).ok())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(Error::api(format!("Cloudflare API error: {}", errors)));
        }
        
        Ok(())
    }
    
    /// List zones
    pub async fn list_zones(&self) -> Result<Vec<Zone>> {
        let url = format!("https://api.cloudflare.com/client/v4/zones?account.id={}", self.account_id);
        
        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to list zones: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Failed to get response text".to_string());
            return Err(Error::api(format!("Failed to list zones: HTTP {} - {}", status, text)));
        }
        
        let response_json: serde_json::Value = response.json()
            .await
            .map_err(|e| Error::protocol(format!("Failed to parse response: {}", e)))?;
            
        let success = response_json.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if !success {
            let errors = response_json.get("errors")
                .and_then(|v| serde_json::to_string_pretty(v).ok())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(Error::api(format!("Cloudflare API error: {}", errors)));
        }
        
        let zones = response_json.get("result")
            .ok_or_else(|| Error::protocol("Missing 'result' field in response".to_string()))?;
            
        let zones: Vec<Zone> = serde_json::from_value(zones.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse zones: {}", e)))?;
            
        Ok(zones)
    }
    
    /// Get zone by name
    pub async fn get_zone_by_name(&self, name: &str) -> Result<Option<Zone>> {
        let zones = self.list_zones().await?;
        
        Ok(zones.into_iter().find(|z| z.name == name))
    }
    
    /// Deploy website to Cloudflare Pages
    pub async fn deploy_website(&self, domain: &str, _site_path: &str) -> Result<WebsiteDeployment> {
        // This is a placeholder implementation as actual deployment would require 
        // interacting with Cloudflare Pages API which is more complex
        // In a real implementation, this would:
        // 1. Create/update a Cloudflare Pages project
        // 2. Upload files from site_path
        // 3. Trigger a deployment and wait for it to complete
        
        // For now, we'll just return a mock deployment
        Ok(WebsiteDeployment {
            id: "mock-deployment-id".to_string(),
            url: format!("https://{}", domain),
            status: "success".to_string(),
        })
    }
} 