use serde::{Deserialize, Serialize};

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub name: String,
    pub info: String,
    pub icon: Option<Vec<u8>>,
    pub limit_text_size: usize,
    pub limit_file_size: usize,
    pub limit_poll_size: usize,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            name: Self::DEFAULT_NAME.to_string(),
            info: Self::DEFAULT_INFO.to_string(),
            icon: None,
            limit_text_size: Self::LIMIT_TEXT_SIZE,
            limit_file_size: Self::LIMIT_FILE_SIZE,
            limit_poll_size: Self::LIMIT_POLL_SIZE,
        }
    }
}

impl Configuration {
    const DEFAULT_NAME: &str = "Nimbus Server";
    const DEFAULT_INFO: &str = "A default Nimbus server, for the people, by the people.\nhttps://github.com/luxreduxdelux/nimbus";
    const LIMIT_TEXT_SIZE: usize = 256;
    const LIMIT_FILE_SIZE: usize = 16;
    const LIMIT_POLL_SIZE: usize = 16;
}
