use crate::db::enums::UserStatus;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::{user, vless_identity};

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = user)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub status: i32,
    pub created_at: String,
}

impl User {
    pub fn status_enum(&self) -> UserStatus {
        self.status.into()
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = vless_identity)]
pub struct VlessIdentity {
    pub uuid: String,
    pub user_id: Option<i64>,
}
