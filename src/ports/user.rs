use crate::db::{enums::UserStatus, User};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserRepoError {
    #[error("User not found: {0}")]
    NotFound(i64),
    #[error("Database error: {0}")]
    Database(String),
}

pub trait IUserRepo: Send + Sync {
    fn select(&self, id: i64) -> Result<Option<User>, UserRepoError>;
    fn insert(&self, user: &crate::db::NewUser) -> Result<(), UserRepoError>;
    fn get_by_status(&self, status: UserStatus) -> Result<Vec<User>, UserRepoError>;
    fn set_status(&self, id: i64, status: UserStatus) -> Result<(), UserRepoError>;
}
