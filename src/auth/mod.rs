use crate::error::{Error, Result};
use oauth2::CsrfToken;
use url::Url;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod oauth;
use oauth::*;

/// Authorization provider types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthProviderType {
    /// OAuth 2.1 authorization
    OAuth,
    /// API key authorization
    ApiKey,
    /// JWT token authorization
    JWT,
}

/// Authorization credentials
#[derive(Debug, Clone)]
pub struct Credentials {
    /// Token value
    pub token: String,
    /// Token type (e.g., "Bearer")
    pub token_type: String,
    /// Expiration time in seconds
    pub expires_in: Option<u64>,
    /// Refresh token
    pub refresh_token: Option<String>,
    /// Scope
    pub scope: Option<String>,
}

/// OAuth configuration
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// Client ID
    pub client_id: String,
    /// Client secret
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
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),
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
}

/// Authentication manager
pub struct AuthManager {
    /// OAuth client
    oauth_client: Option<OAuthClient>,
    /// Current credentials
    credentials: Arc<Mutex<Option<Credentials>>>,
    /// Auth provider type
    provider_type: AuthProviderType,
    /// API key
    api_key: Option<String>,
    /// JWT token
    jwt_token: Option<String>,
}

impl AuthManager {
    /// Create a new auth manager
    pub fn new() -> Self {
        Self {
            oauth_client: None,
            credentials: Arc::new(Mutex::new(None)),
            provider_type: AuthProviderType::OAuth,
            api_key: None,
            jwt_token: None,
        }
    }
    
    /// Initialize OAuth client
    pub fn with_oauth(mut self, oauth_config: &crate::config::OAuthConfig) -> Result<Self> {
        let config = OAuthConfig {
            client_id: oauth_config.client_id.clone(),
            client_secret: oauth_config.client_secret.clone(),
            auth_url: oauth_config.auth_url.clone(),
            token_url: oauth_config.token_url.clone(),
            redirect_url: oauth_config.redirect_url.clone(),
            scopes: oauth_config.scopes.clone(),
        };
        
        self.oauth_client = Some(OAuthClient::new(config)?);
        self.provider_type = AuthProviderType::OAuth;
        Ok(self)
    }
    
    /// Initialize with API key
    pub fn with_api_key(api_key: &str) -> Self {
        let mut auth_manager = Self::new();
        auth_manager.provider_type = AuthProviderType::ApiKey;
        auth_manager.api_key = Some(api_key.to_string());
        auth_manager
    }
    
    /// Get OAuth authorization URL
    pub async fn get_oauth_auth_url(&self) -> Result<(Url, CsrfToken)> {
        if let Some(client) = &self.oauth_client {
            let csrf_token = CsrfToken::new_random();
            let (url, _token) = client.authorize_url(csrf_token.clone()).await;
            Ok((url, csrf_token))
        } else {
            Err(Error::auth("OAuth client not initialized"))
        }
    }
    
    /// Exchange authorization code for token
    pub async fn exchange_oauth_code(&self, code: &str) -> Result<Credentials> {
        if let Some(client) = &self.oauth_client {
            let token = client.exchange_code(code.to_string()).await?;
            
            let credentials = Credentials {
                token: token.access_token.clone(),
                token_type: token.token_type.clone(),
                expires_in: token.expires_in,
                refresh_token: token.refresh_token.clone(),
                scope: token.scope.clone(),
            };
            
            // Store credentials
            let mut creds = self.credentials.lock().await;
            *creds = Some(credentials.clone());
            
            Ok(credentials)
        } else {
            Err(Error::auth("OAuth client not initialized"))
        }
    }
    
    /// Refresh OAuth token
    pub async fn refresh_oauth_token(&self) -> Result<Credentials> {
        if let Some(client) = &self.oauth_client {
            // Get current credentials to extract refresh token
            let current_creds = self.credentials.lock().await;
            
            if let Some(ref creds) = *current_creds {
                if let Some(ref refresh_token) = creds.refresh_token {
                    // Use refresh token to get a new token
                    let token = client.refresh_token(refresh_token.to_string()).await?;
                    
                    let credentials = Credentials {
                        token: token.access_token.clone(),
                        token_type: token.token_type.clone(),
                        expires_in: token.expires_in,
                        refresh_token: token.refresh_token.clone(),
                        scope: token.scope.clone(),
                    };
                    
                    // Store credentials
                    let mut creds = self.credentials.lock().await;
                    *creds = Some(credentials.clone());
                    
                    return Ok(credentials);
                }
            }
            
            Err(Error::auth("No refresh token available"))
        } else {
            Err(Error::auth("OAuth client not initialized"))
        }
    }
    
    /// Set credentials directly
    pub async fn set_credentials(&self, credentials: Credentials) {
        let mut creds = self.credentials.lock().await;
        *creds = Some(credentials);
    }
    
    /// Get current credentials
    pub async fn get_credentials(&self) -> Option<Credentials> {
        let creds = self.credentials.lock().await;
        creds.clone()
    }
    
    /// Get authorization header
    pub async fn get_auth_header(&self) -> Result<String> {
        let creds = self.credentials.lock().await;
        
        if let Some(credentials) = &*creds {
            Ok(format!("{} {}", credentials.token_type, credentials.token))
        } else if let Some(client) = &self.oauth_client {
            // Try to get a token from OAuth client
            let token = client.get_token().await?;
            Ok(format!("Bearer {}", token.access_token))
        } else {
            Err(Error::auth("No credentials available"))
        }
    }
    
    /// Clear credentials
    pub async fn clear_credentials(&self) {
        let mut creds = self.credentials.lock().await;
        *creds = None;
    }
    
    /// Get the auth provider type
    pub fn get_provider_type(&self) -> AuthProviderType {
        self.provider_type.clone()
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
    pub client_id: String,
    pub client_secret: Option<String>,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
}

impl AuthConfig {
    pub fn new(
        client_id: String,
        client_secret: String,
        auth_url: String,
        token_url: String,
        redirect_url: String,
        scopes: Vec<String>,
    ) -> Self {
        Self {
            client_id,
            client_secret: Some(client_secret),
            auth_url,
            token_url,
            redirect_url,
            scopes,
        }
    }
}

pub async fn authorize(config: AuthConfig) -> AuthResult<(String, String)> {
    let oauth_config = OAuthConfig {
        client_id: config.client_id,
        client_secret: config.client_secret,
        auth_url: config.auth_url,
        token_url: config.token_url,
        redirect_url: config.redirect_url,
        scopes: config.scopes,
    };
    
    match OAuthClient::new(oauth_config) {
        Ok(client) => {
            let csrf_token = CsrfToken::new_random();
            let (url, _token) = client.authorize_url(csrf_token.clone()).await;
            Ok(AuthResultData::NeedsAuthorization(url.to_string()))
        },
        Err(e) => {
            Ok(AuthResultData::Error(format!("Failed to initialize OAuth client: {}", e)))
        }
    }
}

pub async fn exchange_code(config: AuthConfig, code: &str) -> AuthResult<oauth::OAuthToken> {
    let client = oauth::OAuthClient::new(OAuthConfig {
        client_id: config.client_id,
        client_secret: config.client_secret,
        auth_url: config.auth_url,
        token_url: config.token_url,
        redirect_url: config.redirect_url,
        scopes: config.scopes,
    })?;

    let token = client.exchange_code(code.to_string()).await?;
    Ok(AuthResultData::Authenticated(token))
}

pub async fn refresh_token(config: AuthConfig, token: oauth::OAuthToken) -> AuthResult<oauth::OAuthToken> {
    let client = oauth::OAuthClient::new(OAuthConfig {
        client_id: config.client_id,
        client_secret: config.client_secret,
        auth_url: config.auth_url,
        token_url: config.token_url,
        redirect_url: config.redirect_url,
        scopes: config.scopes,
    })?;

    if let Some(refresh_token) = token.refresh_token {
        let new_token = client.refresh_token(refresh_token).await?;
        Ok(AuthResultData::Authenticated(new_token))
    } else {
        Ok(AuthResultData::Error("No refresh token available".to_string()))
    }
} 