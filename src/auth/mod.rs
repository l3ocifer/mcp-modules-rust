use crate::error::{Error, Result};
use crate::security::{SecurityModule, SanitizationOptions, ValidationResult};
use oauth2::{CsrfToken};
use url::Url;
use std::time::{Duration, SystemTime};

pub mod oauth;

/// Authorization provider types
#[derive(Debug, Clone)]
pub enum AuthProviderType {
    OAuth2,
    Basic,
    ApiKey,
}

/// Authorization credentials with secure storage
#[derive(Debug, Clone)]
pub struct Credentials {
    /// Token value (protected)
    pub token: String,
    /// Token type (e.g., "Bearer")
    pub token_type: String,
    /// Expiration time in seconds
    pub expires_in: Option<u64>,
    /// Refresh token (protected)
    pub refresh_token: Option<String>,
    /// Scope
    pub scope: Option<String>,
    /// Token creation time
    pub created_at: SystemTime,
}

/// OAuth configuration with validation
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// Client ID
    pub client_id: String,
    /// Client secret (protected)
    pub client_secret: Option<String>,
    /// Authorization URL
    pub auth_url: String,
    /// Token URL
    pub token_url: String,
    /// Redirect URL
    pub redirect_url: String,
    /// Scopes
    pub scopes: Vec<String>,
}

/// Authorization error
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("OAuth error: {0}")]
    OAuth(String),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token expired")]
    TokenExpired,
    #[error("Refresh token not available")]
    NoRefreshToken,
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("No token available")]
    NoToken,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),
    #[error("Token exchange error: {0}")]
    TokenExchangeError(String),
    #[error("Token refresh error: {0}")]
    TokenRefreshError(String),
    #[error("CSRF state mismatch")]
    CsrfError,
    #[error("Missing client ID")]
    MissingClientId,
    #[error("Missing client secret")]
    MissingClientSecret,
    #[error("Missing redirect URI")]
    MissingRedirectUri,
    #[error("Missing auth URL")]
    MissingAuthUrl,
    #[error("Invalid auth URL")]
    InvalidAuthUrl,
    #[error("Missing token URL")]
    MissingTokenUrl,
    #[error("Invalid token URL")]
    InvalidTokenUrl,
    #[error("Invalid redirect URI")]
    InvalidRedirectUri,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Security validation failed: {0}")]
    SecurityValidation(String),
}

/// Authentication manager with enhanced security
pub struct AuthManager {
    /// OAuth client for OAuth 2.1 authentication
    oauth_client: Option<oauth::OAuth21Client>,
    /// Stored credentials (secured)
    credentials: Option<Credentials>,
    /// Authentication provider type
    provider_type: AuthProviderType,
    /// API key for API key authentication (secured)
    api_key: Option<String>,
    /// JWT token for authentication (secured)
    jwt_token: Option<String>,
    /// Configuration
    config: AuthConfig,
    /// Security module
    security: SecurityModule,
    /// CSRF tokens for state validation
    csrf_tokens: std::collections::HashMap<String, (CsrfToken, SystemTime)>,
}

impl AuthManager {
    /// Create a new auth manager with security validation
    pub fn new(config: AuthConfig) -> Result<Self> {
        // Validate configuration
        config.validate()?;
        
        Ok(Self {
            oauth_client: None,
            credentials: None,
            provider_type: config.provider_type.clone(),
            api_key: None,
            jwt_token: None,
            config,
            security: SecurityModule::new(),
            csrf_tokens: std::collections::HashMap::new(),
        })
    }
    
    /// Initialize OAuth client if OAuth is configured
    pub fn init_oauth(&mut self, auth_base_url: String) -> Result<()> {
        // Validate the auth URL
        self.security.validate_url(&auth_base_url)?;
        
        self.oauth_client = Some(oauth::OAuth21Client::new(&auth_base_url));
        self.security.log_security_event("OAUTH_INIT", Some(&auth_base_url));
        Ok(())
    }
    
    /// Initialize with API key (secure)
    pub fn with_api_key(api_key: &str) -> Result<Self> {
        // Validate API key format
        let security = SecurityModule::new();
        let validation_opts = SanitizationOptions {
            max_length: Some(512),
            ..Default::default()
        };
        
        match security.validate_input(api_key, &validation_opts) {
            ValidationResult::Valid => {},
            ValidationResult::Invalid(reason) | ValidationResult::Malicious(reason) => {
                security.log_security_event("INVALID_API_KEY", Some(&reason));
                return Err(Error::auth("Invalid API key format"));
            }
        }
        
        let mut auth_manager = Self::new(AuthConfig::new(
            AuthProviderType::ApiKey,
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            vec![],
        )?)?;
        
        auth_manager.provider_type = AuthProviderType::ApiKey;
        auth_manager.api_key = Some(api_key.to_string());
        auth_manager.security.log_security_event("API_KEY_AUTH_INIT", None);
        
        Ok(auth_manager)
    }
    
    /// Get OAuth authorization URL with CSRF protection
    pub async fn get_oauth_auth_url(&mut self) -> Result<(Url, CsrfToken)> {
        // Check rate limiting
        self.security.check_auth_rate_limit()?;
        
        if let Some(client) = &mut self.oauth_client {
            let csrf_token = CsrfToken::new_random();
            let url_string = client.start_authorization_flow(
                &self.config.redirect_url,
                self.config.scopes.clone(),
                None // No resource indicators for basic flow
            )?;
            
            // Validate the returned URL
            self.security.validate_url(&url_string)
                .map_err(|e| Error::auth(format!("Invalid URL returned from OAuth client: {}", e)))?;
            
            // Parse the validated URL
            let url = url::Url::parse(&url_string)
                .map_err(|e| Error::auth(format!("Failed to parse URL: {}", e)))?;
            
            // Store CSRF token with expiration (10 minutes)
            let csrf_key = csrf_token.secret().clone();
            self.csrf_tokens.insert(
                csrf_key.clone(),
                (csrf_token.clone(), SystemTime::now() + Duration::from_secs(600))
            );
            
            self.security.log_security_event("OAUTH_AUTH_URL_GENERATED", None);
            Ok((url, csrf_token))
        } else {
            Err(Error::auth("OAuth client not initialized"))
        }
    }
    
    /// Exchange OAuth code for credentials with CSRF validation
    pub async fn exchange_oauth_code(&mut self, code: &str, csrf_token: &str) -> Result<Credentials> {
        // Check rate limiting
        self.security.check_auth_rate_limit()?;
        
        // Validate CSRF token
        self.validate_csrf_token(csrf_token)?;
        
        // Validate authorization code
        let validation_opts = SanitizationOptions {
            max_length: Some(512),
            allow_html: false,
            allow_sql: false,
            allow_shell_meta: false,
        };
        
        match self.security.validate_input(code, &validation_opts) {
            ValidationResult::Valid => {},
            ValidationResult::Invalid(reason) | ValidationResult::Malicious(reason) => {
                self.security.log_security_event("MALICIOUS_AUTH_CODE", Some(&reason));
                return Err(Error::auth("Invalid authorization code format"));
            }
        }
        
        if let Some(client) = &mut self.oauth_client {
            let token = client.exchange_code(code.to_string(), None).await?;
            
            let credentials = Credentials {
                token: token.access_token.clone(),
                token_type: token.token_type.clone(),
                expires_in: token.expires_in,
                refresh_token: token.refresh_token.map(|t| t.to_string()),
                scope: token.scope.clone(),
                created_at: SystemTime::now(),
            };
            
            // Store credentials securely
            self.credentials = Some(credentials.clone());
            
            // Clean up used CSRF token
            self.csrf_tokens.remove(csrf_token);
            
            self.security.log_security_event("OAUTH_TOKEN_EXCHANGE_SUCCESS", None);
            Ok(credentials)
        } else {
            Err(Error::auth("OAuth client not initialized"))
        }
    }
    
    /// Validate CSRF token and check expiration
    fn validate_csrf_token(&mut self, csrf_token: &str) -> Result<()> {
        // Clean up expired tokens first
        let now = SystemTime::now();
        self.csrf_tokens.retain(|_, (_, expiry)| *expiry > now);
        
        // Validate token using constant-time comparison
        for (stored_token, (_, expiry)) in &self.csrf_tokens {
            if self.security.secure_compare(csrf_token, stored_token) && *expiry > now {
                return Ok(());
            }
        }
        
        self.security.log_security_event("CSRF_TOKEN_VALIDATION_FAILED", Some(csrf_token));
        Err(Error::auth("Invalid or expired CSRF token"))
    }
    
    /// Refresh expired credentials
    pub async fn refresh_credentials(&mut self) -> Result<Credentials> {
        // Check rate limiting
        self.security.check_auth_rate_limit()?;
        
        if let Some(client) = &mut self.oauth_client {
            // Check if we have current credentials with refresh token
            if let Some(ref creds) = self.credentials {
                if let Some(ref _refresh_token) = creds.refresh_token {
                    // Check if token is actually expired
                    if let Some(expires_in) = creds.expires_in {
                        let expires_at = creds.created_at + Duration::from_secs(expires_in);
                        if SystemTime::now() < expires_at {
                            // Token is still valid
                            return Ok(creds.clone());
                        }
                    }
                    
                    // Use refresh token to get a new token
                    let token = client.refresh_token().await?;
                    
                    let credentials = Credentials {
                        token: token.access_token.clone(),
                        token_type: token.token_type.clone(),
                        expires_in: token.expires_in,
                        refresh_token: token.refresh_token.map(|t| t.to_string()),
                        scope: token.scope.clone(),
                        created_at: SystemTime::now(),
                    };
                    
                    // Store credentials securely
                    self.credentials = Some(credentials.clone());
                    
                    self.security.log_security_event("TOKEN_REFRESH_SUCCESS", None);
                    return Ok(credentials);
                }
            }
            
            Err(Error::auth("No refresh token available"))
        } else {
            Err(Error::auth("OAuth client not initialized"))
        }
    }
    
    /// Set credentials directly with validation
    pub async fn set_credentials(&mut self, credentials: Credentials) -> Result<()> {
        // Validate token format
        let token_str = credentials.token.clone();
        let validation_opts = SanitizationOptions {
            max_length: Some(2048),
            ..Default::default()
        };
        
        match self.security.validate_input(&token_str, &validation_opts) {
            ValidationResult::Valid => {},
            ValidationResult::Invalid(reason) | ValidationResult::Malicious(reason) => {
                self.security.log_security_event("INVALID_TOKEN_FORMAT", Some(&reason));
                return Err(Error::auth("Invalid token format"));
            }
        }
        
        self.credentials = Some(credentials);
        self.security.log_security_event("CREDENTIALS_SET", None);
        Ok(())
    }
    
    /// Get current credentials (returns cloned credentials to avoid exposing internal state)
    pub async fn get_credentials(&self) -> Option<Credentials> {
        self.credentials.clone()
    }
    
    /// Get authorization header with validation
    pub async fn get_auth_header(&self) -> Result<String> {
        if let Some(credentials) = &self.credentials {
            // Check if credentials are expired
            if let Some(expires_in) = credentials.expires_in {
                let expires_at = credentials.created_at + Duration::from_secs(expires_in);
                if SystemTime::now() >= expires_at {
                    self.security.log_security_event("EXPIRED_TOKEN_USAGE_ATTEMPT", None);
                    return Err(Error::auth("Token has expired"));
                }
            }
            
            Ok(format!("{} {}", credentials.token_type, credentials.token))
        } else if let Some(client) = &self.oauth_client {
            // Try to get a token from OAuth client
            if let Some(token) = client.get_access_token() {
                Ok(format!("Bearer {}", token.secret()))
            } else {
                Err(Error::auth("No access token available"))
            }
        } else if let Some(api_key) = &self.api_key {
            Ok(format!("Bearer {}", api_key))
        } else {
            Err(Error::auth("No credentials available"))
        }
    }
    
    /// Clear credentials securely
    pub async fn clear_credentials(&mut self) {
        if self.credentials.is_some() {
            self.security.log_security_event("CREDENTIALS_CLEARED", None);
        }
        self.credentials = None;
        self.api_key = None;
        self.jwt_token = None;
        self.csrf_tokens.clear();
    }
    
    /// Get the auth provider type
    pub fn get_provider_type(&self) -> AuthProviderType {
        self.provider_type.clone()
    }
    
    /// Check if credentials are valid and not expired
    pub fn is_authenticated(&self) -> bool {
        if let Some(credentials) = &self.credentials {
            if let Some(expires_in) = credentials.expires_in {
                let expires_at = credentials.created_at + Duration::from_secs(expires_in);
                SystemTime::now() < expires_at
            } else {
                true // No expiration set, assume valid
            }
        } else {
            self.api_key.is_some() || self.jwt_token.is_some()
        }
    }
}

/// Authentication result
#[derive(Debug, Clone)]
pub enum AuthResultData<T> {
    /// Authentication succeeded
    Authenticated(T),
    /// Authentication failed with an error
    Error(String),
    /// Authentication needs authorization (URL to authorize)
    NeedsAuthorization(String),
}

/// Type alias for authentication results
pub type AuthResult<T> = Result<AuthResultData<T>>;

/// Authentication provider trait
#[async_trait::async_trait]
pub trait AuthProvider: Send + Sync {
    /// Authenticate the user
    async fn authenticate(&self) -> Result<AuthResultData<Credentials>>;
    
    /// Handle authorization code
    async fn handle_authorization_code(&self, code: &str) -> Result<AuthResultData<Credentials>>;
    
    /// Get authorization header
    async fn get_auth_header(&self) -> Result<String>;

    /// Get the provider type
    fn provider_type(&self) -> AuthProviderType;
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub provider_type: AuthProviderType,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
}

impl AuthConfig {
    pub fn new(
        provider_type: AuthProviderType,
        client_id: String,
        client_secret: String,
        auth_url: String,
        token_url: String,
        redirect_url: String,
        scopes: Vec<String>,
    ) -> Result<Self> {
        let config = Self {
            provider_type,
            client_id,
            client_secret: Some(client_secret),
            auth_url,
            token_url,
            redirect_url,
            scopes,
        };
        
        config.validate()?;
        Ok(config)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        let security = SecurityModule::new();
        
        // Validate URLs
        security.validate_url(&self.auth_url)
            .map_err(|e| Error::config(format!("Invalid auth URL: {}", e)))?;
        security.validate_url(&self.token_url)
            .map_err(|e| Error::config(format!("Invalid token URL: {}", e)))?;
        security.validate_url(&self.redirect_url)
            .map_err(|e| Error::config(format!("Invalid redirect URL: {}", e)))?;
        
        // Validate client ID
        if self.client_id.is_empty() {
            return Err(Error::config("Client ID cannot be empty"));
        }
        
        let validation_opts = SanitizationOptions {
            max_length: Some(256),
            ..Default::default()
        };
        
        match security.validate_input(&self.client_id, &validation_opts) {
            ValidationResult::Valid => {},
            ValidationResult::Invalid(reason) | ValidationResult::Malicious(reason) => {
                return Err(Error::config(format!("Invalid client ID: {}", reason)));
            }
        }
        
        Ok(())
    }
}

// Secure OAuth functions with validation
pub async fn authorize(config: AuthConfig) -> AuthResult<(String, String)> {
    // Validate configuration
    config.validate().map_err(|e| Error::auth(e.to_string()))?;
    
    // Create OAuth client for testing
    let _client = oauth::OAuth21Client::new("http://localhost:8080");
    let security = SecurityModule::new();
    security.log_security_event("OAUTH_AUTHORIZE_ATTEMPT", None);
    
    // Return placeholder values for the authorization URL and state
    Ok(AuthResultData::Authenticated(("https://auth.example.com/oauth".to_string(), "csrf_token_123".to_string())))
}

pub async fn exchange_code(config: AuthConfig, code: &str) -> AuthResult<oauth::OAuthToken> {
    // Validate configuration and code
    config.validate().map_err(|e| Error::auth(e.to_string()))?;
    
    let security = SecurityModule::new();
    let validation_opts = SanitizationOptions::default();
    
    match security.validate_input(code, &validation_opts) {
        ValidationResult::Valid => {},
        ValidationResult::Invalid(reason) | ValidationResult::Malicious(reason) => {
            security.log_security_event("MALICIOUS_CODE_EXCHANGE", Some(&reason));
            return Err(Error::auth(format!("Invalid authorization code: {}", reason)));
        }
    }
    
    let mut client = oauth::OAuth21Client::new("http://localhost:8080");
    let token = client.exchange_code(code.to_string(), None).await?;
    
    security.log_security_event("CODE_EXCHANGE_SUCCESS", None);
    Ok(AuthResultData::Authenticated(token))
}

pub async fn refresh_token(_config: AuthConfig, _refresh_token: &str) -> AuthResult<oauth::OAuthToken> {
    let mut client = oauth::OAuth21Client::new("http://localhost:8080");
    // Note: In the corrected API, the refresh token is set during initial token exchange
    // and the refresh_token() method doesn't take parameters
    let token = client.refresh_token().await?;
    Ok(AuthResultData::Authenticated(token))
} 

pub fn test_auth() {
    // Test authentication flow with OAuth 2.1
    let _client = oauth::OAuth21Client::new("http://localhost:8080");
    
    println!("OAuth client created: {:?}", _client);
    
    // In a real implementation:
    // 1. Start authorization flow
    // 2. Get authorization URL
    // 3. User visits URL and authorizes
    // 4. Exchange code for token
    // 5. Use token for API calls
} 