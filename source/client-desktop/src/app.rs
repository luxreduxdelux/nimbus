use chrono::{DateTime, Local, Utc};
use eframe::egui::{
    self, Color32, ColorImage, FontId, ImageSource, Response, RichText, Spinner, TextureHandle,
    Vec2,
};
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
    state: String,
    image: HashMap<u64, TextureHandle>,
    image_icon_main: Option<TextureHandle>,
    image_icon_side: Option<TextureHandle>,
    image_identifier: TextureHandle,
    modal_poll: Option<Poll>,
    modal: Option<fn(&mut Self, &mut egui::Ui)>,
    exit: bool,
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
            system: System::new(&user, &ui),
            client: None,
            user,
            index_channel: 0,
            setup: Default::default(),
            entry: Default::default(),
            state: Default::default(),
            image: Default::default(),
            image_icon_main,
            image_icon_side,
            image_identifier,
            modal_poll: Default::default(),
            modal: Default::default(),
            exit: Default::default(),
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

        if let Some(modal) = &mut self.modal {
            modal(self, ui);
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
                if Self::button_image(
                    ui,
                    egui::include_image!("../asset/back.svg"),
                    Vec2::new(24.0, 24.0),
                )
                .clicked()
                {
                    self.modal = None;
                };

                ui.selectable_value(&mut self.setup, Setup::Account, "Account");
                ui.selectable_value(&mut self.setup, Setup::Window, "Window");
                ui.selectable_value(&mut self.setup, Setup::Notify, "Notify");
                ui.selectable_value(&mut self.setup, Setup::Input, "Input");
            });

            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| match &self.setup {
                Setup::Account => {
                    ui.horizontal(|ui| {
                        ui.heading("Identifier");

                        if ui.button("?").clicked() {}
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

                    // show identifier QR code

                    //================

                    // create identifier

                    //================

                    // import identifier

                    //================

                    ui.button("Export Identifier");

                    ui.heading("Persona");
                    ui.separator();

                    ui.label("User Icon (Main/Side)");
                    ui.horizontal(|ui| {
                        if let Some(icon) = &self.image_icon_main {
                            ui.add(egui::Image::new(icon).fit_to_exact_size(Vec2::new(64.0, 64.0)));
                        } else {
                            ui.add(
                                egui::Image::new(egui::include_image!("../asset/user.svg"))
                                    .fit_to_exact_size(Vec2::new(64.0, 64.0)),
                            );
                        };

                        if let Some(icon) = &self.image_icon_side {
                            ui.add(
                                egui::Image::new(icon).fit_to_exact_size(Vec2::new(256.0, 64.0)),
                            );
                        } else {
                            ui.add(
                                egui::Image::new(egui::include_image!("../asset/user.svg"))
                                    .fit_to_exact_size(Vec2::new(256.0, 64.0)),
                            );
                        };
                    });

                    ui.horizontal(|ui| {
                        if ui.button("Set Main Icon").clicked()
                            && let Some(file) = rfd::FileDialog::new().pick_file()
                        {
                            self.user.icon_main = Some(std::fs::read(file).unwrap());
                        }
                        if ui.button("Set Side Icon").clicked()
                            && let Some(file) = rfd::FileDialog::new().pick_file()
                        {
                            self.user.icon_side = Some(std::fs::read(file).unwrap());
                        }
                    });

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

    fn draw_account(
        ui: &mut egui::Ui,
        client: Option<&mut Client>,
        account: &Account,
    ) -> (bool, bool) {
        let mut modal_state = false;
        let mut modal_setup = false;

        ui.horizontal(|ui| {
            if let Some(client) = &client {
                let response = Self::button_image(
                    ui,
                    ImageSource::Uri("file:///home/lux/Desktop/deer.png".to_string().into()),
                    Vec2::new(32.0, 32.0),
                );

                Self::pop_up("status", response, ui, |ui| {
                    let mut state = client.server.account[&client.index].presence.clone();
                    let mut click = false;
                    let state_list = [
                        (AccountPresence::Online, "Online"),
                        (AccountPresence::Away, "Away"),
                        (AccountPresence::Busy, "Busy"),
                        (AccountPresence::Offline, "Offline"),
                    ];

                    for (s, n) in state_list {
                        if ui.selectable_value(&mut state, s, n).clicked() {
                            click = true;
                        }
                    }

                    ui.separator();

                    if ui.button("Set State").clicked() {
                        modal_state = true;
                    }

                    if click {
                        client.send(CommandClient::AccountPresence(state));
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

            let color = match account.presence {
                AccountPresence::Online => Color32::GREEN,
                AccountPresence::Away => Color32::ORANGE,
                AccountPresence::Busy => Color32::RED,
                AccountPresence::Offline => Color32::DARK_GRAY,
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
                {
                    modal_setup = true;
                };
            }
        });

        (modal_state, modal_setup)
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
        self.draw_chat_l(ui);
        self.draw_chat_r(ui);
        self.draw_chat_c(ui);
    }

    fn draw_chat_l(&mut self, ui: &mut egui::Ui) {
        let client = self.client.as_mut().unwrap();

        egui::Panel::left("left")
            .resizable(false)
            .show_inside(ui, |ui| {
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
                        let (state, setup) = Self::draw_account(ui, Some(client), &account);

                        if state {
                            self.modal = Some(|this, ui| {
                                egui::Modal::new("state".into()).show(ui.ctx(), |ui| {
                                    ui.heading("Set State");
                                    ui.separator();

                                    ui.label("State");
                                    ui.text_edit_singleline(&mut this.state);

                                    ui.separator();
                                    ui.horizontal(|ui| {
                                        if ui.button("Accept").clicked() {
                                            this.client.as_mut().unwrap().send(
                                                CommandClient::AccountState(Some(
                                                    this.state.clone(),
                                                )),
                                            );
                                            this.modal = None;
                                            this.state.clear();
                                        }
                                        if ui.button("Cancel").clicked() {
                                            this.modal = None;
                                            this.state.clear();
                                        }
                                    });
                                });
                            });
                        }

                        if setup {
                            self.modal = Some(|this, ui| {
                                egui::Modal::new("setup".into()).show(ui.ctx(), |ui| {
                                    // TO-DO make this proportional to window
                                    ui.set_min_height(512.0);
                                    this.draw_setup(ui);
                                });
                            });
                        }
                    });
            });
    }

    fn draw_chat_r(&mut self, ui: &mut egui::Ui) {
        let client = self.client.as_mut().unwrap();

        egui::Panel::right("right")
            .resizable(false)
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(6.0);
                        for (_, account) in &client.server.account {
                            Self::draw_account(ui, None, account);
                        }
                    });
            });
    }

    fn draw_chat_c(&mut self, ui: &mut egui::Ui) {
        let client = self.client.as_mut().unwrap();

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
                    let mut previous = None;

                    for (index, message) in &channel.message {
                        let draw_avatar = if let Some(previous) = previous {
                            std::mem::discriminant(&message.kind) != previous
                        } else {
                            true
                        };

                        let response = ui.horizontal(|ui| {
                            if draw_avatar {
                                let image = egui::Image::new(ImageSource::Uri(
                                    "file:///home/lux/Desktop/deer.png".to_string().into(),
                                ))
                                .fit_to_exact_size(Vec2::new(32.0, 32.0))
                                .corner_radius(32.0);

                                ui.add(image);
                            }

                            ui.horizontal(|ui| {
                                if !draw_avatar {
                                    ui.add_space(40.0);
                                }

                                egui::Frame::group(ui.style())
                                    .corner_radius(8.0)
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            if draw_avatar {
                                                let from = message.account(&client.server);

                                                ui.horizontal(|ui| {
                                                    ui.label(RichText::new(&from.name_nick).strong());
                                                    ui.label(RichText::new("9:41").weak());
                                                });
                                            }

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
                                                MessageKind::Poll(poll) => {
                                                    ui.label(RichText::new(&poll.name).strong());
                                                    ui.separator();

                                                    let mut add = 0;

                                                    for choice in &poll.choice {
                                                        add += choice.vote.len();
                                                    }

                                                    for (index_choice, choice) in poll.choice.iter().enumerate() {
                                                        let progress = if add > 0 {
                                                            (choice.vote.len() as f32 / add as f32)
                                                        } else {
                                                            0.0
                                                        };

                                                        ui.label(&choice.name);

                                                        ui.horizontal(|ui| {
                                                            if ui.checkbox(&mut choice.vote.contains(&client.index), "").clicked() {
                                                                client.send(CommandClient::PollVote(self.index_channel, *index, index_choice));
                                                            };
                                                            ui.add(egui::ProgressBar::new(progress).text(format!("{:.0}% - {}", progress * 100.0, choice.vote.len().to_string()))).on_hover_ui(|ui| {
                                                                for account in &choice.vote {
                                                                    let account = &client.server.account[account];
                                                                    // TO-DO somehow broken?
                                                                    ui.label(&account.name_nick);
                                                                }
                                                            });
                                                        });
                                                    }
                                                }
                                                MessageKind::Sticker(sticker) => {
                                                    if let Some(image) = self.image.get(&sticker) {
                                                        ui.image(image);
                                                    }
                                                }
                                            }
                                        });
                                });
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

                        previous = Some(std::mem::discriminant(&message.kind));
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
                                self.modal_poll = Some(Poll::default());
                            };
                        });

                        if let Some(mut poll) = self.modal_poll.clone()  {
                            egui::Modal::new("blah".into()).show(ui.ctx(), |ui| {
                                ui.heading("Submit Poll");
                                ui.separator();

                                ui.label("Poll Name");
                                ui.text_edit_singleline(&mut poll.name);

                                // TO-DO add remove choice
                                for (i, choice) in &mut poll.choice.iter_mut().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("Choice #{}", i + 1));
                                        if let Some(correct) = poll.correct {
                                            let mut pick_a = correct == i;
                                            let mut pick_b = correct == i;

                                            ui.checkbox(&mut pick_a, "Correct answer");

                                            if pick_a && !pick_b {
                                                poll.correct = Some(i);
                                            }
                                        }
                                    });
                                    ui.text_edit_singleline(&mut choice.name);
                                }

                                if ui.button("Add Choice").clicked() {
                                    poll.choice.push(PollChoice::default());
                                }

                                ui.checkbox(&mut poll.hidden, "Hide voter name");
                                ui.checkbox(&mut poll.single, "Allow more than one choice");
                                ui.checkbox(&mut poll.attach, "Allow adding a new choice");
                                ui.checkbox(&mut poll.revoke, "Allow re-voting");

                                let mut correct_a = poll.correct.is_some();
                                let mut correct_b = poll.correct.is_some();

                                ui.checkbox(&mut correct_a, "Set correct choice");

                                if correct_a && !correct_b {
                                    poll.correct = Some(0);
                                }
                                if !correct_a && correct_b {
                                    poll.correct = None;
                                }

                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.add_enabled_ui(poll.is_valid().is_ok(), |ui| {
                                        if ui.button("Submit").clicked() {
                                            client.send(CommandClient::Message(self.index_channel, MessageKind::Poll(poll.clone())));
                                            self.modal_poll = None;
                                        }
                                    });

                                    if ui.button("Close").clicked() {
                                        self.modal_poll = None;
                                    }
                                });
                            });

                            if self.modal_poll.is_some() {
                                self.modal_poll = Some(poll);
                            }
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


                        });
                        Self::pop_up("plus", Self::button_image(
                            ui,
                            egui::include_image!("../asset/emote.svg"),
                            Vec2::new(32.0, 32.0),
                        ), ui, |ui| {
                            ui.vertical(|ui| {
                                for y in 0..8 {
                                    ui.horizontal(|ui| {
                                        for x in 0..8 {
                                            if ui.button("😊").clicked() {
                                                self.entry.push('😊');
                                            }
                                        }
                                    });
                                }
                            });
                        });

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
