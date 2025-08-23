use crate::error::{Error, Result};
use crate::memory::{Memory, MemoryType, Relationship, RelationType, MemorySearchParams};
use crate::security::SecurityModule;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::Row;

/// Trait for memory persistence backends
#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn store_memory(&self, memory: &Memory) -> Result<()>;
    async fn get_memory(&self, id: &str) -> Result<Option<Memory>>;
    async fn update_memory(&self, memory: &Memory) -> Result<()>;
    async fn delete_memory(&self, id: &str) -> Result<()>;
    async fn search_memories(&self, params: &MemorySearchParams) -> Result<Vec<Memory>>;
    async fn store_relationship(&self, relationship: &Relationship) -> Result<()>;
    async fn get_relationships(&self, memory_id: &str) -> Result<Vec<Relationship>>;
    async fn delete_relationships(&self, memory_id: &str) -> Result<()>;
    async fn health_check(&self) -> Result<bool>;
}

/// PostgreSQL-based memory store with optimized performance
pub struct PostgreSQLMemoryStore {
    pool: sqlx::PgPool,
    security: SecurityModule,
    cache: Arc<RwLock<HashMap<String, Memory>>>,
    cache_size_limit: usize,
}

impl PostgreSQLMemoryStore {
    /// Create a new PostgreSQL memory store
    pub async fn new(connection_string: String) -> Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(&connection_string)
            .await
            .map_err(|e| Error::service(&format!("Failed to connect to PostgreSQL: {}", e)))?;

        // Initialize database schema
        Self::init_schema(&pool).await?;

        Ok(Self {
            pool,
            security: SecurityModule::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_size_limit: 10000, // Cache up to 10k memories
        })
    }

    /// Initialize database schema for memory storage
    async fn init_schema(pool: &sqlx::PgPool) -> Result<()> {
        // Create memories table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS memories (
                id VARCHAR(255) PRIMARY KEY,
                memory_type VARCHAR(50) NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata JSONB DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
        "#)
        .execute(pool)
        .await
        .map_err(|e| Error::service(&format!("Failed to create memories table: {}", e)))?;

        // Create relationships table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS memory_relationships (
                from_id VARCHAR(255) NOT NULL,
                to_id VARCHAR(255) NOT NULL,
                relation_type VARCHAR(50) NOT NULL,
                metadata JSONB DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (from_id, to_id, relation_type),
                FOREIGN KEY (from_id) REFERENCES memories(id) ON DELETE CASCADE,
                FOREIGN KEY (to_id) REFERENCES memories(id) ON DELETE CASCADE
            );
        "#)
        .execute(pool)
        .await
        .map_err(|e| Error::service(&format!("Failed to create relationships table: {}", e)))?;

        // Create indexes for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_memories_type ON memories(memory_type);")
            .execute(pool)
            .await
            .map_err(|e| Error::service(&format!("Failed to create memory type index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_memories_created_at ON memories(created_at);")
            .execute(pool)
            .await
            .map_err(|e| Error::service(&format!("Failed to create created_at index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_memories_content_gin ON memories USING GIN (to_tsvector('english', content));")
            .execute(pool)
            .await
            .map_err(|e| Error::service(&format!("Failed to create content search index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_relationships_from_id ON memory_relationships(from_id);")
            .execute(pool)
            .await
            .map_err(|e| Error::service(&format!("Failed to create from_id index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_relationships_to_id ON memory_relationships(to_id);")
            .execute(pool)
            .await
            .map_err(|e| Error::service(&format!("Failed to create to_id index: {}", e)))?;

        Ok(())
    }

    /// Validate input for security
    fn validate_input(&self, input: &str) -> Result<()> {
        use crate::security::{SanitizationOptions, ValidationResult};
        
        let options = SanitizationOptions {
            allow_html: false,
            allow_sql: false,
            allow_shell_meta: false,
            max_length: Some(100000), // 100KB limit
        };

        match self.security.validate_input(input, &options) {
            ValidationResult::Valid => Ok(()),
            ValidationResult::Invalid(reason) => {
                Err(Error::validation(&format!("Invalid input: {}", reason)))
            },
            ValidationResult::Malicious(reason) => {
                self.security.log_security_event("malicious_memory_input", Some(&reason));
                Err(Error::validation(&format!("Malicious input detected: {}", reason)))
            }
        }
    }

    /// Update cache with memory
    async fn update_cache(&self, memory: &Memory) {
        let mut cache = self.cache.write().await;
        
        // Simple LRU eviction if cache is full
        if cache.len() >= self.cache_size_limit {
            if let Some(oldest_key) = cache.keys().next().cloned() {
                cache.remove(&oldest_key);
            }
        }
        
        cache.insert(memory.id.clone(), memory.clone());
    }

    /// Remove from cache
    async fn remove_from_cache(&self, id: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(id);
    }
}

#[async_trait]
impl MemoryStore for PostgreSQLMemoryStore {
    async fn store_memory(&self, memory: &Memory) -> Result<()> {
        // Validate inputs
        self.validate_input(&memory.title)?;
        self.validate_input(&memory.content)?;

        sqlx::query(r#"
            INSERT INTO memories (id, memory_type, title, content, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                memory_type = EXCLUDED.memory_type,
                title = EXCLUDED.title,
                content = EXCLUDED.content,
                metadata = EXCLUDED.metadata,
                updated_at = EXCLUDED.updated_at
        "#)
        .bind(&memory.id)
        .bind(&memory.memory_type.to_string())
        .bind(&memory.title)
        .bind(&memory.content)
        .bind(sqlx::types::Json(&memory.metadata))
        .bind(&memory.created_at)
        .bind(&memory.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::service(&format!("Failed to store memory: {}", e)))?;

        // Update cache
        self.update_cache(memory).await;

        Ok(())
    }

    async fn get_memory(&self, id: &str) -> Result<Option<Memory>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(memory) = cache.get(id) {
                return Ok(Some(memory.clone()));
            }
        }

        // Query database
        let row = sqlx::query(r#"
            SELECT id, memory_type, title, content, metadata, created_at, updated_at
            FROM memories
            WHERE id = $1
        "#)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::service(&format!("Failed to get memory: {}", e)))?;

        if let Some(row) = row {
            let memory_type_str: String = row.try_get("memory_type")?;
            let memory_type = match memory_type_str.as_str() {
                "project" => MemoryType::Project,
                "issue" => MemoryType::Issue,
                "system" => MemoryType::System,
                "config" => MemoryType::Config,
                "finance" => MemoryType::Finance,
                "todo" => MemoryType::Todo,
                "knowledge" => MemoryType::Knowledge,
                _ => MemoryType::Custom(memory_type_str),
            };

            let memory = Memory {
                id: row.try_get("id")?,
                memory_type,
                title: row.try_get("title")?,
                content: row.try_get("content")?,
                metadata: row.try_get::<sqlx::types::Json<HashMap<String, serde_json::Value>>, _>("metadata")?.0,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            };

            // Update cache
            self.update_cache(&memory).await;

            Ok(Some(memory))
        } else {
            Ok(None)
        }
    }

    async fn update_memory(&self, memory: &Memory) -> Result<()> {
        // Validate inputs
        self.validate_input(&memory.title)?;
        self.validate_input(&memory.content)?;

        let result = sqlx::query(r#"
            UPDATE memories 
            SET title = $2, content = $3, metadata = $4, updated_at = $5
            WHERE id = $1
        "#)
        .bind(&memory.id)
        .bind(&memory.title)
        .bind(&memory.content)
        .bind(sqlx::types::Json(&memory.metadata))
        .bind(&memory.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::service(&format!("Failed to update memory: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(Error::not_found(&format!("Memory with ID '{}' not found", memory.id)));
        }

        // Update cache
        self.update_cache(memory).await;

        Ok(())
    }

    async fn delete_memory(&self, id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM memories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::service(&format!("Failed to delete memory: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(Error::not_found(&format!("Memory with ID '{}' not found", id)));
        }

        // Remove from cache
        self.remove_from_cache(id).await;

        Ok(())
    }

    async fn search_memories(&self, params: &MemorySearchParams) -> Result<Vec<Memory>> {
        let mut query = String::from(r#"
            SELECT id, memory_type, title, content, metadata, created_at, updated_at
            FROM memories
            WHERE 1=1
        "#);

        let mut bind_count = 0;
        let mut query_params: Vec<String> = Vec::new();

        // Add memory type filter
        if let Some(memory_type) = &params.memory_type {
            bind_count += 1;
            query.push_str(&format!(" AND memory_type = ${}", bind_count));
            query_params.push(memory_type.to_string());
        }

        // Add keyword search using full-text search
        if let Some(keyword) = &params.keyword {
            // Validate keyword
            self.validate_input(keyword)?;
            
            bind_count += 1;
            query.push_str(&format!(" AND (title ILIKE ${} OR content ILIKE ${} OR to_tsvector('english', content) @@ plainto_tsquery('english', ${}))", 
                bind_count, bind_count, bind_count));
            query_params.push(format!("%{}%", keyword));
        }

        // Add metadata filters
        if let Some(metadata_filters) = &params.metadata_filters {
            for (key, value) in metadata_filters {
                bind_count += 1;
                query.push_str(&format!(" AND metadata->>${}::text = ${}", bind_count, bind_count + 1));
                query_params.push(key.clone());
                bind_count += 1;
                if let Some(value_str) = value.as_str() {
                    query_params.push(value_str.to_string());
                } else {
                    query_params.push(value.to_string());
                }
            }
        }

        // Add ordering and limit
        query.push_str(" ORDER BY updated_at DESC");
        if let Some(limit) = params.limit {
            bind_count += 1;
            query.push_str(&format!(" LIMIT ${}", bind_count));
            query_params.push(limit.to_string());
        }

        // Execute query
        let mut sqlx_query = sqlx::query(&query);
        for param in &query_params {
            sqlx_query = sqlx_query.bind(param);
        }

        let rows = sqlx_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::service(&format!("Failed to search memories: {}", e)))?;

        let mut memories = Vec::with_capacity(rows.len());
        for row in rows {
            let memory_type_str: String = row.try_get("memory_type")?;
            let memory_type = match memory_type_str.as_str() {
                "project" => MemoryType::Project,
                "issue" => MemoryType::Issue,
                "system" => MemoryType::System,
                "config" => MemoryType::Config,
                "finance" => MemoryType::Finance,
                "todo" => MemoryType::Todo,
                "knowledge" => MemoryType::Knowledge,
                _ => MemoryType::Custom(memory_type_str),
            };

            let memory = Memory {
                id: row.try_get("id")?,
                memory_type,
                title: row.try_get("title")?,
                content: row.try_get("content")?,
                metadata: row.try_get::<sqlx::types::Json<HashMap<String, serde_json::Value>>, _>("metadata")?.0,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            };

            memories.push(memory);
        }

        Ok(memories)
    }

    async fn store_relationship(&self, relationship: &Relationship) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO memory_relationships (from_id, to_id, relation_type, metadata, created_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (from_id, to_id, relation_type) DO UPDATE SET
                metadata = EXCLUDED.metadata
        "#)
        .bind(&relationship.from_id)
        .bind(&relationship.to_id)
        .bind(&relationship.relation_type.to_string())
        .bind(sqlx::types::Json(&relationship.metadata))
        .bind(&relationship.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::service(&format!("Failed to store relationship: {}", e)))?;

        Ok(())
    }

    async fn get_relationships(&self, memory_id: &str) -> Result<Vec<Relationship>> {
        let rows = sqlx::query(r#"
            SELECT from_id, to_id, relation_type, metadata, created_at
            FROM memory_relationships
            WHERE from_id = $1 OR to_id = $1
            ORDER BY created_at
        "#)
        .bind(memory_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::service(&format!("Failed to get relationships: {}", e)))?;

        let mut relationships = Vec::with_capacity(rows.len());
        for row in rows {
            let relation_type_str: String = row.try_get("relation_type")?;
            let relation_type = match relation_type_str.as_str() {
                "RELATED_TO" => RelationType::RelatedTo,
                "PART_OF" => RelationType::PartOf,
                "DEPENDS_ON" => RelationType::DependsOn,
                "BLOCKS" => RelationType::Blocks,
                "SUPERSEDES" => RelationType::Supersedes,
                "REFERENCES" => RelationType::References,
                _ => RelationType::Custom(relation_type_str),
            };

            let relationship = Relationship {
                from_id: row.try_get("from_id")?,
                to_id: row.try_get("to_id")?,
                relation_type,
                metadata: row.try_get::<sqlx::types::Json<HashMap<String, serde_json::Value>>, _>("metadata")?.0,
                created_at: row.try_get("created_at")?,
            };

            relationships.push(relationship);
        }

        Ok(relationships)
    }

    async fn delete_relationships(&self, memory_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM memory_relationships WHERE from_id = $1 OR to_id = $1")
            .bind(memory_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::service(&format!("Failed to delete relationships: {}", e)))?;

        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        match sqlx::query("SELECT 1").execute(&self.pool).await {
            Ok(_) => Ok(true),
            Err(e) => {
                log::warn!("Memory store health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

/// In-memory store with Redis caching (for development/testing)
pub struct InMemoryStore {
    memories: Arc<RwLock<HashMap<String, Memory>>>,
    relationships: Arc<RwLock<Vec<Relationship>>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            memories: Arc::new(RwLock::new(HashMap::new())),
            relationships: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl MemoryStore for InMemoryStore {
    async fn store_memory(&self, memory: &Memory) -> Result<()> {
        let mut memories = self.memories.write().await;
        memories.insert(memory.id.clone(), memory.clone());
        Ok(())
    }

    async fn get_memory(&self, id: &str) -> Result<Option<Memory>> {
        let memories = self.memories.read().await;
        Ok(memories.get(id).cloned())
    }

    async fn update_memory(&self, memory: &Memory) -> Result<()> {
        let mut memories = self.memories.write().await;
        if memories.contains_key(&memory.id) {
            memories.insert(memory.id.clone(), memory.clone());
            Ok(())
        } else {
            Err(Error::not_found(&format!("Memory with ID '{}' not found", memory.id)))
        }
    }

    async fn delete_memory(&self, id: &str) -> Result<()> {
        let mut memories = self.memories.write().await;
        if memories.remove(id).is_some() {
            // Remove relationships
            let mut relationships = self.relationships.write().await;
            relationships.retain(|r| r.from_id != id && r.to_id != id);
            Ok(())
        } else {
            Err(Error::not_found(&format!("Memory with ID '{}' not found", id)))
        }
    }

    async fn search_memories(&self, params: &MemorySearchParams) -> Result<Vec<Memory>> {
        let memories = self.memories.read().await;
        
        let filtered_memories: Vec<Memory> = memories
            .values()
            .filter(|memory| {
                // Filter by memory type
                if let Some(ref memory_type) = params.memory_type {
                    if memory.memory_type != *memory_type {
                        return false;
                    }
                }

                // Filter by keyword
                if let Some(ref keyword) = params.keyword {
                    let keyword_lower = keyword.to_lowercase();
                    if !memory.title.to_lowercase().contains(&keyword_lower)
                        && !memory.content.to_lowercase().contains(&keyword_lower)
                    {
                        return false;
                    }
                }

                // Filter by metadata
                if let Some(ref metadata_filters) = params.metadata_filters {
                    for (key, value) in metadata_filters {
                        if !memory
                            .metadata
                            .get(key)
                            .map(|v| v == value)
                            .unwrap_or(false)
                        {
                            return false;
                        }
                    }
                }

                true
            })
            .take(params.limit.unwrap_or(usize::MAX))
            .cloned()
            .collect();

        Ok(filtered_memories)
    }

    async fn store_relationship(&self, relationship: &Relationship) -> Result<()> {
        let mut relationships = self.relationships.write().await;
        relationships.push(relationship.clone());
        Ok(())
    }

    async fn get_relationships(&self, memory_id: &str) -> Result<Vec<Relationship>> {
        let relationships = self.relationships.read().await;
        Ok(relationships
            .iter()
            .filter(|r| r.from_id == memory_id || r.to_id == memory_id)
            .cloned()
            .collect())
    }

    async fn delete_relationships(&self, memory_id: &str) -> Result<()> {
        let mut relationships = self.relationships.write().await;
        relationships.retain(|r| r.from_id != memory_id && r.to_id != memory_id);
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}