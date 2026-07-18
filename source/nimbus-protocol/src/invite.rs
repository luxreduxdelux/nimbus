use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use serde::{Deserialize, Serialize};

//================================================================

pub type InviteID = String;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Invite {
    pub date: DateTime<Utc>,
    pub value: InviteValue,
}

impl Invite {
    pub fn new(value: InviteValue) -> Self {
        Self {
            date: Utc::now(),
            value,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InviteValue {
    pub index: InviteID,
    pub count: Option<u64>,
    pub time: Duration,
}
