use serde::{Deserialize, Serialize};

//================================================================

pub type ChannelID = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub index: ChannelID,
    pub name: String,
    pub info: String,
}

impl Channel {
    pub const DEFAULT_NAME: &str = "general";
    pub const DEFAULT_INFO: &str = "General channel.";

    pub fn new(index: ChannelID, name: String, info: String) -> Self {
        Self { index, name, info }
    }

    pub fn default() -> Self {
        Self {
            index: ChannelID::default(),
            name: Self::DEFAULT_NAME.to_string(),
            info: Self::DEFAULT_INFO.to_string(),
        }
    }
}
