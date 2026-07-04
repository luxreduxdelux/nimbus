use egui::{self, FontFamily, FontId, TextStyle};

//================================================================

use crate::layout::*;
use crate::system::*;
use crate::user::*;
use nimbus_client::prelude::*;

//================================================================

pub struct App {
    pub system: System,
    pub layout: Layout,
    pub client: ClientMulti,
    pub user: User,
}

impl App {
    pub fn new(ui: &egui::Context) -> Self {
        let user = User::default();

        ui.all_styles_mut(|style| {
            style
                .text_styles
                .insert(TextStyle::Body, FontId::new(16.0, FontFamily::Proportional));
            style.text_styles.insert(
                TextStyle::Button,
                FontId::new(16.0, FontFamily::Proportional),
            );
            style.text_styles.insert(
                TextStyle::Heading,
                FontId::new(24.0, FontFamily::Proportional),
            );
            style.text_styles.insert(
                TextStyle::Monospace,
                FontId::new(16.0, FontFamily::Monospace),
            );
            style.text_styles.insert(
                TextStyle::Small,
                FontId::new(12.0, FontFamily::Proportional),
            );
        });
        ui.set_visuals(user.theme.clone());
        ui.set_zoom_factor(user.zoom);

        let mut client = ClientMulti::default();

        for address in &user.address {
            client.client.push(Client::new(
                address.to_string(),
                user.identifier.key,
                user.clone().into(),
            ));
        }

        Self {
            system: System::new(&user, ui),
            user,
            layout: Default::default(),
            client,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        self.client.update(|_| {});
        Layout::draw(self, ui);

        if self.system.tray {
            let mut exit = false;

            ui.input(|i| {
                if i.viewport().close_requested() {
                    exit = true;
                }
            });

            if exit && !self.system.exit() {
                ui.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                ui.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
        }
    }
}
