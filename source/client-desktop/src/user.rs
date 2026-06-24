use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

//================================================================

use client::common::prelude::*;

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub identifier: Identifier,
    pub name_nick: String,
    pub name_user: String,
    pub info: String,
    pub icon: Option<Vec<u8>>,
    pub address: String,
}

impl Into<AccountConnect> for User {
    fn into(self) -> AccountConnect {
        let icon = if let Some(icon) = &self.icon {
            // TO-DO
            None
        } else {
            None
        };

        AccountConnect {
            key: self.identifier.public_key(),
            name_nick: self.name_nick.clone(),
            name_user: self.name_user.clone(),
            info: self.info.clone(),
            icon,
        }
    }
}

impl User {
    const PATH_FILE: &str = "user.data";
}

impl Default for User {
    fn default() -> Self {
        if let Ok(data) = std::fs::read(Self::PATH_FILE)
            && let Ok((user, _)) =
                bincode::serde::decode_from_slice(&data, bincode::config::standard())
        {
            user
        } else {
            Self {
                identifier: Default::default(),
                name_nick: Default::default(),
                name_user: Default::default(),
                info: Default::default(),
                icon: Default::default(),
                address: "127.0.0.1".to_string(),
            }
        }
    }
}

impl Drop for User {
    fn drop(&mut self) {
        std::fs::write(
            Self::PATH_FILE,
            bincode::serde::encode_to_vec(self, bincode::config::standard()).unwrap(),
        )
        .unwrap();
    }
}

//================================================================

///The signing key (otherwise known as "private key").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier(pub AccountKey);

impl Identifier {
    const PATH_FILE: &str = "key.nimbus";

    pub fn public_key(&self) -> AccountKey {
        let key = SigningKey::from(self.0);
        key.verifying_key().to_bytes()
    }
}

impl Default for Identifier {
    fn default() -> Self {
        if let Ok(data) = std::fs::read(Self::PATH_FILE)
            && let Ok((identifier, _)) =
                bincode::serde::decode_from_slice(&data, bincode::config::standard())
        {
            identifier
        } else {
            Self(SigningKey::generate(&mut OsRng).to_bytes())
        }
    }
}

impl Drop for Identifier {
    fn drop(&mut self) {
        std::fs::write(
            Self::PATH_FILE,
            bincode::serde::encode_to_vec(self, bincode::config::standard()).unwrap(),
        )
        .unwrap();
    }
}
