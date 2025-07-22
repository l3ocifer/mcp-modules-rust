use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::tools::ToolDefinition;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::process::{Command, Stdio};

/// Azure resource group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGroup {
    /// Resource group name
    pub name: String,
    /// Location
    pub location: String,
    /// Provisioning state
    pub provisioning_state: String,
    /// Tags
    pub tags: Option<HashMap<String, String>>,
}

/// Azure resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource ID
    pub id: String,
    /// Resource name
    pub name: String,
    /// Resource type
    pub resource_type: String,
    /// Location
    pub location: String,
    /// Tags
    pub tags: Option<HashMap<String, String>>,
}

/// Azure subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Subscription ID
    pub id: String,
    /// Subscription name
    pub name: String,
    /// State
    pub state: String,
}

/// Azure location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// Location name
    pub name: String,
    /// Display name
    pub display_name: String,
    /// Region type
    pub region_type: String,
    /// Region category
    pub region_category: String,
}

/// Azure DevOps work item type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    /// Work item ID
    pub id: i32,
    /// Work item type
    pub work_item_type: String,
    /// Work item title
    pub title: String,
    /// Work item state
    pub state: String,
    /// Created by
    pub created_by: Option<String>,
    /// Assigned to
    pub assigned_to: Option<String>,
    /// Tags
    pub tags: Option<Vec<String>>,
    /// Fields
    pub fields: HashMap<String, Value>,
}

/// Work item query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItemQueryResult {
    /// Work items
    pub work_items: Vec<WorkItem>,
    /// Total count
    pub count: usize,
}

/// Azure DevOps build definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildDefinition {
    /// Build definition ID
    pub id: i32,
    /// Name
    pub name: String,
    /// Path
    pub path: String,
    /// Queue status
    pub queue_status: String,
    /// Repository
    pub repository: Option<Repository>,
}

/// Azure DevOps repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Repository ID
    pub id: String,
    /// Repository name
    pub name: String,
    /// Repository type
    pub repository_type: String,
    /// URL
    pub url: Option<String>,
}

/// Azure DevOps build
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    /// Build ID
    pub id: i32,
    /// Build number
    pub build_number: String,
    /// Status
    pub status: String,
    /// Result
    pub result: Option<String>,
    /// Definition
    pub definition: BuildDefinition,
    /// Started on
    pub started_on: Option<String>,
    /// Finished on
    pub finished_on: Option<String>,
    /// Requested by
    pub requested_by: Option<String>,
    /// Source branch
    pub source_branch: String,
}

/// Azure DevOps release definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseDefinition {
    /// Release definition ID
    pub id: i32,
    /// Name
    pub name: String,
    /// Path
    pub path: String,
    /// Release name format
    pub release_name_format: String,
}

/// Azure DevOps release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    /// Release ID
    pub id: i32,
    /// Release name
    pub name: String,
    /// Status
    pub status: String,
    /// Created on
    pub created_on: String,
    /// Created by
    pub created_by: Option<String>,
    /// Definition
    pub definition: ReleaseDefinition,
    /// Description
    pub description: Option<String>,
}

/// Build query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildQueryParams {
    /// Definition ID
    pub definition_id: Option<i32>,
    /// Branch
    pub branch: Option<String>,
    /// Status filter
    pub status_filter: Option<String>,
    /// Result filter
    pub result_filter: Option<String>,
    /// Top (maximum number of builds to return)
    pub top: Option<i32>,
}

/// Azure client for MCP
pub struct AzureClient<'a> {
    /// Lifecycle manager
    lifecycle: &'a LifecycleManager,
    /// Selected subscription ID
    subscription_id: Arc<Mutex<Option<String>>>,
    /// Selected tenant ID
    tenant_id: Arc<Mutex<Option<String>>>,
}

impl<'a> AzureClient<'a> {
    /// Create a new Azure client
    pub fn new(lifecycle: &'a LifecycleManager) -> Result<Self> {
        // Check if Azure CLI is available
        Self::check_azure_cli()?;
        
        Ok(Self {
            lifecycle,
            subscription_id: Arc::new(Mutex::new(None)),
            tenant_id: Arc::new(Mutex::new(None)),
        })
    }
    
    /// Check if Azure CLI is available
    fn check_azure_cli() -> Result<()> {
        match Command::new("az").arg("--version").stdout(Stdio::null()).status() {
            Ok(status) if status.success() => Ok(()),
            _ => Err(Error::config("Azure CLI not found or not properly configured".to_string())),
        }
    }
    
    /// Set subscription ID
    pub fn set_subscription(&self, subscription_id: String) -> Result<()> {
        let mut subscription = self.subscription_id.lock()
            .map_err(|_| Error::internal("Failed to lock subscription ID".to_string()))?;
            
        *subscription = Some(subscription_id);
        Ok(())
    }
    
    /// Set tenant ID
    pub fn set_tenant(&self, tenant_id: String) -> Result<()> {
        let mut tenant = self.tenant_id.lock()
            .map_err(|_| Error::internal("Failed to lock tenant ID".to_string()))?;
            
        *tenant = Some(tenant_id);
        Ok(())
    }
    
    /// Get current subscription ID
    pub fn get_subscription(&self) -> Result<Option<String>> {
        let subscription = self.subscription_id.lock()
            .map_err(|_| Error::internal("Failed to lock subscription ID".to_string()))?;
            
        Ok(subscription.clone())
    }
    
    /// Get current tenant ID
    pub fn get_tenant(&self) -> Result<Option<String>> {
        let tenant = self.tenant_id.lock()
            .map_err(|_| Error::internal("Failed to lock tenant ID".to_string()))?;
            
        Ok(tenant.clone())
    }
    
    /// Execute resource script
    pub async fn execute_resource_script(&self, script: &str) -> Result<Value> {
        let method = "tools/execute";
        let params = json!({
            "name": "execute_resource_script",
            "arguments": {
                "script": script
            }
        });
        
        let response = self.lifecycle.call_method(method, Some(params)).await?;
        
        Ok(response)
    }
    
    /// List resource groups
    pub async fn list_resource_groups(&self) -> Result<Vec<ResourceGroup>> {
        let script = r#"
            // List all resource groups in current subscription
            async function listResourceGroups() {
                try {
                    const groups = [];
                    
                    for await (const group of resourceClient.resourceGroups.list()) {
                        groups.push({
                            name: group.name,
                            location: group.location,
                            provisioningState: group.properties?.provisioningState || 'Unknown',
                            tags: group.tags
                        });
                    }
                    
                    return { resourceGroups: groups };
                } catch (error) {
                    throw new Error(`Failed to list resource groups: ${error.message}`);
                }
            }
            
            return await listResourceGroups();
        "#;
        
        let response = self.execute_resource_script(script).await?;
        
        // Parse resource groups from response
        let content = Self::extract_content_as_json(&response)?;
        
        let groups_data = content.get("resourceGroups")
            .ok_or_else(|| Error::protocol("Missing 'resourceGroups' field in response".to_string()))?;
            
        let groups: Vec<ResourceGroup> = serde_json::from_value(groups_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse resource groups: {}", e)))?;
            
        Ok(groups)
    }
    
    /// Get a resource group
    pub async fn get_resource_group(&self, name: &str) -> Result<ResourceGroup> {
        let script = format!(r#"
            // Get a specific resource group
            async function getResourceGroup() {{
                try {{
                    const group = await resourceClient.resourceGroups.get("{}");
                    
                    return {{
                        resourceGroup: {{
                            name: group.name,
                            location: group.location,
                            provisioningState: group.properties?.provisioningState || 'Unknown',
                            tags: group.tags
                        }}
                    }};
                }} catch (error) {{
                    throw new Error(`Failed to get resource group: ${{error.message}}`);
                }}
            }}
            
            return await getResourceGroup();
        "#, name);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse resource group from response
        let content = Self::extract_content_as_json(&response)?;
        
        let group_data = content.get("resourceGroup")
            .ok_or_else(|| Error::protocol("Missing 'resourceGroup' field in response".to_string()))?;
            
        let group: ResourceGroup = serde_json::from_value(group_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse resource group: {}", e)))?;
            
        Ok(group)
    }
    
    /// Create a resource group
    pub async fn create_resource_group(&self, name: &str, location: &str, tags: Option<HashMap<String, String>>) -> Result<ResourceGroup> {
        let tags_json = match tags {
            Some(t) => serde_json::to_string(&t)
                .map_err(|e| Error::internal(format!("Failed to serialize tags: {}", e)))?,
            None => "null".to_string(),
        };
        
        let script = format!(r#"
            // Create a resource group
            async function createResourceGroup() {{
                try {{
                    const params = {{
                        location: "{}",
                        tags: {}
                    }};
                    
                    const group = await resourceClient.resourceGroups.createOrUpdate("{}", params);
                    
                    return {{
                        resourceGroup: {{
                            name: group.name,
                            location: group.location,
                            provisioningState: group.properties?.provisioningState || 'Unknown',
                            tags: group.tags
                        }}
                    }};
                }} catch (error) {{
                    throw new Error(`Failed to create resource group: ${{error.message}}`);
                }}
            }}
            
            return await createResourceGroup();
        "#, location, tags_json, name);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse resource group from response
        let content = Self::extract_content_as_json(&response)?;
        
        let group_data = content.get("resourceGroup")
            .ok_or_else(|| Error::protocol("Missing 'resourceGroup' field in response".to_string()))?;
            
        let group: ResourceGroup = serde_json::from_value(group_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse resource group: {}", e)))?;
            
        Ok(group)
    }
    
    /// Delete a resource group
    pub async fn delete_resource_group(&self, name: &str) -> Result<()> {
        let script = format!(r#"
            // Delete a resource group
            async function deleteResourceGroup() {{
                try {{
                    await resourceClient.resourceGroups.beginDeleteAndWait("{}");
                    return {{ success: true }};
                }} catch (error) {{
                    throw new Error(`Failed to delete resource group: ${{error.message}}`);
                }}
            }}
            
            return await deleteResourceGroup();
        "#, name);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Check success
        let content = Self::extract_content_as_json(&response)?;
        
        let success = content.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if success {
            Ok(())
        } else {
            Err(Error::service(format!("Failed to delete resource group {}", name)))
        }
    }
    
    /// List resources in a resource group
    pub async fn list_resources(&self, resource_group: Option<&str>) -> Result<Vec<Resource>> {
        let filter = match resource_group {
            Some(rg) => format!(r#"resourceGroup eq '{}'"#, rg),
            None => "".to_string(),
        };
        
        let script = format!(r#"
            // List resources
            async function listResources() {{
                try {{
                    const resources = [];
                    const filter = {};
                    
                    const options = {{
                        filter: filter
                    }};
                    
                    const resourceList = resourceClient.resources.list({});
                    
                    for await (const resource of resourceList) {{
                        resources.push({{
                            id: resource.id,
                            name: resource.name,
                            resourceType: resource.type,
                            location: resource.location || 'global',
                            tags: resource.tags
                        }});
                    }}
                    
                    return {{ resources }};
                }} catch (error) {{
                    throw new Error(`Failed to list resources: ${{error.message}}`);
                }}
            }}
            
            return await listResources();
        "#, 
        if filter.is_empty() { 
            "undefined".to_string() 
        } else { 
            format!(r#""{}""#, filter) 
        },
        if filter.is_empty() { "" } else { "options" }
        );
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse resources from response
        let content = Self::extract_content_as_json(&response)?;
        
        let resources_data = content.get("resources")
            .ok_or_else(|| Error::protocol("Missing 'resources' field in response".to_string()))?;
            
        let resources: Vec<Resource> = serde_json::from_value(resources_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse resources: {}", e)))?;
            
        Ok(resources)
    }
    
    /// List subscriptions
    pub async fn list_subscriptions(&self) -> Result<Vec<Subscription>> {
        let script = r#"
            // List subscriptions
            async function listSubscriptions() {
                try {
                    const subscriptions = [];
                    
                    for await (const subscription of subscriptionClient.subscriptions.list()) {
                        subscriptions.push({
                            id: subscription.subscriptionId,
                            name: subscription.displayName,
                            state: subscription.state
                        });
                    }
                    
                    return { subscriptions };
                } catch (error) {
                    throw new Error(`Failed to list subscriptions: ${error.message}`);
                }
            }
            
            return await listSubscriptions();
        "#;
        
        let response = self.execute_resource_script(script).await?;
        
        // Parse subscriptions from response
        let content = Self::extract_content_as_json(&response)?;
        
        let subscriptions_data = content.get("subscriptions")
            .ok_or_else(|| Error::protocol("Missing 'subscriptions' field in response".to_string()))?;
            
        let subscriptions: Vec<Subscription> = serde_json::from_value(subscriptions_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse subscriptions: {}", e)))?;
            
        Ok(subscriptions)
    }
    
    /// Get a specific subscription
    pub async fn get_subscription_by_id(&self, subscription_id: &str) -> Result<Subscription> {
        let script = format!(r#"
            // Get a specific subscription
            async function getSubscription() {{
                try {{
                    const subscription = await subscriptionClient.subscriptions.get("{}");
                    
                    return {{
                        subscription: {{
                            id: subscription.subscriptionId,
                            name: subscription.displayName,
                            state: subscription.state
                        }}
                    }};
                }} catch (error) {{
                    throw new Error(`Failed to get subscription: ${{error.message}}`);
                }}
            }}
            
            return await getSubscription();
        "#, subscription_id);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse subscription from response
        let content = Self::extract_content_as_json(&response)?;
        
        let subscription_data = content.get("subscription")
            .ok_or_else(|| Error::protocol("Missing 'subscription' field in response".to_string()))?;
            
        let subscription: Subscription = serde_json::from_value(subscription_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse subscription: {}", e)))?;
            
        Ok(subscription)
    }
    
    /// List locations
    pub async fn list_locations(&self, subscription_id: Option<&str>) -> Result<Vec<Location>> {
        let subscription = match subscription_id {
            Some(sub) => sub.to_string(),
            None => match self.get_subscription()? {
                Some(sub) => sub,
                None => return Err(Error::validation("No subscription selected or provided".to_string())),
            },
        };
        
        let script = format!(r#"
            // List locations
            async function listLocations() {{
                try {{
                    const locations = [];
                    
                    for await (const location of subscriptionClient.subscriptions.listLocations("{}")) {{
                        locations.push({{
                            name: location.name,
                            displayName: location.displayName,
                            regionType: location.metadata?.regionType || 'Unknown',
                            regionCategory: location.metadata?.regionCategory || 'Unknown'
                        }});
                    }}
                    
                    return {{ locations }};
                }} catch (error) {{
                    throw new Error(`Failed to list locations: ${{error.message}}`);
                }}
            }}
            
            return await listLocations();
        "#, subscription);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse locations from response
        let content = Self::extract_content_as_json(&response)?;
        
        let locations_data = content.get("locations")
            .ok_or_else(|| Error::protocol("Missing 'locations' field in response".to_string()))?;
            
        let locations: Vec<Location> = serde_json::from_value(locations_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse locations: {}", e)))?;
            
        Ok(locations)
    }
    
    /// Extract JSON content from response
    fn extract_content_as_json(response: &Value) -> Result<Value> {
        let content = response.get("content")
            .ok_or_else(|| Error::protocol("Missing 'content' field in response".to_string()))?;
            
        if !content.is_array() {
            return Err(Error::protocol("'content' field is not an array".to_string()));
        }
        
        let content_array = content.as_array()
            .ok_or_else(|| Error::invalid_data("Expected array for container list"))?;
        
        for item in content_array {
            if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                    return serde_json::from_str(text)
                        .map_err(|e| Error::protocol(format!("Failed to parse content as JSON: {}", e)));
                }
            }
        }
        
        Err(Error::protocol("No text content found in response".to_string()))
    }
    
    /// Get tool definitions
    pub fn get_tool_definitions(&self) -> Vec<ToolDefinition> {
        use crate::tools::ToolAnnotation;
        
        vec![
            ToolDefinition::from_json_schema(
                "list_resource_groups",
                "List Azure resource groups",
                "azure_resource_management",
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
                Some(ToolAnnotation::new("data_retrieval").with_description("List Azure resource groups")
                    .with_usage_hints(vec!["Use to get all resource groups in subscription".to_string()]))
            ),
            ToolDefinition::from_json_schema(
                "get_resource_group",
                "Get details of an Azure resource group",
                "azure_resource_management",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the resource group"
                        }
                    },
                    "required": ["name"]
                }),
                Some(ToolAnnotation::new("data_retrieval").with_description("Get details of an Azure resource group")
                    .with_usage_hints(vec!["Use to get details of a specific resource group".to_string()]))
            ),
            ToolDefinition::from_json_schema(
                "create_resource_group",
                "Create an Azure resource group",
                "azure_resource_management",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the resource group"
                        },
                        "location": {
                            "type": "string",
                            "description": "Azure region location"
                        },
                        "tags": {
                            "type": "object",
                            "description": "Resource tags as key-value pairs",
                            "additionalProperties": {"type": "string"}
                        }
                    },
                    "required": ["name", "location"]
                }),
                Some(ToolAnnotation::new("resource_management").with_description("Create an Azure resource group")
                    .with_security_notes(vec!["Requires confirmation".to_string(), "Has side effects".to_string()]))
            ),
            ToolDefinition::from_json_schema(
                "delete_resource_group",
                "Delete an Azure resource group",
                "azure_resource_management",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the resource group to delete"
                        }
                    },
                    "required": ["name"]
                }),
                Some(ToolAnnotation::new("resource_management").with_description("Delete an Azure resource group")
                    .with_security_notes(vec!["Destructive operation".to_string(), "Requires confirmation".to_string()]))
            ),
            ToolDefinition::from_json_schema(
                "list_resources",
                "List Azure resources",
                "azure_resource_management",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "resourceGroup": {
                            "type": "string",
                            "description": "Filter by resource group name"
                        }
                    },
                    "required": []
                }),
                Some(ToolAnnotation::new("data_retrieval").with_description("List Azure resources")
                    .with_usage_hints(vec!["Use to list all resources or filter by resource group".to_string()]))
            ),
            ToolDefinition::from_json_schema(
                "list_subscriptions",
                "List Azure subscriptions",
                "azure_subscription_management",
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
                Some(ToolAnnotation::new("data_retrieval").with_description("List Azure subscriptions")
                    .with_usage_hints(vec!["Use to get all available Azure subscriptions".to_string()]))
            ),
            ToolDefinition::from_json_schema(
                "list_locations",
                "List Azure locations",
                "azure_subscription_management",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "subscriptionId": {
                            "type": "string",
                            "description": "Azure subscription ID"
                        }
                    },
                    "required": []
                }),
                Some(ToolAnnotation::new("data_retrieval").with_description("List Azure locations")
                    .with_usage_hints(vec!["Use to get available Azure regions".to_string()]))
            ),
        ]
    }

    /// Azure DevOps work item methods
    /// List work items using WIQL query
    pub async fn list_work_items(&self, project: &str, query: &str) -> Result<WorkItemQueryResult> {
        let script = format!(r#"
            // List work items using WIQL query
            async function listWorkItems() {{
                try {{
                    const wiqlQuery = {{
                        query: `{}`
                    }};

                    const witClient = getClient('WorkItemTrackingRestClient');
                    const queryResult = await witClient.queryByWiql(wiqlQuery, '{}');
                    
                    if (!queryResult || !queryResult.workItems || !queryResult.workItems.length) {{
                        return {{ workItems: [], count: 0 }};
                    }}

                    // Get the full work items
                    const ids = queryResult.workItems.map(wi => wi.id);
                    const workItems = await witClient.getWorkItems(ids, null, null, null, '{}');
                    
                    const formattedWorkItems = workItems.map(wi => {{
                        const fields = wi.fields || {{}};
                        return {{
                            id: wi.id,
                            work_item_type: fields['System.WorkItemType'] || 'Unknown',
                            title: fields['System.Title'] || 'Untitled',
                            state: fields['System.State'] || 'Unknown',
                            created_by: fields['System.CreatedBy'] ? fields['System.CreatedBy'].displayName : null,
                            assigned_to: fields['System.AssignedTo'] ? fields['System.AssignedTo'].displayName : null,
                            tags: fields['System.Tags'] ? fields['System.Tags'].split(';').map(t => t.trim()) : [],
                            fields: fields
                        }};
                    }});

                    return {{ 
                        workItems: formattedWorkItems,
                        count: formattedWorkItems.length
                    }};
                }} catch (error) {{
                    throw new Error(`Failed to query work items: ${{error.message}}`);
                }}
            }}
            
            return await listWorkItems();
        "#, query, project, project);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse work items from response
        let content = Self::extract_content_as_json(&response)?;
        
        let work_items_data = content.get("workItems")
            .ok_or_else(|| Error::protocol("Missing 'workItems' field in response".to_string()))?;
            
        let count = content.get("count")
            .and_then(|c| c.as_u64())
            .unwrap_or(0) as usize;
            
        let work_items: Vec<WorkItem> = serde_json::from_value(work_items_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse work items: {}", e)))?;
            
        Ok(WorkItemQueryResult {
            work_items,
            count,
        })
    }

    /// Get work item by ID
    pub async fn get_work_item(&self, project: &str, id: i32) -> Result<WorkItem> {
        let script = format!(r#"
            // Get work item by ID
            async function getWorkItem() {{
                try {{
                    const witClient = getClient('WorkItemTrackingRestClient');
                    const workItem = await witClient.getWorkItem({}, null, null, null, '{}');
                    
                    if (!workItem) {{
                        throw new Error(`Work item with ID {} not found`);
                    }}
                    
                    const fields = workItem.fields || {{}};
                    const formattedWorkItem = {{
                        id: workItem.id,
                        work_item_type: fields['System.WorkItemType'] || 'Unknown',
                        title: fields['System.Title'] || 'Untitled',
                        state: fields['System.State'] || 'Unknown',
                        created_by: fields['System.CreatedBy'] ? fields['System.CreatedBy'].displayName : null,
                        assigned_to: fields['System.AssignedTo'] ? fields['System.AssignedTo'].displayName : null,
                        tags: fields['System.Tags'] ? fields['System.Tags'].split(';').map(t => t.trim()) : [],
                        fields: fields
                    }};
                    
                    return {{ workItem: formattedWorkItem }};
                }} catch (error) {{
                    throw new Error(`Failed to get work item: ${{error.message}}`);
                }}
            }}
            
            return await getWorkItem();
        "#, id, project, id);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse work item from response
        let content = Self::extract_content_as_json(&response)?;
        
        let work_item_data = content.get("workItem")
            .ok_or_else(|| Error::protocol("Missing 'workItem' field in response".to_string()))?;
            
        let work_item: WorkItem = serde_json::from_value(work_item_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse work item: {}", e)))?;
            
        Ok(work_item)
    }

    /// Create a new work item
    pub async fn create_work_item(&self, project: &str, work_item_type: &str, title: &str, fields: Option<HashMap<String, Value>>) -> Result<WorkItem> {
        // Construct document with operations
        let mut operations = Vec::new();
        
        // Add title field
        operations.push(serde_json::json!({
            "op": "add",
            "path": "/fields/System.Title",
            "value": title
        }));
        
        // Add additional fields if provided
        if let Some(field_map) = fields {
            for (field_name, field_value) in field_map {
                operations.push(serde_json::json!({
                    "op": "add",
                    "path": format!("/fields/{}", field_name),
                    "value": field_value
                }));
            }
        }
        
        // Serialize operations
        let operations_json = serde_json::to_string(&operations)
            .map_err(|e| Error::internal(format!("Failed to serialize operations: {}", e)))?;

        let script = format!(r#"
            // Create a new work item
            async function createWorkItem() {{
                try {{
                    const operations = {};
                    
                    const witClient = getClient('WorkItemTrackingRestClient');
                    const workItem = await witClient.createWorkItem(
                        null, operations, '{}', '{}', false
                    );
                    
                    if (!workItem) {{
                        throw new Error('Failed to create work item');
                    }}
                    
                    const fields = workItem.fields || {{}};
                    const formattedWorkItem = {{
                        id: workItem.id,
                        work_item_type: fields['System.WorkItemType'] || 'Unknown',
                        title: fields['System.Title'] || 'Untitled',
                        state: fields['System.State'] || 'Unknown',
                        created_by: fields['System.CreatedBy'] ? fields['System.CreatedBy'].displayName : null,
                        assigned_to: fields['System.AssignedTo'] ? fields['System.AssignedTo'].displayName : null,
                        tags: fields['System.Tags'] ? fields['System.Tags'].split(';').map(t => t.trim()) : [],
                        fields: fields
                    }};
                    
                    return {{ workItem: formattedWorkItem }};
                }} catch (error) {{
                    throw new Error(`Failed to create work item: ${{error.message}}`);
                }}
            }}
            
            return await createWorkItem();
        "#, operations_json, project, work_item_type);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse work item from response
        let content = Self::extract_content_as_json(&response)?;
        
        let work_item_data = content.get("workItem")
            .ok_or_else(|| Error::protocol("Missing 'workItem' field in response".to_string()))?;
            
        let work_item: WorkItem = serde_json::from_value(work_item_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse work item: {}", e)))?;
            
        Ok(work_item)
    }

    /// Update a work item
    pub async fn update_work_item(&self, project: &str, id: i32, fields: HashMap<String, Value>) -> Result<WorkItem> {
        // Construct document with operations
        let mut operations = Vec::new();
        
        // Add field operations
        for (field_name, field_value) in fields {
            operations.push(serde_json::json!({
                "op": "add",
                "path": format!("/fields/{}", field_name),
                "value": field_value
            }));
        }
        
        // Serialize operations
        let operations_json = serde_json::to_string(&operations)
            .map_err(|e| Error::internal(format!("Failed to serialize operations: {}", e)))?;

        let script = format!(r#"
            // Update a work item
            async function updateWorkItem() {{
                try {{
                    const operations = {};
                    
                    const witClient = getClient('WorkItemTrackingRestClient');
                    const workItem = await witClient.updateWorkItem(
                        null, operations, {}, '{}', false
                    );
                    
                    if (!workItem) {{
                        throw new Error(`Work item with ID {} not found`);
                    }}
                    
                    const fields = workItem.fields || {{}};
                    const formattedWorkItem = {{
                        id: workItem.id,
                        work_item_type: fields['System.WorkItemType'] || 'Unknown',
                        title: fields['System.Title'] || 'Untitled',
                        state: fields['System.State'] || 'Unknown',
                        created_by: fields['System.CreatedBy'] ? fields['System.CreatedBy'].displayName : null,
                        assigned_to: fields['System.AssignedTo'] ? fields['System.AssignedTo'].displayName : null,
                        tags: fields['System.Tags'] ? fields['System.Tags'].split(';').map(t => t.trim()) : [],
                        fields: fields
                    }};
                    
                    return {{ workItem: formattedWorkItem }};
                }} catch (error) {{
                    throw new Error(`Failed to update work item: ${{error.message}}`);
                }}
            }}
            
            return await updateWorkItem();
        "#, operations_json, id, project, id);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse work item from response
        let content = Self::extract_content_as_json(&response)?;
        
        let work_item_data = content.get("workItem")
            .ok_or_else(|| Error::protocol("Missing 'workItem' field in response".to_string()))?;
            
        let work_item: WorkItem = serde_json::from_value(work_item_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse work item: {}", e)))?;
            
        Ok(work_item)
    }

    /// Azure DevOps build and release methods
    /// List build definitions
    pub async fn list_build_definitions(&self, project: &str) -> Result<Vec<BuildDefinition>> {
        let script = format!(r#"
            // List build definitions
            async function listBuildDefinitions() {{
                try {{
                    const buildClient = getClient('BuildRestClient');
                    const definitions = await buildClient.getDefinitions('{}');
                    
                    if (!definitions || !definitions.length) {{
                        return {{ definitions: [] }};
                    }}
                    
                    const formattedDefinitions = definitions.map(def => {{
                        return {{
                            id: def.id,
                            name: def.name,
                            path: def.path || '\\',
                            queue_status: def.queueStatus || 'enabled',
                            repository: def.repository ? {{
                                id: def.repository.id,
                                name: def.repository.name,
                                repository_type: def.repository.type,
                                url: def.repository.url
                            }} : null
                        }};
                    }});
                    
                    return {{ definitions: formattedDefinitions }};
                }} catch (error) {{
                    throw new Error(`Failed to list build definitions: ${{error.message}}`);
                }}
            }}
            
            return await listBuildDefinitions();
        "#, project);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse build definitions from response
        let content = Self::extract_content_as_json(&response)?;
        
        let definitions_data = content.get("definitions")
            .ok_or_else(|| Error::protocol("Missing 'definitions' field in response".to_string()))?;
            
        let definitions: Vec<BuildDefinition> = serde_json::from_value(definitions_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse build definitions: {}", e)))?;
            
        Ok(definitions)
    }
    
    /// Get a build definition
    pub async fn get_build_definition(&self, project: &str, definition_id: i32) -> Result<BuildDefinition> {
        let script = format!(r#"
            // Get a build definition
            async function getBuildDefinition() {{
                try {{
                    const buildClient = getClient('BuildRestClient');
                    const definition = await buildClient.getDefinition('{}', {});
                    
                    if (!definition) {{
                        throw new Error(`Build definition with ID {} not found`);
                    }}
                    
                    const formattedDefinition = {{
                        id: definition.id,
                        name: definition.name,
                        path: definition.path || '\\',
                        queue_status: definition.queueStatus || 'enabled',
                        repository: definition.repository ? {{
                            id: definition.repository.id,
                            name: definition.repository.name,
                            repository_type: definition.repository.type,
                            url: definition.repository.url
                        }} : null
                    }};
                    
                    return {{ definition: formattedDefinition }};
                }} catch (error) {{
                    throw new Error(`Failed to get build definition: ${{error.message}}`);
                }}
            }}
            
            return await getBuildDefinition();
        "#, project, definition_id, definition_id);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse build definition from response
        let content = Self::extract_content_as_json(&response)?;
        
        let definition_data = content.get("definition")
            .ok_or_else(|| Error::protocol("Missing 'definition' field in response".to_string()))?;
            
        let definition: BuildDefinition = serde_json::from_value(definition_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse build definition: {}", e)))?;
            
        Ok(definition)
    }
    
    /// Queue a new build
    pub async fn queue_build(&self, project: &str, definition_id: i32, source_branch: Option<&str>, parameters: Option<HashMap<String, Value>>) -> Result<Build> {
        // Create build parameters
        let mut build_params = json!({
            "definition": {
                "id": definition_id
            }
        });
        
        if let Some(branch) = source_branch {
            build_params["sourceBranch"] = json!(branch);
        }
        
        if let Some(params) = parameters {
            build_params["parameters"] = serde_json::to_value(params)
                .map_err(|e| Error::internal(format!("Failed to serialize parameters: {}", e)))?;
        }
        
        let build_params_json = serde_json::to_string(&build_params)
            .map_err(|e| Error::internal(format!("Failed to serialize build parameters: {}", e)))?;
        
        let script = format!(r#"
            // Queue a new build
            async function queueBuild() {{
                try {{
                    const buildParams = {};
                    
                    const buildClient = getClient('BuildRestClient');
                    const build = await buildClient.queueBuild(buildParams, '{}');
                    
                    if (!build) {{
                        throw new Error('Failed to queue build');
                    }}
                    
                    const formattedBuild = {{
                        id: build.id,
                        build_number: build.buildNumber,
                        status: build.status,
                        result: build.result,
                        definition: {{
                            id: build.definition.id,
                            name: build.definition.name,
                            path: build.definition.path || '\\',
                            queue_status: 'enabled',
                            repository: null
                        }},
                        started_on: build.startTime,
                        finished_on: build.finishTime,
                        requested_by: build.requestedBy ? build.requestedBy.displayName : null,
                        source_branch: build.sourceBranch
                    }};
                    
                    return {{ build: formattedBuild }};
                }} catch (error) {{
                    throw new Error(`Failed to queue build: ${{error.message}}`);
                }}
            }}
            
            return await queueBuild();
        "#, build_params_json, project);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse build from response
        let content = Self::extract_content_as_json(&response)?;
        
        let build_data = content.get("build")
            .ok_or_else(|| Error::protocol("Missing 'build' field in response".to_string()))?;
            
        let build: Build = serde_json::from_value(build_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse build: {}", e)))?;
            
        Ok(build)
    }
    
    /// List builds
    pub async fn list_builds(&self, project: &str, params: Option<BuildQueryParams>) -> Result<Vec<Build>> {
        // Convert params to query parameters
        let mut query_params = Vec::new();
        
        if let Some(p) = &params {
            if let Some(def_id) = p.definition_id {
                query_params.push(format!("definitions={}", def_id));
            }
            
            if let Some(branch) = &p.branch {
                query_params.push(format!("branchName={}", branch));
            }
            
            if let Some(status) = &p.status_filter {
                query_params.push(format!("statusFilter={}", status));
            }
            
            if let Some(result) = &p.result_filter {
                query_params.push(format!("resultFilter={}", result));
            }
            
            if let Some(top) = p.top {
                query_params.push(format!("$top={}", top));
            }
        }
        
        let _query_string = if query_params.is_empty() {
            "".to_string()
        } else {
            format!("&{}", query_params.join("&"))
        };
        
        let script = format!(r#"
            // List builds
            async function listBuilds() {{
                try {{
                    const buildClient = getClient('BuildRestClient');
                    const builds = await buildClient.getBuilds('{}', null /* definitions */, null /* queues */, 
                        null /* buildNumber */, null /* minFinishTime */, null /* maxFinishTime */, 
                        null /* requestedFor */, null /* reasonFilter */, null /* statusFilter */, 
                        null /* resultFilter */, null /* tagFilters */, null /* properties */, 
                        null /* top */, null /* continuationToken */, null /* maxBuildsPerDefinition */, 
                        null /* deletedFilter */, null /* queryOrder */, null /* branchName */,
                        null /* buildIds */, null /* repositoryId */, null /* repositoryType */);
                    
                    if (!builds || !builds.length) {{
                        return {{ builds: [] }};
                    }}
                    
                    const formattedBuilds = builds.map(build => {{
                        return {{
                            id: build.id,
                            build_number: build.buildNumber,
                            status: build.status,
                            result: build.result,
                            definition: {{
                                id: build.definition.id,
                                name: build.definition.name,
                                path: build.definition.path || '\\',
                                queue_status: 'enabled',
                                repository: null
                            }},
                            started_on: build.startTime,
                            finished_on: build.finishTime,
                            requested_by: build.requestedBy ? build.requestedBy.displayName : null,
                            source_branch: build.sourceBranch
                        }};
                    }});
                    
                    return {{ builds: formattedBuilds }};
                }} catch (error) {{
                    throw new Error(`Failed to list builds: ${{error.message}}`);
                }}
            }}
            
            return await listBuilds();
        "#, project);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse builds from response
        let content = Self::extract_content_as_json(&response)?;
        
        let builds_data = content.get("builds")
            .ok_or_else(|| Error::protocol("Missing 'builds' field in response".to_string()))?;
            
        let builds: Vec<Build> = serde_json::from_value(builds_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse builds: {}", e)))?;
            
        Ok(builds)
    }
    
    /// List release definitions
    pub async fn list_release_definitions(&self, project: &str) -> Result<Vec<ReleaseDefinition>> {
        let script = format!(r#"
            // List release definitions
            async function listReleaseDefinitions() {{
                try {{
                    const releaseClient = getClient('ReleaseRestClient');
                    const definitions = await releaseClient.getReleaseDefinitions('{}');
                    
                    if (!definitions || !definitions.length) {{
                        return {{ definitions: [] }};
                    }}
                    
                    const formattedDefinitions = definitions.map(def => {{
                        return {{
                            id: def.id,
                            name: def.name,
                            path: def.path || '\\',
                            release_name_format: def.releaseNameFormat
                        }};
                    }});
                    
                    return {{ definitions: formattedDefinitions }};
                }} catch (error) {{
                    throw new Error(`Failed to list release definitions: ${{error.message}}`);
                }}
            }}
            
            return await listReleaseDefinitions();
        "#, project);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse release definitions from response
        let content = Self::extract_content_as_json(&response)?;
        
        let definitions_data = content.get("definitions")
            .ok_or_else(|| Error::protocol("Missing 'definitions' field in response".to_string()))?;
            
        let definitions: Vec<ReleaseDefinition> = serde_json::from_value(definitions_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse release definitions: {}", e)))?;
            
        Ok(definitions)
    }
    
    /// Create a release
    pub async fn create_release(&self, project: &str, definition_id: i32, description: Option<&str>, artifacts: Option<Vec<Value>>) -> Result<Release> {
        // Create release parameters
        let mut release_params = json!({
            "definitionId": definition_id,
            "isDraft": false,
            "reason": "none"
        });
        
        if let Some(desc) = description {
            release_params["description"] = json!(desc);
        }
        
        if let Some(arts) = artifacts {
            release_params["artifacts"] = json!(arts);
        }
        
        let release_params_json = serde_json::to_string(&release_params)
            .map_err(|e| Error::internal(format!("Failed to serialize release parameters: {}", e)))?;
        
        let script = format!(r#"
            // Create a release
            async function createRelease() {{
                try {{
                    const releaseParams = {};
                    
                    const releaseClient = getClient('ReleaseRestClient');
                    const release = await releaseClient.createRelease(releaseParams, '{}');
                    
                    if (!release) {{
                        throw new Error('Failed to create release');
                    }}
                    
                    const formattedRelease = {{
                        id: release.id,
                        name: release.name,
                        status: release.status,
                        created_on: release.createdOn,
                        created_by: release.createdBy ? release.createdBy.displayName : null,
                        definition: {{
                            id: release.releaseDefinition.id,
                            name: release.releaseDefinition.name,
                            path: release.releaseDefinition.path || '\\',
                            release_name_format: release.releaseDefinition.releaseNameFormat || ''
                        }},
                        description: release.description
                    }};
                    
                    return {{ release: formattedRelease }};
                }} catch (error) {{
                    throw new Error(`Failed to create release: ${{error.message}}`);
                }}
            }}
            
            return await createRelease();
        "#, release_params_json, project);
        
        let response = self.execute_resource_script(&script).await?;
        
        // Parse release from response
        let content = Self::extract_content_as_json(&response)?;
        
        let release_data = content.get("release")
            .ok_or_else(|| Error::protocol("Missing 'release' field in response".to_string()))?;
            
        let release: Release = serde_json::from_value(release_data.clone())
            .map_err(|e| Error::protocol(format!("Failed to parse release: {}", e)))?;
            
        Ok(release)
    }
} 