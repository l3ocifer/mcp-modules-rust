#[cfg(feature = "database")]
use crate::database::{Database, DatabaseStatus, QueryResult, Table, Column};
use crate::error::{Error, Result};
#[cfg(feature = "database")]
use crate::security::SecurityModule;
#[cfg(feature = "database")]
use serde_json::{json, Value};
#[cfg(feature = "database")]
use sqlx::{postgres::PgPoolOptions, PgPool, Row, Column as SqlxColumn, TypeInfo};
#[cfg(feature = "database")]
use std::time::Instant;

/// PostgreSQL provider for database module with connection pooling and performance optimization
#[cfg(feature = "database")]
pub struct PostgreSQLProvider {
    pool: PgPool,
    #[allow(dead_code)]
    connection_string: String,
    security: SecurityModule,
    #[allow(dead_code)]
    database_name: String,
}

#[cfg(feature = "database")]
impl PostgreSQLProvider {
    /// Create a new PostgreSQL provider with optimized connection pool
    pub async fn new(connection_string: String) -> Result<Self> {
        // Extract database name from connection string
        let database_name = connection_string
            .split('/')
            .last()
            .unwrap_or("postgres")
            .split('?')
            .next()
            .unwrap_or("postgres")
            .to_string();

        let pool = PgPoolOptions::new()
            .max_connections(32)
            .min_connections(4)
            .connect(&connection_string)
            .await
            .map_err(|e| Error::service(format!("Failed to connect to PostgreSQL: {}", e)))?;

        // Test connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| Error::service(format!("PostgreSQL connection test failed: {}", e)))?;

        Ok(Self {
            pool,
            connection_string,
            security: SecurityModule::new(),
            database_name,
        })
    }

    /// Convert a SQL row to JSON value
    fn row_to_value(row: &sqlx::postgres::PgRow) -> Result<Value> {
        let mut object = serde_json::Map::new();
        
        for (i, column) in row.columns().iter().enumerate() {
            let name = column.name();
            let value: Value = if let Ok(v) = row.try_get::<Option<i32>, _>(i) {
                json!(v)
            } else if let Ok(v) = row.try_get::<Option<i64>, _>(i) {
                json!(v)
            } else if let Ok(v) = row.try_get::<Option<f64>, _>(i) {
                json!(v)
            } else if let Ok(v) = row.try_get::<Option<String>, _>(i) {
                json!(v)
            } else if let Ok(v) = row.try_get::<Option<bool>, _>(i) {
                json!(v)
            } else if let Ok(v) = row.try_get::<Option<chrono::NaiveDateTime>, _>(i) {
                json!(v.map(|dt| dt.to_string()))
            } else if let Ok(v) = row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(i) {
                json!(v.map(|dt| dt.to_rfc3339()))
            } else if let Ok(v) = row.try_get::<Option<serde_json::Value>, _>(i) {
                v.unwrap_or(Value::Null)
            } else {
                Value::Null
            };
            
            object.insert(name.to_string(), value);
        }
        
        Ok(Value::Object(object))
    }

    /// Get column information from a row
    fn get_columns(row: &sqlx::postgres::PgRow) -> Vec<Column> {
        row.columns()
            .iter()
            .map(|col| {
                Column {
                    name: col.name().to_string(),
                    data_type: col.type_info().name().to_string(),
                    nullable: true, // Would need additional schema query to determine
                    primary_key: false, // Would need additional schema query to determine
                    unique: false, // Would need additional schema query to determine
                    default: None, // Would need additional schema query to determine
                }
            })
            .collect()
    }
}

#[cfg(feature = "database")]
#[async_trait::async_trait]
impl Database for PostgreSQLProvider {
    async fn execute_query(&self, query: &str, _database: Option<&str>) -> Result<QueryResult> {
        let start = Instant::now();
        
        // Basic SQL injection prevention (in production, use parameterized queries)
        use crate::security::{SanitizationOptions, ValidationResult};
        let options = SanitizationOptions::default();
        if let ValidationResult::Malicious(reason) = self.security.validate_input(query, &options) {
            return Err(Error::config(format!("Potentially malicious query: {}", reason)));
        }
        
        // Determine query type
        let query_lower = query.trim().to_lowercase();
        
        if query_lower.starts_with("select") || query_lower.starts_with("with") {
            // Execute SELECT query
            let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(query)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| Error::service(format!("Query execution failed: {}", e)))?;
            
            let columns = if !rows.is_empty() {
                Self::get_columns(&rows[0])
            } else {
                vec![]
            };
            
            let row_values: Result<Vec<Value>> = rows
                .iter()
                .map(Self::row_to_value)
                .collect();
            
            Ok(QueryResult {
                rows: row_values?,
                columns,
                rows_affected: 0,
                execution_time_ms: start.elapsed().as_millis() as u64,
            })
        } else {
            // Execute DML query (INSERT, UPDATE, DELETE)
            let result = sqlx::query(query)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::service(format!("Query execution failed: {}", e)))?;
            
            Ok(QueryResult {
                rows: vec![],
                columns: vec![],
                rows_affected: result.rows_affected(),
                execution_time_ms: start.elapsed().as_millis() as u64,
            })
        }
    }

    async fn list_databases(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT datname FROM pg_database WHERE datistemplate = false"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::service(format!("Failed to list databases: {}", e)))?;
        
        Ok(rows.into_iter().map(|(name,)| name).collect())
    }

    async fn list_tables(&self, database: Option<&str>) -> Result<Vec<Table>> {
        let schema = database.unwrap_or("public");
        
        let _query = format!(
            r#"
            SELECT 
                t.table_name,
                pg_relation_size(quote_ident(t.table_schema)||'.'||quote_ident(t.table_name)) as size_bytes,
                (SELECT COUNT(*) FROM {}."{}" ) as row_count
            FROM information_schema.tables t
            WHERE t.table_schema = '{}'
                AND t.table_type = 'BASE TABLE'
            "#,
            schema, "", schema
        );
        
        // This is a simplified version - a proper implementation would need dynamic SQL
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = $1
                AND table_type = 'BASE TABLE'
            ORDER BY table_name
            "#
        )
        .bind(schema)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::service(format!("Failed to list tables: {}", e)))?;
        
        let mut tables = Vec::new();
        for (table_name,) in rows {
            // Get table size and row count
            let stats_query = format!(
                "SELECT pg_relation_size('{}') as size_bytes, 
                 (SELECT COUNT(*) FROM {}) as row_count",
                table_name, table_name
            );
            
            if let Ok(row) = sqlx::query(&stats_query).fetch_one(&self.pool).await {
                let size_bytes: Option<i64> = row.try_get("size_bytes").ok().flatten();
                let row_count: Option<i64> = row.try_get("row_count").ok().flatten();
                
                tables.push(Table {
                    name: table_name,
                    columns: vec![], // Will be populated by describe_table
                    row_count: row_count.map(|c| c as u64),
                    size_bytes: size_bytes.map(|s| s as u64),
                });
            } else {
                tables.push(Table {
                    name: table_name,
                    columns: vec![],
                    row_count: None,
                    size_bytes: None,
                });
            }
        }
        
        Ok(tables)
    }

    async fn describe_table(&self, table_name: &str, database: Option<&str>) -> Result<Table> {
        let schema = database.unwrap_or("public");
        
        // Get column information
        let columns: Vec<(String, String, bool, Option<String>)> = sqlx::query_as(
            r#"
            SELECT 
                column_name,
                data_type,
                is_nullable = 'YES' as nullable,
                column_default
            FROM information_schema.columns
            WHERE table_schema = $1 AND table_name = $2
            ORDER BY ordinal_position
            "#
        )
        .bind(schema)
        .bind(table_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::service(format!("Failed to describe table: {}", e)))?;
        
        // Get primary key information
        let pk_columns: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT kcu.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            WHERE tc.constraint_type = 'PRIMARY KEY'
                AND tc.table_schema = $1
                AND tc.table_name = $2
            "#
        )
        .bind(schema)
        .bind(table_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::service(format!("Failed to get primary keys: {}", e)))?;
        
        let pk_set: std::collections::HashSet<String> = 
            pk_columns.into_iter().map(|(name,)| name).collect();
        
        // Get unique constraint information
        let unique_columns: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT kcu.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            WHERE tc.constraint_type = 'UNIQUE'
                AND tc.table_schema = $1
                AND tc.table_name = $2
            "#
        )
        .bind(schema)
        .bind(table_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::service(format!("Failed to get unique constraints: {}", e)))?;
        
        let unique_set: std::collections::HashSet<String> = 
            unique_columns.into_iter().map(|(name,)| name).collect();
        
        let table_columns: Vec<Column> = columns
            .into_iter()
            .map(|(name, data_type, nullable, default)| Column {
                name: name.clone(),
                data_type,
                nullable,
                primary_key: pk_set.contains(&name),
                unique: unique_set.contains(&name),
                default,
            })
            .collect();
        
        // Get table size and row count
        let stats_query = format!(
            "SELECT pg_relation_size('{}') as size_bytes, 
             (SELECT COUNT(*) FROM {}) as row_count",
            table_name, table_name
        );
        
        let (size_bytes, row_count) = if let Ok(row) = sqlx::query(&stats_query).fetch_one(&self.pool).await {
            let size_bytes: Option<i64> = row.try_get("size_bytes").ok().flatten();
            let row_count: Option<i64> = row.try_get("row_count").ok().flatten();
            (size_bytes.map(|s| s as u64), row_count.map(|c| c as u64))
        } else {
            (None, None)
        };
        
        Ok(Table {
            name: table_name.to_string(),
            columns: table_columns,
            row_count,
            size_bytes,
        })
    }

    async fn health_check(&self) -> Result<DatabaseStatus> {
        let start = Instant::now();
        
        match sqlx::query("SELECT 1").execute(&self.pool).await {
            Ok(_) => Ok(DatabaseStatus {
                healthy: true,
                latency_ms: start.elapsed().as_millis() as u64,
                message: Some("PostgreSQL connection healthy".to_string()),
            }),
            Err(e) => Ok(DatabaseStatus {
                healthy: false,
                latency_ms: start.elapsed().as_millis() as u64,
                message: Some(format!("PostgreSQL health check failed: {}", e)),
            }),
        }
    }
}

// Stub implementation for when database feature is not enabled
#[cfg(not(feature = "database"))]
pub struct PostgreSQLProvider;

#[cfg(not(feature = "database"))]
impl PostgreSQLProvider {
    pub async fn new(_connection_string: String) -> Result<Self> {
        Err(Error::config("PostgreSQL support requires 'database' feature to be enabled"))
    }
}