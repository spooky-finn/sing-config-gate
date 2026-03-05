use serde::{Deserialize, Serialize};

use crate::db::enums::UserStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationCmd {
    #[serde(rename = "i")]
    pub user_id: i64,
    #[serde(rename = "s")]
    pub status: UserStatus,
}

impl InvitationCmd {
    pub fn new(user_id: i64, status: UserStatus) -> Self {
        Self { user_id, status }
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let cmd: InvitationCmd =
            serde_json::from_str(text).map_err(|e| format!("Failed to parse JSON: {}", e))?;
        Ok(cmd)
    }

    pub fn to_callback_data(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
