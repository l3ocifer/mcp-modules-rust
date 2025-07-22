use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use reqwest::Client;
use std::collections::HashMap;

/// Coordinate point (longitude, latitude)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    /// Longitude
    pub lon: f64,
    /// Latitude
    pub lat: f64,
}

/// Bounding box for geographic queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    /// Minimum longitude
    pub min_lon: f64,
    /// Minimum latitude
    pub min_lat: f64,
    /// Maximum longitude
    pub max_lon: f64,
    /// Maximum latitude
    pub max_lat: f64,
}

/// OSM node representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Node ID
    pub id: i64,
    /// Latitude
    pub lat: f64,
    /// Longitude
    pub lon: f64,
    /// Tags (key-value pairs)
    pub tags: HashMap<String, String>,
}

/// OSM way representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Way {
    /// Way ID
    pub id: i64,
    /// Node IDs
    pub nodes: Vec<i64>,
    /// Tags (key-value pairs)
    pub tags: HashMap<String, String>,
    /// Whether the way is closed (first node == last node)
    pub is_closed: bool,
}

/// OSM relation representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    /// Relation ID
    pub id: i64,
    /// Members
    pub members: Vec<RelationMember>,
    /// Tags (key-value pairs)
    pub tags: HashMap<String, String>,
}

/// OSM relation member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationMember {
    /// Member type
    pub member_type: String,
    /// Member reference
    pub ref_: i64,
    /// Member role
    pub role: String,
}

/// Query result from OSM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmQueryResult {
    /// Nodes
    pub nodes: Vec<Node>,
    /// Ways
    pub ways: Vec<Way>,
    /// Relations
    pub relations: Vec<Relation>,
}

/// OpenStreetMap postgresql column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresColumn {
    /// Column name
    pub column_name: String,
    /// Data type
    pub data_type: String,
    /// Whether the column is nullable
    pub is_nullable: String,
}

/// OpenStreetMap postgresql index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresIndex {
    /// Index name
    pub indexname: String,
    /// Index definition
    pub indexdef: String,
}

/// PostgreSQL table information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresTableInfo {
    /// Table name
    pub name: String,
    /// Columns
    pub columns: Vec<PostgresColumn>,
    /// Indexes
    pub indexes: Vec<PostgresIndex>,
    /// Approximate row count
    pub approximate_row_count: i64,
}

/// Query parameters for raw SQL queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlQueryParams {
    /// SQL query
    pub query: String,
    /// Query parameters
    pub params: Option<HashMap<String, Value>>,
    /// Maximum number of rows to return
    pub max_rows: Option<usize>,
}

/// Client for OpenStreetMap data
pub struct OsmClient<'a> {
    /// Lifecycle manager
    #[allow(dead_code)]
    lifecycle: &'a LifecycleManager,
    /// HTTP client
    client: Client,
    /// Base URL for Overpass API
    overpass_url: String,
    /// PostgreSQL connection info
    pg_host: Option<String>,
    /// PostgreSQL port
    pg_port: Option<u16>,
    /// PostgreSQL database name
    pg_db: Option<String>,
    /// PostgreSQL username
    pg_user: Option<String>,
    /// PostgreSQL password
    pg_password: Option<String>,
}

impl<'a> OsmClient<'a> {
    /// Create a new OpenStreetMap client
    pub fn new(lifecycle: &'a LifecycleManager) -> Self {
        Self {
            lifecycle,
            client: Client::new(),
            overpass_url: "https://overpass-api.de/api/interpreter".to_string(),
            pg_host: None,
            pg_port: None,
            pg_db: None,
            pg_user: None,
            pg_password: None,
        }
    }

    /// Set Overpass API URL
    pub fn with_overpass_url(mut self, url: impl Into<String>) -> Self {
        self.overpass_url = url.into();
        self
    }

    /// Set PostgreSQL connection info
    pub fn with_postgres_connection(
        mut self,
        host: impl Into<String>,
        port: u16,
        db: impl Into<String>,
        user: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.pg_host = Some(host.into());
        self.pg_port = Some(port);
        self.pg_db = Some(db.into());
        self.pg_user = Some(user.into());
        self.pg_password = Some(password.into());
        self
    }

    /// Query OpenStreetMap data using Overpass QL with performance optimizations
    pub async fn query_overpass(&self, overpass_query: &str) -> Result<OsmQueryResult> {
        let response = self.client
            .post(&self.overpass_url)
            .body(overpass_query.to_string()) // Convert to owned string to fix lifetime
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to query Overpass API: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Unable to read error response".into());
            return Err(Error::network(format!("Overpass API returned error {}: {}", status, text)));
        }
        
        let data: Value = response.json()
            .await
            .map_err(|e| Error::parsing(format!("Failed to parse Overpass response: {}", e)))?;
            
        // Parse elements with pre-allocated collections
        let elements = data.get("elements")
            .and_then(|e| e.as_array())
            .ok_or_else(|| Error::parsing("Invalid Overpass response format"))?;
            
        // Pre-allocate vectors with estimated capacity based on element count
        let element_count = elements.len();
        let mut nodes = Vec::with_capacity(element_count / 2); // Estimate 50% nodes
        let mut ways = Vec::with_capacity(element_count / 3);  // Estimate 33% ways  
        let mut relations = Vec::with_capacity(element_count / 10); // Estimate 10% relations
        
        for element in elements {
            let element_type = element.get("type")
                .and_then(|t| t.as_str())
                .ok_or_else(|| Error::parsing("Element missing type"))?;
                
            match element_type {
                "node" => {
                    let id = element.get("id")
                        .and_then(|id| id.as_i64())
                        .ok_or_else(|| Error::parsing("Node missing ID"))?;
                        
                    let lat = element.get("lat")
                        .and_then(|lat| lat.as_f64())
                        .ok_or_else(|| Error::parsing("Node missing latitude"))?;
                        
                    let lon = element.get("lon")
                        .and_then(|lon| lon.as_f64())
                        .ok_or_else(|| Error::parsing("Node missing longitude"))?;
                        
                    // Optimize tag parsing to avoid unnecessary allocations
                    let tags = element.get("tags")
                        .and_then(|tags| tags.as_object())
                        .map(|obj| {
                            let mut tag_map = HashMap::with_capacity(obj.len());
                            tag_map.extend(
                                obj.iter()
                                    .filter_map(|(k, v)| {
                                        v.as_str().map(|s| (k.clone(), s.to_string()))
                                    })
                            );
                            tag_map
                        })
                        .unwrap_or_else(|| HashMap::with_capacity(0));
                        
                    nodes.push(Node { id, lat, lon, tags });
                },
                "way" => {
                    let id = element.get("id")
                        .and_then(|id| id.as_i64())
                        .ok_or_else(|| Error::parsing("Way missing ID"))?;
                        
                    let nodes_array = element.get("nodes")
                        .and_then(|nodes| nodes.as_array())
                        .ok_or_else(|| Error::parsing("Way missing nodes"))?;
                        
                    // Pre-allocate way nodes vector
                    let mut way_nodes = Vec::with_capacity(nodes_array.len());
                    way_nodes.extend(
                        nodes_array
                            .iter()
                            .filter_map(|n| n.as_i64())
                    );
                        
                    // Optimize tag parsing with pre-allocation
                    let tags = element.get("tags")
                        .and_then(|tags| tags.as_object())
                        .map(|obj| {
                            let mut tag_map = HashMap::with_capacity(obj.len());
                            tag_map.extend(
                                obj.iter()
                                    .filter_map(|(k, v)| {
                                        v.as_str().map(|s| (k.clone(), s.to_string()))
                                    })
                            );
                            tag_map
                        })
                        .unwrap_or_else(|| HashMap::with_capacity(0));
                        
                    let is_closed = !way_nodes.is_empty() && way_nodes.first() == way_nodes.last();
                    ways.push(Way { id, nodes: way_nodes, tags, is_closed });
                },
                "relation" => {
                    let id = element.get("id")
                        .and_then(|id| id.as_i64())
                        .ok_or_else(|| Error::parsing("Relation missing ID"))?;
                        
                    let members_array = element.get("members")
                        .and_then(|members| members.as_array())
                        .ok_or_else(|| Error::parsing("Relation missing members"))?;
                        
                    // Pre-allocate members vector
                    let mut members = Vec::with_capacity(members_array.len());
                    members.extend(
                        members_array.iter()
                            .filter_map(|m| {
                                let member_type = m.get("type")?.as_str()?;
                                let ref_ = m.get("ref")?.as_i64()?;
                                let role = m.get("role")?.as_str()?;
                                
                                Some(RelationMember {
                                    member_type: member_type.to_string(),
                                    ref_,
                                    role: role.to_string(),
                                })
                            })
                    );
                        
                    // Optimize tag parsing with pre-allocation
                    let tags = element.get("tags")
                        .and_then(|tags| tags.as_object())
                        .map(|obj| {
                            let mut tag_map = HashMap::with_capacity(obj.len());
                            tag_map.extend(
                                obj.iter()
                                    .filter_map(|(k, v)| {
                                        v.as_str().map(|s| (k.clone(), s.to_string()))
                                    })
                            );
                            tag_map
                        })
                        .unwrap_or_else(|| HashMap::with_capacity(0));
                        
                    relations.push(Relation { id, members, tags });
                },
                _ => continue,
            }
        }
        
        Ok(OsmQueryResult {
            nodes,
            ways,
            relations,
        })
    }
    
    /// Find points of interest near a location
    pub async fn find_pois_near_location(&self, lon: f64, lat: f64, radius_meters: u32) -> Result<Vec<Node>> {
        let overpass_query = format!(r#"
            [out:json];
            (
              node["amenity"](around:{radius_meters},{lat},{lon});
              node["tourism"](around:{radius_meters},{lat},{lon});
              node["shop"](around:{radius_meters},{lat},{lon});
              node["leisure"](around:{radius_meters},{lat},{lon});
              node["historic"](around:{radius_meters},{lat},{lon});
            );
            out body;
        "#);
        
        let result = self.query_overpass(&overpass_query).await?;
        Ok(result.nodes)
    }
    
    /// Search for named places
    pub async fn search_places(&self, query: &str, bbox: Option<BoundingBox>) -> Result<Vec<Node>> {
        let bbox_str = if let Some(bbox) = bbox {
            format!("({},{},{},{})", 
                bbox.min_lat, bbox.min_lon, bbox.max_lat, bbox.max_lon)
        } else {
            "".to_string()
        };
        
        let overpass_query = format!(r#"
            [out:json];
            (
              node["name"~"{query}",i]{bbox_str};
              way["name"~"{query}",i]{bbox_str};
              relation["name"~"{query}",i]{bbox_str};
            );
            out center;
        "#);
        
        let result = self.query_overpass(&overpass_query).await?;
        Ok(result.nodes)
    }
    
    /// Get routes between points
    pub async fn get_route(&self, from: Point, to: Point, transport_mode: &str) -> Result<String> {
        // This would typically use a routing service like OSRM
        // For now, we'll return a placeholder
        Err(Error::service(format!(
            "Routing functionality requires a routing service like OSRM. 
            Consider using the points ({}, {}) to ({}, {}) with mode {} 
            to query an external routing service.",
            from.lon, from.lat, to.lon, to.lat, transport_mode
        )))
    }
    
    /// Generate an image of a map
    pub async fn generate_map_image(&self, 
        center: Point, 
        zoom: u8, 
        width: u32, 
        height: u32
    ) -> Result<Vec<u8>> {
        // This would typically use a service like Mapbox or OpenStreetMap tiles
        // For now, we'll return a placeholder
        Err(Error::service(format!(
            "Map image generation requires a map rendering service. 
            Consider using the center point ({}, {}) with zoom level {} 
            and dimensions {}x{} to query an external map service.",
            center.lon, center.lat, zoom, width, height
        )))
    }

    /// Get registered tools
    pub fn get_tools(&self) -> Vec<(String, String, serde_json::Value)> {
        vec![
            (
                "query_overpass".to_string(),
                "Query OpenStreetMap data using Overpass QL".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["query"],
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Overpass QL query"
                        }
                    }
                }),
            ),
            (
                "find_pois_near_location".to_string(),
                "Find points of interest near a location".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["longitude", "latitude", "radius"],
                    "properties": {
                        "longitude": {
                            "type": "number",
                            "description": "Longitude of the center point"
                        },
                        "latitude": {
                            "type": "number",
                            "description": "Latitude of the center point"
                        },
                        "radius": {
                            "type": "integer",
                            "description": "Radius in meters"
                        }
                    }
                }),
            ),
            (
                "search_places".to_string(),
                "Search for named places".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["query"],
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "min_lon": {
                            "type": "number",
                            "description": "Minimum longitude for bounding box"
                        },
                        "min_lat": {
                            "type": "number",
                            "description": "Minimum latitude for bounding box"
                        },
                        "max_lon": {
                            "type": "number",
                            "description": "Maximum longitude for bounding box"
                        },
                        "max_lat": {
                            "type": "number",
                            "description": "Maximum latitude for bounding box"
                        }
                    }
                }),
            ),
            (
                "get_route".to_string(),
                "Get route between two points".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["from_lon", "from_lat", "to_lon", "to_lat"],
                    "properties": {
                        "from_lon": {
                            "type": "number",
                            "description": "Longitude of the starting point"
                        },
                        "from_lat": {
                            "type": "number",
                            "description": "Latitude of the starting point"
                        },
                        "to_lon": {
                            "type": "number",
                            "description": "Longitude of the destination point"
                        },
                        "to_lat": {
                            "type": "number",
                            "description": "Latitude of the destination point"
                        },
                        "transport_mode": {
                            "type": "string",
                            "description": "Mode of transport (car, bicycle, foot)"
                        }
                    }
                }),
            ),
            (
                "generate_map_image".to_string(),
                "Generate an image of a map".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["center_lon", "center_lat", "zoom"],
                    "properties": {
                        "center_lon": {
                            "type": "number",
                            "description": "Longitude of the center point"
                        },
                        "center_lat": {
                            "type": "number",
                            "description": "Latitude of the center point"
                        },
                        "zoom": {
                            "type": "integer",
                            "description": "Zoom level (0-19)"
                        },
                        "width": {
                            "type": "integer",
                            "description": "Image width in pixels"
                        },
                        "height": {
                            "type": "integer",
                            "description": "Image height in pixels"
                        }
                    }
                }),
            ),
        ]
    }
} 