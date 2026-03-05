use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum UserStatus {
    New = 0,
    Accepted = 1,
    Rejected = 2,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserStatus::New => "New",
            UserStatus::Accepted => "Accepted",
            UserStatus::Rejected => "Rejected",
        }
    }
}
