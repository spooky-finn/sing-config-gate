use crate::db::enums::UserStatus;
use crate::ports::user::IUserRepo;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use teloxide::types::CallbackQuery;
use tracing::{error, info};

const OP_CODE: &str = "invate-confirm";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationCmd {
    #[serde(rename = "opcode")]
    op_code: String,
    pub user_id: i64,
    pub status: UserStatus,
}

impl InvitationCmd {
    pub fn new(user_id: i64, status: UserStatus) -> Self {
        Self {
            op_code: OP_CODE.to_string(),
            user_id,
            status,
        }
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let data: serde_json::Value =
            serde_json::from_str(text).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let opcode = data["opcode"]
            .as_str()
            .ok_or_else(|| format!("Missing or invalid opcode"))?;

        if opcode != OP_CODE {
            return Err(format!("Wrong operation code: received {}", opcode));
        }

        let status_str = data["status"]
            .as_i64()
            .ok_or_else(|| "Invalid status".to_string())?;

        let status = match status_str {
            0 => UserStatus::New,
            1 => UserStatus::Accepted,
            2 => UserStatus::Rejected,
            _ => return Err("Invalid user status".to_string()),
        };

        let user_id = data["userId"]
            .as_i64()
            .ok_or_else(|| "Invalid user id".to_string())?;

        Ok(Self {
            op_code: OP_CODE.to_string(),
            user_id,
            status,
        })
    }

    pub fn to_callback_data(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

pub struct AdminService {
    user_repo: Arc<dyn IUserRepo>,
    pub admin_id: i64,
}

impl AdminService {
    pub fn new(user_repo: Arc<dyn IUserRepo>, admin_id: i64) -> Self {
        Self { user_repo, admin_id }
    }

    pub fn is_admin_callback(&self, msg: &CallbackQuery) -> Option<InvitationCmd> {
        if msg.from.id.0 as i64 != self.admin_id {
            return None;
        }

        let data = msg.data.as_ref()?;

        match InvitationCmd::parse(data) {
            Ok(cmd) => Some(cmd),
            Err(e) => {
                error!(error = %e, "Failed to parse admin command");
                None
            }
        }
    }

    pub async fn handle_admin_callback(&self, cmd: &InvitationCmd) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.user_repo.set_status(cmd.user_id, cmd.status)?;
        info!(user_id = cmd.user_id, status = ?cmd.status, "Admin callback handled");
        Ok(())
    }
}
