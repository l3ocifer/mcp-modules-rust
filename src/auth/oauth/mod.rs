use crate::error::Result;
use crate::auth::{AuthError, AuthProvider, AuthResultData, OAuthConfig, Credentials};
use oauth2::{
    AuthUrl, TokenUrl, ClientId, ClientSecret, RedirectUrl,
    AuthorizationCode, CsrfToken,
    basic::BasicClient, reqwest::async_http_client, TokenType,
    EmptyExtraTokenFields, StandardTokenResponse,
    Scope, TokenResponse,
};
use std::sync::Arc;
use std::time::{SystemTime, Duration};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as TokioMutex;
use async_trait::async_trait;

// Add missing http_client module for native client
mod http_client {
    pub fn native() -> impl Fn(oauth2::HttpRequest) -> Result<oauth2::HttpResponse, oauth2::reqwest::Error<reqwest::Error>> {
        oauth2::reqwest::http_client
    }
}

/// OAuth token details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    /// Access token
    pub access_token: String,
    /// Token type (e.g., "Bearer")
    pub token_type: String,
    /// Expiration time (seconds from creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
    /// Refresh token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Scopes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Creation time (as unix timestamp)
    pub created_at: SystemTime,
}

impl OAuthToken {
    /// Creates a new OAuth token
    pub fn new(
        access_token: String,
        token_type: String,
        expires_in: Option<u64>,
        refresh_token: Option<String>,
        scope: Option<String>,
    ) -> Self {
        Self {
            access_token,
            token_type,
            expires_in,
            refresh_token,
            scope,
            created_at: SystemTime::now(),
        }
    }

    /// Create a new OAuth token from a standard token response
    pub fn from_response<T: TokenType + AsRef<str>>(
        response: StandardTokenResponse<EmptyExtraTokenFields, T>
    ) -> Self {
        Self {
            access_token: response.access_token().secret().to_string(),
            token_type: response.token_type().as_ref().to_string(),
            expires_in: response.expires_in().map(|e| e.as_secs()),
            refresh_token: response.refresh_token().map(|r| r.secret().to_string()),
            scope: response.scopes().map(|s| s.iter().map(|scope| scope.to_string()).collect::<Vec<_>>().join(" ")),
            created_at: SystemTime::now(),
        }
    }

    /// Checks if the token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_in) = self.expires_in {
            let expiry_time = self.created_at + Duration::from_secs(expires_in);
            SystemTime::now() > expiry_time
        } else {
            false
        }
    }
    
    /// Get authorization header value
    pub fn authorization_header(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
    }
}

/// OAuth client implementation
pub struct OAuthClient {
    /// OAuth configuration
    config: OAuthConfig,
    /// Current token
    token: Arc<TokioMutex<Option<OAuthToken>>>,
    /// HTTP client
    http_client: reqwest::Client,
}

impl OAuthClient {
    /// Create a new OAuth client
    pub fn new(config: OAuthConfig) -> Result<Self> {
        let client_id = config.client_id.clone();
        let client_secret = config.client_secret.clone();
        let redirect_url = config.redirect_url.clone();
        
        let _client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret.unwrap_or_default())),
            AuthUrl::new(config.auth_url.clone())
                .map_err(|_| AuthError::InvalidAuthUrl)?,
            Some(TokenUrl::new(config.token_url.clone())
                .map_err(|_| AuthError::InvalidTokenUrl)?),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_url).map_err(|_| AuthError::InvalidRedirectUri)?);

        Ok(Self {
            config,
            token: Arc::new(TokioMutex::new(None)),
            http_client: reqwest::Client::new(),
        })
    }
    
    /// Gets the authorization URL for the OAuth2 flow
    pub async fn authorize_url(&self, csrf_state: CsrfToken) -> (reqwest::Url, CsrfToken) {
        // Create the OAuth client
        let client = self.create_oauth_client().expect("Failed to create OAuth client");
        
        // Start the authorization process
        let mut auth_request = client.authorize_url(|| csrf_state.clone());
        
        // Add scopes if configured
        if !self.config.scopes.is_empty() {
            for scope in &self.config.scopes {
                auth_request = auth_request.add_scope(Scope::new(scope.clone()));
            }
        }
        
        // Get the URL and CSRF token
        let (auth_url, csrf_token) = auth_request.url();
        
        (auth_url, csrf_token)
    }
    
    /// Exchange an authorization code for an access token (async version)
    pub async fn exchange_code(&self, code: String) -> Result<OAuthToken> {
        // Create a CSRF token (not validated in this flow as we're handling the code directly)
        let csrf_token = CsrfToken::new_random();
        
        // Exchange the code for a token
        let token_response = self
            .create_oauth_client()?
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .map_err(|e| AuthError::TokenExchangeError(e.to_string()))?;

        let token = OAuthToken::from_response(token_response);
        *self.token.lock().await = Some(token.clone());
        
        Ok(token)
    }
    
    /// Refresh an access token using a refresh token
    pub async fn refresh_token(&self, refresh_token: String) -> Result<OAuthToken> {
        let token_result = self
            .create_oauth_client()?
            .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token))
            .request_async(async_http_client)
            .await
            .map_err(|e| AuthError::TokenRefreshError(e.to_string()))?;

        let token = OAuthToken::from_response(token_result);
        *self.token.lock().await = Some(token.clone());
        
        Ok(token)
    }
    
    /// Checks if the token is expired
    pub async fn is_token_expired(&self) -> bool {
        if let Some(token) = self.token.lock().await.as_ref() {
            token.is_expired()
        } else {
            false
        }
    }

    /// Gets the current token, refreshing it if expired
    pub async fn get_token(&self) -> Result<OAuthToken> {
        let token_guard = self.token.lock().await;
        
        // Check if we have a token
        if let Some(token) = token_guard.as_ref() {
            // If token is expired, try to refresh it
            if self.is_token_expired().await {
                // Check if we have a refresh token
                if let Some(refresh_token) = &token.refresh_token {
                    // Clone the refresh token before dropping the guard
                    let refresh_token = refresh_token.clone();
                    // Release the lock before async operation
                    drop(token_guard);
                    
                    // Refresh the token
                    return self.refresh_token(refresh_token).await;
                } else {
                    return Err(AuthError::NoRefreshToken.into());
                }
            }
            
            // Return the token if it's not expired
            Ok(token.clone())
        } else {
            Err(AuthError::NoToken.into())
        }
    }
    
    fn create_oauth_client(&self) -> Result<BasicClient> {
        let client = BasicClient::new(
            ClientId::new(self.config.client_id.clone()),
            self.config.client_secret.as_ref().map(|s| ClientSecret::new(s.clone())),
            AuthUrl::new(self.config.auth_url.clone())
                .map_err(|_| AuthError::InvalidAuthUrl)?,
            Some(TokenUrl::new(self.config.token_url.clone())
                .map_err(|_| AuthError::InvalidTokenUrl)?),
        )
        .set_redirect_uri(RedirectUrl::new(self.config.redirect_url.clone())
            .map_err(|_| AuthError::InvalidRedirectUri)?);
        
        Ok(client)
    }

    async fn get_auth_url(&self) -> Result<String> {
        let csrf_token = CsrfToken::new_random();
        let (auth_url, _) = self.authorize_url(csrf_token.clone()).await;
        
        Ok(auth_url.to_string())
    }
}

#[async_trait]
impl AuthProvider for OAuthClient {
    async fn authenticate(&self) -> Result<AuthResultData<Credentials>> {
        if let Some(token) = &*self.token.lock().await {
            if !self.is_token_expired().await {
                let credentials = Credentials {
                    token: token.access_token.clone(),
                    token_type: token.token_type.clone(),
                    expires_in: token.expires_in,
                    refresh_token: token.refresh_token.clone(),
                    scope: token.scope.clone(),
                };
                return Ok(AuthResultData::Authenticated(credentials));
            }
            
            if let Some(refresh_token) = &token.refresh_token {
                match self.refresh_token(refresh_token.clone()).await {
                    Ok(new_token) => {
                        let credentials = Credentials {
                            token: new_token.access_token.clone(),
                            token_type: new_token.token_type.clone(),
                            expires_in: new_token.expires_in,
                            refresh_token: new_token.refresh_token.clone(),
                            scope: new_token.scope.clone(),
                        };
                        return Ok(AuthResultData::Authenticated(credentials));
                    },
                    Err(e) => return Ok(AuthResultData::Error(e.to_string())),
                }
            }
        }
        
        // Create a random CSRF token
        let csrf_token = CsrfToken::new_random();
        let (auth_url, _) = self.authorize_url(csrf_token.clone()).await;
        Ok(AuthResultData::NeedsAuthorization(auth_url.to_string()))
    }
    
    async fn handle_authorization_code(&self, code: &str) -> Result<AuthResultData<Credentials>> {
        match self.exchange_code(code.to_string()).await {
            Ok(token) => {
                let credentials = Credentials {
                    token: token.access_token,
                    token_type: token.token_type,
                    expires_in: token.expires_in,
                    refresh_token: token.refresh_token,
                    scope: token.scope,
                };
                
                Ok(AuthResultData::Authenticated(credentials))
            },
            Err(_) => {
                // If exchange fails, generate new auth URL
                let csrf_token = CsrfToken::new_random();
                let (auth_url, _) = self.authorize_url(csrf_token.clone()).await;
                
                Ok(AuthResultData::NeedsAuthorization(auth_url.to_string()))
            }
        }
    }
    
    async fn get_auth_header(&self) -> Result<String> {
        let token = self.get_token().await?;
        Ok(token.authorization_header())
    }
    
    /// Get the provider type
    fn provider_type(&self) -> crate::auth::AuthProviderType {
        crate::auth::AuthProviderType::OAuth
    }
} 