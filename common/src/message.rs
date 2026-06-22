use serde::{Deserialize, Serialize};

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub from: String,
    pub kind: MessageKind,
}

impl Message {
    pub fn text(from: String, text: String) -> Self {
        Self {
            from,
            kind: MessageKind::Text(text),
        }
    }

    pub fn file(from: String, name: String, data: Vec<u8>) -> Self {
        Self {
            from,
            kind: MessageKind::File(name, data),
        }
    }

    pub fn sticker(from: String, sticker: u64) -> Self {
        Self {
            from,
            kind: MessageKind::Sticker(sticker),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageKind {
    Text(String),
    File(String, Vec<u8>),
    Sticker(u64),
}
