use std::collections::HashMap;

use eframe::egui::{
    self, Color32, ColorImage, FontId, ImageSource, Response, RichText, Spinner, TextureHandle,
    Vec2,
};

//================================================================

use crate::user::*;
use client::common::prelude::*;
use client::*;

//================================================================

#[derive(Default)]
pub struct App {
    user: User,
    client: Option<Client>,
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

        if let Some(client) = &mut self.client {
            client.update(|command| match command {
                CommandServer::Enter(server) => {
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

    fn draw_main(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.5 - 128.0);

                ui.label(
                    RichText::new("Welcome to Nimbus!")
                        .font(FontId::proportional(32.0))
                        .strong(),
                );

                let icon = if let Some(icon) = &self.user.icon {
                    egui::Image::new(ImageSource::Uri(format!("file://{icon}").into()))
                        .fit_to_exact_size(Vec2::new(96.0, 96.0))
                } else {
                    egui::Image::new(egui::include_image!("../asset/user.svg"))
                        .fit_to_exact_size(Vec2::new(96.0, 96.0))
                };

                ui.label("Icon");
                if ui.button(icon).clicked()
                    && let Some(icon) = rfd::FileDialog::new()
                        .set_title("Select an user icon.")
                        .pick_file()
                {
                    self.user.icon = Some(icon.display().to_string());
                };

                ui.label("Name");
                ui.text_edit_singleline(&mut self.user.name);

                ui.label("Server Address");
                ui.text_edit_singleline(&mut self.user.address);

                if ui.button("Log In").clicked() {
                    self.client = Some(Client::new(
                        self.user.address.to_string(),
                        self.user.clone().into(),
                    ));
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

                let icon = if let Some(icon) = &self.user.icon {
                    //println!("{icon}");
                    egui::Image::new(ImageSource::Uri(format!("file://{icon}").into()))
                        .fit_to_exact_size(Vec2::new(96.0, 96.0))
                } else {
                    egui::Image::new(egui::include_image!("../asset/user.svg"))
                        .fit_to_exact_size(Vec2::new(96.0, 96.0))
                };

                ui.add(icon);

                ui.label(&self.user.name);

                ui.add(Spinner::new().size(32.0));

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
                    Vec2::new(40.0, 40.0),
                );

                if response.clicked() {
                    egui::Popup::open_id(ui.ctx(), id);
                }

                #[rustfmt::skip]
                egui::Popup::from_toggle_button_response(&response).show(|ui| {
                    if ui.button("Online").clicked()         { client.set_state(AccountState::Online);       };
                    if ui.button("Away").clicked()           { client.set_state(AccountState::Away);         };
                    if ui.button("Do Not Disturb").clicked() { client.set_state(AccountState::DoNotDisturb); };
                    if ui.button("Offline").clicked()        { client.set_state(AccountState::Offline);      };
                });
            } else {
                ui.add(
                    egui::Image::new(ImageSource::Uri(
                        "file:///home/lux/Desktop/deer.png".to_string().into(),
                    ))
                    .fit_to_exact_size(Vec2::new(40.0, 40.0))
                    .corner_radius(40.0),
                );
            }

            let point = ui
                .min_rect()
                .translate(Vec2::new(14.0, 14.0))
                .scale_from_center(0.25);

            let color = match account.state {
                AccountState::Online => Color32::GREEN,
                AccountState::Away => Color32::ORANGE,
                AccountState::DoNotDisturb => Color32::RED,
                AccountState::Offline => Color32::DARK_GRAY,
            };

            egui::Image::new(egui::include_image!("../asset/dot.svg"))
                .tint(color)
                .paint_at(ui, point);

            ui.label(RichText::new(&account.name).strong());

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
            ui.horizontal(|ui| {
                /*
                egui::ScrollArea::vertical()
                    .id_salt("server")
                    .auto_shrink([true, true])
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            for x in 0..32 {
                                Self::button_image(
                                    ui,
                                    ImageSource::Uri(
                                        "file:///home/lux/Desktop/deer.png".to_string().into(),
                                    ),
                                    Vec2::new(40.0, 40.0),
                                );
                            }
                        });
                    });

                ui.separator();
                */

                egui::ScrollArea::vertical()
                    .id_salt("channel")
                    .auto_shrink([true, true])
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            for (_, channel) in &client.server.channel {
                                ui.button(&format!("#{}", channel.name));
                            }
                        });
                    });
            });

            ui.separator();

            /*
            // TO-DO use index from server for us
            let account = client
                .server
                .account
                .iter()
                .find(|x| x.name == self.user.name)
                .cloned()
                .unwrap();

            Self::draw_account(ui, Some(client), &account);
            */
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
            ui.label("channel");

            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .max_height(ui.available_height() - 56.0)
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

                    let channel = client.server.channel.get(&0).unwrap();

                    for (index, message) in &channel.message {
                        ui.horizontal(|ui| {
                            let image = egui::Image::new(ImageSource::Uri(
                                "file:///home/lux/Desktop/deer.png".to_string().into(),
                            ))
                            .fit_to_exact_size(Vec2::new(40.0, 40.0))
                            .corner_radius(40.0);

                            ui.add(image);

                            ui.vertical(|ui| {
                                match &message.kind {
                                    //
                                    MessageKind::Text(text) => {
                                        ui.label(RichText::new(&message.from).strong());
                                        ui.label(text);
                                    }
                                    MessageKind::File(name, data) => {
                                        ui.label(RichText::new(&message.from).strong());
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
                                        ui.label(RichText::new(&message.from).strong());

                                        if let Some(image) = self.image.get(&sticker) {
                                            ui.image(image);
                                        }
                                    }
                                }

                                #[rustfmt::skip]
                                ui.horizontal(|ui| {
                                    Self::button_image(ui, egui::include_image!("../asset/reply.svg"),  Vec2::new(24.0, 24.0));
                                    Self::button_image(ui, egui::include_image!("../asset/emote.svg"),  Vec2::new(24.0, 24.0));
                                    Self::button_image(ui, egui::include_image!("../asset/copy.svg"),   Vec2::new(24.0, 24.0));
                                    Self::button_image(ui, egui::include_image!("../asset/edit.svg"),   Vec2::new(24.0, 24.0));
                                    Self::button_image(ui, egui::include_image!("../asset/delete.svg"), Vec2::new(24.0, 24.0));
                                });
                            });
                        });

                        ui.add_space(4.0);
                    }
                });

            egui::Area::new("floating_text".into())
                .fixed_pos([ui.cursor().min.x, ui.viewport_rect().max.y - 32.0])
                .show(ui, |ui| {
                    let mut write = Vec::new();

                    for (_, account) in &client.server.account {
                        if account.write {
                            write.push(account.name.clone());
                        }
                    }

                    if !write.is_empty() {
                        egui::Frame::new()
                            .fill(egui::Color32::from_black_alpha(224))
                            .corner_radius(egui::CornerRadius::same(4))
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                if write.len() == 1 {
                                    ui.label(format!("{} is typing...", write[0]));
                                } else if write.len() == 2 {
                                    ui.label(format!("{} and {} are typing...", write[0], write[1]));
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
                                client.send_file(
                                    &path
                                        .file_name()
                                        .map(|x| x.display().to_string())
                                        .unwrap_or("file".to_string()),
                                    file,
                                );
                            }
                        };

                        // TO-DO
                        let empty = self.entry.is_empty();
                        let text = ui.add_sized(
                            [(ui.available_width() - 144.0).max(0.0), 0.0],
                            egui::TextEdit::singleline(&mut self.entry).id("client_buffer".into()),
                        );

                        if text.changed() {
                            if empty && !self.entry.is_empty() {
                                client.set_write(true);
                            } else if !empty && self.entry.is_empty() {
                                client.set_write(false);
                            }
                        }

                        if text.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            client.send_text(&self.entry);
                            client.set_write(false);
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
                                    client.send_sticker(*index);
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
                            // TO-DO
                            client.send_text(&self.entry);
                            client.set_write(false);
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
