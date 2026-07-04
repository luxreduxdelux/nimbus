use serde::{Deserialize, Serialize};

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub limit_text_size: usize,
    pub limit_file_size: usize,
    pub limit_poll_size: usize,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            limit_text_size: Self::LIMIT_TEXT_SIZE,
            limit_file_size: Self::LIMIT_FILE_SIZE,
            limit_poll_size: Self::LIMIT_POLL_SIZE,
        }
    }
}

impl Configuration {
    const LIMIT_TEXT_SIZE: usize = 256;
    const LIMIT_FILE_SIZE: usize = 1_000_000 * 10;
    const LIMIT_POLL_SIZE: usize = 16;
}
