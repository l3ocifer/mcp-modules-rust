use crate::database::{Database, DatabaseStatus, QueryResult, Table, Column};
use crate::error::{Error, Result};
use crate::security::SecurityModule;
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool, Row, Column as SqlxColumn, TypeInfo};
use std::time::Instant;
use std::collections::HashMap;

/// PostgreSQL provider for database module with connection pooling and performance optimization
pub struct PostgreSQLProvider {
    pool: PgPool,
    connection_string: String,
    security: SecurityModule,
    database_name: String,
}

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
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(std::time::Duration::from_secs(600))
            .max_lifetime(std::time::Duration::from_secs(1800))
            .connect(&connection_string)
            .await
            .map_err(|e| Error::service(&format!("Failed to connect to PostgreSQL: {}", e)))?;

        // Test connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| Error::service(&format!("PostgreSQL connection test failed: {}", e)))?;

        Ok(Self {
            pool,
            connection_string,
            security: SecurityModule::new(),
            database_name,
        })
    }

    /// Validate SQL query for security
    fn validate_query(&self, query: &str) -> Result<()> {
        use crate::security::{SanitizationOptions, ValidationResult};
        
        let options = SanitizationOptions {
            allow_html: false,
            allow_sql: true, // SQL is expected, but we still validate for injection
            allow_shell_meta: false,
            max_length: Some(50000),
        };

        // Additional SQL injection checks
        let query_lower = query.to_lowercase();
        let dangerous_patterns = [
            "; drop table", "; delete from", "; truncate", 
            "union select", "or 1=1", "' or '1'='1",
            "exec xp_", "exec sp_", "xp_cmdshell",
            "information_schema", "pg_stat_",
        ];

        for pattern in &dangerous_patterns {
            if query_lower.contains(pattern) {
                self.security.log_security_event("sql_injection_attempt", Some(&format!("Pattern detected: {}", pattern)));
                return Err(Error::validation(&format!("Potentially malicious SQL pattern detected: {}", pattern)));
            }
        }

        match self.security.validate_input(query, &options) {
            ValidationResult::Valid => Ok(()),
            ValidationResult::Invalid(reason) => {
                Err(Error::validation(&format!("Invalid SQL query: {}", reason)))
            },
            ValidationResult::Malicious(reason) => {
                self.security.log_security_event("malicious_sql_query", Some(&reason));
                Err(Error::validation(&format!("Malicious SQL query detected: {}", reason)))
            }
        }
    }

    /// List databases with performance optimization
    pub async fn list_databases(&self) -> Result<Vec<Database>> {
        let start_time = Instant::now();
        
        let query = r#"
            SELECT 
                d.datname as name,
                pg_database_size(d.datname) as size,
                (SELECT count(*) FROM information_schema.tables 
                 WHERE table_catalog = d.datname AND table_schema = 'public') as table_count,
                (SELECT count(*) FROM pg_stat_user_indexes 
                 WHERE schemaname = 'public') as index_count
            FROM pg_database d
            WHERE d.datistemplate = false
            AND d.datname NOT IN ('postgres', 'template0', 'template1')
            ORDER BY d.datname
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::service(&format!("Failed to list databases: {}", e)))?;

        let mut results = Vec::with_capacity(rows.len());

        for row in rows {
            let name: String = row.try_get("name")
                .map_err(|e| Error::service(&format!("Failed to get database name: {}", e)))?;
            let size: i64 = row.try_get("size")
                .map_err(|e| Error::service(&format!("Failed to get database size: {}", e)))?;
            let table_count: i64 = row.try_get("table_count").unwrap_or(0);
            let index_count: i64 = row.try_get("index_count").unwrap_or(0);

            results.push(Database {
                id: format!("postgresql/{}", name),
                name,
                provider: "postgresql".to_string(),
                status: DatabaseStatus::Online,
                size: Some(size as u64),
                metadata: json!({
                    "tables": table_count,
                    "indexes": index_count,
                    "version": "15.0",
                    "encoding": "UTF8",
                    "connection_time_ms": start_time.elapsed().as_millis() as u64
                }),
            });
        }

        Ok(results)
    }

    /// List tables in a database with optimization
    pub async fn list_tables(&self, schema: Option<&str>) -> Result<Vec<String>> {
        let schema = schema.unwrap_or("public");
        
        let query = r#"
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = $1 
            AND table_type = 'BASE TABLE'
            ORDER BY table_name
        "#;

        let rows = sqlx::query(query)
            .bind(schema)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::service(&format!("Failed to list tables: {}", e)))?;

        let tables = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("table_name"))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::service(&format!("Failed to parse table names: {}", e)))?;

        Ok(tables)
    }

    /// Get table schema information
    pub async fn describe_table(&self, table_name: &str, schema: Option<&str>) -> Result<Table> {
        let schema = schema.unwrap_or("public");

        // Get table info and column details
        let query = r#"
            SELECT 
                c.column_name,
                c.data_type,
                c.is_nullable,
                tc.constraint_type
            FROM information_schema.columns c
            LEFT JOIN information_schema.key_column_usage kcu 
                ON c.table_name = kcu.table_name 
                AND c.column_name = kcu.column_name
                AND c.table_schema = kcu.table_schema
            LEFT JOIN information_schema.table_constraints tc 
                ON kcu.constraint_name = tc.constraint_name
                AND tc.constraint_type = 'PRIMARY KEY'
            WHERE c.table_name = $1 
            AND c.table_schema = $2
            ORDER BY c.ordinal_position
        "#;

        let rows = sqlx::query(query)
            .bind(table_name)
            .bind(schema)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::service(&format!("Failed to describe table: {}", e)))?;

        // Get row count and size
        let stats_query = format!(
            "SELECT count(*) as row_count FROM {}.{}",
            schema, table_name
        );
        
        let row_count: i64 = sqlx::query_scalar(&stats_query)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        let mut columns = Vec::with_capacity(rows.len());
        for row in rows {
            let column_name: String = row.try_get("column_name")?;
            let data_type: String = row.try_get("data_type")?;
            let is_nullable: String = row.try_get("is_nullable")?;
            let constraint_type: Option<String> = row.try_get("constraint_type").ok();

            columns.push(Column {
                name: column_name,
                data_type,
                is_nullable: is_nullable == "YES",
                is_primary: constraint_type.as_deref() == Some("PRIMARY KEY"),
            });
        }

        Ok(Table {
            name: table_name.to_string(),
            schema: schema.to_string(),
            row_count: row_count as u64,
            size: 0, // Would need additional query to get table size
            columns,
        })
    }

    /// Execute a SQL query with security validation and performance optimization
    pub async fn execute_query(&self, query: &str) -> Result<QueryResult> {
        let start_time = Instant::now();

        // Validate query for security
        self.validate_query(query)?;

        // Determine if it's a SELECT query or modification query
        let is_select = query.trim().to_lowercase().starts_with("select");

        if is_select {
            self.execute_select_query(query, start_time).await
        } else {
            self.execute_modification_query(query, start_time).await
        }
    }

    /// Execute SELECT query
    async fn execute_select_query(&self, query: &str, start_time: Instant) -> Result<QueryResult> {
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::service(&format!("Failed to execute SELECT query: {}", e)))?;

        if rows.is_empty() {
            return Ok(QueryResult {
                rows: vec![],
                columns: vec![],
                affected_rows: Some(0),
                execution_time: Some(start_time.elapsed().as_millis() as u64),
            });
        }

        // Get column names from the first row
        let columns: Vec<String> = rows[0]
            .columns()
            .iter()
            .map(|col| col.name().to_string())
            .collect();

        // Convert rows to JSON values
        let mut json_rows = Vec::with_capacity(rows.len());
        for row in rows {
            let mut json_row = serde_json::Map::new();
            
            for (i, column) in row.columns().iter().enumerate() {
                let value = match column.type_info().name() {
                    "INT4" => {
                        row.try_get::<Option<i32>, _>(i)
                            .map(|v| v.map(|n| json!(n)).unwrap_or(Value::Null))
                    },
                    "INT8" => {
                        row.try_get::<Option<i64>, _>(i)
                            .map(|v| v.map(|n| json!(n)).unwrap_or(Value::Null))
                    },
                    "TEXT" | "VARCHAR" => {
                        row.try_get::<Option<String>, _>(i)
                            .map(|v| v.map(|s| json!(s)).unwrap_or(Value::Null))
                    },
                    "BOOL" => {
                        row.try_get::<Option<bool>, _>(i)
                            .map(|v| v.map(|b| json!(b)).unwrap_or(Value::Null))
                    },
                    "TIMESTAMPTZ" | "TIMESTAMP" => {
                        row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(i)
                            .map(|v| v.map(|dt| json!(dt.to_rfc3339())).unwrap_or(Value::Null))
                    },
                    _ => {
                        // Generic string conversion for unknown types
                        row.try_get::<Option<String>, _>(i)
                            .map(|v| v.map(|s| json!(s)).unwrap_or(Value::Null))
                    }
                }.unwrap_or(Value::Null);

                json_row.insert(column.name().to_string(), value);
            }
            
            json_rows.push(Value::Object(json_row));
        }

        let rows_count = json_rows.len() as u64;
        Ok(QueryResult {
            rows: json_rows,
            columns,
            affected_rows: Some(rows_count),
            execution_time: Some(start_time.elapsed().as_millis() as u64),
        })
    }

    /// Execute modification query (INSERT, UPDATE, DELETE)
    async fn execute_modification_query(&self, query: &str, start_time: Instant) -> Result<QueryResult> {
        let result = sqlx::query(query)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::service(&format!("Failed to execute query: {}", e)))?;

        Ok(QueryResult {
            rows: vec![],
            columns: vec![],
            affected_rows: Some(result.rows_affected()),
            execution_time: Some(start_time.elapsed().as_millis() as u64),
        })
    }

    /// Test connection health
    pub async fn health_check(&self) -> Result<bool> {
        match sqlx::query("SELECT 1").execute(&self.pool).await {
            Ok(_) => Ok(true),
            Err(e) => {
                log::warn!("PostgreSQL health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get connection pool statistics
    pub fn get_pool_stats(&self) -> HashMap<String, Value> {
        let mut stats = HashMap::new();
        stats.insert("provider".to_string(), json!("postgresql"));
        stats.insert("database_name".to_string(), json!(self.database_name));
        stats.insert("connection_string".to_string(), json!(self.connection_string));
        stats.insert("pool_size".to_string(), json!(self.pool.size()));
        stats.insert("pool_idle".to_string(), json!(self.pool.num_idle()));
        stats
    }

    /// Close the connection pool
    pub async fn close(&self) {
        self.pool.close().await;
    }
}
