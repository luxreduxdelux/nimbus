use serde::{Deserialize, Serialize};

//================================================================

use crate::file::*;
use crate::storage::*;

//================================================================

pub type EmoteID = u64;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Emote {
    pub index: EmoteID,
    pub value: EmoteValue,
}

impl Emote {
    pub fn new(index: EmoteID, value: EmoteValue) -> Self {
        Self { index, value }
    }
}

//================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmoteValue {
    pub name: String,
    pub file: FileID,
}

impl EmoteValue {
    pub fn from_request(emote: EmoteValueRequest, storage: &mut Storage) -> anyhow::Result<Self> {
        let name = emote.name.clone();
        let file = FileValue::new(String::default(), emote.data).insert(storage)?;
        Ok(Self {
            name,
            file: file.index,
        })
    }
}

//================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmoteValueRequest {
    pub name: String,
    pub data: Vec<u8>,
}

impl EmoteValueRequest {
    pub fn new(name: String, data: Vec<u8>) -> Self {
        Self { name, data }
    }
}
