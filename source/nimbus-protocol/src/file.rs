use serde::{Deserialize, Serialize};

//================================================================

use crate::storage::*;

//================================================================

pub type FileID = u64;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct File {
    pub name: String,
    pub data: Vec<u8>,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileKind {
    Text,
    Image,
    Video,
    Audio,
    Other,
}

impl File {
    pub fn new(name: String, data: Vec<u8>) -> Self {
        let kind = infer::get(&data).map(|kind| kind.mime_type().to_string());

        Self { name, data, kind }
    }

    pub fn kind(&self) -> FileKind {
        if let Some(kind) = &self.kind {
            if kind.starts_with("text") {
                return FileKind::Text;
            } else if kind.starts_with("image") {
                return FileKind::Image;
            } else if kind.starts_with("video") {
                return FileKind::Video;
            } else if kind.starts_with("audio") {
                return FileKind::Audio;
            }
        }

        FileKind::Other
    }
}

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileValue {
    pub name: String,
    pub data: Vec<u8>,
}

impl FileValue {
    pub fn new(name: String, data: Vec<u8>) -> Self {
        Self { name, data }
    }

    pub fn insert(self, storage: &mut Storage) -> anyhow::Result<FileMeta> {
        let index = storage.count_file()?;
        let file = File::new(self.name.clone(), self.data);
        let meta = FileMeta::new(index, &file);
        storage.insert_file(index, file.clone())?;

        Ok(meta)
    }
}

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    pub index: FileID,
    pub name: String,
    pub size: usize,
    pub kind: Option<String>,
}

impl FileMeta {
    pub fn new(index: FileID, file: &File) -> Self {
        Self {
            index,
            name: file.name.clone(),
            size: file.data.len(),
            kind: file.kind.clone(),
        }
    }

    pub fn kind(&self) -> FileKind {
        if let Some(kind) = &self.kind {
            if kind.starts_with("text") {
                return FileKind::Text;
            } else if kind.starts_with("image") {
                return FileKind::Image;
            } else if kind.starts_with("video") {
                return FileKind::Video;
            } else if kind.starts_with("audio") {
                return FileKind::Audio;
            }
        }

        FileKind::Other
    }
}
