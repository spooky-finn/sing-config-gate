pub mod enums;
pub mod schema;

use crate::db::enums::UserStatus;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

pub use schema::{user, vpn_uuid};

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = user)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub status: i32,
    pub auth_key: String,
    pub created_at: String,
}

impl User {
    pub fn status_enum(&self) -> UserStatus {
        match self.status {
            0 => UserStatus::New,
            1 => UserStatus::Accepted,
            2 => UserStatus::Rejected,
            _ => UserStatus::New,
        }
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = user)]
pub struct NewUser {
    pub id: i64,
    pub username: String,
    pub status: i32,
    pub auth_key: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = vpn_uuid)]
pub struct VpnUuidRow {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub created_at: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = vpn_uuid)]
pub struct NewVpnUuid {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub created_at: String,
}
