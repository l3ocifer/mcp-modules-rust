use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use oauth2::{
    ClientId, RedirectUrl, Scope, TokenUrl,
    AuthUrl, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    TokenResponse, AccessToken, RefreshToken, AuthorizationCode,
    StandardRevocableToken, ConfigurationError,
    basic::BasicClient,
};

// Type alias for the fully configured OAuth client
type ConfiguredOAuthClient = oauth2::Client<
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::StandardTokenIntrospectionResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::StandardRevocableToken,
    oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet,
    oauth2::EndpointSet,
>;
use reqwest::Client as HttpClient;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use url::Url;

/// OAuth 2.1 client with MCP 2025-06-18 enhancements
#[derive(Debug)]
pub struct OAuth21Client {
    /// Base URL for the authorization server
    auth_base_url: String,
    /// HTTP client for requests
    #[allow(dead_code)]
    http_client: HttpClient,
    /// OAuth2 client
    oauth_client: Option<ConfiguredOAuthClient>,
    /// PKCE code verifier
    pkce_verifier: Option<PkceCodeVerifier>,
    /// Current access token
    access_token: Option<AccessToken>,
    /// Current refresh token
    refresh_token: Option<RefreshToken>,
    /// Token expiration time
    token_expires_at: Option<SystemTime>,
    /// Authorization server metadata
    server_metadata: Option<AuthServerMetadata>,
    /// Dynamic client registration data
    client_registration: Option<ClientRegistration>,
}

/// OAuth 2.0 Authorization Server Metadata (RFC 8414)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthServerMetadata {
    /// Authorization server issuer identifier
    pub issuer: String,
    /// Authorization endpoint URL
    pub authorization_endpoint: String,
    /// Token endpoint URL
    pub token_endpoint: String,
    /// Client registration endpoint URL (RFC 7591)
    pub registration_endpoint: Option<String>,
    /// Revocation endpoint URL
    pub revocation_endpoint: Option<String>,
    /// Supported response types
    pub response_types_supported: Vec<String>,
    /// Supported grant types
    pub grant_types_supported: Option<Vec<String>>,
    /// Supported scopes
    pub scopes_supported: Option<Vec<String>>,
    /// Supported token endpoint auth methods
    pub token_endpoint_auth_methods_supported: Option<Vec<String>>,
    /// Whether PKCE is required
    pub require_request_uri_registration: Option<bool>,
    /// Code challenge methods supported
    pub code_challenge_methods_supported: Option<Vec<String>>,
}

/// Dynamic Client Registration Request (RFC 7591)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRegistrationRequest {
    /// Redirect URIs
    pub redirect_uris: Vec<String>,
    /// Client name
    pub client_name: Option<String>,
    /// Client URI
    pub client_uri: Option<String>,
    /// Logo URI
    pub logo_uri: Option<String>,
    /// Scope string
    pub scope: Option<String>,
    /// Contacts
    pub contacts: Option<Vec<String>>,
    /// Terms of service URI
    pub tos_uri: Option<String>,
    /// Policy URI
    pub policy_uri: Option<String>,
    /// Software ID
    pub software_id: Option<String>,
    /// Software version
    pub software_version: Option<String>,
    /// Grant types
    pub grant_types: Option<Vec<String>>,
    /// Response types
    pub response_types: Option<Vec<String>>,
    /// Token endpoint auth method
    pub token_endpoint_auth_method: Option<String>,
}

/// Dynamic Client Registration Response (RFC 7591)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRegistration {
    /// Client identifier
    pub client_id: String,
    /// Client secret (for confidential clients)
    pub client_secret: Option<String>,
    /// Registration access token
    pub registration_access_token: Option<String>,
    /// Registration client URI
    pub registration_client_uri: Option<String>,
    /// Client secret expires at
    pub client_secret_expires_at: Option<u64>,
    /// All other registration data
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

/// Resource Indicator for RFC 8707 compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceIndicator {
    /// Resource server identifier
    pub resource: String,
    /// Optional audience
    pub audience: Option<Vec<String>>,
}

impl OAuth21Client {
    /// Create a new OAuth 2.1 client
    pub fn new(auth_base_url: &str) -> Self {
        Self {
            auth_base_url: auth_base_url.to_string(),
            http_client: HttpClient::new(),
            oauth_client: None,
            pkce_verifier: None,
            access_token: None,
            refresh_token: None,
            token_expires_at: None,
            server_metadata: None,
            client_registration: None,
        }
    }

    /// Discover authorization server metadata (RFC 8414)
    pub async fn discover_metadata(&mut self) -> Result<()> {
        let well_known_url = format!("{}/.well-known/oauth-authorization-server", self.auth_base_url);
        
        let response = self.http_client
            .get(&well_known_url)
            .header("MCP-Protocol-Version", "2025-06-18")
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to fetch metadata: {}", e)))?;

        if response.status().is_success() {
            let metadata: AuthServerMetadata = response
                .json()
                .await
                .map_err(|e| Error::parsing(format!("Failed to parse metadata: {}", e)))?;
                
            self.server_metadata = Some(metadata);
            Ok(())
        } else {
            // Fallback to default endpoints
            self.server_metadata = Some(AuthServerMetadata {
                issuer: self.auth_base_url.clone(),
                authorization_endpoint: format!("{}/authorize", self.auth_base_url),
                token_endpoint: format!("{}/token", self.auth_base_url),
                registration_endpoint: Some(format!("{}/register", self.auth_base_url)),
                revocation_endpoint: Some(format!("{}/revoke", self.auth_base_url)),
                response_types_supported: vec!["code".to_string()],
                grant_types_supported: Some(vec!["authorization_code".to_string(), "refresh_token".to_string()]),
                scopes_supported: Some(vec!["openid".to_string(), "mcp".to_string()]),
                token_endpoint_auth_methods_supported: Some(vec!["none".to_string(), "client_secret_basic".to_string()]),
                require_request_uri_registration: Some(false),
                code_challenge_methods_supported: Some(vec!["S256".to_string()]),
            });
            Ok(())
        }
    }

    /// Register client dynamically (RFC 7591)
    pub async fn register_client(&mut self, registration_request: ClientRegistrationRequest) -> Result<()> {
        let _metadata = self.server_metadata.as_ref()
            .ok_or_else(|| Error::auth("Server metadata not available".to_string()))?;

        let registration_endpoint = self.server_metadata.as_ref()
            .and_then(|m| m.registration_endpoint.as_ref())
            .ok_or_else(|| Error::auth("Registration endpoint not available".to_string()))?;

        let response = self.http_client
            .post(registration_endpoint)
            .header("Content-Type", "application/json")
            .json(&registration_request)
            .send()
            .await
            .map_err(|e| Error::network(format!("Registration request failed: {}", e)))?;

        if response.status().is_success() {
            let registration: ClientRegistration = response
                .json()
                .await
                .map_err(|e| Error::parsing(format!("Failed to parse registration response: {}", e)))?;
            
            self.client_registration = Some(registration);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(Error::auth(format!("Client registration failed: {}", error_text)))
        }
    }

    /// Initialize OAuth client with discovered metadata and registered client
    pub fn init_oauth_client(&mut self) -> Result<()> {
        let _metadata = self.server_metadata.as_ref()
            .ok_or_else(|| Error::auth("Server metadata not available".to_string()))?;
            
        let registration = self.client_registration.as_ref()
            .ok_or_else(|| Error::auth("Client not registered".to_string()))?;

        // Create OAuth client with proper endpoint configuration
        let client_id = ClientId::new(registration.client_id.clone());
        let auth_url = AuthUrl::new(self.auth_base_url.clone())
            .map_err(|e| Error::config(format!("Invalid auth URL: {}", e)))?;
        let token_url = TokenUrl::new(self.auth_base_url.clone())
            .map_err(|e| Error::config(format!("Invalid token URL: {}", e)))?;
        
        // Add a default revocation URL for the type system
        let revocation_url = oauth2::RevocationUrl::new(format!("{}/revoke", self.auth_base_url))
            .map_err(|e| Error::config(format!("Invalid revocation URL: {}", e)))?;

        // Build client with all required endpoints to match the type
        let client: ConfiguredOAuthClient = BasicClient::new(client_id)
            .set_auth_uri(auth_url)
            .set_token_uri(token_url)
            .set_revocation_url(revocation_url);

        // Store the configured client
        self.oauth_client = Some(client);
        Ok(())
    }

    /// Start authorization flow with PKCE and resource indicators
    pub fn start_authorization_flow(
        &mut self,
        redirect_uri: &str,
        scopes: Vec<String>,
        resource_indicators: Option<Vec<ResourceIndicator>>,
    ) -> Result<String> {
        let client = self.oauth_client.as_ref()
            .ok_or_else(|| Error::auth("OAuth client not initialized".to_string()))?;

        // Generate PKCE challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        self.pkce_verifier = Some(pkce_verifier);

        let _redirect_url = RedirectUrl::new(redirect_uri.to_string())
            .map_err(|e| Error::auth(format!("Invalid redirect URL: {}", e)))?;

        let mut auth_request = client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge);

        // Add scopes
        for scope in scopes {
            auth_request = auth_request.add_scope(Scope::new(scope));
        }

        let (auth_url, _csrf_token) = auth_request.url();
        
        // Add resource indicators as query parameters if provided (RFC 8707)
        let mut final_url = auth_url;
        if let Some(indicators) = resource_indicators {
            let mut url = Url::parse(final_url.as_ref())
                .map_err(|e| Error::auth(format!("Invalid URL: {}", e)))?;
            
            for indicator in indicators {
                url.query_pairs_mut().append_pair("resource", &indicator.resource);
                if let Some(audience) = indicator.audience {
                    for aud in audience {
                        url.query_pairs_mut().append_pair("audience", &aud);
                    }
                }
            }
            final_url = url;
        }

        Ok(final_url.to_string())
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(&mut self, code: String, resource_indicators: Option<Vec<ResourceIndicator>>) -> Result<OAuthToken> {
        let client = self.oauth_client.as_ref()
            .ok_or_else(|| Error::auth("OAuth client not initialized".to_string()))?;

        let pkce_verifier = self.pkce_verifier.take()
            .ok_or_else(|| Error::auth("PKCE verifier not found".to_string()))?;

        let mut token_request = client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier);

        // Add resource indicators if provided (RFC 8707)
        if let Some(indicators) = resource_indicators {
            for indicator in indicators {
                token_request = token_request.add_extra_param("resource", indicator.resource);
                if let Some(audience) = indicator.audience {
                    for aud in audience {
                        token_request = token_request.add_extra_param("audience", aud);
                    }
                }
            }
        }

        // Create HTTP client for token exchange
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| Error::auth(format!("Failed to create HTTP client: {}", e)))?;

        let token_response = token_request
            .request_async(&http_client)
            .await
            .map_err(|e| Error::auth(format!("Token exchange failed: {}", e)))?;

        // Store tokens
        self.access_token = Some(token_response.access_token().clone());
        self.refresh_token = token_response.refresh_token().cloned();
        
        // Calculate expiration time
        self.token_expires_at = token_response.expires_in().map(|duration| {
            SystemTime::now() + duration
        });

        Ok(OAuthToken {
            access_token: token_response.access_token().secret().clone(),
            refresh_token: token_response.refresh_token().map(|t| t.secret().clone()),
            expires_in: token_response.expires_in().map(|d| d.as_secs()),
            token_type: "Bearer".to_string(),
            scope: None, // Could be extracted from response if needed
        })
    }

    /// Refresh access token
    pub async fn refresh_token(&mut self) -> Result<OAuthToken> {
        let client = self.oauth_client.as_ref()
            .ok_or_else(|| Error::auth("OAuth client not initialized".to_string()))?;

        let refresh_token = self.refresh_token.as_ref()
            .ok_or_else(|| Error::auth("No refresh token available".to_string()))?;

        // Create HTTP client for token refresh
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| Error::auth(format!("Failed to create HTTP client: {}", e)))?;

        let token_response = client
            .exchange_refresh_token(refresh_token)
            .request_async(&http_client)
            .await
            .map_err(|e| Error::auth(format!("Token refresh failed: {}", e)))?;

        // Update stored tokens
        self.access_token = Some(token_response.access_token().clone());
        if let Some(new_refresh_token) = token_response.refresh_token() {
            self.refresh_token = Some(new_refresh_token.clone());
        }
        
        // Update expiration time
        self.token_expires_at = token_response.expires_in().map(|duration| {
            SystemTime::now() + duration
        });

        Ok(OAuthToken {
            access_token: token_response.access_token().secret().clone(),
            refresh_token: token_response.refresh_token().map(|t| t.secret().clone()),
            expires_in: token_response.expires_in().map(|d| d.as_secs()),
            token_type: "Bearer".to_string(),
            scope: None,
        })
    }

    /// Check if token is valid and not expired
    pub fn is_token_valid(&self) -> bool {
        if self.access_token.is_none() {
            return false;
        }

        if let Some(expires_at) = self.token_expires_at {
            // Add 60 second buffer for clock skew
            SystemTime::now() + Duration::from_secs(60) < expires_at
        } else {
            // If no expiration is set, assume token is valid
            true
        }
    }

    /// Get current access token
    pub fn get_access_token(&self) -> Option<&AccessToken> {
        self.access_token.as_ref()
    }

    /// Revoke token
    pub async fn revoke_token(&mut self) -> Result<()> {
        let client = self.oauth_client.as_ref()
            .ok_or_else(|| Error::auth("OAuth client not initialized".to_string()))?;

        if let Some(access_token) = &self.access_token {
            let revocable_token = StandardRevocableToken::AccessToken(access_token.clone());
            
            // Create HTTP client for token revocation
            let http_client = reqwest::ClientBuilder::new()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|e| Error::auth(format!("Failed to create HTTP client: {}", e)))?;

            client
                .revoke_token(revocable_token)
                .map_err(|e: ConfigurationError| Error::auth(format!("Token revocation not supported: {}", e)))?
                .request_async(&http_client)
                .await
                .map_err(|e| Error::auth(format!("Token revocation failed: {}", e)))?;

            self.access_token = None;
            self.refresh_token = None;
            self.token_expires_at = None;
        }

        Ok(())
    }
}

/// OAuth token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: String,
    pub scope: Option<String>,
}

/// Default client registration for MCP
impl Default for ClientRegistrationRequest {
    fn default() -> Self {
        Self {
            redirect_uris: vec!["http://localhost:8080/callback".to_string()],
            client_name: Some("MCP Client".to_string()),
            client_uri: Some("https://modelcontextprotocol.io".to_string()),
            logo_uri: None,
            scope: Some("mcp".to_string()),
            contacts: None,
            tos_uri: None,
            policy_uri: None,
            software_id: Some("mcp-client".to_string()),
            software_version: Some("2025-06-18".to_string()),
            grant_types: Some(vec!["authorization_code".to_string(), "refresh_token".to_string()]),
            response_types: Some(vec!["code".to_string()]),
            token_endpoint_auth_method: Some("none".to_string()), // Public client
        }
    }
} 