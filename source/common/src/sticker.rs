use serde::{Deserialize, Serialize};

//================================================================

pub type StickerID = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sticker {
    pub data: Vec<u8>,
}
