use egui::{self, FontFamily, FontId, TextStyle};

//================================================================

use crate::layout::*;
use crate::system::*;
use crate::user::*;
use nimbus_protocol::prelude::*;

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
            let call_ui = ui.clone();

            let call: Box<dyn FnMut(CommandServer) + Send> = Box::new(move |_: CommandServer| {
                call_ui.request_repaint();
            });

            client.client.push(Client::new(
                address.to_string(),
                user.identifier.key,
                user.clone().into(),
                Some(call),
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
        self.client.update(|client, command| match command {
            CommandServer::Message(message) => {
                if let MessageValue::Text(text) = &message.value
                    && let Some(account) = client.get_local_account()
                    && message.is_mention(account)
                    && let Some(account) = message.account(&mut client.cache)
                {
                    self.system
                        .push_notification(account.name_nick.to_string(), text.to_string());
                }
            }
            CommandServer::ViewFile(identifier, file) => {
                if let Some(kind) = &file.kind {
                    if kind.starts_with("image") {
                        self.layout.load_texture_raw(ui, *identifier, &file.data);
                    }
                }
                println!("got file: {:?}, {}", file.kind, file.name);
            }
            _ => {}
        });

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
