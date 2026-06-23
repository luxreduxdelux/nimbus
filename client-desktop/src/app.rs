use eframe::egui::{
    self, Color32, ColorImage, FontId, ImageSource, Response, RichText, Spinner, TextureHandle,
    Vec2,
};
use std::collections::HashMap;

//================================================================

use crate::user::*;
use client::common::prelude::*;
use client::*;

//================================================================

#[derive(Default)]
pub struct App {
    user: User,
    client: Option<Client>,
    index_channel: ChannelID,
    index_message: Option<u64>,
    entry: String,
    image: HashMap<u64, TextureHandle>,
}

impl App {
    fn load_image(
        cache: &mut HashMap<u64, TextureHandle>,
        index: u64,
        data: &[u8],
        ui: &eframe::egui::Ui,
    ) -> anyhow::Result<()> {
        let image = image::load_from_memory(data)?.to_rgba8();
        let color_image = ColorImage::from_rgba_unmultiplied(
            [image.width() as usize, image.height() as usize],
            image.as_raw(),
        );
        let texture = ui.ctx().load_texture(
            index.to_string(),
            color_image,
            eframe::egui::TextureOptions::default(),
        );

        cache.insert(index, texture);

        Ok(())
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        //ui.request_repaint();

        //self.draw_setup(ui);
        //return;

        if let Some(client) = &mut self.client {
            client.update(|command| match command {
                CommandServer::Enter(_, server) => {
                    for (index, sticker) in &server.sticker {
                        println!("[CLIENT] Load image: {index}");
                        Self::load_image(&mut self.image, *index, &sticker.data, ui);
                    }
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
        egui::Panel::top("top").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.button("<-");
                ui.label(RichText::new("Setup").strong());
            });
        });
        egui::Panel::left("left").show_inside(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.selectable_value(&mut 1, 1, "1");
                });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.label("Nick Name");
            ui.text_edit_singleline(&mut self.user.name_nick);

            ui.label("User Name");
            ui.text_edit_singleline(&mut self.user.name_user);

            ui.label("User Info");
            ui.text_edit_multiline(&mut self.user.info);
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

                ui.add_enabled_ui(true, |ui| {
                    if ui.button("Log In").clicked() {
                        self.client = Some(Client::new(
                            self.user.address.to_string(),
                            self.user.identifier.0,
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

    fn draw_account(ui: &mut egui::Ui, client: Option<&mut Client>, account: &Account) {
        ui.horizontal(|ui| {
            if let Some(client) = &client {
                let id = ui.make_persistent_id("status");

                let response = Self::button_image(
                    ui,
                    ImageSource::Uri("file:///home/lux/Desktop/deer.png".to_string().into()),
                    Vec2::new(32.0, 32.0),
                );

                if response.clicked() {
                    egui::Popup::open_id(ui.ctx(), id);
                }

                #[rustfmt::skip]
                egui::Popup::from_toggle_button_response(&response).show(|ui| {
                    let mut state = client.server.account[&client.index].state.clone();
                    let mut click = false;
                    let state_list = [
                        (AccountState::Online, "Online"),
                        (AccountState::Away, "Away"),
                        (AccountState::Busy, "Busy"),
                        (AccountState::Offline, "Offline")
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
                Self::button_image(
                    ui,
                    egui::include_image!("../asset/cog.svg"),
                    Vec2::new(32.0, 32.0),
                );
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

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(ui.available_height() - 48.0)
                .show_rows(ui, 8.0, client.server.channel.len(), |ui, range| {
                    for i in range {
                        if ui
                            .selectable_value(
                                &mut self.index_channel,
                                i as u64,
                                &format!("#{}", client.server.channel[&(i as u64)].name),
                            )
                            .clicked()
                        {
                            client.send(CommandClient::AccountChannel(i as ChannelID));
                        };
                    }
                });

            //ui.button("New Channel");

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
                        if Self::button_image(
                            ui,
                            egui::include_image!("../asset/file.svg"),
                            Vec2::new(32.0, 32.0),
                        )
                        .clicked()
                        {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Upload a file.")
                                .pick_file()
                                && let Ok(file) = std::fs::read(path.clone())
                            {
                                /*
                                client.send_file(self.index_channel,
                                    &path
                                        .file_name()
                                        .map(|x| x.display().to_string())
                                        .unwrap_or("file".to_string()),
                                    file,
                                );
                                */
                            }
                        };

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
