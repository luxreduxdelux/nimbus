use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

//================================================================

use crate::message::*;

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    // TO-DO make private
    pub count_message: u64,
    pub message: BTreeMap<u64, Message>,
    pub name: String,
    pub info: String,
}

impl Channel {
    const DEFAULT_NAME: &str = "general";
    const DEFAULT_INFO: &str = "General channel.";

    pub fn default() -> Self {
        Self {
            count_message: Default::default(),
            message: Default::default(),
            name: Self::DEFAULT_NAME.to_string(),
            info: Self::DEFAULT_INFO.to_string(),
        }
    }
}
