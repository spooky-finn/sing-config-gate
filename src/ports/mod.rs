pub mod user;
pub mod vless_identity;

use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum RepoError {
    #[error("VpnUuid not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(String),
}
