/// Government grants module for accessing government grant data
pub mod grants;

// Re-export key types
pub use grants::{GrantsClient, Grant, GrantsSearchParams}; 