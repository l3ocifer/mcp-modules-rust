#[cfg(feature = "database")]
use crate::database::{Database, DatabaseStatus, QueryResult, Table, Column};
use crate::error::{Error, Result};
#[cfg(feature = "database")]
use serde_json::Value;
#[cfg(feature = "database")]
use crate::security::SecurityModule;
#[cfg(feature = "database")]
use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, FindOptions},
    Client, Database as MongoDatabase,
};
#[cfg(feature = "database")]
use futures::TryStreamExt;
#[cfg(feature = "database")]
use std::time::Instant;
#[cfg(feature = "database")]
use tokio::sync::RwLock;
#[cfg(feature = "database")]
use std::sync::Arc;
#[cfg(feature = "database")]
use std::collections::HashMap;

/// MongoDB provider for database module with connection pooling and performance optimization
#[cfg(feature = "database")]
pub struct MongoDBProvider {
    client: Client,
    #[allow(dead_code)]
    connection_string: String,
    default_database: String,
    #[allow(dead_code)]
    security: SecurityModule,
    connection_pool: Arc<RwLock<HashMap<String, MongoDatabase>>>,
    max_pool_size: u32,
}

#[cfg(feature = "database")]
impl MongoDBProvider {
    /// Create a new MongoDB provider with optimized settings
    pub async fn new(connection_string: String) -> Result<Self> {
        let client_options = ClientOptions::parse(&connection_string)
            .await
            .map_err(|e| Error::service(format!("Failed to parse MongoDB connection string: {}", e)))?;

        let client = Client::with_options(client_options)
            .map_err(|e| Error::service(format!("Failed to create MongoDB client: {}", e)))?;

        // Test connection
        client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await
            .map_err(|e| Error::service(format!("Failed to connect to MongoDB: {}", e)))?;

        // Extract default database name from connection string
        let default_database = connection_string
            .split('/')
            .last()
            .unwrap_or("test")
            .split('?')
            .next()
            .unwrap_or("test")
            .to_string();

        Ok(Self {
            client,
            connection_string,
            default_database,
            security: SecurityModule::new(),
            connection_pool: Arc::new(RwLock::new(HashMap::with_capacity(10))),
            max_pool_size: 100,
        })
    }

    /// Get or create a database connection from the pool
    async fn get_database(&self, database_name: Option<&str>) -> MongoDatabase {
        let db_name = database_name.unwrap_or(&self.default_database);
        
        // Try to get from pool first
        {
            let pool = self.connection_pool.read().await;
            if let Some(db) = pool.get(db_name) {
                return db.clone();
            }
        }
        
        // Create new connection if not in pool
        let db = self.client.database(db_name);
        
        // Add to pool if not at max capacity
        {
            let mut pool = self.connection_pool.write().await;
            if pool.len() < self.max_pool_size as usize {
                pool.insert(db_name.to_string(), db.clone());
            }
        }
        
        db
    }

    /// Convert MongoDB document to JSON value
    fn document_to_value(doc: &Document) -> Result<Value> {
        let value: Value = mongodb::bson::from_bson(mongodb::bson::Bson::Document(doc.clone()))
            .map_err(|e| Error::service(format!("Failed to convert BSON to JSON: {}", e)))?;
        Ok(value)
    }

    /// Convert JSON value to MongoDB document
    fn value_to_document(value: &Value) -> Result<Document> {
        match mongodb::bson::to_bson(value) {
            Ok(mongodb::bson::Bson::Document(doc)) => Ok(doc),
            Ok(_) => Err(Error::config("JSON value must be an object")),
            Err(e) => Err(Error::service(format!("Failed to convert JSON to BSON: {}", e))),
        }
    }
}

#[cfg(feature = "database")]
#[async_trait::async_trait]
impl Database for MongoDBProvider {
    async fn execute_query(&self, query: &str, database: Option<&str>) -> Result<QueryResult> {
        let start = Instant::now();
        
        // Parse the query as a MongoDB command
        let command: Value = serde_json::from_str(query)
            .map_err(|e| Error::config(format!("Invalid MongoDB query JSON: {}", e)))?;
        
        let db = self.get_database(database).await;
        
        // Extract collection name and operation from the command
        let collection_name = command.get("collection")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::config("Missing 'collection' field in query"))?;
        
        let operation = command.get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::config("Missing 'operation' field in query"))?;
        
        let collection = db.collection::<Document>(collection_name);
        
        let result = match operation {
            "find" => {
                let filter = command.get("filter")
                    .map(Self::value_to_document)
                    .transpose()?
                    .unwrap_or_else(|| doc! {});
                
                let options = FindOptions::builder()
                    .limit(command.get("limit").and_then(|v| v.as_i64()))
                    .skip(command.get("skip").and_then(|v| v.as_u64()))
                    .build();
                
                let mut cursor = collection.find(filter, options).await
                    .map_err(|e| Error::service(format!("MongoDB find failed: {}", e)))?;
                
                let mut rows = Vec::new();
                while let Some(doc) = cursor.try_next().await
                    .map_err(|e| Error::service(format!("Failed to iterate cursor: {}", e)))? {
                    rows.push(Self::document_to_value(&doc)?);
                }
                
                QueryResult {
                    rows,
                    columns: vec![],
                    rows_affected: 0,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                }
            },
            "insert" => {
                let document = command.get("document")
                    .ok_or_else(|| Error::config("Missing 'document' field for insert"))?;
                let doc = Self::value_to_document(document)?;
                
                collection.insert_one(doc, None).await
                    .map_err(|e| Error::service(format!("MongoDB insert failed: {}", e)))?;
                
                QueryResult {
                    rows: vec![],
                    columns: vec![],
                    rows_affected: 1,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                }
            },
            "update" => {
                let filter = command.get("filter")
                    .map(Self::value_to_document)
                    .transpose()?
                    .unwrap_or_else(|| doc! {});
                
                let update = command.get("update")
                    .ok_or_else(|| Error::config("Missing 'update' field"))?;
                let update_doc = Self::value_to_document(update)?;
                
                let result = collection.update_many(filter, update_doc, None).await
                    .map_err(|e| Error::service(format!("MongoDB update failed: {}", e)))?;
                
                QueryResult {
                    rows: vec![],
                    columns: vec![],
                    rows_affected: result.modified_count as u64,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                }
            },
            "delete" => {
                let filter = command.get("filter")
                    .map(Self::value_to_document)
                    .transpose()?
                    .unwrap_or_else(|| doc! {});
                
                let result = collection.delete_many(filter, None).await
                    .map_err(|e| Error::service(format!("MongoDB delete failed: {}", e)))?;
                
                QueryResult {
                    rows: vec![],
                    columns: vec![],
                    rows_affected: result.deleted_count as u64,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                }
            },
            _ => return Err(Error::config(format!("Unknown operation: {}", operation))),
        };
        
        Ok(result)
    }

    async fn list_databases(&self) -> Result<Vec<String>> {
        let names = self.client.list_database_names(None, None).await
            .map_err(|e| Error::service(format!("Failed to list databases: {}", e)))?;
        Ok(names)
    }

    async fn list_tables(&self, database: Option<&str>) -> Result<Vec<Table>> {
        let db = self.get_database(database).await;
        let collection_names = db.list_collection_names(None).await
            .map_err(|e| Error::service(format!("Failed to list collections: {}", e)))?;
        
        let mut tables = Vec::new();
        for name in collection_names {
            // Get collection stats
            let stats = db.run_command(doc! {
                "collStats": &name,
                "scale": 1
            }, None).await
                .map_err(|e| Error::service(format!("Failed to get collection stats: {}", e)))?;
            
            let row_count = stats.get_i64("count").unwrap_or(0) as u64;
            
            tables.push(Table {
                name: name.clone(),
                columns: vec![], // MongoDB is schemaless
                row_count: Some(row_count),
                size_bytes: stats.get_i64("size").ok().map(|s| s as u64),
            });
        }
        
        Ok(tables)
    }

    async fn describe_table(&self, table_name: &str, database: Option<&str>) -> Result<Table> {
        let db = self.get_database(database).await;
        
        // Get collection stats
        let stats = db.run_command(doc! {
            "collStats": table_name,
            "scale": 1
        }, None).await
            .map_err(|e| Error::service(format!("Failed to get collection stats: {}", e)))?;
        
        let row_count = stats.get_i64("count").unwrap_or(0) as u64;
        
        // Sample documents to infer schema
        let collection = db.collection::<Document>(table_name);
        let sample_docs: Vec<Document> = collection
            .find(None, FindOptions::builder().limit(100).build())
            .await
            .map_err(|e| Error::service(format!("Failed to sample collection: {}", e)))?
            .try_collect()
            .await
            .map_err(|e| Error::service(format!("Failed to collect samples: {}", e)))?;
        
        // Infer schema from sample documents
        let mut field_types: HashMap<String, String> = HashMap::new();
        for doc in &sample_docs {
            for (key, value) in doc {
                if !field_types.contains_key(key) {
                    let data_type = match value {
                        mongodb::bson::Bson::Double(_) => "double",
                        mongodb::bson::Bson::String(_) => "string",
                        mongodb::bson::Bson::Array(_) => "array",
                        mongodb::bson::Bson::Document(_) => "object",
                        mongodb::bson::Bson::Boolean(_) => "boolean",
                        mongodb::bson::Bson::Int32(_) => "int32",
                        mongodb::bson::Bson::Int64(_) => "int64",
                        mongodb::bson::Bson::ObjectId(_) => "objectId",
                        mongodb::bson::Bson::DateTime(_) => "date",
                        _ => "mixed",
                    };
                    field_types.insert(key.clone(), data_type.to_string());
                }
            }
        }
        
        let columns: Vec<Column> = field_types
            .into_iter()
            .map(|(name, data_type)| Column {
                name,
                data_type,
                nullable: true, // All fields are nullable in MongoDB
                primary_key: false,
                unique: false,
                default: None,
            })
            .collect();
        
        Ok(Table {
            name: table_name.to_string(),
            columns,
            row_count: Some(row_count),
            size_bytes: stats.get_i64("size").ok().map(|s| s as u64),
        })
    }

    async fn health_check(&self) -> Result<DatabaseStatus> {
        let start = Instant::now();
        
        match self.client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await {
            Ok(_) => Ok(DatabaseStatus {
                healthy: true,
                latency_ms: start.elapsed().as_millis() as u64,
                message: Some("MongoDB connection healthy".to_string()),
            }),
            Err(e) => Ok(DatabaseStatus {
                healthy: false,
                latency_ms: start.elapsed().as_millis() as u64,
                message: Some(format!("MongoDB health check failed: {}", e)),
            }),
        }
    }
}

// Stub implementation for when database feature is not enabled
#[cfg(not(feature = "database"))]
pub struct MongoDBProvider;

#[cfg(not(feature = "database"))]
impl MongoDBProvider {
    pub async fn new(_connection_string: String) -> Result<Self> {
        Err(Error::config("MongoDB support requires 'database' feature to be enabled"))
    }
}