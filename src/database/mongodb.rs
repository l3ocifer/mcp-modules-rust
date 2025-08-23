use crate::database::{Database, DatabaseStatus, QueryResult, Table, Column};
use crate::error::{Error, Result};
use crate::security::SecurityModule;
use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, FindOptions},
    Client, Database as MongoDatabase,
};
use futures::TryStreamExt;
use serde_json::{json, Value};
use std::time::Instant;
use tokio::sync::RwLock;
use std::sync::Arc;
use std::collections::HashMap;

/// MongoDB provider for database module with connection pooling and performance optimization
pub struct MongoDBProvider {
    client: Client,
    connection_string: String,
    default_database: String,
    security: SecurityModule,
    connection_pool: Arc<RwLock<HashMap<String, MongoDatabase>>>,
    max_pool_size: u32,
}

impl MongoDBProvider {
    /// Create a new MongoDB provider with optimized settings
    pub async fn new(connection_string: String) -> Result<Self> {
        let client_options = ClientOptions::parse(&connection_string)
            .await
            .map_err(|e| Error::service(&format!("Failed to parse MongoDB connection string: {}", e)))?;

        let client = Client::with_options(client_options)
            .map_err(|e| Error::service(&format!("Failed to create MongoDB client: {}", e)))?;

        // Test connection
        client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await
            .map_err(|e| Error::service(&format!("Failed to connect to MongoDB: {}", e)))?;

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
            max_pool_size: 10,
        })
    }

    /// Get a database with connection pooling
    async fn get_database(&self, database_name: &str) -> MongoDatabase {
        let pool = self.connection_pool.read().await;
        if let Some(db) = pool.get(database_name) {
            return db.clone();
        }
        drop(pool);

        // Create new connection
        let db = self.client.database(database_name);
        
        let mut pool = self.connection_pool.write().await;
        if pool.len() < self.max_pool_size as usize {
            pool.insert(database_name.to_string(), db.clone());
        }
        
        db
    }

    /// Validate MongoDB query for security
    fn validate_query(&self, query: &str) -> Result<()> {
        use crate::security::{SanitizationOptions, ValidationResult};
        
        let options = SanitizationOptions {
            allow_html: false,
            allow_sql: false,
            allow_shell_meta: false,
            max_length: Some(10000),
        };

        match self.security.validate_input(query, &options) {
            ValidationResult::Valid => Ok(()),
            ValidationResult::Invalid(reason) => {
                Err(Error::validation(&format!("Invalid MongoDB query: {}", reason)))
            },
            ValidationResult::Malicious(reason) => {
                self.security.log_security_event("malicious_mongodb_query", Some(&reason));
                Err(Error::validation(&format!("Malicious MongoDB query detected: {}", reason)))
            }
        }
    }

    /// List databases with performance optimization
    pub async fn list_databases(&self) -> Result<Vec<Database>> {
        let start_time = Instant::now();
        
        let databases = self
            .client
            .list_database_names(None, None)
            .await
            .map_err(|e| Error::service(&format!("Failed to list databases: {}", e)))?;

        // Pre-allocate with estimated capacity
        let mut results = Vec::with_capacity(databases.len());

        for db_name in databases {
            // Skip system databases
            if &db_name == "admin" || &db_name == "local" || &db_name == "config" {
                continue;
            }

            let db = self.get_database(&db_name).await;
            
            // Get database stats
            let stats = match db.run_command(doc! {"dbStats": 1}, None).await {
                Ok(stats) => stats,
                Err(_) => doc! {}, // Continue even if stats fail
            };

            let size = stats.get_i64("dataSize").unwrap_or(0) as u64;
            let collections = stats.get_i32("collections").unwrap_or(0);
            let indexes = stats.get_i32("indexes").unwrap_or(0);

            results.push(Database {
                id: format!("mongodb/{}", db_name),
                name: db_name,
                provider: "mongodb".to_string(),
                status: DatabaseStatus::Online,
                size: Some(size),
                metadata: json!({
                    "collections": collections,
                    "indexes": indexes,
                    "engine": "WiredTiger",
                    "connection_time_ms": start_time.elapsed().as_millis() as u64
                }),
            });
        }

        Ok(results)
    }

    /// List collections in a database with optimization
    pub async fn list_collections(&self, database: &str) -> Result<Vec<String>> {
        let db = self.get_database(database).await;
        
        let collections = db
            .list_collection_names(None)
            .await
            .map_err(|e| Error::service(&format!("Failed to list collections: {}", e)))?;

        Ok(collections)
    }

    /// Get collection schema information
    pub async fn describe_collection(&self, database: &str, collection: &str) -> Result<Table> {
        let db = self.get_database(database).await;
        let coll = db.collection::<Document>(collection);

        // Get collection stats
        let _stats = coll
            .aggregate(vec![
                doc! {"$collStats": {"storageStats": {}}},
            ], None)
            .await
            .map_err(|e| Error::service(&format!("Failed to get collection stats: {}", e)))?;

        let count = coll.count_documents(None, None).await.unwrap_or(0);

        // Sample documents to infer schema
        let find_options = FindOptions::builder().limit(100).build();
        let mut cursor = coll
            .find(None, find_options)
            .await
            .map_err(|e| Error::service(&format!("Failed to sample documents: {}", e)))?;

        let mut field_types = HashMap::new();
        let mut _doc_count = 0;

        while let Ok(Some(doc)) = cursor.try_next().await {
            _doc_count += 1;
            for (key, value) in doc.iter() {
                let type_name = match value {
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
                field_types.insert(key.clone(), type_name.to_string());
            }
        }

        // Convert to columns
        let mut columns = Vec::with_capacity(field_types.len());
        for (name, data_type) in field_types {
            columns.push(Column {
                name: name.clone(),
                data_type,
                is_nullable: true, // MongoDB fields are generally optional
                is_primary: name == "_id",
            });
        }

        Ok(Table {
            name: collection.to_string(),
            schema: database.to_string(),
            row_count: count,
            size: 0, // Would need additional stats call
            columns,
        })
    }

    /// Execute a MongoDB query with security validation and performance optimization
    pub async fn execute_query(
        &self,
        database: &str,
        collection: &str,
        query: &str,
    ) -> Result<QueryResult> {
        let start_time = Instant::now();

        // Validate query for security
        self.validate_query(query)?;

        let db = self.get_database(database).await;
        let coll = db.collection::<Document>(collection);

        // Parse the query as BSON document
        let query_doc: Document = if query.is_empty() {
            doc! {}
        } else {
            // Try to parse as JSON and convert to BSON
            let json_value: serde_json::Value = serde_json::from_str(query)
                .map_err(|e| Error::parsing(&format!("Invalid MongoDB query JSON: {}", e)))?;
            match mongodb::bson::to_bson(&json_value) {
                Ok(mongodb::bson::Bson::Document(doc)) => doc,
                Ok(_) => return Err(Error::parsing("Query must be a JSON object")),
                Err(e) => return Err(Error::parsing(&format!("Failed to convert query to BSON: {}", e))),
            }
        };

        // Execute find query with performance optimization
        let find_options = FindOptions::builder()
            .limit(1000) // Prevent excessive results
            .build();

        let mut cursor = coll
            .find(Some(query_doc), find_options)
            .await
            .map_err(|e| Error::service(&format!("Failed to execute query: {}", e)))?;

        // Pre-allocate results vector
        let mut rows = Vec::with_capacity(100);
        let mut columns = Vec::new();
        let mut first_doc = true;

        while let Ok(Some(doc)) = cursor.try_next().await {
            if first_doc {
                // Extract column names from first document
                columns = doc.keys().map(|k| k.to_string()).collect();
                first_doc = false;
            }

            // Convert BSON document to JSON
            let value: Value = mongodb::bson::from_bson(mongodb::bson::Bson::Document(doc))
                .map_err(|e| Error::parsing(&format!("Failed to parse BSON: {}", e)))?;
            rows.push(value);
        }

        let rows_count = rows.len() as u64;
        Ok(QueryResult {
            rows,
            columns,
            affected_rows: Some(rows_count),
            execution_time: Some(start_time.elapsed().as_millis() as u64),
        })
    }

    /// Insert document with performance optimization
    pub async fn insert_document(
        &self,
        database: &str,
        collection: &str,
        document: Value,
    ) -> Result<String> {
        let db = self.get_database(database).await;
        let coll = db.collection::<Document>(collection);

        // Convert JSON to BSON
        let bson_doc: Document = match mongodb::bson::to_bson(&document) {
            Ok(mongodb::bson::Bson::Document(doc)) => doc,
            Ok(_) => return Err(Error::parsing("Document must be a JSON object")),
            Err(e) => return Err(Error::parsing(&format!("Failed to convert JSON to BSON: {}", e))),
        };

        let result = coll
            .insert_one(bson_doc, None)
            .await
            .map_err(|e| Error::service(&format!("Failed to insert document: {}", e)))?;

        Ok(result.inserted_id.to_string())
    }

    /// Update documents with performance optimization
    pub async fn update_documents(
        &self,
        database: &str,
        collection: &str,
        filter: Value,
        update: Value,
    ) -> Result<u64> {
        let db = self.get_database(database).await;
        let coll = db.collection::<Document>(collection);

        let filter_doc: Document = match mongodb::bson::to_bson(&filter) {
            Ok(mongodb::bson::Bson::Document(doc)) => doc,
            Ok(_) => return Err(Error::parsing("Filter must be a JSON object")),
            Err(e) => return Err(Error::parsing(&format!("Failed to convert filter to BSON: {}", e))),
        };

        let update_doc: Document = match mongodb::bson::to_bson(&update) {
            Ok(mongodb::bson::Bson::Document(doc)) => doc,
            Ok(_) => return Err(Error::parsing("Update must be a JSON object")),
            Err(e) => return Err(Error::parsing(&format!("Failed to convert update to BSON: {}", e))),
        };

        let result = coll
            .update_many(filter_doc, update_doc, None)
            .await
            .map_err(|e| Error::service(&format!("Failed to update documents: {}", e)))?;

        Ok(result.modified_count)
    }

    /// Delete documents with performance optimization
    pub async fn delete_documents(
        &self,
        database: &str,
        collection: &str,
        filter: Value,
    ) -> Result<u64> {
        let db = self.get_database(database).await;
        let coll = db.collection::<Document>(collection);

        let filter_doc: Document = match mongodb::bson::to_bson(&filter) {
            Ok(mongodb::bson::Bson::Document(doc)) => doc,
            Ok(_) => return Err(Error::parsing("Filter must be a JSON object")),
            Err(e) => return Err(Error::parsing(&format!("Failed to convert filter to BSON: {}", e))),
        };

        let result = coll
            .delete_many(filter_doc, None)
            .await
            .map_err(|e| Error::service(&format!("Failed to delete documents: {}", e)))?;

        Ok(result.deleted_count)
    }

    /// Test connection health
    pub async fn health_check(&self) -> Result<bool> {
        match self
            .client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                log::warn!("MongoDB health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get connection info
    pub fn connection_info(&self) -> HashMap<String, Value> {
        let mut info = HashMap::new();
        info.insert("provider".to_string(), json!("mongodb"));
        info.insert("connection_string".to_string(), json!(self.connection_string));
        info.insert("default_database".to_string(), json!(self.default_database));
        info.insert("max_pool_size".to_string(), json!(self.max_pool_size));
        info
    }
}
