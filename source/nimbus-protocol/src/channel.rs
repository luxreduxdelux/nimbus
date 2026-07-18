use serde::{Deserialize, Serialize};

//================================================================

pub type ChannelID = u64;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Channel {
    pub index: ChannelID,
    pub value: ChannelValue,
}

impl Channel {
    pub fn new(index: ChannelID, value: ChannelValue) -> Self {
        Self { index, value }
    }
}

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelValue {
    pub name: String,
    pub info: String,
}

impl ChannelValue {
    pub const DEFAULT_NAME: &str = "general";
    pub const DEFAULT_INFO: &str = "General channel.";

    pub fn new(name: String, info: String) -> Self {
        Self { name, info }
    }
}

impl Default for ChannelValue {
    fn default() -> Self {
        Self {
            name: Self::DEFAULT_NAME.to_string(),
            info: Self::DEFAULT_INFO.to_string(),
        }
    }
}
