use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use serde::{Deserialize, Serialize};
use serde_json::json;
use reqwest::Client;

/// Grant agency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agency {
    /// Agency code
    pub code: String,
    /// Agency name
    pub name: String,
}

/// Grant details summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantSummary {
    /// Award ceiling amount
    pub award_ceiling: Option<f64>,
    /// Award floor amount
    pub award_floor: Option<f64>,
    /// Publication date
    pub post_date: Option<String>,
    /// Close date
    pub close_date: Option<String>,
    /// Description of the grant
    pub summary_description: Option<String>,
    /// URL for additional information
    pub additional_info_url: Option<String>,
    /// Agency contact description
    pub agency_contact_description: Option<String>,
    /// Agency email address
    pub agency_email_address: Option<String>,
    /// Agency phone number
    pub agency_phone_number: Option<String>,
    /// Applicant eligibility description
    pub applicant_eligibility_description: Option<String>,
}

/// Government grant representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grant {
    /// Agency short name
    pub agency: String,
    /// Agency code
    pub agency_code: String,
    /// Agency full name
    pub agency_name: String,
    /// Opportunity ID
    pub opportunity_id: u64,
    /// Opportunity number
    pub opportunity_number: String,
    /// Opportunity title
    pub opportunity_title: String,
    /// Opportunity status
    pub opportunity_status: String,
    /// Grant summary
    pub summary: GrantSummary,
    /// Grant category
    pub category: String,
    /// Top level agency name
    pub top_level_agency_name: Option<String>,
}

/// Grants API response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantsApiResponse {
    /// Grant data
    pub data: Vec<Grant>,
    /// Pagination information
    pub pagination_info: PaginationInfo,
    /// Facet counts
    pub facet_counts: FacetCounts,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    /// Total number of records
    pub total_records: u64,
}

/// Facet counts for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetCounts {
    /// Agency counts
    pub agency: std::collections::HashMap<String, u64>,
}

/// Search parameters for grants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantsSearchParams {
    /// Search query
    pub query: String,
    /// Page number (starting from 1)
    pub page: u32,
    /// Number of grants per page
    pub grants_per_page: u32,
}

/// Client for accessing government grants data
pub struct GrantsClient<'a> {
    /// Lifecycle manager
    lifecycle: &'a LifecycleManager,
    /// HTTP client
    client: Client,
    /// API key for grants API
    api_key: Option<String>,
}

impl<'a> GrantsClient<'a> {
    /// Create a new grants client
    pub fn new(lifecycle: &'a LifecycleManager, api_key: Option<String>) -> Self {
        Self {
            lifecycle,
            client: Client::new(),
            api_key,
        }
    }

    /// Set API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Search for government grants based on a query
    pub async fn search_grants(&self, params: GrantsSearchParams) -> Result<Vec<Grant>> {
        let api_key = self.api_key.as_ref()
            .ok_or_else(|| Error::config("Grants API key not provided"))?;
            
        let url = "https://api.simpler.grants.gov/v1/opportunities/search";
        
        let search_data = json!({
            "filters": {
                "opportunity_status": {
                    "one_of": ["forecasted", "posted"]
                }
            },
            "pagination": {
                "order_by": "opportunity_id",
                "page_offset": params.page,
                "page_size": params.grants_per_page,
                "sort_direction": "descending"
            },
            "query": params.query
        });
        
        let response = self.client
            .post(url)
            .header("accept", "application/json")
            .header("X-Auth", api_key)
            .header("Content-Type", "application/json")
            .json(&search_data)
            .send()
            .await
            .map_err(|e| Error::External(format!("Failed to send grants search request: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Unable to read error response".into());
            return Err(Error::External(format!("Grants API returned error {}: {}", status, text)));
        }
        
        let api_response: GrantsApiResponse = response.json()
            .await
            .map_err(|e| Error::External(format!("Failed to parse grants API response: {}", e)))?;
            
        Ok(api_response.data)
    }
    
    /// Format grant details as a string
    pub fn format_grant_details(&self, grant: &Grant) -> String {
        format!(r#"
OPPORTUNITY DETAILS
------------------
Title: {}
Opportunity Number: {}
Agency: {} ({})
Status: {}

FUNDING INFORMATION
------------------
Award Floor: {}
Award Ceiling: {}
Category: {}

DATES AND DEADLINES
------------------
Posted Date: {}
Close Date: {}

CONTACT INFORMATION
------------------
Agency Contact: {}
Email: {}
Phone: {}

ELIGIBILITY
------------------
{}

ADDITIONAL INFORMATION
------------------
More Details URL: {}

Description:
{}

==========================================================================
"#,
            grant.opportunity_title,
            grant.opportunity_number,
            grant.agency_name, grant.agency_code,
            grant.opportunity_status,
            grant.summary.award_floor.map_or("Not specified".to_string(), |v| format!("${}", v)),
            grant.summary.award_ceiling.map_or("Not specified".to_string(), |v| format!("${}", v)),
            grant.category,
            grant.summary.post_date.as_deref().unwrap_or("N/A"),
            grant.summary.close_date.as_deref().unwrap_or("N/A"),
            grant.summary.agency_contact_description.as_deref().unwrap_or("Not provided"),
            grant.summary.agency_email_address.as_deref().unwrap_or("Not provided"),
            grant.summary.agency_phone_number.as_deref().unwrap_or("Not provided"),
            grant.summary.applicant_eligibility_description.as_deref().unwrap_or("Eligibility information not provided")
                .replace("<[^>]*>", "").trim(),
            grant.summary.additional_info_url.as_deref().unwrap_or("Not available"),
            grant.summary.summary_description.as_deref().unwrap_or("No description available")
                .replace("<[^>]*>", "").trim()
        )
    }
    
    /// Create a summary of search results
    pub fn create_summary(&self, grants: &[Grant], search_query: &str, page: u32, grants_per_page: u32) -> String {
        let start_idx = (page as usize - 1) * grants_per_page as usize;
        let end_idx = start_idx + grants_per_page as usize;
        let displayed_grants = &grants[start_idx.min(grants.len())..end_idx.min(grants.len())];
        let total_pages = (grants.len() as f64 / grants_per_page as f64).ceil() as u32;

        let mut summary = format!(r#"Search Results for "{}":

OVERVIEW
--------
Total Grants Found: {}
Showing grants {} to {} of {}
Page {} of {}

DETAILED GRANT LISTINGS
----------------------
"#,
            search_query,
            grants.len(),
            start_idx + 1,
            (start_idx + displayed_grants.len()).min(grants.len()),
            grants.len(),
            page,
            total_pages
        );

        for grant in displayed_grants {
            summary.push_str(&self.format_grant_details(grant));
        }

        summary.push_str(&format!("\nNote: Showing {} grants per page. Total grants available: {}", 
            grants_per_page, grants.len()));
            
        summary
    }
    
    /// Get registered tools
    pub fn get_tools(&self) -> Vec<(String, String, serde_json::Value)> {
        vec![
            (
                "search_grants".to_string(),
                "Search for government grants based on keywords".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["query"],
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query for grants (e.g., 'Artificial intelligence', 'Climate change')"
                        },
                        "page": {
                            "type": "integer",
                            "description": "Page number for pagination (default: 1)"
                        },
                        "grants_per_page": {
                            "type": "integer",
                            "description": "Number of grants per page (default: 3)"
                        }
                    }
                }),
            ),
        ]
    }
} 