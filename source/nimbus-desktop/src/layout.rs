use egui::RichText;
use egui::{
    self, Color32, ColorImage, FontFamily, FontId, ImageSource, IntoAtoms, Response, TextStyle,
    TextureHandle, Vec2,
};
use rust_i18n::t;

//================================================================

use crate::app::*;
use crate::user::*;
use nimbus_client::prelude::*;

//================================================================

#[derive(Default, Copy, Clone, PartialEq, Eq)]
enum SetupIndex {
    #[default]
    Account,
    Window,
    Notify,
    Input,
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
enum FriendIndex {
    #[default]
    Online,
    All,
}

#[derive(Default, Clone)]
pub struct Layout {
    modal: Option<fn(&mut App, &mut egui::Ui)>,
    modal_address: String,
    index_server: Option<usize>,
    index_channel: Option<usize>,
    index_account: Option<usize>,
    index_setup: SetupIndex,
    index_friend: FriendIndex,
    setup_user: User,
    entry_text: String,
    entry_reply: Option<MessageID>,
    panel_user: bool,
    panel_server: bool,
}

impl Layout {
    const IMAGE_SEARCH: ImageSource<'_> = egui::include_image!("../asset/search.svg");
    const IMAGE_STAR_MESSAGE: ImageSource<'_> = egui::include_image!("../asset/star_message.svg");
    const IMAGE_COG: ImageSource<'_> = egui::include_image!("../asset/cog.svg");
    const IMAGE_REPLY: ImageSource<'_> = egui::include_image!("../asset/reply.svg");
    const IMAGE_EMOTE: ImageSource<'_> = egui::include_image!("../asset/emote.svg");
    const IMAGE_COPY: ImageSource<'_> = egui::include_image!("../asset/copy.svg");
    const IMAGE_EDIT: ImageSource<'_> = egui::include_image!("../asset/edit.svg");
    const IMAGE_STAR_A: ImageSource<'_> = egui::include_image!("../asset/star_a.svg");
    const IMAGE_DELETE: ImageSource<'_> = egui::include_image!("../asset/delete.svg");
    const IMAGE_STICKER: ImageSource<'_> = egui::include_image!("../asset/sticker.svg");
    const IMAGE_SEND: ImageSource<'_> = egui::include_image!("../asset/send.svg");
    const IMAGE_BACK: ImageSource<'_> = egui::include_image!("../asset/back.svg");
    const IMAGE_USER: ImageSource<'_> = egui::include_image!("../asset/user.svg");
    const IMAGE_USER_SIDE: ImageSource<'_> = egui::include_image!("../asset/user_side.svg");
    const IMAGE_USER_FRIEND: ImageSource<'_> = egui::include_image!("../asset/user_friend.svg");
    const IMAGE_USER_ADD: ImageSource<'_> = egui::include_image!("../asset/user_add.svg");
    const IMAGE_USER_CODE: ImageSource<'_> = egui::include_image!("../asset/user_code.svg");
    const IMAGE_USER_ONLINE: ImageSource<'_> = egui::include_image!("../asset/user_online.svg");
    const IMAGE_WINDOW: ImageSource<'_> = egui::include_image!("../asset/window.svg");
    const IMAGE_NOTIFY: ImageSource<'_> = egui::include_image!("../asset/notify.svg");
    const IMAGE_INPUT: ImageSource<'_> = egui::include_image!("../asset/input.svg");
    const IMAGE_APPLY: ImageSource<'_> = egui::include_image!("../asset/apply.svg");
    const IMAGE_RESET: ImageSource<'_> = egui::include_image!("../asset/reset.svg");
    const IMAGE_CLOSE: ImageSource<'_> = egui::include_image!("../asset/close.svg");
    const IMAGE_ERROR: ImageSource<'_> = egui::include_image!("../asset/error.svg");
    const IMAGE_ENTER: ImageSource<'_> = egui::include_image!("../asset/enter.svg");
    const IMAGE_LEAVE: ImageSource<'_> = egui::include_image!("../asset/leave.svg");
    const IMAGE_LOGO: ImageSource<'_> = egui::include_image!("../asset/logo.svg");
    const IMAGE_DOT: ImageSource<'_> = egui::include_image!("../asset/dot.svg");
    const IMAGE_PLUS: ImageSource<'_> = egui::include_image!("../asset/plus.svg");
    const IMAGE_TEST: ImageSource<'_> = egui::include_image!("../asset/test.png");
    const BUTTON_IMAGE_SCALE: Vec2 = Vec2::new(40.0, 40.0);
    const PANEL_L_SIZE: f32 = 320.0; //376.0;
    const PANEL_R_SIZE: f32 = 256.0; //264.0;

    //================================================================

    pub fn draw(app: &mut App, ui: &mut egui::Ui) {
        let height = ui.available_height() - 66.0;
        let size = if app.layout.panel_server {
            Self::PANEL_L_SIZE
        } else {
            68.0
        };

        egui::Panel::left("panel_l")
            .min_size(size)
            .max_size(size)
            .resizable(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_min_height(height);
                        ui.add_space(4.0);
                        Self::draw_picker_l(app, ui);
                    });

                    if app.layout.panel_server {
                        //ui.separator();

                        ui.vertical(|ui| {
                            ui.set_min_height(height);
                            ui.add_space(4.0);
                            if app.layout.index_server.is_none() {
                                Self::draw_picker_r_account(app, ui);
                            } else {
                                Self::draw_picker_r_channel(app, ui);
                            }
                        });
                    }
                });

                ui.separator();

                let (account, cog) = Self::draw_account_self(app, ui);

                if cog {
                    app.layout.modal = Some(Self::modal_setup);
                }
            });

        if app.layout.index_server.is_some() && app.layout.panel_user {
            egui::Panel::right("panel_r")
                .min_size(Self::PANEL_R_SIZE)
                .max_size(Self::PANEL_R_SIZE)
                .resizable(false)
                .show(ui, |ui| {
                    ui.label("foo");
                });
        }

        egui::CentralPanel::default().show(ui, |ui| {
            if app.layout.index_server.is_none() {
                if app.layout.index_account.is_none() {
                    Self::draw_friend_list(app, ui);
                } else {
                    Self::draw_chat_account(app, ui);
                }
            } else {
                Self::draw_chat_channel(app, ui);
            }
        });

        if let Some(modal) = &mut app.layout.modal {
            modal(app, ui);
        }
    }

    fn draw_image_label(ui: &mut egui::Ui, image: egui::Image, label: &str) {
        ui.horizontal(|ui| {
            ui.add(image);
            ui.label(label);
        });
    }

    fn draw_message_system(ui: &mut egui::Ui, message: &MessageSystem, server: &Server) {
        match message {
            MessageSystem::Enter(account) => {
                if let Some(account) = server.account.get(account) {
                    Self::draw_image_label(
                        ui,
                        Self::image(Self::IMAGE_ENTER, Vec2::splat(32.0)).tint(Color32::GREEN),
                        &account.name_nick,
                    );
                }
            }
            MessageSystem::Leave(account) => {
                if let Some(account) = server.account.get(account) {
                    Self::draw_image_label(
                        ui,
                        Self::image(Self::IMAGE_LEAVE, Vec2::splat(32.0)).tint(Color32::RED),
                        &account.name_nick,
                    );
                }
            }
            MessageSystem::Star(_) => {
                Self::draw_image_label(
                    ui,
                    Self::image(Self::IMAGE_STAR_A, Vec2::splat(32.0)),
                    "TO-DO",
                );
            }
        }
    }

    fn draw_message_in_line(ui: &mut egui::Ui, message: &Message, server: &Server) {
        match &message.kind {
            MessageKind::System(message) => Self::draw_message_system(ui, message, server),
            MessageKind::Text(text) => {
                ui.label(RichText::new(text).italics());
            }
            MessageKind::File(_, _) => {
                ui.label(RichText::new("[File]").italics());
            }
            MessageKind::Poll(_) => {
                ui.label(RichText::new("[Poll]").italics());
            }
            MessageKind::Sticker(_) => {
                ui.label(RichText::new("[Sticker]").italics());
            }
        }
    }

    fn draw_message_reply(
        ui: &mut egui::Ui,
        channel: ChannelID,
        message: MessageID,
        server: &Server,
    ) {
        if let Some(channel) = server.channel.get(&channel) {
            if let Some(message) = channel.message.get(&message)
                && let Some(account) = message.account(server)
            {
                ui.label(RichText::new(&account.name_nick).weak());
                Self::draw_message_in_line(ui, message, server);
            } else {
                ui.label(RichText::new(t!("message.error")).italics());
            }
        }
    }

    fn draw_chat_channel(app: &mut App, ui: &mut egui::Ui) {
        if let Some(channel_index) = app.layout.index_channel {
            let client = &app.client.client[app.layout.index_server.unwrap()];
            let channel = &client.server.channel[&(channel_index as u64)];

            let width = (ui.available_width() - (Self::BUTTON_IMAGE_SCALE.x + 14.0) * 4.0).max(0.0);

            ui.horizontal(|ui| {
                Self::allocate_size(ui, width, |ui| {
                    ui.vertical(|ui| {
                        ui.add(egui::Label::new(RichText::new(&channel.name).strong()).truncate());
                        ui.add(egui::Label::new(&channel.info).truncate());
                    });
                });

                ui.add_space(
                    (ui.available_width() - (Self::BUTTON_IMAGE_SCALE.x + 14.0) * 4.0).max(0.0),
                );

                Self::draw_button_image(ui, Self::IMAGE_COG);
                Self::draw_button_image(ui, Self::IMAGE_SEARCH);
                Self::draw_button_image(ui, Self::IMAGE_STAR_MESSAGE);

                let mut panel = app.layout.panel_user;

                if Self::draw_selectable(ui, &mut panel, true, Self::IMAGE_USER_SIDE, "").clicked()
                {
                    app.layout.panel_user = !app.layout.panel_user;
                }
            });

            ui.separator();

            let (_, which, index) = Token::parse(&app.layout.entry_text);
            let height = if which == TokenKind::Account
                || which == TokenKind::Channel
                || app.layout.entry_reply.is_some()
            {
                137.0
            } else {
                77.0
            };

            let height = ui.available_height() - height;

            egui::ScrollArea::vertical()
                .id_salt("scroll_chat")
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .max_height(height)
                .show(ui, |ui| {
                    for (i, message) in &channel.message {
                        let message_system = message.account.is_none();

                        ui.horizontal(|ui| {
                            if !message_system {
                                ui.add(
                                    egui::Image::new(Self::IMAGE_TEST)
                                        .fit_to_exact_size(Self::BUTTON_IMAGE_SCALE),
                                );
                            }

                            ui.vertical(|ui| {
                                let response = egui::Frame::group(ui.style()).show(ui, |ui| {
                                    let available = ui.available_size();

                                    let (rect, response) =
                                        ui.allocate_exact_size(available, egui::Sense::click());

                                    ui.allocate_ui_at_rect(rect, |ui| {
                                        if let Some(account) = message.account(&client.server) {
                                            ui.label(RichText::new(&account.name_nick).strong());
                                        }

                                        if let Some(reply) = message.reply {
                                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                                Self::draw_message_reply(
                                                    ui,
                                                    channel_index as ChannelID,
                                                    reply,
                                                    &client.server,
                                                );
                                            });
                                        }

                                        match &message.kind {
                                            MessageKind::System(message) => {
                                                Self::draw_message_system(
                                                    ui,
                                                    message,
                                                    &client.server,
                                                );
                                            }
                                            MessageKind::Text(text) => {
                                                ui.label(text);
                                            }
                                            _ => {}
                                        };
                                    });

                                    response
                                });

                                response.inner.context_menu(|ui| {
                                    if Self::draw_button_image_label(
                                        ui,
                                        Self::IMAGE_REPLY,
                                        &t!("message.reply"),
                                    )
                                    .clicked()
                                    {
                                        app.layout.entry_reply = Some(*i);
                                    }
                                    Self::draw_button_image_label(
                                        ui,
                                        Self::IMAGE_EMOTE,
                                        &t!("message.emote"),
                                    );
                                    Self::draw_button_image_label(
                                        ui,
                                        Self::IMAGE_COPY,
                                        &t!("message.copy"),
                                    );
                                    Self::draw_button_image_label(
                                        ui,
                                        Self::IMAGE_EDIT,
                                        &t!("message.edit"),
                                    );
                                    Self::draw_button_image_label(
                                        ui,
                                        Self::IMAGE_STAR_A,
                                        &t!("message.star"),
                                    );
                                    if Self::draw_button_image_label(
                                        ui,
                                        Self::IMAGE_DELETE,
                                        &t!("message.delete"),
                                    )
                                    .clicked()
                                    {
                                        client.send(CommandClient::MessageDelete(
                                            channel_index as ChannelID,
                                            *i,
                                        ));
                                    };
                                })
                            });
                        });
                    }
                });

            let mut entry_focus = false;

            if which == TokenKind::Account || which == TokenKind::Channel {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("scroll_chat_side")
                        .auto_shrink([false, false])
                        .max_height(64.0)
                        .show(ui, |ui| {
                            if which == TokenKind::Account {
                                for (_, account) in &client.server.account {
                                    if ui.button(&account.name_user).clicked() {
                                        app.layout.entry_text.truncate(index);
                                        app.layout.entry_text.push('@');
                                        app.layout.entry_text.push_str(&account.name_user);
                                        app.layout.entry_text.push(' ');
                                        entry_focus = true;
                                    }
                                }
                            } else {
                                for (_, channel) in &client.server.channel {
                                    if ui.button(&channel.name).clicked() {
                                        app.layout.entry_text.truncate(index);
                                        app.layout.entry_text.push('#');
                                        app.layout.entry_text.push_str(&channel.name);
                                        app.layout.entry_text.push(' ');
                                        entry_focus = true;
                                    }
                                }
                            }
                        });
                });
            } else if let Some(reply) = app.layout.entry_reply {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("scroll_chat_side")
                        .auto_shrink([false, false])
                        .max_height(64.0)
                        .show(ui, |ui| {
                            Self::draw_message_reply(
                                ui,
                                channel_index as ChannelID,
                                reply,
                                &client.server,
                            );
                        });
                });
            } else {
                ui.label("");
            };

            ui.separator();

            ui.horizontal(|ui| {
                Self::draw_button_image(ui, Self::IMAGE_PLUS);

                let response = ui.add_sized(
                    [
                        (ui.available_width() - (Self::BUTTON_IMAGE_SCALE.x + 16.0) * 3.0).max(0.0),
                        Self::BUTTON_IMAGE_SCALE.y,
                    ],
                    egui::TextEdit::singleline(&mut app.layout.entry_text)
                        .id("text_edit".into())
                        .font(FontId::proportional(Self::BUTTON_IMAGE_SCALE.y * 0.5))
                        .vertical_align(egui::Align::Center),
                );

                if entry_focus {
                    response.request_focus();
                }

                Self::draw_button_image(ui, Self::IMAGE_STICKER);
                Self::draw_button_image(ui, Self::IMAGE_EMOTE);

                if Self::draw_button_image(ui, Self::IMAGE_SEND).clicked() {
                    if let Some(message_index) = app.layout.entry_reply {
                        client.send(CommandClient::MessageReply(
                            channel_index as ChannelID,
                            message_index,
                            MessageKind::Text(app.layout.entry_text.clone()),
                        ));
                    } else {
                        client.send(CommandClient::Message(
                            channel_index as ChannelID,
                            MessageKind::Text(app.layout.entry_text.clone()),
                        ));
                    }

                    app.layout.entry_reply = None;
                    app.layout.entry_text.clear();
                }
            });
        }
    }

    fn draw_chat_account(app: &mut App, ui: &mut egui::Ui) {
        ui.label("Account");
        ui.separator();

        egui::ScrollArea::vertical()
            .id_salt("scroll_chat")
            .show(ui, |ui| {
                for x in 0..32 {
                    ui.label(format!("{x}"));
                }
            });
    }

    fn draw_picker_l(app: &mut App, ui: &mut egui::Ui) {
        let response =
            Self::draw_selectable(ui, &mut app.layout.index_server, None, Self::IMAGE_LOGO, "");

        if response.middle_clicked() && app.layout.index_server.is_none() {
            app.layout.panel_server = !app.layout.panel_server;
        }

        ui.add_sized(
            [Self::BUTTON_IMAGE_SCALE.x + 8.0, 0.0],
            egui::Separator::default(),
        );

        egui::ScrollArea::vertical()
            .id_salt("scroll_picker_l")
            .show(ui, |ui| {
                for (i, client) in app.client.client.iter().enumerate() {
                    if client.ready {
                        let response = Self::draw_selectable(
                            ui,
                            &mut app.layout.index_server,
                            Some(i),
                            Self::IMAGE_LOGO,
                            "",
                        );

                        if response.clicked() {
                            app.layout.index_server = Some(i);
                            app.layout.panel_server = true;
                        }

                        if response.middle_clicked()
                            && let Some(index) = app.layout.index_server
                            && index == i
                        {
                            app.layout.panel_server = !app.layout.panel_server;
                        }

                        response.on_hover_text(&client.server.name);
                    } else if let Some(error) = &client.error {
                        ui.horizontal(|ui| {
                            ui.add_space(4.0);
                            ui.add(
                                egui::Image::new(Self::IMAGE_ERROR)
                                    .fit_to_exact_size(Self::BUTTON_IMAGE_SCALE),
                            )
                            .on_hover_text(t!("general.error"));
                            //.on_hover_text(format!("{}: {error}", t!("general.error")));
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.add_space(4.0);
                            ui.add(egui::Spinner::new().size(Self::BUTTON_IMAGE_SCALE.x))
                                .on_hover_text("Connecting...");
                        });
                    }

                    ui.add_space(2.0);
                }

                if Self::draw_button_image(ui, Self::IMAGE_PLUS).clicked() {
                    app.layout.modal = Some(Self::modal_server_join);
                }
            });
    }

    fn draw_picker_r_account(app: &mut App, ui: &mut egui::Ui) {
        let select = app.layout.index_account.is_none();
        let select = if select {
            Color32::DARK_BLUE
        } else {
            Color32::TRANSPARENT
        };

        Self::draw_selectable(
            ui,
            &mut app.layout.index_account,
            None,
            Self::IMAGE_USER_FRIEND,
            &t!("friend.list"),
        );

        ui.separator();

        egui::ScrollArea::vertical()
            .id_salt("scroll_picker_r")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for x in 0..32 {
                    let select = if let Some(i) = app.layout.index_account {
                        x == i
                    } else {
                        false
                    };
                    let select = if select {
                        Color32::DARK_BLUE
                    } else {
                        Color32::TRANSPARENT
                    };

                    Self::draw_selectable(
                        ui,
                        &mut app.layout.index_account,
                        Some(x),
                        Self::IMAGE_USER,
                        &format!("{x}"),
                    );
                }
            });
    }

    fn allocate_size<F: FnMut(&mut egui::Ui)>(ui: &mut egui::Ui, width: f32, call: F) {
        ui.allocate_ui_with_layout(
            egui::vec2(width, ui.spacing().interact_size.y),
            egui::Layout::left_to_right(egui::Align::Center),
            call,
        );
    }

    fn draw_picker_r_channel(app: &mut App, ui: &mut egui::Ui) {
        let client = &app.client.client[app.layout.index_server.unwrap()];
        let width = (ui.available_width() - Self::BUTTON_IMAGE_SCALE.x - 16.0).max(0.0);

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.add_space(Self::BUTTON_IMAGE_SCALE.y * 0.5 - 8.0);
                Self::allocate_size(ui, width, |ui| {
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Label::new(RichText::new(&client.server.name).strong())
                                .truncate(),
                        );
                    });
                });
            });

            ui.add_space(ui.available_width() - Self::BUTTON_IMAGE_SCALE.x - 8.0);

            if Self::draw_button_image(ui, Self::IMAGE_COG).clicked() {
                app.layout.modal = Some(Self::modal_server_setup);
            }
        });

        ui.separator();

        egui::ScrollArea::vertical()
            .id_salt("scroll_picker_r")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (i, channel) in &client.server.channel {
                    let select = if let Some(index) = app.layout.index_channel {
                        (index as u64) == *i
                    } else {
                        false
                    };

                    if ui
                        .add_sized(
                            [ui.available_width(), 0.0],
                            egui::Button::new(&channel.name).selected(select),
                        )
                        .clicked()
                    {
                        app.layout.index_channel = Some(*i as usize);
                    }
                }
            });
    }

    #[rustfmt::skip]
    fn draw_friend_list(app: &mut App, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            Self::draw_selectable(ui, &mut app.layout.index_friend, FriendIndex::Online, Self::IMAGE_USER_ONLINE, &t!("status.online"));
            Self::draw_selectable(ui, &mut app.layout.index_friend, FriendIndex::All,  Self::IMAGE_USER,          &t!("general.all"));
        });

        ui.separator();

        let h = ui.available_height() - 56.0;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(h)
            .show(ui, |ui| match app.layout.index_friend {
                FriendIndex::Online => {},
                FriendIndex::All    => {},
            });

        ui.separator();

        ui.horizontal(|ui| {
            if Self::draw_button_image_label(ui, Self::IMAGE_USER_ADD, &t!("friend.add")).clicked() {
            }
            if Self::draw_button_image_label(ui, Self::IMAGE_USER_CODE, &t!("friend.code")).clicked() {
            }
        });
    }

    fn draw_setup_account(app: &mut App, ui: &mut egui::Ui) {
        ui.label(t!("setup_account.user_icon"));

        if app.user.icon.is_some() {
            ui.add(egui::Image::new(Self::IMAGE_TEST).fit_to_exact_size([96.0, 96.0].into()));
            ui.horizontal(|ui| {
                ui.button(t!("setup_account.user_icon_apply"));
                ui.button(t!("setup_account.user_icon_clear"));
            });
        } else {
            ui.add(egui::Image::new(Self::IMAGE_TEST).fit_to_exact_size([96.0, 96.0].into()));
            ui.horizontal(|ui| {
                ui.button(t!("setup_account.user_icon_apply"));
            });
        }

        Self::draw_edit_mono(
            ui,
            &t!("setup_account.nick_name"),
            &mut app.layout.setup_user.name_nick,
        );
        Self::draw_edit_mono(
            ui,
            &t!("setup_account.user_name"),
            &mut app.layout.setup_user.name_user,
        );
        Self::draw_edit_multi(
            ui,
            &t!("setup_account.user_info"),
            &mut app.layout.setup_user.info,
        );
        ui.checkbox(
            &mut app.user.notify_push,
            t!("setup_account.indicator_read"),
        );
        ui.checkbox(
            &mut app.user.notify_push,
            t!("setup_account.indicator_type"),
        );
        ui.checkbox(
            &mut app.user.notify_push,
            t!("setup_account.indicator_seen"),
        );
        // TO-DO auto-delete picker
    }

    fn draw_setup_window(app: &mut App, ui: &mut egui::Ui) {
        ui.label(t!("setup_window.zoom"));
        let zoom = ui.add(egui::Slider::new(&mut app.user.zoom, 0.5..=2.0));

        if zoom.drag_stopped() {
            ui.set_zoom_factor(app.user.zoom);
        }

        ui.checkbox(&mut app.user.tray_show, t!("setup_window.tray"));
        ui.checkbox(&mut app.user.tray_show, t!("setup_window.embed_link"));
        ui.checkbox(&mut app.user.tray_show, t!("setup_window.embed_file"));
        ui.checkbox(&mut app.user.tray_show, t!("setup_window.show_hidden"));
        ui.checkbox(&mut app.user.tray_show, t!("setup_window.message_compact"));
        // TO-DO language picker
    }

    fn draw_setup_notify(app: &mut App, ui: &mut egui::Ui) {
        ui.checkbox(&mut app.user.notify_push, t!("setup_notify.push"));
        ui.checkbox(&mut app.user.notify_tray, t!("setup_notify.tray"));
        ui.checkbox(&mut app.user.notify_sound, t!("setup_notify.sound"));
    }

    fn draw_setup_input(app: &mut App, ui: &mut egui::Ui) {
        ui.label(t!("setup_input.audio_device"));
        ui.label(t!("setup_input.server_lower"));
        ui.label(t!("setup_input.server_upper"));
        ui.label(t!("setup_input.channel_lower"));
        ui.label(t!("setup_input.channel_upper"));
        ui.label(t!("setup_input.toggle_quick"));
        ui.label(t!("setup_input.edit_message"));
    }

    #[rustfmt::skip]
    fn modal_setup(app: &mut App, ui: &mut egui::Ui) {
        egui::Modal::new("modal_setup".into()).show(ui, |ui| {
            ui.set_min_size([1024.0 - 64.0, 768.0 - 64.0].into());

            ui.horizontal(|ui| {
                Self::draw_selectable(ui, &mut app.layout.index_setup, SetupIndex::Account, Self::IMAGE_USER,   "Account");
                Self::draw_selectable(ui, &mut app.layout.index_setup, SetupIndex::Window,  Self::IMAGE_WINDOW, "Window");
                Self::draw_selectable(ui, &mut app.layout.index_setup, SetupIndex::Notify,  Self::IMAGE_NOTIFY, "Notify");
                Self::draw_selectable(ui, &mut app.layout.index_setup, SetupIndex::Input,   Self::IMAGE_INPUT,  "Input");
            });

            ui.separator();

            let h = ui.available_height() - 56.0;

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(h)
                .show(ui, |ui| match app.layout.index_setup {
                    SetupIndex::Account => Self::draw_setup_account(app, ui),
                    SetupIndex::Window  => Self::draw_setup_window(app, ui),
                    SetupIndex::Notify  => Self::draw_setup_notify(app, ui),
                    SetupIndex::Input   => Self::draw_setup_input(app, ui),
                });

            ui.separator();

            ui.horizontal(|ui| {
                if Self::draw_button_image_label(ui, Self::IMAGE_APPLY, &t!("general.apply")).clicked() {
                    app.user = app.layout.setup_user.clone();
                    app.layout.modal = None;
                }
                if Self::draw_button_image_label(ui, Self::IMAGE_RESET, &t!("general.reset")).clicked() {
                    app.layout.setup_user = app.user.clone();
                }
                if Self::draw_button_image_label(ui, Self::IMAGE_CLOSE, &t!("general.close")).clicked() {
                    app.layout.modal = None;
                }
            });
        });
    }

    fn modal_server_join(app: &mut App, ui: &mut egui::Ui) {
        egui::Modal::new("modal_server_join".into()).show(ui, |ui| {
            ui.heading(t!("general.join"));
            ui.separator();

            Self::draw_edit_mono(ui, &t!("server.address"), &mut app.layout.modal_address);

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button(t!("general.join")).clicked() {
                    app.client.client.push(Client::new(
                        app.layout.modal_address.clone(),
                        app.user.identifier.key,
                        app.user.clone().into(),
                    ));

                    app.layout.modal = None;
                    app.layout.modal_address.clear();
                }
                if ui.button(t!("general.close")).clicked() {
                    app.layout.modal = None;
                    app.layout.modal_address.clear();
                }
            });
        });
    }

    fn modal_server_setup(app: &mut App, ui: &mut egui::Ui) {
        let server = &app.client.client[app.layout.index_server.unwrap()].server;

        egui::Modal::new("modal_setup".into()).show(ui, |ui| {
            ui.set_min_size([1024.0 - 64.0, 768.0 - 64.0].into());

            ui.label(format!(
                "Message Text Size Limit: {}",
                server.configuration.limit_text_size
            ));
            ui.label(format!(
                "Message File Size Limit: {} MB",
                server.configuration.limit_file_size / 1_000_000
            ));
            ui.label(format!(
                "Message Poll Size Limit: {}",
                server.configuration.limit_poll_size
            ));
        });
    }

    //================================================================

    fn image(image: ImageSource, size: Vec2) -> egui::Image {
        egui::Image::new(image).fit_to_exact_size(size)
    }

    fn draw_account_self(app: &App, ui: &mut egui::Ui) -> (bool, bool) {
        let mut account = false;
        let mut cog = false;

        ui.horizontal(|ui| {
            account = ui
                .add_sized(
                    [
                        (ui.available_width() - Self::BUTTON_IMAGE_SCALE.x - 16.0).max(0.0),
                        Self::BUTTON_IMAGE_SCALE.y,
                    ],
                    egui::Button::new(Self::image(Self::IMAGE_TEST, Self::BUTTON_IMAGE_SCALE)),
                )
                .clicked();

            if app.layout.panel_server {
                cog = Self::draw_button_image(ui, Self::IMAGE_COG).clicked();
            }
        });

        (account, cog)
    }

    fn draw_selectable<V: PartialEq>(
        ui: &mut egui::Ui,
        current: &mut V,
        select: V,
        image: ImageSource,
        text: &str,
    ) -> Response {
        let image = Self::image(image, Self::BUTTON_IMAGE_SCALE);

        if text.is_empty() {
            ui.selectable_value(current, select, image)
        } else {
            ui.selectable_value(current, select, (image, text))
        }
    }

    fn draw_pop_up<F: FnOnce(&mut egui::Ui)>(
        identifier: &str,
        response: egui::Response,
        ui: &mut egui::Ui,
        content: F,
    ) {
        let identifier = ui.make_persistent_id(identifier);

        if response.clicked() {
            egui::Popup::open_id(ui.ctx(), identifier);
        }

        egui::Popup::from_toggle_button_response(&response).show(content);
    }

    fn draw_button_image(ui: &mut egui::Ui, image: ImageSource) -> Response {
        ui.button(Self::image(image, Self::BUTTON_IMAGE_SCALE))
    }

    fn draw_button_image_label(ui: &mut egui::Ui, image: ImageSource, label: &str) -> Response {
        ui.button((Self::image(image, Self::BUTTON_IMAGE_SCALE), label))
    }

    fn draw_edit_mono(ui: &mut egui::Ui, label: &str, value: &mut String) -> Response {
        ui.label(label);
        ui.text_edit_singleline(value)
    }

    fn draw_edit_multi(ui: &mut egui::Ui, label: &str, value: &mut String) -> Response {
        ui.label(label);
        ui.text_edit_multiline(value)
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
}
