use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;
use std::sync::Arc;

pub mod persistence;

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

/// Memory statistics for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatistics {
    /// Total number of memories
    pub total_memories: usize,
    /// Count by memory type
    pub type_counts: HashMap<String, usize>,
    /// Total number of relationships
    pub total_relationships: usize,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Memory client for storing and retrieving long-term memories with persistence
pub struct MemoryClient {
    /// Lifecycle manager
    #[allow(dead_code)]
    lifecycle: Arc<LifecycleManager>,
    /// Persistence backend
    store: Arc<dyn persistence::MemoryStore>,
}

impl MemoryClient {
    /// Create a new memory client with PostgreSQL backend
    pub async fn new_with_postgres(
        lifecycle: Arc<LifecycleManager>, 
        connection_string: String
    ) -> Result<Self> {
        let store = Arc::new(persistence::PostgreSQLMemoryStore::new(connection_string).await?);
        Ok(Self {
            lifecycle,
            store,
        })
    }

    /// Create a new memory client with in-memory backend (for testing/development)
    pub fn new_in_memory(lifecycle: Arc<LifecycleManager>) -> Self {
        let store = Arc::new(persistence::InMemoryStore::new());
        Self {
            lifecycle,
            store,
        }
    }

    /// Backwards compatible constructor (uses in-memory store)
    pub fn new(lifecycle: &LifecycleManager) -> Self {
        Self::new_in_memory(Arc::new(lifecycle.clone()))
    }

    /// Create a new memory
    pub async fn create_memory(
        &self,
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

        self.store.store_memory(&memory).await?;
        Ok(id)
    }

    /// Convenient method to create memory from a Memory struct
    pub async fn create_memory_from_struct(&self, mut memory: Memory) -> Result<String> {
        if memory.id.is_empty() {
            memory.id = format!("{}-{}", memory.memory_type, Uuid::new_v4());
        }
        
        let now = Utc::now();
        if memory.created_at == DateTime::<Utc>::from_timestamp(0, 0).unwrap_or(now) {
            memory.created_at = now;
        }
        memory.updated_at = now;

        self.store.store_memory(&memory).await?;
        Ok(memory.id)
    }

    /// Get a memory by ID
    pub async fn get_memory(&self, id: &str) -> Result<Memory> {
        match self.store.get_memory(id).await? {
            Some(memory) => Ok(memory),
            None => Err(Error::not_found_with_resource(
                format!("Memory with ID '{}' not found", id),
                "memory",
                id,
            ))
        }
    }

    /// Update an existing memory
    pub async fn update_memory(
        &self,
        id: &str,
        title: Option<String>,
        content: Option<String>,
        metadata: Option<HashMap<String, Value>>,
    ) -> Result<()> {
        // Get existing memory
        let mut memory = self.get_memory(id).await?;

        // Update fields
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

        // Store updated memory
        self.store.update_memory(&memory).await?;
        Ok(())
    }

    /// Delete a memory by ID
    pub async fn delete_memory(&self, id: &str) -> Result<()> {
        // Delete relationships first
        self.store.delete_relationships(id).await?;
        
        // Delete the memory
        self.store.delete_memory(id).await?;
        Ok(())
    }

    /// Create a relationship between two memories
    pub async fn create_relationship(
        &self,
        from_id: &str,
        to_id: &str,
        relation_type: RelationType,
        metadata: Option<HashMap<String, Value>>,
    ) -> Result<()> {
        // Verify both memories exist
        self.get_memory(from_id).await.map_err(|_| {
            Error::not_found_with_resource(
                format!("Source memory with ID '{}' not found", from_id),
                "memory",
                from_id,
            )
        })?;

        self.get_memory(to_id).await.map_err(|_| {
            Error::not_found_with_resource(
                format!("Target memory with ID '{}' not found", to_id),
                "memory",
                to_id,
            )
        })?;

        let relationship = Relationship {
            from_id: from_id.to_string(),
            to_id: to_id.to_string(),
            relation_type,
            metadata: metadata.unwrap_or_default(),
            created_at: Utc::now(),
        };

        self.store.store_relationship(&relationship).await?;
        Ok(())
    }

    /// Search for memories with optimized database queries
    pub async fn search_memories(&self, params: MemorySearchParams) -> Result<Vec<Memory>> {
        self.store.search_memories(&params).await
    }

    /// Get all relationships for a memory
    pub async fn get_relationships(&self, memory_id: &str) -> Result<Vec<Relationship>> {
        // Verify memory exists
        self.get_memory(memory_id).await?;
        
        self.store.get_relationships(memory_id).await
    }

    /// Get related memories with optimized queries
    pub async fn get_related_memories(&self, memory_id: &str) -> Result<Vec<Memory>> {
        let relationships = self.get_relationships(memory_id).await?;

        // Pre-allocate based on relationship count
        let mut related_memories = Vec::with_capacity(relationships.len());

        // Use async operations to fetch related memories
        for relationship in relationships {
            let related_id = if relationship.from_id == memory_id {
                &relationship.to_id
            } else {
                &relationship.from_id
            };
            
            if let Ok(memory) = self.get_memory(related_id).await {
                related_memories.push(memory);
            }
        }

        Ok(related_memories)
    }

    /// Get health status of the memory store
    pub async fn health_check(&self) -> Result<bool> {
        self.store.health_check().await
    }

    /// Get memory statistics
    pub async fn get_statistics(&self) -> Result<MemoryStatistics> {
        // For now, we'll implement basic stats
        // In a full implementation, the store would provide efficient counting
        let empty_params = MemorySearchParams {
            memory_type: None,
            keyword: None,
            metadata_filters: None,
            limit: None,
        };
        
        let all_memories = self.store.search_memories(&empty_params).await?;
        let total_memories = all_memories.len();
        
        // Count by type
        let mut type_counts = HashMap::new();
        for memory in &all_memories {
            let count = type_counts.entry(memory.memory_type.to_string()).or_insert(0);
            *count += 1;
        }

        Ok(MemoryStatistics {
            total_memories,
            type_counts,
            total_relationships: 0, // Would need separate query
            last_updated: Utc::now(),
        })
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

