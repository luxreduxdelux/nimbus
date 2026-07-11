use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

//================================================================

use crate::account::*;
use crate::channel::*;
use crate::configuration::*;
use crate::storage::*;

//================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Server {
    pub configuration: Configuration,
}

impl Server {
    pub fn from_storage(storage: &Storage) -> anyhow::Result<Self> {
        Ok(Self {
            configuration: Configuration::default(),
        })
    }
}
