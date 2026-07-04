use egui::TextureHandle;
use resvg::render;
use resvg::usvg::{Options, Tree};
use serde::{Deserialize, Serialize};
use tiny_skia::Pixmap;

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
    pub zoom: f32,
    pub tray_show: bool,
    pub notify_push: bool,
    pub notify_tray: bool,
    pub notify_sound: bool,
    pub address: Vec<String>,
}

impl User {
    const PATH_FILE: &str = "client.data";

    pub fn generate_image_identifier(&self, ui: &egui::Context) -> TextureHandle {
        let code = serde_json::to_string(&self.identifier).unwrap();
        let code = qrcode::QrCode::new(code).unwrap();
        let icon = code
            .render::<qrcode::render::svg::Color>()
            .max_dimensions(256, 256)
            .build();

        let tree = Tree::from_str(&icon, &Options::default()).unwrap();
        let mut map = Pixmap::new(256, 256).unwrap();

        render(&tree, tiny_skia::Transform::default(), &mut map.as_mut());

        let icon = map.data().to_vec();
        let icon = egui::ColorImage::from_rgba_unmultiplied([256, 256], &icon);

        ui.load_texture(
            "identifier_image",
            icon,
            eframe::egui::TextureOptions::default(),
        )
    }
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
        if let Ok(data) = std::fs::read(Self::PATH_FILE)
            && let Ok((user, _)) =
                bincode::serde::decode_from_slice(&data, bincode::config::standard())
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
                zoom: 1.0,
                tray_show: true,
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
            bincode::serde::encode_to_vec(self, bincode::config::standard()).unwrap(),
        )
        .unwrap();
    }
}
