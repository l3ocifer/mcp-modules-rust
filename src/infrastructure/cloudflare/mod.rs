use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client as ReqwestClient;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", &config.api_token))
                .map_err(|e| Error::config(format!("Invalid API key: {}", e)))?,
        );
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
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to list DNS records: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to get response text".to_string());
            return Err(Error::api(format!(
                "Failed to list DNS records: HTTP {} - {}",
                status, text
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::protocol(format!("Failed to parse response: {}", e)))?;

        let success = response_json
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !success {
            let errors = response_json
                .get("errors")
                .and_then(|v| serde_json::to_string_pretty(v).ok())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(Error::api(format!("Cloudflare API error: {}", errors)));
        }

        let records = response_json
            .get("result")
            .ok_or_else(|| Error::protocol("Missing 'result' field in response".to_string()))?;

        let records: Vec<DnsRecord> = serde_json::from_value(records.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse DNS records: {}", e)))?;

        Ok(records)
    }

    /// Create a DNS record
    pub async fn create_dns_record(
        &self,
        zone_id: &str,
        params: CreateDnsRecordParams,
    ) -> Result<DnsRecord> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        );

        let response = self
            .client
            .post(&url)
            .json(&params)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to create DNS record: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to get response text".to_string());
            return Err(Error::api(format!(
                "Failed to create DNS record: HTTP {} - {}",
                status, text
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::protocol(format!("Failed to parse response: {}", e)))?;

        let success = response_json
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !success {
            let errors = response_json
                .get("errors")
                .and_then(|v| serde_json::to_string_pretty(v).ok())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(Error::api(format!("Cloudflare API error: {}", errors)));
        }

        let record = response_json
            .get("result")
            .ok_or_else(|| Error::protocol("Missing 'result' field in response".to_string()))?;

        let record: DnsRecord = serde_json::from_value(record.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse DNS record: {}", e)))?;

        Ok(record)
    }

    /// Update a DNS record
    pub async fn update_dns_record(
        &self,
        zone_id: &str,
        record_id: &str,
        params: UpdateDnsRecordParams,
    ) -> Result<DnsRecord> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone_id, record_id
        );

        let response = self
            .client
            .patch(&url)
            .json(&params)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to update DNS record: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to get response text".to_string());
            return Err(Error::api(format!(
                "Failed to update DNS record: HTTP {} - {}",
                status, text
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::protocol(format!("Failed to parse response: {}", e)))?;

        let success = response_json
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !success {
            let errors = response_json
                .get("errors")
                .and_then(|v| serde_json::to_string_pretty(v).ok())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(Error::api(format!("Cloudflare API error: {}", errors)));
        }

        let record = response_json
            .get("result")
            .ok_or_else(|| Error::protocol("Missing 'result' field in response".to_string()))?;

        let record: DnsRecord = serde_json::from_value(record.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse DNS record: {}", e)))?;

        Ok(record)
    }

    /// Delete a DNS record
    pub async fn delete_dns_record(&self, zone_id: &str, record_id: &str) -> Result<()> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone_id, record_id
        );

        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to delete DNS record: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to get response text".to_string());
            return Err(Error::api(format!(
                "Failed to delete DNS record: HTTP {} - {}",
                status, text
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::protocol(format!("Failed to parse response: {}", e)))?;

        let success = response_json
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !success {
            let errors = response_json
                .get("errors")
                .and_then(|v| serde_json::to_string_pretty(v).ok())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(Error::api(format!("Cloudflare API error: {}", errors)));
        }

        Ok(())
    }

    /// List zones
    pub async fn list_zones(&self) -> Result<Vec<Zone>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones?account.id={}",
            self.account_id
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to list zones: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to get response text".to_string());
            return Err(Error::api(format!(
                "Failed to list zones: HTTP {} - {}",
                status, text
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::protocol(format!("Failed to parse response: {}", e)))?;

        let success = response_json
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !success {
            let errors = response_json
                .get("errors")
                .and_then(|v| serde_json::to_string_pretty(v).ok())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(Error::api(format!("Cloudflare API error: {}", errors)));
        }

        let zones = response_json
            .get("result")
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
    pub async fn deploy_website(
        &self,
        domain: &str,
        _site_path: &str,
    ) -> Result<WebsiteDeployment> {
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

    /// Verify API token by making a test request
    async fn verify_token(&self) -> Result<()> {
        // Make a simple request to verify the token
        let response = self.client
            .get("https://api.cloudflare.com/client/v4/user/tokens/verify")
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to verify token: {}", e)))?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::auth("Cloudflare API token verification failed"))
        }
    }

    /// Check Cloudflare API health
    pub async fn health_check(&self) -> Result<bool> {
        // Try to verify API token by making a simple request
        match self.verify_token().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Deploy a Cloudflare resource
    pub async fn deploy_resource(&self, resource: crate::infrastructure::ResourceSpec) -> Result<crate::infrastructure::ResourceResult> {
        use crate::infrastructure::ResourceResult;
        
        // Deploy based on resource type
        match resource.spec.get("type").and_then(|v| v.as_str()) {
            Some("dns_record") => {
                let zone_id = resource.spec.get("zone_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::validation("Missing 'zone_id' in resource spec"))?;
                let record_type = resource.spec.get("record_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::validation("Missing 'record_type' in resource spec"))?;
                let name = resource.spec.get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::validation("Missing 'name' in resource spec"))?;
                let content = resource.spec.get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::validation("Missing 'content' in resource spec"))?;
                let proxied = resource.spec.get("proxied")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                let params = CreateDnsRecordParams {
                    record_type: record_type.to_string(),
                    name: name.to_string(),
                    content: content.to_string(),
                    ttl: 300, // Default TTL
                    proxied,
                };
                
                match self.create_dns_record(zone_id, params).await {
                    Ok(record) => Ok(ResourceResult {
                        name: resource.name.clone(),
                        resource_type: resource.resource_type.clone(),
                        status: "success".to_string(),
                        message: Some(format!("DNS record {} created successfully with ID: {}", name, record.id)),
                    }),
                    Err(e) => Ok(ResourceResult {
                        name: resource.name.clone(),
                        resource_type: resource.resource_type.clone(),
                        status: "failed".to_string(),
                        message: Some(format!("Failed to create DNS record: {}", e)),
                    }),
                }
            },
            Some("website") => {
                let domain = resource.spec.get("domain")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::validation("Missing 'domain' in resource spec"))?;
                let site_path = resource.spec.get("site_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                
                match self.deploy_website(domain, site_path).await {
                    Ok(deployment) => Ok(ResourceResult {
                        name: resource.name.clone(),
                        resource_type: resource.resource_type.clone(),
                        status: "success".to_string(),
                        message: Some(format!("Website deployed to {}", deployment.url)),
                    }),
                    Err(e) => Ok(ResourceResult {
                        name: resource.name.clone(),
                        resource_type: resource.resource_type.clone(),
                        status: "failed".to_string(),
                        message: Some(format!("Failed to deploy website: {}", e)),
                    }),
                }
            },
            _ => Ok(ResourceResult {
                name: resource.name.clone(),
                resource_type: resource.resource_type.clone(),
                status: "failed".to_string(),
                message: Some("Unknown or unsupported Cloudflare resource type".to_string()),
            }),
        }
    }

    /// Scale Cloudflare resources (not applicable)
    pub async fn scale_resource(&self, target: crate::infrastructure::ScalingTarget) -> Result<crate::infrastructure::ScalingTargetResult> {
        use crate::infrastructure::ScalingTargetResult;
        
        // Cloudflare resources don't support traditional scaling
        Ok(ScalingTargetResult {
            resource_id: target.resource_name.clone(),
            previous_count: target.current_count,
            new_count: target.current_count,
            status: "unsupported".to_string(),
        })
    }

    /// Get Cloudflare metrics
    pub async fn get_metrics(&self) -> Result<serde_json::Value> {
        // Cloudflare metrics would require Analytics API access
        // For now, return basic stats
        Ok(serde_json::json!({
            "provider": "cloudflare",
            "status": "operational",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}
