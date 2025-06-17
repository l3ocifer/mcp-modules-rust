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
    pub async fn get_memory(&self, id: &str) -> Result<Memory> {
        self.memories.get(id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Memory with ID '{}' not found", id)))
    }
    
    /// Update an existing memory
    pub async fn update_memory(&mut self, 
        id: &str,
        title: Option<String>, 
        content: Option<String>,
        metadata: Option<HashMap<String, Value>>,
    ) -> Result<()> {
        let memory = self.memories.get_mut(id)
            .ok_or_else(|| Error::NotFound(format!("Memory with ID '{}' not found", id)))?;
            
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
            return Err(Error::NotFound(format!("Memory with ID '{}' not found", id)));
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
            return Err(Error::NotFound(format!("Source memory with ID '{}' not found", from_id)));
        }
        
        if !self.memories.contains_key(to_id) {
            return Err(Error::NotFound(format!("Target memory with ID '{}' not found", to_id)));
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
    
    /// Search for memories
    pub async fn search_memories(&self, params: MemorySearchParams) -> Result<Vec<Memory>> {
        let mut results: Vec<Memory> = self.memories.values().cloned().collect();
        
        // Filter by memory type
        if let Some(memory_type) = params.memory_type {
            results.retain(|m| m.memory_type == memory_type);
        }
        
        // Filter by keyword
        if let Some(keyword) = params.keyword {
            let keyword = keyword.to_lowercase();
            results.retain(|m| {
                m.title.to_lowercase().contains(&keyword) || 
                m.content.to_lowercase().contains(&keyword)
            });
        }
        
        // Filter by metadata
        if let Some(metadata_filters) = params.metadata_filters {
            for (key, value) in metadata_filters {
                results.retain(|m| {
                    m.metadata.get(&key)
                        .map(|v| v == &value)
                        .unwrap_or(false)
                });
            }
        }
        
        // Apply limit
        if let Some(limit) = params.limit {
            results.truncate(limit);
        }
        
        Ok(results)
    }
    
    /// Get all relationships for a memory
    pub async fn get_relationships(&self, memory_id: &str) -> Result<Vec<Relationship>> {
        if !self.memories.contains_key(memory_id) {
            return Err(Error::NotFound(format!("Memory with ID '{}' not found", memory_id)));
        }
        
        let relationships: Vec<Relationship> = self.relationships.iter()
            .filter(|r| r.from_id == memory_id || r.to_id == memory_id)
            .cloned()
            .collect();
            
        Ok(relationships)
    }
    
    /// Get related memories
    pub async fn get_related_memories(&self, memory_id: &str) -> Result<Vec<Memory>> {
        let relationships = self.get_relationships(memory_id).await?;
        
        let related_ids: Vec<String> = relationships.iter()
            .flat_map(|r| {
                if r.from_id == memory_id {
                    vec![r.to_id.clone()]
                } else {
                    vec![r.from_id.clone()]
                }
            })
            .collect();
            
        let related_memories: Vec<Memory> = related_ids.iter()
            .filter_map(|id| self.memories.get(id).cloned())
            .collect();
            
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