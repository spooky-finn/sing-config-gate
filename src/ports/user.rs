use crate::{
    db::{enums::UserStatus, models::User},
    ports::RepoError,
};

#[allow(dead_code)]
pub trait UserRepoTrait: Send + Sync {
    fn get(&self, id: i64) -> Result<Option<User>, RepoError>;
    fn get_by_status(&self, status: UserStatus) -> Result<Vec<User>, RepoError>;
    fn insert(&self, user: &User) -> Result<(), RepoError>;
    fn set_status(&self, id: i64, status: UserStatus) -> Result<(), RepoError>;
}
