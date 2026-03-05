use crate::{db::models::VlessIdentity, ports::RepoError};

#[allow(dead_code)]
pub trait VlessIdentityRepoTrait: Send + Sync {
    fn assign(&self, user_id: i64) -> Result<(), RepoError>;
    fn get(&self, uuid: &str) -> Result<Option<VlessIdentity>, RepoError>;
    fn get_by_user_id(&self, user_id: i64) -> Result<VlessIdentity, RepoError>;
    fn insert(&self, vless_identity: &VlessIdentity) -> Result<(), RepoError>;
    fn delete(&self, uuid: &str) -> Result<(), RepoError>;
}
