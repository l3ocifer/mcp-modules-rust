use std::fmt;
use crate::transport::TransportError;

/// Authentication error type
#[derive(Debug)]
pub enum AuthError {
    /// Invalid credentials
    InvalidCredentials(String),
    /// OAuth error
    OAuth(String),
    /// Not authenticated
    NotAuthenticated,
    /// Configuration error
    Config(String),
    /// Token expired
    TokenExpired,
    /// No refresh token available
    NoRefreshToken,
    /// No token available
    NoToken,
    /// Token exchange error
    TokenExchangeError(String),
    /// Token refresh error
    TokenRefreshError(String),
    /// Invalid auth URL
    InvalidAuthUrl,
    /// Invalid token URL
    InvalidTokenUrl,
    /// Invalid redirect URI
    InvalidRedirectUri,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::InvalidCredentials(msg) => write!(f, "Invalid credentials: {}", msg),
            AuthError::OAuth(msg) => write!(f, "OAuth error: {}", msg),
            AuthError::NotAuthenticated => write!(f, "Not authenticated"),
            AuthError::Config(msg) => write!(f, "Configuration error: {}", msg),
            AuthError::TokenExpired => write!(f, "Token expired"),
            AuthError::NoRefreshToken => write!(f, "No refresh token available"),
            AuthError::NoToken => write!(f, "No token available"),
            AuthError::TokenExchangeError(msg) => write!(f, "Token exchange error: {}", msg),
            AuthError::TokenRefreshError(msg) => write!(f, "Token refresh error: {}", msg),
            AuthError::InvalidAuthUrl => write!(f, "Invalid authorization URL"),
            AuthError::InvalidTokenUrl => write!(f, "Invalid token URL"),
            AuthError::InvalidRedirectUri => write!(f, "Invalid redirect URI"),
        }
    }
}

impl std::error::Error for AuthError {}

/// Main error type
#[derive(Debug, Clone)]
pub enum Error {
    /// Configuration error
    Config(String),
    /// Service error
    Service(String),
    /// Authentication error
    Auth(String),
    /// Network error
    Network(String),
    /// Validation error
    Validation(String),
    /// Not found error
    NotFound(String),
    /// Rate limit error
    RateLimit(String),
    /// Internal error
    Internal(String),
    /// Protocol error
    Protocol(String),
    /// Connection error
    Connection(String),
    /// Capability error - server does not support the requested capability
    Capability(String),
    /// Permission error - client does not have permission to perform the action
    Permission(String),
    /// Cancellation error - operation was cancelled
    Cancelled(String),
    /// Transport error
    Transport(String),
    /// Parse error
    Parse(String),
    /// External error
    External(String),
    /// Invalid input error
    InvalidInput(String),
    /// IO error
    Io(String),
    /// Operation error
    Operation(String),
    /// Invalid parameters error
    InvalidParameters(String),
    /// API error - error from API calls
    Api(String),
}

impl Error {
    /// Create a new configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Error::Config(message.into())
    }
    
    /// Create a new service error
    pub fn service<S: Into<String>>(message: S) -> Self {
        Error::Service(message.into())
    }
    
    /// Create a new authentication error
    pub fn auth<S: Into<String>>(message: S) -> Self {
        Error::Auth(message.into())
    }
    
    /// Create a new network error
    pub fn network<S: Into<String>>(message: S) -> Self {
        Error::Network(message.into())
    }
    
    /// Create a new validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Error::Validation(message.into())
    }
    
    /// Create a new not found error
    pub fn not_found<S: Into<String>>(message: S) -> Self {
        Error::NotFound(message.into())
    }
    
    /// Create a new rate limit error
    pub fn rate_limit<S: Into<String>>(message: S) -> Self {
        Error::RateLimit(message.into())
    }
    
    /// Create a new internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Error::Internal(message.into())
    }
    
    /// Create a new protocol error
    pub fn protocol<S: Into<String>>(message: S) -> Self {
        Error::Protocol(message.into())
    }
    
    /// Create a new connection error
    pub fn connection<S: Into<String>>(message: S) -> Self {
        Error::Connection(message.into())
    }
    
    /// Create a new capability error
    pub fn capability<S: Into<String>>(message: S) -> Self {
        Error::Capability(message.into())
    }
    
    /// Create a new permission error
    pub fn permission<S: Into<String>>(message: S) -> Self {
        Error::Permission(message.into())
    }
    
    /// Create a new cancellation error
    pub fn cancelled<S: Into<String>>(message: S) -> Self {
        Error::Cancelled(message.into())
    }
    
    /// Create a transport error
    pub fn transport<S: Into<String>>(message: S) -> Self {
        Error::Transport(message.into())
    }
    
    /// Create a parse error
    pub fn parsing<S: Into<String>>(message: S) -> Self {
        Error::Parse(message.into())
    }
    
    /// Create an external error
    pub fn external<S: Into<String>>(message: S) -> Self {
        Error::External(message.into())
    }
    
    /// Create an invalid input error
    pub fn invalid_input<S: Into<String>>(message: S) -> Self {
        Error::InvalidInput(message.into())
    }
    
    /// Create an IO error
    pub fn io<S: Into<String>>(message: S) -> Self {
        Error::Io(message.into())
    }
    
    /// Create an operation error
    pub fn operation<S: Into<String>>(message: S) -> Self {
        Error::Operation(message.into())
    }
    
    /// Create an invalid parameters error
    pub fn invalid_parameters<S: Into<String>>(message: S) -> Self {
        Error::InvalidParameters(message.into())
    }
    
    /// Create an API error
    pub fn api<S: Into<String>>(message: S) -> Self {
        Error::Api(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(message) => write!(f, "Configuration error: {}", message),
            Error::Service(message) => write!(f, "Service error: {}", message),
            Error::Auth(message) => write!(f, "Authentication error: {}", message),
            Error::Network(message) => write!(f, "Network error: {}", message),
            Error::Validation(message) => write!(f, "Validation error: {}", message),
            Error::NotFound(message) => write!(f, "Not found error: {}", message),
            Error::RateLimit(message) => write!(f, "Rate limit error: {}", message),
            Error::Internal(message) => write!(f, "Internal error: {}", message),
            Error::Protocol(message) => write!(f, "Protocol error: {}", message),
            Error::Connection(message) => write!(f, "Connection error: {}", message),
            Error::Capability(message) => write!(f, "Capability error: {}", message),
            Error::Permission(message) => write!(f, "Permission error: {}", message),
            Error::Cancelled(message) => write!(f, "Cancelled: {}", message),
            Error::Transport(message) => write!(f, "Transport error: {}", message),
            Error::Parse(message) => write!(f, "Parse error: {}", message),
            Error::External(message) => write!(f, "External error: {}", message),
            Error::InvalidInput(message) => write!(f, "Invalid input error: {}", message),
            Error::Io(message) => write!(f, "IO error: {}", message),
            Error::Operation(message) => write!(f, "Operation error: {}", message),
            Error::InvalidParameters(message) => write!(f, "Invalid parameters error: {}", message),
            Error::Api(message) => write!(f, "API error: {}", message),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for DevOps Architect MCP
pub type Result<T> = std::result::Result<T, Error>;

/// From reqwest::Error to Error
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Error::Network(format!("Request timed out: {}", err))
        } else if err.is_connect() {
            Error::Connection(format!("Connection failed: {}", err))
        } else if err.is_status() {
            let status = err.status().unwrap_or_default();
            match status.as_u16() {
                401 | 403 => Error::Auth(format!("Authentication failed: {}", err)),
                404 => Error::NotFound(format!("Resource not found: {}", err)),
                429 => Error::RateLimit(format!("Rate limited: {}", err)),
                500..=599 => Error::Service(format!("Server error: {}", err)),
                _ => Error::Service(format!("HTTP error: {}", err)),
            }
        } else {
            Error::Network(format!("Request failed: {}", err))
        }
    }
}

/// From serde_json::Error to Error
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Protocol(format!("JSON error: {}", err))
    }
}

/// From std::io::Error to Error
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Internal(format!("I/O error: {}", err))
    }
}

/// From url::ParseError to Error
impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::Config(format!("URL parse error: {}", err))
    }
}

impl From<TransportError> for Error {
    fn from(err: TransportError) -> Self {
        match err {
            TransportError::ConnectionError(msg) => Error::network(msg),
            TransportError::SendError(msg) => Error::network(msg),
            TransportError::ReceiveError(msg) => Error::network(msg),
            TransportError::ParseError(msg) => Error::parsing(msg),
            TransportError::Timeout(msg) => Error::network(msg),
            TransportError::Protocol { code, message } => Error::protocol(format!("Code: {}, Message: {}", code, message)),
            TransportError::Json(err) => Error::from(err),
            TransportError::WebSocket(msg) => Error::network(msg),
            TransportError::Io(err) => Error::io(err.to_string()),
            TransportError::InvalidResponse(msg) => Error::invalid_input(msg),
            TransportError::Connection(msg) => Error::network(msg),
        }
    }
}

impl From<AuthError> for Error {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::OAuth(msg) => Error::auth(msg),
            AuthError::Config(msg) => Error::config(msg),
            AuthError::InvalidCredentials(msg) => Error::auth(msg),
            AuthError::NotAuthenticated => Error::auth("Not authenticated".to_string()),
            AuthError::TokenExpired => Error::auth("Token expired".to_string()),
            AuthError::NoRefreshToken => Error::auth("No refresh token available".to_string()),
            AuthError::NoToken => Error::auth("No token available".to_string()),
            AuthError::TokenExchangeError(msg) => Error::auth(msg),
            AuthError::TokenRefreshError(msg) => Error::auth(msg),
            AuthError::InvalidAuthUrl => Error::auth("Invalid authorization URL".to_string()),
            AuthError::InvalidTokenUrl => Error::auth("Invalid token URL".to_string()),
            AuthError::InvalidRedirectUri => Error::auth("Invalid redirect URI".to_string()),
        }
    }
}

// Also add from_auth_error implementation
impl From<crate::auth::AuthError> for Error {
    fn from(error: crate::auth::AuthError) -> Self {
        Error::auth(error.to_string())
    }
} 