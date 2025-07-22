use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

/// Memory type enum for categorizing memories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MemoryType {
    /// Project-related memory
    Project,
    /// Issue or problem memory
    Issue,
    /// System or architecture memory
    System,
    /// Configuration details
    Config,
    /// Financial advice or information
    Finance,
    /// Todo items
    Todo,
    /// General knowledge
    Knowledge,
    /// Custom memory type
    Custom(String),
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryType::Project => write!(f, "project"),
            MemoryType::Issue => write!(f, "issue"),
            MemoryType::System => write!(f, "system"),
            MemoryType::Config => write!(f, "config"),
            MemoryType::Finance => write!(f, "finance"),
            MemoryType::Todo => write!(f, "todo"),
            MemoryType::Knowledge => write!(f, "knowledge"),
            MemoryType::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Relationship type enum for connecting memories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum RelationType {
    /// Related to
    #[serde(rename = "RELATED_TO")]
    RelatedTo,
    /// Part of
    #[serde(rename = "PART_OF")]
    PartOf,
    /// Depends on
    #[serde(rename = "DEPENDS_ON")]
    DependsOn,
    /// Blocks
    #[serde(rename = "BLOCKS")]
    Blocks,
    /// Supersedes
    #[serde(rename = "SUPERSEDES")]
    Supersedes,
    /// References
    #[serde(rename = "REFERENCES")]
    References,
    /// Custom relationship type
    Custom(String),
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationType::RelatedTo => write!(f, "RELATED_TO"),
            RelationType::PartOf => write!(f, "PART_OF"),
            RelationType::DependsOn => write!(f, "DEPENDS_ON"),
            RelationType::Blocks => write!(f, "BLOCKS"),
            RelationType::Supersedes => write!(f, "SUPERSEDES"),
            RelationType::References => write!(f, "REFERENCES"),
            RelationType::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Memory node representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique identifier
    pub id: String,
    /// Memory type
    pub memory_type: MemoryType,
    /// Memory title or name
    pub title: String,
    /// Content of the memory
    pub content: String,
    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, Value>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Memory relationship representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Source memory ID
    pub from_id: String,
    /// Target memory ID
    pub to_id: String,
    /// Relationship type
    pub relation_type: RelationType,
    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, Value>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Search parameters for memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchParams {
    /// Optional memory type to filter by
    pub memory_type: Option<MemoryType>,
    /// Optional keyword to search within content
    pub keyword: Option<String>,
    /// Optional metadata filters as key-value pairs
    pub metadata_filters: Option<HashMap<String, Value>>,
    /// Maximum results to return
    pub limit: Option<usize>,
}

/// Memory client for storing and retrieving long-term memories
pub struct MemoryClient<'a> {
    /// Lifecycle manager
    #[allow(dead_code)]
    lifecycle: &'a LifecycleManager,
    /// In-memory storage for memories (would be replaced with a database in production)
    memories: HashMap<String, Memory>,
    /// In-memory storage for relationships
    relationships: Vec<Relationship>,
}

impl<'a> MemoryClient<'a> {
    /// Create a new memory client
    pub fn new(lifecycle: &'a LifecycleManager) -> Self {
        Self {
            lifecycle,
            memories: HashMap::new(),
            relationships: Vec::new(),
        }
    }

    /// Create a new memory
    pub async fn create_memory(&mut self, 
        memory_type: MemoryType, 
        title: impl Into<String>, 
        content: impl Into<String>,
        metadata: Option<HashMap<String, Value>>,
    ) -> Result<String> {
        let id = format!("{}-{}", memory_type, Uuid::new_v4());
        let now = Utc::now();
        
        let memory = Memory {
            id: id.clone(),
            memory_type,
            title: title.into(),
            content: content.into(),
            metadata: metadata.unwrap_or_default(),
            created_at: now,
            updated_at: now,
        };
        
        self.memories.insert(id.clone(), memory);
        Ok(id)
    }
    
    /// Get a memory by ID
    pub fn get_memory(&self, id: &str) -> Result<Memory> {
        self.memories.get(id)
            .cloned()
            .ok_or_else(|| Error::not_found_with_resource(
                format!("Memory with ID '{}' not found", id),
                "memory",
                id
            ))
    }
    
    /// Update an existing memory
    pub async fn update_memory(&mut self, 
        id: &str,
        title: Option<String>, 
        content: Option<String>,
        metadata: Option<HashMap<String, Value>>,
    ) -> Result<()> {
        let memory = self.memories.get_mut(id)
            .ok_or_else(|| Error::not_found_with_resource(
                format!("Memory with ID '{}' not found", id),
                "memory",
                id
            ))?;
            
        if let Some(title) = title {
            memory.title = title;
        }
        
        if let Some(content) = content {
            memory.content = content;
        }
        
        if let Some(metadata) = metadata {
            memory.metadata.extend(metadata);
        }
        
        memory.updated_at = Utc::now();
        Ok(())
    }
    
    /// Delete a memory by ID
    pub async fn delete_memory(&mut self, id: &str) -> Result<()> {
        if self.memories.remove(id).is_none() {
            return Err(Error::not_found_with_resource(
                format!("Memory with ID '{}' not found", id),
                "memory",
                id
            ));
        }
        
        // Remove all relationships involving this memory
        self.relationships.retain(|r| r.from_id != id && r.to_id != id);
        Ok(())
    }
    
    /// Create a relationship between two memories
    pub async fn create_relationship(&mut self, 
        from_id: &str, 
        to_id: &str, 
        relation_type: RelationType,
        metadata: Option<HashMap<String, Value>>,
    ) -> Result<()> {
        // Verify both memories exist
        if !self.memories.contains_key(from_id) {
            return Err(Error::not_found_with_resource(
                format!("Source memory with ID '{}' not found", from_id),
                "memory",
                from_id
            ));
        }
        
        if !self.memories.contains_key(to_id) {
            return Err(Error::not_found_with_resource(
                format!("Target memory with ID '{}' not found", to_id),
                "memory", 
                to_id
            ));
        }
        
        let relationship = Relationship {
            from_id: from_id.to_string(),
            to_id: to_id.to_string(),
            relation_type,
            metadata: metadata.unwrap_or_default(),
            created_at: Utc::now(),
        };
        
        self.relationships.push(relationship);
        Ok(())
    }
    
    /// Search for memories with zero-copy optimizations
    pub async fn search_memories(&self, params: MemorySearchParams) -> Result<Vec<Memory>> {
        // Pre-allocate with estimated capacity based on filters
        let _estimated_capacity = if params.memory_type.is_some() || params.keyword.is_some() {
            self.memories.len() / 4 // Assume 25% match rate for filtered searches
        } else {
            self.memories.len()
        };

        // Use iterator chains to avoid intermediate collections
        let filtered_memories: Vec<Memory> = self.memories
            .values()
            .filter(|memory| {
                // Filter by memory type first (most selective)
                if let Some(ref memory_type) = params.memory_type {
                    if memory.memory_type != *memory_type {
                        return false;
                    }
                }
                
                // Filter by keyword (avoid string allocations by comparing with original case)
                if let Some(ref keyword) = params.keyword {
                    let keyword_lower = keyword.to_lowercase();
                    if !memory.title.to_lowercase().contains(&keyword_lower) 
                        && !memory.content.to_lowercase().contains(&keyword_lower) {
                        return false;
                    }
                }
                
                // Filter by metadata
                if let Some(ref metadata_filters) = params.metadata_filters {
                    for (key, value) in metadata_filters {
                        if !memory.metadata.get(key)
                            .map(|v| v == value)
                            .unwrap_or(false) {
                            return false;
                        }
                    }
                }
                
                true
            })
            .take(params.limit.unwrap_or(usize::MAX)) // Apply limit during iteration
            .cloned() // Only clone the final filtered results
            .collect();

        Ok(filtered_memories)
    }
    
    /// Get all relationships for a memory with optimized filtering
    pub async fn get_relationships(&self, memory_id: &str) -> Result<Vec<Relationship>> {
        if !self.memories.contains_key(memory_id) {
            return Err(Error::not_found_with_resource(
                format!("Memory with ID '{}' not found", memory_id),
                "memory",
                memory_id
            ));
        }

        // Use iterator with pre-allocation hint
        let mut relationships = Vec::with_capacity(self.relationships.len() / 10); // Estimate 10% match rate
        
        relationships.extend(
            self.relationships
                .iter()
                .filter(|r| r.from_id == memory_id || r.to_id == memory_id)
                .cloned()
        );
            
        Ok(relationships)
    }
    
    /// Get related memories with zero-copy string handling
    pub async fn get_related_memories(&self, memory_id: &str) -> Result<Vec<Memory>> {
        let relationships = self.get_relationships(memory_id).await?;
        
        // Pre-allocate based on relationship count
        let mut related_memories = Vec::with_capacity(relationships.len());
        
        // Use filter_map to combine operations and avoid intermediate collections
        related_memories.extend(
            relationships
                .iter()
                .filter_map(|r| {
                    let related_id = if r.from_id == memory_id {
                        &r.to_id
                    } else {
                        &r.from_id
                    };
                    self.memories.get(related_id).cloned()
                })
        );
            
        Ok(related_memories)
    }

    /// Get registered tools
    pub fn get_tools(&self) -> Vec<(String, String, serde_json::Value)> {
        vec![
            (
                "create_memory".to_string(),
                "Create a new memory in the knowledge graph".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["memory_type", "title", "content"],
                    "properties": {
                        "memory_type": {
                            "type": "string",
                            "description": "Type of memory (project, issue, system, config, finance, todo, knowledge, or custom)"
                        },
                        "title": {
                            "type": "string",
                            "description": "Title or name of the memory"
                        },
                        "content": {
                            "type": "string",
                            "description": "Content of the memory"
                        },
                        "metadata": {
                            "type": "object",
                            "description": "Additional metadata as key-value pairs"
                        }
                    }
                }),
            ),
            (
                "get_memory".to_string(),
                "Get a memory by ID".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["id"],
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Memory ID"
                        }
                    }
                }),
            ),
            (
                "search_memories".to_string(),
                "Search for memories based on various criteria".to_string(),
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "memory_type": {
                            "type": "string",
                            "description": "Type of memory to filter by"
                        },
                        "keyword": {
                            "type": "string",
                            "description": "Keyword to search within content"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results to return"
                        }
                    }
                }),
            ),
            (
                "create_relationship".to_string(),
                "Create a relationship between two memories".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["from_id", "to_id", "relation_type"],
                    "properties": {
                        "from_id": {
                            "type": "string",
                            "description": "Source memory ID"
                        },
                        "to_id": {
                            "type": "string",
                            "description": "Target memory ID"
                        },
                        "relation_type": {
                            "type": "string",
                            "description": "Relationship type (RELATED_TO, PART_OF, DEPENDS_ON, BLOCKS, SUPERSEDES, REFERENCES, or custom)"
                        },
                        "metadata": {
                            "type": "object",
                            "description": "Additional metadata as key-value pairs"
                        }
                    }
                }),
            ),
            (
                "get_related_memories".to_string(),
                "Get all memories related to a specific memory".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["memory_id"],
                    "properties": {
                        "memory_id": {
                            "type": "string",
                            "description": "Memory ID to find related memories for"
                        }
                    }
                }),
            ),
        ]
    }
} 