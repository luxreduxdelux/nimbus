use chrono::{DateTime, Local, Utc};
use eframe::egui::{
    self, Color32, ColorImage, FontId, ImageSource, Response, RichText, Spinner, TextureHandle,
    Vec2,
};
use egui_modal::Modal;
use std::collections::HashMap;

//================================================================

use crate::system::*;
use crate::user::*;
use client::common::prelude::*;
use client::*;

//================================================================

#[derive(Default, PartialEq, Eq)]
enum Setup {
    #[default]
    Account,
    Window,
    Notify,
    Input,
}

pub struct App {
    system: System,
    client: Option<Client>,
    user: User,
    index_channel: ChannelID,
    setup: Setup,
    entry: String,
    image: HashMap<u64, TextureHandle>,
    image_icon_main: Option<TextureHandle>,
    image_icon_side: Option<TextureHandle>,
    image_identifier: TextureHandle,
    show_setup: bool,
    show_poll: bool,
}

impl App {
    pub fn new(ui: &egui::Context) -> Self {
        let user = User::default();
        let image_icon_main = if let Some(icon) = &user.icon_main {
            Some(Self::load_image("icon_main", icon, ui).unwrap())
        } else {
            None
        };
        let image_icon_side = if let Some(icon) = &user.icon_side {
            Some(Self::load_image("icon_side", icon, ui).unwrap())
        } else {
            None
        };
        let image_identifier = user.generate_image_identifier(ui);

        ui.set_visuals(user.theme.clone());
        ui.set_zoom_factor(user.zoom);

        Self {
            system: System::new(&user),
            client: None,
            user,
            index_channel: 0,
            setup: Default::default(),
            entry: Default::default(),
            image: Default::default(),
            image_icon_main,
            image_icon_side,
            image_identifier,
            show_setup: Default::default(),
            show_poll: Default::default(),
        }
    }

    fn load_image(index: &str, data: &[u8], ui: &egui::Context) -> anyhow::Result<TextureHandle> {
        let image = image::load_from_memory(data)?.to_rgba8();
        let color_image = ColorImage::from_rgba_unmultiplied(
            [image.width() as usize, image.height() as usize],
            image.as_raw(),
        );
        Ok(ui.load_texture(
            index.to_string(),
            color_image,
            eframe::egui::TextureOptions::default(),
        ))
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        //ui.request_repaint();

        //self.draw_setup(ui);
        //return;

        if let Some(client) = &mut self.client {
            client.update(|command| match command {
                CommandServer::Enter(_, server) => {
                    //for (index, sticker) in &server.sticker {
                    //    println!("[CLIENT] Load image: {index}");
                    //    Self::load_image(&mut self.image, *index, &sticker.data, ui);
                    //}
                }
                _ => {}
            });

            if let Some(error) = &client.error {
                self.draw_error(ui);
            } else if client.ready {
                self.draw_chat(ui);
            } else {
                self.draw_load(ui);
            }
        } else {
            self.draw_main(ui);
        }
    }

    fn draw_setup(&mut self, ui: &mut egui::Ui) {
        /*
        Account
            key
            nick
            name
            info
            icon
        Notify
            sound
            push
            tray
        Window
            zoom
            tray
            theme
        Input
            voice
            hot-key
        */
        egui::Panel::top("top").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                Self::button_image(
                    ui,
                    egui::include_image!("../asset/back.svg"),
                    Vec2::new(24.0, 24.0),
                );

                ui.selectable_value(&mut self.setup, Setup::Account, "Account");
                ui.selectable_value(&mut self.setup, Setup::Window, "Window");
                ui.selectable_value(&mut self.setup, Setup::Notify, "Notify");
                ui.selectable_value(&mut self.setup, Setup::Input, "Input");
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| match &self.setup {
                Setup::Account => {
                    ui.horizontal(|ui| {
                        ui.heading("Identifier");

                        let modal = Modal::new(ui, "identifier_help");

                        modal.show(|ui| {
                            modal.title(ui, "Identifier Help");
                            modal.frame(ui, |ui| {
                                ui.label("An identifier can uniquely identify you across any Nimbus community. When connecting to a Nimbus server for the first time, it will register you using your identifier.\n
                                    If you or another person using your identifier connect again to that community, they must prove ownership to your identifier.\n
                                    For that reason, you must NOT share your identifier with anyone else.");
                            });
                            modal.buttons(ui, |ui| {
                                modal.button(ui, "Close");
                            });
                        });

                        if ui.button("?").clicked() {
                            modal.open();
                        }
                    });

                    ui.separator();

                    let date =
                        DateTime::<Utc>::from_timestamp(self.user.identifier.date, 0).unwrap();
                    let date = date.with_timezone(&Local);

                    ui.label("Identifier Name");
                    ui.text_edit_singleline(&mut self.user.identifier.name);

                    ui.label("Identifier Date");
                    ui.label(RichText::new(date.to_string()).strong());

                    //================

                    let modal = Modal::new(ui, "identifier_image");

                    modal.show(|ui| {
                        modal.title(ui, "Identifier");
                        modal.frame(ui, |ui| {
                            ui.image(&self.image_identifier);
                        });
                        modal.buttons(ui, |ui| {
                            modal.button(ui, "Close");
                        });
                    });

                    if ui.button("Show Identifier QR Code").clicked() {
                        modal.open();
                    }

                    //================

                    let modal = Modal::new(ui, "identifier_create");

                    modal.show(|ui| {
                        modal.title(ui, "Create Identifier");
                        modal.frame(ui, |ui| {
                            ui.label("WARNING! This will discard the current identifier. Make sure to export your identifier before continuing.")
                        });
                        modal.buttons(ui, |ui| {
                            if modal.button(ui, "Create").clicked() {
                                self.user.identifier  = Identifier::default();
                                self.image_identifier = self.user.generate_image_identifier(ui.ctx());
                            }
                            modal.button(ui, "Cancel");
                        });
                    });

                    if ui.button("Create Identifier").clicked() {
                        modal.open();
                    }

                    //================

                    let modal = Modal::new(ui, "identifier_import");

                    modal.show(|ui| {
                        modal.title(ui, "Import Identifier");
                        modal.frame(ui, |ui| {
                            ui.label("WARNING! This will discard the current identifier. Make sure to export your identifier before continuing.")
                        });
                        modal.buttons(ui, |ui| {
                            if modal.button(ui, "Import").clicked() {
                                self.user.identifier  = Identifier::default();
                                self.image_identifier = self.user.generate_image_identifier(ui.ctx());
                            }
                            modal.button(ui, "Cancel");
                        });
                    });

                    if ui.button("Import Identifier").clicked() {
                        modal.open();
                    }

                    //================

                    ui.button("Export Identifier");


                    ui.heading("Persona");
                    ui.separator();


                    ui.label("User Icon (Main/Side)");
                    ui.horizontal(|ui| {
                        if let Some(icon) = &self.image_icon_main {
                            ui.add(
                                egui::Image::new(icon)
                                    .fit_to_exact_size(Vec2::new(64.0, 64.0)),
                            );
                        } else {
                            ui.add(
                                egui::Image::new(egui::include_image!("../asset/user.svg"))
                                    .fit_to_exact_size(Vec2::new(64.0, 64.0)),
                            );
                        };

                        if let Some(icon) = &self.image_icon_side {
                            ui.add(
                                egui::Image::new(icon)
                                    .fit_to_exact_size(Vec2::new(256.0, 64.0)),
                            );
                        } else {
                            ui.add(
                                egui::Image::new(egui::include_image!("../asset/user.svg"))
                                    .fit_to_exact_size(Vec2::new(256.0, 64.0)),
                            );
                        };
                    });

                    if ui.button("Set Main User Icon").clicked() && let Some(file) = rfd::FileDialog::new().pick_file() {
                        self.user.icon_main = Some(std::fs::read(file).unwrap());
                    }
                    if ui.button("Set Side User Icon").clicked() && let Some(file) = rfd::FileDialog::new().pick_file() {
                        self.user.icon_side = Some(std::fs::read(file).unwrap());
                    }

                    ui.label("Nick Name");
                    ui.text_edit_singleline(&mut self.user.name_nick);

                    ui.label("User Name");
                    ui.text_edit_singleline(&mut self.user.name_user);

                    ui.label("User Info");
                    ui.text_edit_multiline(&mut self.user.info);

                    ui.heading("Privacy");
                    ui.separator();

                    ui.checkbox(&mut true, "Send Type Indicator");
                    ui.checkbox(&mut true, "Send Read Indicator");
                    ui.checkbox(&mut true, "Send Rich Presence");

                    // block list, automatic message deletion, delete all if away for X time
                }
                Setup::Window => {
                    ui.label("Zoom Scale");

                    let response = ui.add(egui::Slider::new(&mut self.user.zoom, 0.5..=2.0));

                    if response.drag_stopped() || (!response.dragged() && response.changed()) {
                        ui.set_zoom_factor(self.user.zoom);
                    }

                    ui.checkbox(&mut false, "Show Tray Icon");

                    ui.heading("Theme");
                    ui.separator();

                    if ui.button("Import Theme").clicked() {
                        ui.copy_text(serde_json::to_string_pretty(ui.visuals()).unwrap());
                    };
                    if ui.button("Export Theme").clicked() {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::RequestPaste);
                    };
                }
                Setup::Notify => {
                    ui.checkbox(&mut true, "Play Sound");
                    ui.checkbox(&mut true, "Push Notification");
                    ui.checkbox(&mut true, "Icon Notification");
                }
                Setup::Input => {
                    ui.label("Voice Input");
                    ui.label("Voice Check");
                    ui.label("Double-Click Message Action");
                    ui.label("Channel Navigation (Upper)");
                    ui.label("Channel Navigation (Lower)");
                    ui.label("Edit Message (On Hover)");
                    ui.label("Star Message (On Hover)");
                    ui.label("React Message (On Hover)");
                    ui.label("Edit Last Message");
                    ui.label("Toggle Navigator");
                }
            });
        });
    }

    fn draw_main(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.5 - 224.0);

                ui.add(
                    egui::Image::new(egui::include_image!("../asset/logo.svg"))
                        .fit_to_exact_size(Vec2::new(96.0, 96.0)),
                );

                ui.label(
                    RichText::new("Welcome to Nimbus!")
                        .font(FontId::proportional(32.0))
                        .strong(),
                );

                ui.separator();

                /*
                ui.label("Identifier (?)")
                    .on_hover_text("An identifier is what will represent you in a Nimbus server and can validate that it is you who is logging into a server.\nDo NOT distribute your identifier file to anyone.");

                if let Some(identifier) = &self.user.identifier {
                    ui.label(RichText::new(&identifier.name).strong());

                    ui.button("New Identifier");
                    ui.button("Save Identifier");
                    ui.button("Load Identifier");
                } else {
                    ui.label(RichText::new("...").weak());

                    ui.button("New Identifier");
                    ui.button("Load Identifier");
                }
                */

                /*
                if self.user.icon.is_some() {
                    if ui.button("Remove Icon").clicked() {
                        self.user.icon = None;
                    };
                } else {
                    if ui.button("Add Icon").clicked()
                        && let Some(icon) = rfd::FileDialog::new()
                            .set_title("Select an user icon.")
                            .pick_file()
                        && let Ok(file) = std::fs::read(icon)
                    {
                        self.user.icon = Some(file);
                    };
                }

                if let Some(icon) = &self.user.icon {
                    ui.add(
                        egui::Image::new(ImageSource::Uri(format!("file://{icon}").into()))
                            .fit_to_exact_size(Vec2::new(96.0, 96.0))
                            .corner_radius(96.0),
                    );
                } else {
                    ui.add(
                        egui::Image::new(egui::include_image!("../asset/user.svg"))
                            .fit_to_exact_size(Vec2::new(96.0, 96.0)),
                    );
                };
                */

                ui.label("Nick Name");
                ui.text_edit_singleline(&mut self.user.name_nick);

                ui.label("User Name");
                ui.text_edit_singleline(&mut self.user.name_user);

                ui.label("Info");
                ui.text_edit_multiline(&mut self.user.info);

                ui.label("Server Address");
                ui.text_edit_singleline(&mut self.user.address);

                ui.separator();

                let valid_nick = AccountConnect::is_valid_nick(&self.user.name_nick);
                let valid_user = AccountConnect::is_valid_user(&self.user.name_user);
                let valid_info = AccountConnect::is_valid_info(&self.user.info);
                let valid = valid_nick.is_ok() && valid_user.is_ok() && valid_info.is_ok();

                ui.add_enabled_ui(valid, |ui| {
                    if ui.button("Log In").clicked() {
                        self.client = Some(Client::new(
                            self.user.address.to_string(),
                            self.user.identifier.key,
                            self.user.clone().into(),
                        ));
                    }
                });

                if let Err(error) = valid_nick {
                    #[rustfmt::skip]
                    match error {
                        NickError::Empty  => ui.label("Nick name cannot be empty."),
                        NickError::Length => ui.label(format!("Nick name cannot be greater than {} characters.", AccountConnect::LIMIT_NICK_NAME)),
                    };
                }

                if let Err(error) = valid_user {
                    #[rustfmt::skip]
                    match error {
                        UserError::Empty        => ui.label("User name cannot be empty."),
                        UserError::Length       => ui.label(format!("User name cannot be greater than {} characters.", AccountConnect::LIMIT_USER_NAME)),
                        UserError::InvalidASCII => ui.label("User name must use a-z, A-Z, 0-9 characters."),
                    };
                }
            });
        });
    }

    fn draw_load(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.5 - 128.0);

                ui.label(
                    RichText::new("Connecting...")
                        .font(FontId::proportional(32.0))
                        .strong(),
                );

                /*
                let icon = if let Some(icon) = &self.user.icon {
                    egui::Image::new(ImageSource::Uri(format!("file://{icon}").into()))
                        .fit_to_exact_size(Vec2::new(96.0, 96.0))
                        .corner_radius(96.0)
                } else {
                    egui::Image::new(egui::include_image!("../asset/user.svg"))
                        .fit_to_exact_size(Vec2::new(96.0, 96.0))
                };

                ui.add(icon);
                */

                ui.label(&self.user.name_user);

                ui.add(Spinner::new().size(48.0));

                if ui.button("Cancel").clicked() {
                    self.client = None;
                }
            });
        });
    }

    fn pop_up<F: FnOnce(&mut egui::Ui)>(
        identifier: &str,
        response: egui::Response,
        ui: &mut egui::Ui,
        content: F,
    ) {
        let identifier = ui.make_persistent_id(identifier);

        if response.clicked() {
            egui::Popup::open_id(ui.ctx(), identifier);
        }

        #[rustfmt::skip]
        egui::Popup::from_toggle_button_response(&response).show(content);
    }

    fn draw_account(ui: &mut egui::Ui, client: Option<&mut Client>, account: &Account) {
        ui.horizontal(|ui| {
            if let Some(client) = &client {
                let response = Self::button_image(
                    ui,
                    ImageSource::Uri("file:///home/lux/Desktop/deer.png".to_string().into()),
                    Vec2::new(32.0, 32.0),
                );

                Self::pop_up("status", response, ui, |ui| {
                    let mut state = client.server.account[&client.index].state.clone();
                    let mut click = false;
                    let state_list = [
                        (AccountState::Online, "Online"),
                        (AccountState::Away, "Away"),
                        (AccountState::Busy, "Busy"),
                        (AccountState::Offline, "Offline"),
                    ];

                    for (s, n) in state_list {
                        if ui.selectable_value(&mut state, s, n).clicked() {
                            click = true;
                        }
                    }

                    if click {
                        client.send(CommandClient::AccountState(state));
                    }
                });
            } else {
                ui.add(
                    egui::Image::new(ImageSource::Uri(
                        "file:///home/lux/Desktop/deer.png".to_string().into(),
                    ))
                    .fit_to_exact_size(Vec2::new(32.0, 32.0))
                    .corner_radius(32.0),
                );
            }

            let point = ui
                .min_rect()
                .translate(Vec2::new(14.0, 14.0))
                .scale_from_center(0.25);

            let color = match account.state {
                AccountState::Online => Color32::GREEN,
                AccountState::Away => Color32::ORANGE,
                AccountState::Busy => Color32::RED,
                AccountState::Offline => Color32::DARK_GRAY,
            };

            egui::Image::new(egui::include_image!("../asset/dot.svg"))
                .tint(color)
                .paint_at(ui, point);

            ui.label(RichText::new(&account.name_nick).strong());

            if client.is_some() {
                if Self::button_image(
                    ui,
                    egui::include_image!("../asset/cog.svg"),
                    Vec2::new(32.0, 32.0),
                )
                .clicked()
                {};
            }
        });
    }

    fn draw_error(&mut self, ui: &mut egui::Ui) {
        // TO-DO are you kidding me
        let error = self
            .client
            .as_ref()
            .unwrap()
            .error
            .as_ref()
            .unwrap()
            .clone();

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.5 - 128.0);

                ui.label(
                    RichText::new("Error!")
                        .font(FontId::proportional(32.0))
                        .strong(),
                );

                ui.label(error.to_string());

                if ui.button("Accept").clicked() {
                    self.client = None;
                }
            });
        });
    }

    fn draw_chat(&mut self, ui: &mut egui::Ui) {
        let client = self.client.as_mut().unwrap();

        egui::Panel::left("left").show_inside(ui, |ui| {
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                Self::button_image(
                    ui,
                    ImageSource::Uri("file:///home/lux/Desktop/deer.png".to_string().into()),
                    Vec2::new(32.0, 32.0),
                );

                ui.label(RichText::new(&client.server.name).strong());
            });

            ui.separator();

            let mut seen = vec![];

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(ui.available_height() - 48.0)
                .show(ui, |ui| {
                    //
                    for c in &client.server.category {
                        for i in &c.list {
                            if !seen.contains(i) {
                                seen.push(*i);
                            }
                        }

                        ui.collapsing(&c.name, |ui| {
                            for i in &c.list {
                                let channel = &client.server.channel[&i];

                                if ui
                                    .selectable_value(
                                        &mut self.index_channel,
                                        *i as u64,
                                        &format!("#{}", channel.name),
                                    )
                                    .clicked()
                                {
                                    client.send(CommandClient::AccountChannel(*i as ChannelID));
                                };
                            }
                        });
                    }

                    for (i, c) in &client.server.channel {
                        if !seen.contains(i) {
                            if ui
                                .selectable_value(
                                    &mut self.index_channel,
                                    *i as u64,
                                    &format!("#{}", c.name),
                                )
                                .clicked()
                            {
                                client.send(CommandClient::AccountChannel(*i as ChannelID));
                            };

                            seen.push(*i);
                        }
                    }
                });

            egui::Area::new("account".into())
                .fixed_pos([ui.cursor().min.x, ui.viewport_rect().max.y - 48.0])
                .show(ui, |ui| {
                    ui.separator();

                    let account = client.server.account[&client.index].clone();
                    Self::draw_account(ui, Some(client), &account);
                });
        });

        egui::Panel::right("right").show_inside(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(6.0);
                    for (_, account) in &client.server.account {
                        Self::draw_account(ui, None, account);
                    }
                });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let channel = client.server.channel.get(&self.index_channel).unwrap();
            let shape_x = ui.max_rect().width();

            ui.label(RichText::new(&format!("#{}", channel.name)).strong());
            ui.label(&channel.info);

            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(ui.available_height() - 48.0)
                .show(ui, |ui| {
                    /*
                    for x in 0..32 {
                        let image = egui::Image::new(ImageSource::Uri(
                            "file:///home/lux/Desktop/deer.png".to_string().into(),
                        ))
                        .fit_to_exact_size(Vec2::new(40.0, 40.0))
                        .corner_radius(40.0);

                        ui.add(image);

                        ui.label("foo");
                    }
                    */

                    for (index, message) in &channel.message {
                        let response = ui.horizontal(|ui| {
                            let image = egui::Image::new(ImageSource::Uri(
                                "file:///home/lux/Desktop/deer.png".to_string().into(),
                            ))
                            .fit_to_exact_size(Vec2::new(32.0, 32.0))
                            .corner_radius(32.0);

                            ui.add(image);

                            ui.vertical(|ui| {
                                let from = message.account(&client.server);

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&from.name_nick).strong());
                                    ui.label(RichText::new("9:41").weak());
                                });

                                match &message.kind {
                                    MessageKind::Text(text) => {
                                        ui.label(text);
                                    }
                                    MessageKind::File(name, data) => {
                                        if ui.button(format!("File attachment ({name})")).clicked()
                                        {
                                            if let Some(path) = rfd::FileDialog::new()
                                                .set_title("Save file.")
                                                .set_file_name(name)
                                                .save_file()
                                            {
                                                std::fs::write(path, data);
                                            }
                                        }
                                    }
                                    MessageKind::Sticker(sticker) => {
                                        if let Some(image) = self.image.get(&sticker) {
                                            ui.image(image);
                                        }
                                    }
                                }
                            });
                        }).response;

                        response.context_menu(|ui| {
                            #[rustfmt::skip]
                            Self::button_image(ui, egui::include_image!("../asset/reply.svg"),  Vec2::new(24.0, 24.0));
                            Self::button_image(ui, egui::include_image!("../asset/emote.svg"),  Vec2::new(24.0, 24.0));
                            Self::button_image(ui, egui::include_image!("../asset/copy.svg"),   Vec2::new(24.0, 24.0));
                            Self::button_image(ui, egui::include_image!("../asset/star_a.svg"), Vec2::new(24.0, 24.0));
                            Self::button_image(ui, egui::include_image!("../asset/edit.svg"),   Vec2::new(24.0, 24.0));

                            if Self::button_image(ui, egui::include_image!("../asset/delete.svg"), Vec2::new(24.0, 24.0)).clicked() {
                                client.send(CommandClient::MessageDelete(self.index_channel, *index));
                            };
                        });

                        ui.add_space(4.0);
                    }
                });

            let mut write = Vec::new();

            for (_, account) in &client.server.account {
                if account.index != client.index && account.write && account.channel == self.index_channel {
                    write.push(account.name_nick.clone());
                }
            }

            let push = if write.is_empty() { 48.0 } else { 66.0 };

            egui::Area::new("floating_text".into())
                .fixed_pos([ui.cursor().min.x, ui.viewport_rect().max.y - push])
                .show(ui, |ui| {
                    if !write.is_empty() {
                        egui::Frame::new()
                            .fill(egui::Color32::from_black_alpha(224))
                            .corner_radius(egui::CornerRadius::same(4))
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                if write.len() == 1 {
                                    ui.label(format!("{} is typing...", write[0]));
                                } else if write.len() == 2 {
                                    ui.label(format!(
                                        "{} and {} are typing...",
                                        write[0], write[1]
                                    ));
                                } else if write.len() == 3 {
                                    ui.label(format!(
                                        "{}, {} and {} are typing...",
                                        write[0], write[1], write[2]
                                    ));
                                } else {
                                    ui.label("Several people are typing...");
                                }
                            });
                    }

                    ui.separator();


                    ui.horizontal(|ui| {
                        let add = Self::button_image(
                            ui,
                            egui::include_image!("../asset/plus.svg"),
                            Vec2::new(32.0, 32.0),
                        );

                        //let id = ui.make_persistent_id("my_modal");

                        Self::pop_up("plus", add, ui, |ui| {
                            ui.button("Upload File");
                            if ui.button("Submit Poll").clicked() {
                                self.show_poll = true;
                            };
                        });

                        if self.show_poll {
                            egui::Modal::new("blah".into()).show(ui.ctx(), |ui| {
                                ui.label("Hello");

                                if ui.button("Close").clicked() {
                                    self.show_poll = false;
                                }
                            });
                        }

                        // TO-DO
                        let empty = self.entry.is_empty();
                        let text = ui.add_sized(
                            [(shape_x - 192.0).max(0.0), 32.0],
                            egui::TextEdit::singleline(&mut self.entry)
                                .font(FontId::default())
                                .vertical_align(egui::Align::Center)
                                .id("client_buffer".into()),
                        );

                        if text.changed() {
                            if empty && !self.entry.is_empty() {
                                client.send(CommandClient::AccountWrite(true));
                            } else if !empty && self.entry.is_empty() {
                                client.send(CommandClient::AccountWrite(false));
                            }
                        }

                        if text.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            client.send(CommandClient::Message(
                                self.index_channel,
                                MessageKind::Text(self.entry.clone()),
                            ));
                            client.send(CommandClient::AccountWrite(false));
                            self.entry.clear();
                            text.request_focus();
                        }

                        Self::button_image(
                            ui,
                            egui::include_image!("../asset/sticker.svg"),
                            Vec2::new(32.0, 32.0),
                        )
                        .context_menu(|ui| {
                            for (index, image) in &self.image {
                                if Self::button_image(ui, image.into(), Vec2::new(32.0, 32.0))
                                    .clicked()
                                {
                                    //client.send_sticker(self.index_channel, *index);
                                }
                            }

                            /*
                            ui.vertical(|ui| {
                                for y in 0..4 {
                                    ui.horizontal(|ui| {
                                        for x in 0..4 {
                                            ui.button(format!("{x} x {y}"));
                                        }
                                    });
                                }
                            });
                            */
                        });
                        Self::button_image(
                            ui,
                            egui::include_image!("../asset/emote.svg"),
                            Vec2::new(32.0, 32.0),
                        );
                        if Self::button_image(
                            ui,
                            egui::include_image!("../asset/send.svg"),
                            Vec2::new(32.0, 32.0),
                        )
                        .clicked()
                        {
                            client.send(CommandClient::Message(
                                self.index_channel,
                                MessageKind::Text(self.entry.clone()),
                            ));
                            client.send(CommandClient::AccountWrite(false));
                            self.entry.clear();
                        };
                    });
                    //ui.label("Hello world");
                });
        });
    }

    fn button_image(ui: &mut egui::Ui, image: ImageSource, size: Vec2) -> Response {
        ui.button(egui::Image::new(image).fit_to_exact_size(size))
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        self.draw(ui);
    }
}
