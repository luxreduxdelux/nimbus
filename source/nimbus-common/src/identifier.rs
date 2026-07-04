use chrono::Utc;
use ed25519_dalek::SigningKey;
use rand::prelude::IteratorRandom;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

//================================================================

use crate::account::*;

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub name: String,
    pub date: i64,
    pub key: AccountKey,
}

impl Identifier {
    pub fn public_key(&self) -> AccountKey {
        let key = SigningKey::from(self.key);
        key.verifying_key().to_bytes()
    }
}

impl Default for Identifier {
    fn default() -> Self {
        let key = SigningKey::generate(&mut OsRng);
        let v_k = key.verifying_key().to_bytes();

        Self {
            name: format!(
                "Identifier #{}",
                (1000..=9999).choose(&mut OsRng).unwrap_or(1000)
            ),
            date: Utc::now().timestamp(),
            key: key.to_bytes(),
        }
    }
}
