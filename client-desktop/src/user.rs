use serde::{Deserialize, Serialize};

//================================================================

use client::common::prelude::*;

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub icon: Option<String>,
    pub address: String,
}

impl Into<Account> for User {
    fn into(self) -> Account {
        let icon = if let Some(icon) = &self.icon {
            // TO-DO
            None
        } else {
            None
        };

        Account {
            name: self.name.clone(),
            icon: icon,
            ..Default::default()
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
                name: Default::default(),
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
