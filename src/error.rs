use std::string::ToString;
use tokio::task::JoinError;
use std::sync::PoisonError;
use std::time::SystemTimeError;

/// Comprehensive error type for the DevOps MCP library with enhanced handling
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Authentication and authorization errors
    #[error("Authentication error: {message}")]
    Auth { 
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        recoverable: bool,
    },

    /// Configuration-related errors  
    #[error("Configuration error: {message}")]
    Config { 
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        suggestion: Option<String>,
    },

    /// Network and connection errors
    #[error("Network error: {message}")]
    Network { 
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        retry_after: Option<std::time::Duration>,
        endpoint: Option<String>,
    },

    /// Service-specific errors
    #[error("Service error: {message}")]
    Service { 
        message: String,
        service_name: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        error_code: Option<String>,
    },

    /// Transport layer errors
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),

    /// Protocol-level errors
    #[error("Protocol error: {message}")]
    Protocol { 
        message: String,
        protocol_version: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Data parsing and validation errors
    #[error("Parsing error: {message}")]
    Parsing { 
        message: String,
        expected_format: Option<String>,
        actual_input: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Input validation errors
    #[error("Validation error: {message}")]
    Validation { 
        message: String,
        field: Option<String>,
        constraint: Option<String>,
    },

    /// Resource not found errors
    #[error("Resource not found: {message}")]
    NotFound { 
        message: String,
        resource_type: Option<String>,
        resource_id: Option<String>,
    },

    /// Internal system errors
    #[error("Internal error: {message}")]
    Internal { 
        message: String,
        operation: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Invalid data errors
    #[error("Invalid data: {message}")]
    InvalidData { 
        message: String,
        data_type: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Connection-related errors
    #[error("Connection error: {message}")]
    Connection { 
        message: String,
        endpoint: Option<String>,
        retry_count: Option<u32>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Timeout errors
    #[error("Operation timed out: {message}")]
    Timeout { 
        message: String,
        operation: Option<String>,
        duration: Option<std::time::Duration>,
    },

    /// Capability errors (when feature is not supported)
    #[error("Capability not supported: {message}")]
    Capability { 
        message: String,
        required_feature: Option<String>,
        alternative: Option<String>,
    },

    /// API-specific errors
    #[error("API error: {message}")]
    Api { 
        message: String,
        api_name: Option<String>,
        status_code: Option<u16>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// I/O operation errors
    #[error("I/O error: {message}")]
    Io { 
        message: String,
        operation: Option<String>,
        path: Option<std::path::PathBuf>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// Transport-specific error types
#[derive(Debug, thiserror::Error, Clone)]
pub enum TransportError {
    #[error("Connection failed: {message}")]
    ConnectionFailed { 
        message: String,
        retry_count: Option<u32>,
    },

    #[error("Request failed: {message}")]
    RequestFailed { 
        message: String,
        method: Option<String>,
    },

    #[error("Protocol error: {message} (code: {code})")]
    Protocol { 
        message: String,
        code: i64,
    },

    #[error("Serialization error: {message}")]
    Serialization { 
        message: String,
        format: Option<String>,
    },

    #[error("Authentication failed: {message}")]
    AuthenticationFailed { 
        message: String,
        auth_type: Option<String>,
    },

    #[error("Rate limit exceeded: {message}")]
    RateLimitExceeded { 
        message: String,
        retry_after: Option<std::time::Duration>,
    },

    #[error("Transport not supported: {transport_type}")]
    NotSupported { 
        transport_type: String,
    },

    #[error("Request timeout: {message}")]
    RequestTimeout { 
        message: String,
        duration: Option<std::time::Duration>,
    },
}

/// Error recovery strategy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry { 
        max_attempts: u32,
        delay: std::time::Duration,
    },
    /// Fallback to alternative method
    Fallback { 
        alternative: String,
    },
    /// Skip and continue
    Skip,
    /// Fail immediately
    Fail,
    /// Manual intervention required
    Manual { 
        instructions: String,
    },
}

/// Error context for better diagnostics
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub module: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(operation: &str, module: &str) -> Self {
        Self {
            operation: operation.to_string(),
            module: module.to_string(),
            timestamp: chrono::Utc::now(),
            request_id: None,
            user_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    pub fn with_metadata(mut self, key: &str, value: impl Into<String>) -> Self {
        self.metadata.insert(key.to_string(), value.into());
        self
    }
}

/// Enhanced error with context and recovery information
#[derive(Debug)]
pub struct ContextualError {
    pub error: Error,
    pub context: ErrorContext,
    pub recovery_strategy: Option<RecoveryStrategy>,
    pub severity: ErrorSeverity,
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Low severity - informational
    Info,
    /// Warning - operation may continue
    Warning,
    /// Error - operation failed but system stable
    Error,
    /// Critical - system stability affected
    Critical,
    /// Fatal - immediate attention required
    Fatal,
}

impl Error {
    /// Convenience constructors for common error types
    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth {
            message: message.into(),
            source: None,
            recoverable: true,
        }
    }

    pub fn auth_with_source(message: impl Into<String>, source: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Auth {
            message: message.into(),
            source: Some(source),
            recoverable: true,
        }
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            source: None,
            suggestion: None,
        }
    }

    pub fn config_with_suggestion(message: impl Into<String>, suggestion: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            source: None,
            suggestion: Some(suggestion.into()),
        }
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
            source: None,
            retry_after: None,
            endpoint: None,
        }
    }

    pub fn network_with_endpoint(message: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
            source: None,
            retry_after: None,
            endpoint: Some(endpoint.into()),
        }
    }

    pub fn network_with_retry(message: impl Into<String>, retry_after: std::time::Duration) -> Self {
        Self::Network {
            message: message.into(),
            source: None,
            retry_after: Some(retry_after),
            endpoint: None,
        }
    }

    pub fn service(message: impl Into<String>) -> Self {
        Self::Service {
            message: message.into(),
            service_name: None,
            source: None,
            error_code: None,
        }
    }

    pub fn service_with_code(message: impl Into<String>, service_name: impl Into<String>, error_code: impl Into<String>) -> Self {
        Self::Service {
            message: message.into(),
            service_name: Some(service_name.into()),
            source: None,
            error_code: Some(error_code.into()),
        }
    }

    pub fn transport(err: TransportError) -> Self {
        Self::Transport(err)
    }

    pub fn protocol(message: impl Into<String>) -> Self {
        Self::Protocol {
            message: message.into(),
            protocol_version: None,
            source: None,
        }
    }

    pub fn parsing(message: impl Into<String>) -> Self {
        Self::Parsing {
            message: message.into(),
            expected_format: None,
            actual_input: None,
            source: None,
        }
    }

    pub fn parsing_with_format(message: impl Into<String>, expected: impl Into<String>, actual: Option<String>) -> Self {
        Self::Parsing {
            message: message.into(),
            expected_format: Some(expected.into()),
            actual_input: actual,
            source: None,
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            field: None,
            constraint: None,
        }
    }

    pub fn validation_with_field(message: impl Into<String>, field: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            field: Some(field.into()),
            constraint: None,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            message: message.into(),
            resource_type: None,
            resource_id: None,
        }
    }

    pub fn not_found_with_resource(message: impl Into<String>, resource_type: impl Into<String>, resource_id: impl Into<String>) -> Self {
        Self::NotFound {
            message: message.into(),
            resource_type: Some(resource_type.into()),
            resource_id: Some(resource_id.into()),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            operation: None,
            source: None,
        }
    }

    pub fn internal_with_source(message: impl Into<String>, source: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Internal {
            message: message.into(),
            operation: None,
            source: Some(source),
        }
    }

    pub fn invalid_data(message: impl Into<String>) -> Self {
        Self::InvalidData {
            message: message.into(),
            data_type: None,
            source: None,
        }
    }

    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
            endpoint: None,
            retry_count: None,
            source: None,
        }
    }

    pub fn connection_with_endpoint(message: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
            endpoint: Some(endpoint.into()),
            retry_count: None,
            source: None,
        }
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout {
            message: message.into(),
            operation: None,
            duration: None,
        }
    }

    pub fn timeout_with_duration(message: impl Into<String>, duration: std::time::Duration) -> Self {
        Self::Timeout {
            message: message.into(),
            operation: None,
            duration: Some(duration),
        }
    }

    pub fn capability(message: impl Into<String>) -> Self {
        Self::Capability {
            message: message.into(),
            required_feature: None,
            alternative: None,
        }
    }

    pub fn api(message: impl Into<String>) -> Self {
        Self::Api {
            message: message.into(),
            api_name: None,
            status_code: None,
            source: None,
        }
    }

    pub fn api_with_status(message: impl Into<String>, api_name: impl Into<String>, status_code: u16) -> Self {
        Self::Api {
            message: message.into(),
            api_name: Some(api_name.into()),
            status_code: Some(status_code),
            source: None,
        }
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
            operation: None,
            path: None,
            source: None,
        }
    }

    pub fn io_with_path(message: impl Into<String>, path: std::path::PathBuf) -> Self {
        Self::Io {
            message: message.into(),
            operation: None,
            path: Some(path),
            source: None,
        }
    }

    /// Additional methods for compatibility with existing codebase
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            field: None,
            constraint: Some("input validation failed".to_string()),
        }
    }

    /// External error compatibility (maps to internal errors)
    pub fn external(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            operation: Some("external_service".to_string()),
            source: None,
        }
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Error::Auth { recoverable, .. } => *recoverable,
            Error::Network { retry_after, .. } => retry_after.is_some(),
            Error::Connection { .. } => true,
            Error::Timeout { .. } => true,
            Error::Transport(transport_err) => transport_err.is_recoverable(),
            Error::Service { .. } => false,
            Error::Protocol { .. } => false,
            Error::Parsing { .. } => false,
            Error::Validation { .. } => false,
            Error::NotFound { .. } => false,
            Error::Internal { .. } => false,
            Error::InvalidData { .. } => false,
            Error::Capability { .. } => false,
            Error::Api { status_code, .. } => {
                // Some API errors are recoverable (5xx server errors)
                if let Some(code) = status_code {
                    *code >= 500
                } else {
                    false
                }
            },
            Error::Io { .. } => false,
            Error::Config { .. } => false,
        }
    }

    /// Get error category for logging and metrics
    pub fn category(&self) -> &'static str {
        match self {
            Error::Auth { .. } => "authentication",
            Error::Config { .. } => "configuration",
            Error::Network { .. } => "network",
            Error::Service { .. } => "service",
            Error::Transport(_) => "transport",
            Error::Protocol { .. } => "protocol",
            Error::Parsing { .. } => "parsing",
            Error::Validation { .. } => "validation",
            Error::NotFound { .. } => "not_found",
            Error::Internal { .. } => "internal",
            Error::InvalidData { .. } => "invalid_data",
            Error::Connection { .. } => "connection",
            Error::Timeout { .. } => "timeout",
            Error::Capability { .. } => "capability",
            Error::Api { .. } => "api",
            Error::Io { .. } => "io",
        }
    }

    /// Get suggested recovery strategy
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Error::Network { retry_after, .. } => {
                if let Some(delay) = retry_after {
                    RecoveryStrategy::Retry { max_attempts: 3, delay: *delay }
                } else {
                    RecoveryStrategy::Retry { max_attempts: 3, delay: std::time::Duration::from_secs(1) }
                }
            },
            Error::Connection { .. } => RecoveryStrategy::Retry { max_attempts: 5, delay: std::time::Duration::from_secs(2) },
            Error::Timeout { .. } => RecoveryStrategy::Retry { max_attempts: 2, delay: std::time::Duration::from_secs(5) },
            Error::Auth { recoverable: true, .. } => RecoveryStrategy::Manual { instructions: "Please check your credentials and try again".to_string() },
            Error::Api { status_code: Some(code), .. } if *code >= 500 => RecoveryStrategy::Retry { max_attempts: 3, delay: std::time::Duration::from_secs(30) },
            Error::Service { .. } => RecoveryStrategy::Fallback { alternative: "Try alternative service provider".to_string() },
            Error::NotFound { .. } => RecoveryStrategy::Skip,
            Error::Capability { alternative: Some(alt), .. } => RecoveryStrategy::Fallback { alternative: alt.clone() },
            _ => RecoveryStrategy::Fail,
        }
    }

    /// Create contextual error with recovery information
    pub fn with_context(self, context: ErrorContext) -> ContextualError {
        let severity = match &self {
            Error::Internal { .. } => ErrorSeverity::Critical,
            Error::Auth { .. } => ErrorSeverity::Error,
            Error::Network { .. } | Error::Connection { .. } | Error::Timeout { .. } => ErrorSeverity::Warning,
            Error::NotFound { .. } | Error::Validation { .. } => ErrorSeverity::Info,
            _ => ErrorSeverity::Error,
        };

        let recovery_strategy = Some(self.recovery_strategy());

        ContextualError {
            error: self,
            context,
            recovery_strategy,
            severity,
        }
    }

    pub fn operation(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            operation: Some("operation".to_string()),
            source: None,
        }
    }
}

impl TransportError {
    pub fn connection_failed(message: impl Into<String>) -> Self {
        Self::ConnectionFailed {
            message: message.into(),
            retry_count: None,
        }
    }

    pub fn request_failed(message: impl Into<String>) -> Self {
        Self::RequestFailed {
            message: message.into(),
            method: None,
        }
    }

    pub fn protocol_error(message: impl Into<String>, code: i64) -> Self {
        Self::Protocol {
            message: message.into(),
            code,
        }
    }

    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
            format: None,
        }
    }

    pub fn auth_failed(message: impl Into<String>) -> Self {
        Self::AuthenticationFailed {
            message: message.into(),
            auth_type: None,
        }
    }

    pub fn rate_limited(message: impl Into<String>, retry_after: Option<std::time::Duration>) -> Self {
        Self::RateLimitExceeded {
            message: message.into(),
            retry_after,
        }
    }

    pub fn not_supported(transport_type: impl Into<String>) -> Self {
        Self::NotSupported {
            transport_type: transport_type.into(),
        }
    }

    pub fn request_timeout(message: impl Into<String>, duration: Option<std::time::Duration>) -> Self {
        Self::RequestTimeout {
            message: message.into(),
            duration,
        }
    }

    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::ConnectionFailed { .. } => true,
            Self::RequestFailed { .. } => true,
            Self::Protocol { code, .. } => *code >= -32000 && *code <= -32099, // JSON-RPC recoverable errors
            Self::Serialization { .. } => false,
            Self::AuthenticationFailed { .. } => true,
            Self::RateLimitExceeded { .. } => true,
            Self::NotSupported { .. } => false,
            Self::RequestTimeout { .. } => true,
        }
    }
}

impl ContextualError {
    /// Log the error with appropriate level
    pub fn log(&self) {
        let level = match self.severity {
            ErrorSeverity::Info => log::Level::Info,
            ErrorSeverity::Warning => log::Level::Warn,
            ErrorSeverity::Error => log::Level::Error,
            ErrorSeverity::Critical | ErrorSeverity::Fatal => log::Level::Error,
        };

        log::log!(
            level,
            "[{}::{}] {}: {} | Context: {:?} | Recovery: {:?}",
            self.context.module,
            self.context.operation,
            self.error.category(),
            self.error,
            self.context.metadata,
            self.recovery_strategy
        );

        // Also log to tracing for structured logging
        match self.severity {
            ErrorSeverity::Info => tracing::info!(
                error = %self.error,
                category = self.error.category(),
                module = %self.context.module,
                operation = %self.context.operation,
                timestamp = %self.context.timestamp,
                request_id = %self.context.request_id.as_deref().unwrap_or("none"),
                metadata = ?self.context.metadata,
                recovery_strategy = ?self.recovery_strategy,
                "Error occurred"
            ),
            ErrorSeverity::Warning => tracing::warn!(
                error = %self.error,
                category = self.error.category(),
                module = %self.context.module,
                operation = %self.context.operation,
                timestamp = %self.context.timestamp,
                request_id = %self.context.request_id.as_deref().unwrap_or("none"),
                metadata = ?self.context.metadata,
                recovery_strategy = ?self.recovery_strategy,
                "Warning occurred"
            ),
            ErrorSeverity::Error => tracing::error!(
                error = %self.error,
                category = self.error.category(),
                module = %self.context.module,
                operation = %self.context.operation,
                timestamp = %self.context.timestamp,
                request_id = %self.context.request_id.as_deref().unwrap_or("none"),
                metadata = ?self.context.metadata,
                recovery_strategy = ?self.recovery_strategy,
                "Error occurred"
            ),
            ErrorSeverity::Critical | ErrorSeverity::Fatal => tracing::error!(
                error = %self.error,
                category = self.error.category(),
                module = %self.context.module,
                operation = %self.context.operation,
                timestamp = %self.context.timestamp,
                request_id = %self.context.request_id.as_deref().unwrap_or("none"),
                metadata = ?self.context.metadata,
                recovery_strategy = ?self.recovery_strategy,
                severity = ?self.severity,
                "Critical error occurred"
            ),
        }
    }
}

// Implement From traits for standard error types
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::io(format!("I/O operation failed: {}", err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::parsing_with_format(
            format!("JSON parsing failed: {}", err),
            "valid JSON",
            None
        )
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::validation(format!("Invalid URL format: {}", err))
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Error::timeout("HTTP request timed out")
        } else if err.is_connect() {
            Error::connection(format!("Failed to connect: {}", err))
        } else if err.is_status() {
            let status_code = err.status().map(|s| s.as_u16()).unwrap_or(0);
            Error::api_with_status(format!("HTTP error: {}", err), "HTTP", status_code)
        } else {
            Error::network(format!("Network error: {}", err))
        }
    }
}

impl From<chrono::ParseError> for Error {
    fn from(err: chrono::ParseError) -> Self {
        Error::parsing_with_format(
            format!("Date/time parsing failed: {}", err),
            "valid datetime format",
            None
        )
    }
}

impl From<JoinError> for Error {
    fn from(err: JoinError) -> Self {
        Error::internal(format!("Task join error: {}", err))
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(err: PoisonError<T>) -> Self {
        Error::internal(format!("Mutex poison error: {}", err))
    }
}

impl From<SystemTimeError> for Error {
    fn from(err: SystemTimeError) -> Self {
        Error::internal(format!("System time error: {}", err))
    }
}

// Result type alias for convenience
pub type Result<T> = std::result::Result<T, Error>;

/// Utility functions for error handling
pub mod utils {
    use super::*;
    use std::borrow::Cow;

    /// Retry an operation with exponential backoff and optimized memory usage
    pub async fn retry_with_backoff<T, F, Fut>(
        operation: F,
        max_attempts: u32,
        initial_delay: std::time::Duration,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempts = 0;
        let mut delay = initial_delay;
        const MAX_DELAY: std::time::Duration = std::time::Duration::from_secs(30);

        loop {
            attempts += 1;
            
            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) if attempts >= max_attempts => return Err(err),
                Err(err) if err.is_recoverable() => {
                    tracing::warn!(
                        attempt = attempts,
                        max_attempts = max_attempts,
                        delay_ms = delay.as_millis(),
                        error = %err,
                        "Retrying operation after error"
                    );
                    
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(delay.saturating_mul(2), MAX_DELAY); // Prevent overflow
                },
                Err(err) => return Err(err), // Non-recoverable error
            }
        }
    }

    /// Execute multiple operations and collect successful results with pre-allocation
    pub async fn collect_results<T, E>(
        results: Vec<std::result::Result<T, E>>
    ) -> (Vec<T>, Vec<E>) {
        let total_len = results.len();
        let mut successes = Vec::with_capacity(total_len);
        let mut errors = Vec::with_capacity(total_len / 10); // Assume 10% error rate

        for result in results {
            match result {
                Ok(value) => successes.push(value),
                Err(error) => errors.push(error),
            }
        }

        // Shrink error vector if over-allocated
        errors.shrink_to_fit();

        (successes, errors)
    }

    /// Graceful degradation helper with zero-copy error messages
    pub async fn graceful_fallback<T, F, Fallback>(
        operation: F,
        fallback: Fallback,
        operation_name: &'static str,
    ) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
        Fallback: std::future::Future<Output = Result<T>>,
    {
        match operation.await {
            Ok(result) => Ok(result),
            Err(err) => {
                tracing::warn!(
                    operation = operation_name,
                    error = %err,
                    "Primary operation failed, attempting fallback"
                );
                
                fallback.await.map_err(|fallback_err| {
                    let context = ErrorContext::new(operation_name, "fallback")
                        .with_metadata("primary_error", err.to_string())
                        .with_metadata("fallback_error", fallback_err.to_string());
                    
                    Error::internal("Both primary and fallback operations failed").with_context(context).error
                })
            }
        }
    }

    /// Timeout wrapper with efficient error context
    pub async fn with_timeout<T, F>(
        future: F,
        timeout: std::time::Duration,
        operation_name: &'static str,
    ) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        match tokio::time::timeout(timeout, future).await {
            Ok(result) => result,
            Err(_) => {
                let context = ErrorContext::new(operation_name, "timeout")
                    .with_metadata("timeout_seconds", timeout.as_secs().to_string());
                
                Err(Error::timeout(Cow::Borrowed("Operation timed out")).with_context(context).error)
            }
        }
    }

    /// Batch operation processor with optimized memory usage and proper Clone constraint
    pub async fn process_batch<T, R, F, Fut>(
        items: Vec<T>,
        batch_size: usize,
        processor: F,
    ) -> Result<Vec<R>>
    where
        T: Clone,
        F: Fn(Vec<T>) -> Fut,
        Fut: std::future::Future<Output = Result<Vec<R>>>,
    {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let total_items = items.len();
        let mut results = Vec::with_capacity(total_items);
        
        // Process in optimized chunks
        for chunk in items.chunks(batch_size) {
            let batch_results = processor(chunk.to_vec()).await?;
            results.extend(batch_results);
        }

        Ok(results)
    }

    /// Efficient string formatting for error messages
    pub fn format_error_message(template: &str, args: &[(&str, &dyn std::fmt::Display)]) -> String {
        let mut message = String::with_capacity(template.len() + args.len() * 16); // Estimate capacity
        message.push_str(template);
        
        for (key, value) in args {
            let placeholder = format!("{{{}}}", key);
            if let Some(pos) = message.find(&placeholder) {
                message.replace_range(pos..pos + placeholder.len(), &value.to_string());
            }
        }
        
        message
    }
}

/// Timeout utility for performance optimization
pub async fn with_timeout<F, T>(duration: std::time::Duration, future: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    match tokio::time::timeout(duration, future).await {
        Ok(result) => result,
        Err(_) => Err(Error::timeout(format!("Operation timed out after {:?}", duration))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categorization() {
        let auth_err = Error::auth("Invalid credentials");
        assert_eq!(auth_err.category(), "authentication");
        assert!(auth_err.is_recoverable());

        let config_err = Error::config("Missing API key");
        assert_eq!(config_err.category(), "configuration");
        assert!(!config_err.is_recoverable());
    }

    #[test]
    fn test_contextual_error() {
        let context = ErrorContext::new("test_operation", "test_module")
            .with_request_id("req_123".to_string())
            .with_metadata("user_id", "user_456");

        let err = Error::network("Connection failed");
        let contextual_err = err.with_context(context);

        assert_eq!(contextual_err.context.operation, "test_operation");
        assert_eq!(contextual_err.context.module, "test_module");
        assert_eq!(contextual_err.context.request_id, Some("req_123".to_string()));
    }

    #[test]
    fn test_recovery_strategies() {
        let network_err = Error::network("Temporary network issue");
        match network_err.recovery_strategy() {
            RecoveryStrategy::Retry { max_attempts, .. } => assert_eq!(max_attempts, 3),
            _ => panic!("Expected retry strategy for network error"),
        }

        let not_found_err = Error::not_found("Resource not found");
        assert_eq!(not_found_err.recovery_strategy(), RecoveryStrategy::Skip);
    }

    #[tokio::test]
    async fn test_retry_with_backoff() {
        let mut attempt_count = 0;
        
        let operation = || {
            attempt_count += 1;
            async move {
                if attempt_count < 3 {
                    Err(Error::network("Temporary failure"))
                } else {
                    Ok("Success")
                }
            }
        };

        let result = utils::retry_with_backoff(
            operation,
            5,
            std::time::Duration::from_millis(10)
        ).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        assert_eq!(attempt_count, 3);
    }
} 