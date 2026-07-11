use serde::{Deserialize, Serialize};

//================================================================

use nimbus_protocol::prelude::*;

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub identifier: Identifier,
    pub name_nick: String,
    pub name_user: String,
    pub info: String,
    pub icon: Option<Vec<u8>>,
    pub theme: egui::Visuals,
    pub indicator_read: bool,
    pub indicator_type: bool,
    pub indicator_seen: bool,
    pub zoom: f32,
    pub tray_show: bool,
    pub embed_link: bool,
    pub embed_file: bool,
    pub show_hidden: bool,
    pub message_compact: bool,
    pub notify_push: bool,
    pub notify_tray: bool,
    pub notify_sound: bool,
    pub address: Vec<String>,
}

impl User {
    const PATH_FILE: &str = "client.data";
}

impl Into<AccountConnect> for User {
    fn into(self) -> AccountConnect {
        AccountConnect {
            key: self.identifier.public_key(),
            name_nick: self.name_nick.clone(),
            name_user: self.name_user.clone(),
            info: self.info.clone(),
            icon: self.icon.clone(),
        }
    }
}

impl Default for User {
    fn default() -> Self {
        if let Ok(data) = std::fs::read_to_string(Self::PATH_FILE)
            && let Ok(user) = serde_json::from_str(&data)
        {
            user
        } else {
            Self {
                identifier: Default::default(),
                name_nick: "Nimbus User".to_string(),
                name_user: "nimbus_user".to_string(),
                info: "Sure is nice being a Nimbus user around here!".to_string(),
                icon: Default::default(),
                theme: Default::default(),
                indicator_read: true,
                indicator_type: true,
                indicator_seen: true,
                zoom: 1.0,
                tray_show: true,
                embed_link: true,
                embed_file: true,
                show_hidden: false,
                message_compact: false,
                notify_push: true,
                notify_tray: true,
                notify_sound: true,
                address: vec!["127.0.0.1".to_string()],
            }
        }
    }
}

impl Drop for User {
    fn drop(&mut self) {
        std::fs::write(
            Self::PATH_FILE,
            serde_json::to_string_pretty(&self).unwrap(),
        )
        .unwrap();
    }
}
