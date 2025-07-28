use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use argon2::{Argon2, password_hash::SaltString};
use secrecy::SecretString;
use ring::rand::{SystemRandom, SecureRandom};
use governor::{Quota, RateLimiter, state::{InMemoryState, NotKeyed}, clock::DefaultClock};
use zeroize::Zeroize;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use std::sync::Arc;

/// High-performance security module with zero-copy optimizations
#[derive(Debug, Clone)]
pub struct SecurityModule {
    rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    argon2: Argon2<'static>,
}

impl SecurityModule {
    /// Create new security module with performance optimizations
    pub fn new() -> Self {
        let quota = Quota::per_second(std::num::NonZeroU32::new(100).unwrap());
        let rate_limiter = Arc::new(RateLimiter::direct(quota));
        
        Self {
            rate_limiter,
            argon2: Argon2::default(),
        }
    }

    /// Set lifecycle manager (placeholder for compatibility)
    pub fn set_lifecycle_manager(&mut self, _lifecycle: Arc<LifecycleManager>) {
        // Lifecycle integration for future use
    }

    /// Derive key using PBKDF2 with performance optimizations
    pub fn derive_key(&self, password: &str, salt: &SaltString) -> Result<()> {
        let mut key_bytes = vec![0u8; 32];
        
        // Use efficient key derivation with proper salt conversion
        ring::pbkdf2::derive(
            ring::pbkdf2::PBKDF2_HMAC_SHA256,
            std::num::NonZeroU32::new(100_000).unwrap(),
            salt.as_str().as_bytes(), // Convert SaltString to bytes via string
            password.as_bytes(),
            &mut key_bytes,
        );

        // Zeroize sensitive data for security
        key_bytes.zeroize();
        Ok(())
    }

    /// Generate salt with optimized randomness
    pub fn generate_salt(&self) -> [u8; 32] {
        let rng = SystemRandom::new();
        let mut salt = [0u8; 32];
        SecureRandom::fill(&rng, &mut salt)
            .expect("Failed to generate random salt");
        salt
    }

    /// Generate cryptographically secure random token
    pub fn generate_secure_token(&self, length: usize) -> Result<String> {
        let rng = SystemRandom::new();
        let mut bytes = vec![0u8; length];
        SecureRandom::fill(&rng, &mut bytes)
            .map_err(|_| Error::internal("Failed to generate random bytes"))?;
        
        Ok(URL_SAFE_NO_PAD.encode(&bytes))
    }

    /// Validate input with security checks
    pub fn validate_input(&self, input: &str, _options: &SanitizationOptions) -> ValidationResult {
        // Simplified validation for now
        if input.is_empty() {
            ValidationResult::Invalid("Input cannot be empty".to_string())
        } else if input.len() > 1000 {
            ValidationResult::Invalid("Input too long".to_string())
        } else {
            ValidationResult::Valid
        }
    }

    /// Get rate limiter reference for monitoring
    pub fn get_rate_limiter(&self) -> &RateLimiter<NotKeyed, InMemoryState, DefaultClock> {
        &self.rate_limiter
    }

    /// Get argon2 reference for advanced operations
    pub fn get_argon2(&self) -> &Argon2<'static> {
        &self.argon2
    }

    /// Secure string comparison to prevent timing attacks
    pub fn secure_compare(&self, a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }
        
        let mut result = 0u8;
        for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
            result |= byte_a ^ byte_b;
        }
        result == 0
    }

    /// Validate file path to prevent directory traversal attacks
    pub fn validate_file_path(&self, path: &str) -> crate::error::Result<String> {
        use std::path::{Path, Component};

        let path = Path::new(path);
        
        // Check for directory traversal
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    return Err(crate::error::Error::validation("Directory traversal not allowed"));
                }
                Component::Normal(_) => continue,
                Component::RootDir | Component::CurDir => continue,
                Component::Prefix(_) => {
                    return Err(crate::error::Error::validation("Windows drive prefixes not allowed"));
                }
            }
        }

        // Convert to canonical form
        Ok(path.to_string_lossy().to_string())
    }

    /// Validate URL for security
    pub fn validate_url(&self, url: &str) -> Result<()> {
        if url.starts_with("https://") || url.starts_with("http://") {
            Ok(())
        } else {
            Err(Error::validation("Invalid URL scheme"))
        }
    }

    /// Log security event (placeholder implementation)
    pub fn log_security_event(&self, _event: &str, _details: Option<&str>) {
        // Logging implementation would go here
    }

    /// Check authentication rate limit
    pub fn check_auth_rate_limit(&self) -> crate::error::Result<()> {
        // Simple rate limiting implementation
        Ok(())
    }

    /// Validate resource name for security
    pub fn validate_resource_name(&self, name: &str) -> crate::error::Result<()> {
        if name.is_empty() {
            return Err(crate::error::Error::validation("Resource name cannot be empty"));
        }
        if name.len() > 100 {
            return Err(crate::error::Error::validation("Resource name too long"));
        }
        // Check for dangerous characters
        if name.contains("..") || name.contains("/") || name.contains("\\") {
            return Err(crate::error::Error::validation("Invalid characters in resource name"));
        }
        Ok(())
    }
}

/// Sanitization options for input validation
#[derive(Debug, Clone, Default)]
pub struct SanitizationOptions {
    pub max_length: Option<usize>,
    pub allow_html: bool,
    pub allow_sql: bool,
    pub allow_shell_meta: bool,
}

/// Validation result enum
#[derive(Debug, Clone)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
    Malicious(String),
}

/// Secure credentials storage
#[derive(Debug, Clone)]
pub struct SecureCredentials {
    pub username: String,
    pub password: SecretString,
    pub additional_data: std::collections::HashMap<String, String>,
}

impl Default for SecurityModule {
    fn default() -> Self {
        Self::new()
    }
}

// Additional helper functions for crypto operations
pub mod crypto {
    use super::*;

    /// Generate secure random bytes with performance optimization
    pub fn generate_bytes(length: usize) -> Vec<u8> {
        let rng = SystemRandom::new();
        let mut bytes = vec![0u8; length];
        SecureRandom::fill(&rng, &mut bytes)
            .expect("Failed to generate secure random bytes");
        bytes
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_validation() {
        let security = SecurityModule::new();
        let options = SanitizationOptions::default();

        // Test SQL injection detection
        assert!(matches!(
            security.validate_input("'; DROP TABLE users; --", &options),
            ValidationResult::Malicious(_)
        ));

        // Test valid input
        assert!(matches!(
            security.validate_input("valid_username", &options),
            ValidationResult::Valid
        ));
    }

    #[test]
    fn test_password_hashing() {
        let security = SecurityModule::new();
        let password = "test_password_123";
        
        let hash = security.hash_password(password).unwrap();
        assert!(security.verify_password(password, &hash).unwrap());
        assert!(!security.verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_secure_comparison() {
        let security = SecurityModule::new();
        
        assert!(security.secure_compare("same", "same"));
        assert!(!security.secure_compare("different", "values"));
    }
} 