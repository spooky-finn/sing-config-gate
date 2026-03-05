use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum UserStatus {
    New = 0,
    Accepted = 1,
    Rejected = 2,
}

impl Serialize for UserStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(*self as i32)
    }
}

impl<'de> Deserialize<'de> for UserStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = i32::deserialize(deserializer)?;
        Ok(UserStatus::from(value))
    }
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

impl From<UserStatus> for i32 {
    fn from(value: UserStatus) -> Self {
        match value {
            UserStatus::New => 0,
            UserStatus::Accepted => 1,
            UserStatus::Rejected => 2,
        }
    }
}

impl From<i32> for UserStatus {
    fn from(value: i32) -> Self {
        match value {
            0 => UserStatus::New,
            1 => UserStatus::Accepted,
            2 => UserStatus::Rejected,
            _ => panic!("Invalid UserStatus value: {}", value),
        }
    }
}
