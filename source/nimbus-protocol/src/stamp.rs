use serde::{Deserialize, Serialize};

//================================================================

use crate::file::*;
use crate::storage::*;

//================================================================

pub type StampID = u64;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stamp {
    pub index: StampID,
    pub value: StampValue,
}

impl Stamp {
    pub fn new(index: StampID, value: StampValue) -> Self {
        Self { index, value }
    }
}

//================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StampValue {
    pub name: String,
    pub file: FileID,
}

impl StampValue {
    pub fn from_request(emote: StampValueRequest, storage: &mut Storage) -> anyhow::Result<Self> {
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
pub struct StampValueRequest {
    pub name: String,
    pub data: Vec<u8>,
}

impl StampValueRequest {
    pub fn new(name: String, data: Vec<u8>) -> Self {
        Self { name, data }
    }
}
