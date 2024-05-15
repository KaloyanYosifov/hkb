use hkb_date::date::SimpleDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateReminderData {
    pub note: String,
    pub remind_at: SimpleDate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateReminderData {
    pub id: i64,
    pub note: Option<String>,
    pub remind_at: Option<SimpleDate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReminderData {
    pub id: i64,
    pub note: String,
    pub remind_at: SimpleDate,
    pub created_at: SimpleDate,
}

pub mod fakes {
    use hkb_date::date::SimpleDate;

    use super::ReminderData;

    pub fn create_reminder() -> ReminderData {
        ReminderData {
            id: 1,
            note: "Testing".to_owned(),
            remind_at: SimpleDate::local(),
            created_at: SimpleDate::local(),
        }
    }
}
