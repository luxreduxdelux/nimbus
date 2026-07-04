use serde::{Deserialize, Serialize};

//================================================================

use crate::channel::*;

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub list: Vec<ChannelID>,
}

impl Category {
    pub fn new(name: String, list: Vec<ChannelID>) -> Self {
        Self { name, list }
    }
}
