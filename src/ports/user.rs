use thiserror::Error;

use crate::db::{enums::UserStatus, models::User};

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum UserRepoError {
    #[error("User not found: {0}")]
    NotFound(i64),
    #[error("Database error: {0}")]
    Database(String),
}

#[allow(dead_code)]
pub trait IUserRepo: Send + Sync {
    fn get(&self, id: i64) -> Result<User, UserRepoError>;
    fn select(&self, id: i64) -> Result<Option<User>, UserRepoError>;
    fn insert(&self, user: &User) -> Result<(), UserRepoError>;
    fn get_by_status(&self, status: UserStatus) -> Result<Vec<User>, UserRepoError>;
    fn set_status(&self, id: i64, status: UserStatus) -> Result<(), UserRepoError>;
}
