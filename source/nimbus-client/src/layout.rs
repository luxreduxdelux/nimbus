use egui::{
    self, Color32, ColorImage, FontId, ImageSource, Response, RichText, TextureHandle, Vec2,
};
use rust_i18n::t;
use std::collections::HashMap;

//================================================================

use crate::app::*;
use crate::user::*;
use nimbus_protocol::prelude::*;

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
enum ServerIndex {
    #[default]
    General,
    Account,
    Channel,
    Emote,
    Stamp,
    Role,
    Invite,
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
enum ServerPrompt {
    #[default]
    None,
    Modify(u64),
    Create,
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
    modal_code: String,
    modal_file: FileID,
    modal_poll: PollValue,
    modal_setup: SetupIndex,
    modal_server: ServerIndex,

    modal_server_prompt: ServerPrompt,
    //modal_server_account: AccountValue,
    modal_server_channel: ChannelValue,
    modal_server_emote: EmoteValueRequest,
    modal_server_stamp: StampValueRequest,
    modal_server_role: RoleValue,
    modal_server_invite: InviteValue,

    index_server: Option<usize>,
    index_channel: Option<ChannelID>,
    index_account: Option<AccountID>,
    index_friend: FriendIndex,
    setup_user: User,
    entry_text: String,
    entry_reply: Option<MessageID>,
    panel_user: bool,
    image_cache: HashMap<FileID, TextureHandle>,
}

impl Layout {
    #[rustfmt::skip]    const IMAGE_SEARCH         : ImageSource<'_> = egui::include_image!("../asset/search.svg");
    #[rustfmt::skip]    const IMAGE_STAR_MESSAGE   : ImageSource<'_> = egui::include_image!("../asset/star_message.svg");
    #[rustfmt::skip]    const IMAGE_COG            : ImageSource<'_> = egui::include_image!("../asset/cog.svg");
    #[rustfmt::skip]    const IMAGE_REPLY          : ImageSource<'_> = egui::include_image!("../asset/reply.svg");
    #[rustfmt::skip]    const IMAGE_EMOTE          : ImageSource<'_> = egui::include_image!("../asset/emote.svg");
    #[rustfmt::skip]    const IMAGE_COPY           : ImageSource<'_> = egui::include_image!("../asset/copy.svg");
    #[rustfmt::skip]    const IMAGE_EDIT           : ImageSource<'_> = egui::include_image!("../asset/edit.svg");
    #[rustfmt::skip]    const IMAGE_STAR_A         : ImageSource<'_> = egui::include_image!("../asset/star_a.svg");
    #[rustfmt::skip]    const IMAGE_DELETE         : ImageSource<'_> = egui::include_image!("../asset/delete.svg");
    #[rustfmt::skip]    const IMAGE_STAMP          : ImageSource<'_> = egui::include_image!("../asset/stamp.svg");
    #[rustfmt::skip]    const IMAGE_SEND           : ImageSource<'_> = egui::include_image!("../asset/send.svg");
    #[rustfmt::skip]    const IMAGE_ROLE           : ImageSource<'_> = egui::include_image!("../asset/role.svg");
    #[rustfmt::skip]    const IMAGE_ACCOUNT        : ImageSource<'_> = egui::include_image!("../asset/account.svg");
    #[rustfmt::skip]    const IMAGE_ACCOUNT_SIDE   : ImageSource<'_> = egui::include_image!("../asset/account_side.svg");
    #[rustfmt::skip]    const IMAGE_ACCOUNT_FRIEND : ImageSource<'_> = egui::include_image!("../asset/account_friend.svg");
    #[rustfmt::skip]    const IMAGE_ACCOUNT_ADD    : ImageSource<'_> = egui::include_image!("../asset/account_add.svg");
    #[rustfmt::skip]    const IMAGE_ACCOUNT_CODE   : ImageSource<'_> = egui::include_image!("../asset/account_code.svg");
    #[rustfmt::skip]    const IMAGE_ACCOUNT_ONLINE : ImageSource<'_> = egui::include_image!("../asset/account_online.svg");
    #[rustfmt::skip]    const IMAGE_CHANNEL        : ImageSource<'_> = egui::include_image!("../asset/channel.svg");
    #[rustfmt::skip]    const IMAGE_INVITE         : ImageSource<'_> = egui::include_image!("../asset/invite.svg");
    #[rustfmt::skip]    const IMAGE_SERVER         : ImageSource<'_> = egui::include_image!("../asset/server.svg");
    #[rustfmt::skip]    const IMAGE_WINDOW         : ImageSource<'_> = egui::include_image!("../asset/window.svg");
    #[rustfmt::skip]    const IMAGE_NOTIFY         : ImageSource<'_> = egui::include_image!("../asset/notify.svg");
    #[rustfmt::skip]    const IMAGE_INPUT          : ImageSource<'_> = egui::include_image!("../asset/input.svg");
    #[rustfmt::skip]    const IMAGE_APPLY          : ImageSource<'_> = egui::include_image!("../asset/apply.svg");
    #[rustfmt::skip]    const IMAGE_RESET          : ImageSource<'_> = egui::include_image!("../asset/reset.svg");
    #[rustfmt::skip]    const IMAGE_CLOSE          : ImageSource<'_> = egui::include_image!("../asset/close.svg");
    #[rustfmt::skip]    const IMAGE_ERROR          : ImageSource<'_> = egui::include_image!("../asset/error.svg");
    #[rustfmt::skip]    const IMAGE_ENTER          : ImageSource<'_> = egui::include_image!("../asset/enter.svg");
    #[rustfmt::skip]    const IMAGE_LEAVE          : ImageSource<'_> = egui::include_image!("../asset/leave.svg");
    #[rustfmt::skip]    const IMAGE_LOGO           : ImageSource<'_> = egui::include_image!("../asset/logo.svg");
    #[rustfmt::skip]    const IMAGE_DOT            : ImageSource<'_> = egui::include_image!("../asset/dot.svg");
    #[rustfmt::skip]    const IMAGE_PLUS           : ImageSource<'_> = egui::include_image!("../asset/plus.svg");
    #[rustfmt::skip]    const IMAGE_ATTACHMENT     : ImageSource<'_> = egui::include_image!("../asset/attachment.svg");
    #[rustfmt::skip]    const IMAGE_POLL           : ImageSource<'_> = egui::include_image!("../asset/poll.svg");
    #[rustfmt::skip]    const IMAGE_FILE           : ImageSource<'_> = egui::include_image!("../asset/file.svg");
    #[rustfmt::skip]    const IMAGE_FILE_TEXT      : ImageSource<'_> = egui::include_image!("../asset/file_text.svg");
    #[rustfmt::skip]    const IMAGE_FILE_IMAGE     : ImageSource<'_> = egui::include_image!("../asset/file_image.svg");
    #[rustfmt::skip]    const IMAGE_FILE_VIDEO     : ImageSource<'_> = egui::include_image!("../asset/file_video.svg");
    #[rustfmt::skip]    const IMAGE_FILE_AUDIO     : ImageSource<'_> = egui::include_image!("../asset/file_audio.svg");
    #[rustfmt::skip]    const IMAGE_TEST           : ImageSource<'_> = egui::include_image!("../asset/test.png");
    const BUTTON_IMAGE_SCALE: Vec2 = Vec2::new(40.0, 40.0);
    const PANEL_L_SIZE: f32 = 320.0; //376.0;
    const PANEL_R_SIZE: f32 = 256.0; //264.0;

    //================================================================

    pub fn load_texture_raw(
        &mut self,
        context: &egui::Context,
        file: FileID,
        data: &[u8],
    ) -> anyhow::Result<()> {
        let image = image::load_from_memory(data)?.to_rgba8();
        let scale = [image.width() as usize, image.height() as usize];
        let image = egui::ColorImage::from_rgba_unmultiplied(scale, image.as_raw());
        let image = context.load_texture("image", image, egui::TextureOptions::default());
        self.image_cache.insert(file, image);

        Ok(())
    }

    pub fn draw(app: &mut App, ui: &mut egui::Ui) {
        let height = ui.available_height() - 66.0;

        egui::Panel::left("panel_l")
            .min_size(Self::PANEL_L_SIZE)
            .max_size(Self::PANEL_L_SIZE)
            .resizable(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_min_height(height);
                        ui.add_space(4.0);
                        Self::draw_picker_l(app, ui);
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.set_min_height(height);
                        ui.add_space(4.0);
                        if app.layout.index_server.is_none() {
                            Self::draw_picker_r_account(app, ui);
                        } else {
                            Self::draw_picker_r_channel(app, ui);
                        }
                    });
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
                    if let Some(view) = Self::get_client_mutable(app).cache.get_view_account() {
                        for (_, account) in view {
                            Self::draw_pop_up(
                                "pop_account",
                                Self::draw_account_side(ui, account),
                                ui,
                                |ui| {
                                    Self::draw_account_main(ui, account);
                                },
                            );
                        }
                    }
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

    fn draw_message_system(ui: &mut egui::Ui, message: &MessageSystem, client: &mut Client) {
        match message {
            MessageSystem::Enter(account) => {
                if let Some(account) = client.cache.get_account(*account) {
                    Self::draw_image_label(
                        ui,
                        Self::image(Self::IMAGE_ENTER, Vec2::splat(32.0)).tint(Color32::GREEN),
                        &account.name_nick,
                    );
                }
            }
            MessageSystem::Leave(account) => {
                if let Some(account) = client.cache.get_account(*account) {
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

    fn draw_message_in_line(ui: &mut egui::Ui, message: &Message, client: &mut Client) {
        match &message.value {
            MessageValue::System(message) => Self::draw_message_system(ui, message, client),
            MessageValue::Text(text) => {
                ui.label(RichText::new(text).italics());
            }
            MessageValue::File(_) => {
                //TO-DO
                //ui.label(RichText::new("[File]").italics());
            }
            MessageValue::Poll(_) => {
                ui.label(RichText::new("[Poll]").italics());
            }
            MessageValue::Stamp(_) => {
                // TO-DO
                //ui.label(RichText::new("[Poll]").italics());
            }
        }
    }

    fn draw_message_reply(ui: &mut egui::Ui, message: MessageID, client: &mut Client) {
        if let Some(message) = client.cache.get_message(message).cloned()
            && let Some(account) = message.account(&mut client.cache)
        {
            ui.label(RichText::new(&account.name_nick).weak());
            Self::draw_message_in_line(ui, &message, client);
        } else {
            ui.label(RichText::new(t!("message.error")).italics());
        }
    }

    fn draw_account_main(ui: &mut egui::Ui, account: &Account) {
        ui.horizontal(|ui| {
            ui.add(Self::image(Self::IMAGE_TEST, Vec2::splat(96.0)).corner_radius(96.0));

            ui.vertical(|ui| {
                ui.label(&account.name_nick);
                ui.label(&account.name_user);
            });
        });

        ui.separator();

        if account.info.is_empty() {
            ui.label(RichText::new("No user info.").weak());
        } else {
            ui.label(&account.info);
        }
    }

    fn draw_account_side(ui: &mut egui::Ui, account: &Account) -> Response {
        ui.horizontal(|ui| {
            let response = ui.add(
                Self::image(Self::IMAGE_TEST, Self::BUTTON_IMAGE_SCALE)
                    .sense(egui::Sense::click())
                    .corner_radius(Self::BUTTON_IMAGE_SCALE.x),
            );
            ui.label(&account.name_nick);
            response
        })
        .inner
    }

    fn draw_chat_channel(app: &mut App, ui: &mut egui::Ui) {
        let client = &mut app.client.client[app.layout.index_server.unwrap() as usize];

        if let Some(channel_index) = app.layout.index_channel
            && let Some(channel) = client.cache.get_channel(channel_index)
        {
            let width = (ui.available_width() - (Self::BUTTON_IMAGE_SCALE.x + 14.0) * 4.0).max(0.0);

            ui.horizontal(|ui| {
                Self::allocate_size(ui, width, |ui| {
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Label::new(RichText::new(&channel.value.name).strong())
                                .truncate(),
                        );
                        ui.add(egui::Label::new(&channel.value.info).truncate());
                    });
                });

                ui.add_space(
                    (ui.available_width() - (Self::BUTTON_IMAGE_SCALE.x + 14.0) * 4.0).max(0.0),
                );

                Self::draw_button_image(ui, Self::IMAGE_COG);
                Self::draw_button_image(ui, Self::IMAGE_SEARCH);
                Self::draw_button_image(ui, Self::IMAGE_STAR_MESSAGE);

                let mut panel = app.layout.panel_user;

                if Self::draw_selectable(ui, &mut panel, true, Self::IMAGE_ACCOUNT_SIDE, "")
                    .clicked()
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
                    let account = client.get_local_account().clone();

                    if let Some(view) = client.cache.get_view_message(channel_index).cloned() {
                        for (i, message) in view {
                            let message_system = message.account.is_none();
                            //let cache = &client.cache.get_message_cache(&account, message);
                            //let color = if cache.mention {
                            //    Color32::ORANGE
                            //} else {
                            //    Color32::TRANSPARENT
                            //};

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
                                            if let Some(account) =
                                                message.account(&mut client.cache)
                                            {
                                                ui.label(
                                                    RichText::new(&account.name_nick).strong(),
                                                );
                                            }

                                            if let Some(reply) = message.reply {
                                                egui::Frame::group(ui.style()).show(ui, |ui| {
                                                    Self::draw_message_reply(ui, reply, client);
                                                });
                                            }

                                            match &message.value {
                                                MessageValue::System(message) => {
                                                    Self::draw_message_system(ui, message, client);
                                                }
                                                MessageValue::Text(text) => {
                                                    ui.label(text);
                                                }
                                                MessageValue::File(file) => {
                                                    if client.cache.get_file_state(file.index)
                                                        == ViewState::RequestDone
                                                    {
                                                        if let Some(image) =
                                                            app.layout.image_cache.get(&file.index)
                                                        {
                                                            if ui
                                                                .add(
                                                                    Self::image(
                                                                        image.into(),
                                                                        Vec2::new(256.0, 256.0),
                                                                    )
                                                                    .sense(egui::Sense::click()),
                                                                )
                                                                .clicked()
                                                            {
                                                                app.layout.modal_file = file.index;
                                                                app.layout.modal =
                                                                    Some(Self::modal_image);
                                                            };
                                                        }

                                                        //ui.label(format!("{name} ({} MB)", size));
                                                    } else if client
                                                        .cache
                                                        .get_file_state(file.index)
                                                        == ViewState::RequestMade
                                                    {
                                                        ui.horizontal(|ui| {
                                                            ui.spinner();
                                                            ui.label("Downloading file...");
                                                        });
                                                    } else {
                                                        let image = match file.kind() {
                                                            FileKind::Text => Self::IMAGE_FILE_TEXT,
                                                            FileKind::Image => {
                                                                Self::IMAGE_FILE_IMAGE
                                                            }
                                                            FileKind::Video => {
                                                                Self::IMAGE_FILE_VIDEO
                                                            }
                                                            FileKind::Audio => {
                                                                Self::IMAGE_FILE_AUDIO
                                                            }
                                                            FileKind::Other => Self::IMAGE_FILE,
                                                        };

                                                        Self::draw_image_label(
                                                            ui,
                                                            Self::image(
                                                                image,
                                                                Self::BUTTON_IMAGE_SCALE,
                                                            ),
                                                            &format!(
                                                                "{} ({:.2} MB)",
                                                                file.name,
                                                                file.size as f32 / 1_000_000_f32
                                                            ),
                                                        );

                                                        if ui.button("Download File").clicked() {
                                                            client.cache.get_file(file.index);
                                                        }
                                                    }
                                                }
                                                MessageValue::Poll(poll) => {
                                                    //
                                                    ui.label(&poll.value.name);
                                                    ui.separator();

                                                    for (i, choice) in
                                                        poll.value.choice.iter().enumerate()
                                                    {
                                                        let vote =
                                                            if let Some(vote) = poll.vote.get(&i) {
                                                                vote.clone()
                                                            } else {
                                                                Default::default()
                                                            };

                                                        ui.label(choice);

                                                        ui.horizontal(|ui| {
                                                            if ui
                                                                .checkbox(
                                                                    &mut vote
                                                                        .contains(&client.index),
                                                                    (),
                                                                )
                                                                .clicked()
                                                            {
                                                                client.send(
                                                                    CommandClient::PollVote(
                                                                        message.index,
                                                                        i,
                                                                    ),
                                                                );
                                                            }
                                                            ui.add(
                                                                egui::ProgressBar::new(1.0)
                                                                    .text(vote.len().to_string()),
                                                            );
                                                        });
                                                    }
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
                                            app.layout.entry_reply = Some(message.index);
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
                                            client
                                                .send(CommandClient::MessageRemove(message.index));
                                        };
                                    })
                                });
                            });
                        }
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
                                if let Some(view) = client.cache.get_view_account() {
                                    for (_, account) in view {
                                        if ui.button(&account.name_user).clicked() {
                                            app.layout.entry_text.truncate(index);
                                            app.layout.entry_text.push('@');
                                            app.layout.entry_text.push_str(&account.name_user);
                                            app.layout.entry_text.push(' ');
                                            entry_focus = true;
                                        }
                                    }
                                }
                            } else {
                                if let Some(view) = client.cache.get_view_channel() {
                                    for (_, channel) in view {
                                        if ui.button(&channel.value.name).clicked() {
                                            app.layout.entry_text.truncate(index);
                                            app.layout.entry_text.push('#');
                                            app.layout.entry_text.push_str(&channel.value.name);
                                            app.layout.entry_text.push(' ');
                                            entry_focus = true;
                                        }
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
                            Self::draw_message_reply(ui, reply, client);
                        });
                });
            } else {
                ui.label("");
            };

            ui.separator();

            ui.horizontal(|ui| {
                Self::draw_pop_up(
                    "plus_pop",
                    Self::draw_button_image(ui, Self::IMAGE_PLUS),
                    ui,
                    |ui| {
                        if Self::draw_button_image_label(ui, Self::IMAGE_ATTACHMENT, "Upload File")
                            .clicked()
                        {}
                        if Self::draw_button_image_label(ui, Self::IMAGE_POLL, "Submit Poll")
                            .clicked()
                        {
                            app.layout.modal = Some(Self::modal_poll);
                        }

                        /*
                        if let Some(file) =
                            rfd::FileDialog::new().set_title("Upload File").pick_file()
                        {
                            client.send(CommandClient::Message(
                                channel_index as ChannelID,
                                MessageKindRequest::File(
                                    file.file_name()
                                        .map(|x| x.display().to_string())
                                        .unwrap_or("file".to_string()),
                                    std::fs::read(file).unwrap(),
                                ),
                            ));
                        }
                        */
                    },
                );

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

                Self::draw_button_image(ui, Self::IMAGE_STAMP);
                Self::draw_button_image(ui, Self::IMAGE_EMOTE);

                if Self::draw_button_image(ui, Self::IMAGE_SEND).clicked() {
                    if let Some(message_index) = app.layout.entry_reply {
                        client.send(CommandClient::MessageReply(
                            message_index,
                            MessageValueRequest::Text(app.layout.entry_text.clone()),
                        ));
                    } else {
                        client.send(CommandClient::Message(
                            channel_index as ChannelID,
                            MessageValueRequest::Text(app.layout.entry_text.clone()),
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
        Self::draw_selectable(ui, &mut app.layout.index_server, None, Self::IMAGE_LOGO, "");

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
                        }

                        response.on_hover_text(&client.server.configuration.name);
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
            Self::IMAGE_ACCOUNT_FRIEND,
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
                        Self::IMAGE_ACCOUNT,
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
        let client = &mut app.client.client[app.layout.index_server.unwrap()];
        let width = (ui.available_width() - Self::BUTTON_IMAGE_SCALE.x - 16.0).max(0.0);

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.add_space(Self::BUTTON_IMAGE_SCALE.y * 0.5 - 8.0);
                Self::allocate_size(ui, width, |ui| {
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new(&client.server.configuration.name).strong(),
                            )
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
                if let Some(view) = client.cache.get_view_channel() {
                    for (i, channel) in view {
                        let select = if let Some(index) = app.layout.index_channel {
                            (index) == *i
                        } else {
                            false
                        };

                        if ui
                            .add_sized(
                                [ui.available_width(), 0.0],
                                egui::Button::new(&channel.value.name).selected(select),
                            )
                            .clicked()
                        {
                            app.layout.index_channel = Some(*i);
                        }
                    }
                }
            });
    }

    #[rustfmt::skip]
    fn draw_friend_list(app: &mut App, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            Self::draw_selectable(ui, &mut app.layout.index_friend, FriendIndex::Online, Self::IMAGE_ACCOUNT_ONLINE, &t!("status.online"));
            Self::draw_selectable(ui, &mut app.layout.index_friend, FriendIndex::All,  Self::IMAGE_ACCOUNT,          &t!("general.all"));
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
            if Self::draw_button_image_label(ui, Self::IMAGE_ACCOUNT_ADD, &t!("friend.add")).clicked() {
            }
            if Self::draw_button_image_label(ui, Self::IMAGE_ACCOUNT_CODE, &t!("friend.code")).clicked() {
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
            &mut app.user.indicator_read,
            t!("setup_account.indicator_read"),
        );
        ui.checkbox(
            &mut app.user.indicator_type,
            t!("setup_account.indicator_type"),
        );
        ui.checkbox(
            &mut app.user.indicator_seen,
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
        ui.checkbox(&mut app.user.embed_link, t!("setup_window.embed_link"));
        ui.checkbox(&mut app.user.embed_file, t!("setup_window.embed_file"));
        ui.checkbox(&mut app.user.show_hidden, t!("setup_window.show_hidden"));
        ui.checkbox(
            &mut app.user.message_compact,
            t!("setup_window.message_compact"),
        );
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

    fn draw_server_setup_general(app: &mut App, ui: &mut egui::Ui) {
        ui.label("Server Name");
        ui.text_edit_singleline(&mut "");

        ui.label("Server Info");
        ui.text_edit_singleline(&mut "");
    }

    fn draw_server_setup_account(app: &mut App, ui: &mut egui::Ui) {
        let client = &mut app.client.client[app.layout.index_server.unwrap()];

        if let Some(view) = client.cache.get_view_account() {
            for (i, account) in view {
                ui.horizontal(|ui| {
                    Self::draw_button_image_label(ui, Self::IMAGE_CLOSE, "Remove");
                    ui.add(
                        Self::image(Self::IMAGE_TEST, Self::BUTTON_IMAGE_SCALE)
                            .corner_radius(Self::BUTTON_IMAGE_SCALE.x),
                    );
                    ui.label(&account.name_nick);
                });
            }
        }
    }

    fn draw_modal_editor<
        F: FnMut(&mut App),
        G: FnMut(&mut App, &mut egui::Ui),
        H: FnMut(&mut App, u64),
        J: FnMut(&mut App),
    >(
        app: &mut App,
        ui: &mut egui::Ui,
        text_button: &str,
        text_prompt: &str,
        mut call_click: F,
        mut call_prompt: G,
        mut call_modify: H,
        mut call_create: J,
    ) {
        if Self::draw_button_image_label(ui, Self::IMAGE_PLUS, text_button).clicked() {
            app.layout.modal_server_prompt = ServerPrompt::Create;
            call_click(app);
            //app.layout.modal_server_channel = ChannelValue::default();
        }

        Self::modal_server_prompt(
            app,
            ui,
            text_prompt,
            |app, ui| call_prompt(app, ui),
            |app| {
                if let ServerPrompt::Modify(index) = app.layout.modal_server_prompt {
                    call_modify(app, index);
                } else {
                    call_create(app);
                }
            },
        );
    }

    fn draw_server_setup_channel(app: &mut App, ui: &mut egui::Ui) {
        let mut remove = None;
        let mut modify = None;

        if let Some(view) = Self::get_client_mutable(app).cache.get_view_channel() {
            for (i, channel) in view {
                ui.horizontal(|ui| {
                    if Self::draw_button_image(ui, Self::IMAGE_DELETE).clicked() {
                        remove = Some(*i);
                    }
                    if Self::draw_button_image(ui, Self::IMAGE_EDIT).clicked() {
                        modify = Some((*i, channel.value.clone()));
                    }
                    ui.label(&channel.value.name);
                });
            }
        }

        if let Some(i) = remove {
            Self::get_client_mutable(app).send(CommandClient::ChannelRemove(i));
        }
        if let Some((i, channel)) = modify {
            app.layout.modal_server_prompt = ServerPrompt::Modify(i);
            app.layout.modal_server_channel = channel;
        }

        Self::draw_modal_editor(
            app,
            ui,
            "Create Channel",
            "Channel",
            |app| {
                app.layout.modal_server_channel = ChannelValue::default();
            },
            |app, ui| {
                ui.label("Channel Name");
                ui.text_edit_singleline(&mut app.layout.modal_server_channel.name);
                ui.label("Channel Info");
                ui.text_edit_singleline(&mut app.layout.modal_server_channel.info);
            },
            |app, index| {
                Self::get_client(app).send(CommandClient::ChannelModify(
                    index,
                    app.layout.modal_server_channel.clone(),
                ));
            },
            |app| {
                Self::get_client(app).send(CommandClient::Channel(
                    app.layout.modal_server_channel.clone(),
                ));
            },
        );
    }

    fn draw_server_setup_emote(app: &mut App, ui: &mut egui::Ui) {}

    fn draw_server_setup_stamp(app: &mut App, ui: &mut egui::Ui) {
        let mut remove = None;
        let mut modify = None;

        if let Some(view) = Self::get_client_mutable(app).cache.get_view_role() {
            for (i, role) in view {
                let color = egui::Color32::from_rgb(
                    role.value.color.r,
                    role.value.color.g,
                    role.value.color.b,
                );

                ui.horizontal(|ui| {
                    if Self::draw_button_image(ui, Self::IMAGE_DELETE).clicked() {
                        remove = Some(*i);
                    }
                    if Self::draw_button_image(ui, Self::IMAGE_EDIT).clicked() {
                        modify = Some((*i, role.value.clone()));
                    }
                    ui.add(Self::image(Self::IMAGE_DOT, Vec2::splat(16.0)).tint(color));
                    ui.label(&role.value.name);
                });
            }
        }

        if let Some(i) = remove {
            Self::get_client_mutable(app).send(CommandClient::RoleRemove(i));
        }
        if let Some((i, role)) = modify {
            app.layout.modal_server_prompt = ServerPrompt::Modify(i);
            app.layout.modal_server_role = role.clone();
        }

        Self::draw_modal_editor(
            app,
            ui,
            "Create Role",
            "Role",
            |app| {
                app.layout.modal_server_role = RoleValue::default();
            },
            |app, ui| {
                ui.label("Role Name");
                ui.text_edit_singleline(&mut app.layout.modal_server_role.name);

                let mut color = egui::Color32::from_rgb(
                    app.layout.modal_server_role.color.r,
                    app.layout.modal_server_role.color.g,
                    app.layout.modal_server_role.color.b,
                );

                egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut color,
                    egui::color_picker::Alpha::Opaque,
                );

                app.layout.modal_server_role.color = Color::new(color.r(), color.g(), color.b());
            },
            |app, index| {
                Self::get_client(app).send(CommandClient::RoleModify(
                    index,
                    app.layout.modal_server_role.clone(),
                ));
            },
            |app| {
                Self::get_client(app)
                    .send(CommandClient::Role(app.layout.modal_server_role.clone()));
            },
        );
    }

    fn draw_server_setup_role(app: &mut App, ui: &mut egui::Ui) {
        let mut remove = None;
        let mut modify = None;

        if let Some(view) = Self::get_client_mutable(app).cache.get_view_role() {
            for (i, role) in view {
                let color = egui::Color32::from_rgb(
                    role.value.color.r,
                    role.value.color.g,
                    role.value.color.b,
                );

                ui.horizontal(|ui| {
                    if Self::draw_button_image(ui, Self::IMAGE_DELETE).clicked() {
                        remove = Some(*i);
                    }
                    if Self::draw_button_image(ui, Self::IMAGE_EDIT).clicked() {
                        modify = Some((*i, role.value.clone()));
                    }
                    ui.add(Self::image(Self::IMAGE_DOT, Vec2::splat(16.0)).tint(color));
                    ui.label(&role.value.name);
                });
            }
        }

        if let Some(i) = remove {
            Self::get_client_mutable(app).send(CommandClient::RoleRemove(i));
        }
        if let Some((i, role)) = modify {
            app.layout.modal_server_prompt = ServerPrompt::Modify(i);
            app.layout.modal_server_role = role.clone();
        }

        Self::draw_modal_editor(
            app,
            ui,
            "Create Role",
            "Role",
            |app| {
                app.layout.modal_server_role = RoleValue::default();
            },
            |app, ui| {
                ui.label("Role Name");
                ui.text_edit_singleline(&mut app.layout.modal_server_role.name);

                let mut color = egui::Color32::from_rgb(
                    app.layout.modal_server_role.color.r,
                    app.layout.modal_server_role.color.g,
                    app.layout.modal_server_role.color.b,
                );

                egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut color,
                    egui::color_picker::Alpha::Opaque,
                );

                app.layout.modal_server_role.color = Color::new(color.r(), color.g(), color.b());
            },
            |app, index| {
                Self::get_client(app).send(CommandClient::RoleModify(
                    index,
                    app.layout.modal_server_role.clone(),
                ));
            },
            |app| {
                Self::get_client(app)
                    .send(CommandClient::Role(app.layout.modal_server_role.clone()));
            },
        );
    }

    fn draw_server_setup_invite(app: &mut App, ui: &mut egui::Ui) {
        let mut remove = None;
        //let mut modify = None;

        if let Some(view) = Self::get_client_mutable(app).cache.get_view_invite() {
            for (i, invite) in view {
                ui.horizontal(|ui| {
                    if Self::draw_button_image(ui, Self::IMAGE_DELETE).clicked() {
                        remove = Some(i.clone());
                    }
                    if Self::draw_button_image(ui, Self::IMAGE_EDIT).clicked() {
                        //modify = Some((*i, invite.value.clone()));
                    }
                    ui.label(&invite.value.index);
                });
            }
        }

        if let Some(i) = remove {
            Self::get_client_mutable(app).send(CommandClient::InviteRemove(i));
        }
        /*
        if let Some((i, channel)) = modify {
            app.layout.modal_server_prompt = ServerPrompt::Modify(i);
            app.layout.modal_server_channel = channel;
        }
        */

        Self::draw_modal_editor(
            app,
            ui,
            "Create Invite",
            "Invite",
            |app| {
                app.layout.modal_server_invite = InviteValue::default();
            },
            |app, ui| {
                ui.label("Invite Code");
                ui.text_edit_singleline(&mut app.layout.modal_server_invite.index);
                // TO-DO rest of invite
            },
            |app, index| {
                //Self::get_client(app).send(CommandClient::ChannelModify(
                //    index,
                //    app.layout.modal_server_invite.clone(),
                //));
            },
            |app| {
                Self::get_client(app).send(CommandClient::Invite(
                    app.layout.modal_server_invite.clone(),
                ));
            },
        );
    }

    fn modal_server_prompt<F: FnMut(&mut App, &mut egui::Ui), G: FnMut(&mut App)>(
        app: &mut App,
        ui: &mut egui::Ui,
        text: &str,
        mut call_prompt: F,
        mut call_accept: G,
    ) {
        if app.layout.modal_server_prompt != ServerPrompt::None {
            egui::Modal::new("modal_server_prompt".into()).show(ui, |ui| {
                ui.heading(text);
                ui.separator();

                call_prompt(app, ui);

                ui.separator();
                if ui.button("Accept").clicked() {
                    call_accept(app);
                    app.layout.modal_server_prompt = ServerPrompt::None;
                }
                if ui.button("Cancel").clicked() {
                    app.layout.modal_server_prompt = ServerPrompt::None;
                }
            });
        }
    }

    fn modal_poll(app: &mut App, ui: &mut egui::Ui) {
        egui::Modal::new("modal_poll".into()).show(ui, |ui| {
            ui.heading("Submit Poll");
            ui.separator();

            // Poll name
            ui.label("Poll Name");
            ui.text_edit_singleline(&mut app.layout.modal_poll.name);

            let mut remove = None;

            for (i, choice) in app.layout.modal_poll.choice.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("Choice #{}", i + 1));
                    if ui.button("Remove").clicked() {
                        remove = Some(i);
                    }
                });
                ui.text_edit_singleline(choice);
            }

            if let Some(remove) = remove {
                app.layout.modal_poll.choice.remove(remove);
            }

            if ui.button("Add Choice").clicked() {
                app.layout.modal_poll.choice.push(Default::default());
            }

            ui.checkbox(&mut app.layout.modal_poll.hidden, "Hide Voter Name");
            ui.checkbox(
                &mut app.layout.modal_poll.single,
                "Allow More Than One Choice",
            );
            ui.checkbox(&mut app.layout.modal_poll.attach, "Allow Adding New Choice");
            ui.checkbox(&mut app.layout.modal_poll.revoke, "Allow Re-Voting");
            //ui.checkbox(&mut app.layout.modal_poll.revoke, "Set Correct Choice");

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Accept").clicked() {
                    Self::get_client(app).send(CommandClient::Message(
                        app.layout.index_channel.unwrap(),
                        MessageValueRequest::Poll(app.layout.modal_poll.clone()),
                    ));
                    app.layout.modal = None;
                }
                if ui.button("Cancel").clicked() {
                    app.layout.modal = None;
                }
            });
        });
    }

    fn modal_image(app: &mut App, ui: &mut egui::Ui) {
        egui::Modal::new("modal_image".into()).show(ui, |ui| {
            if let Some(image) = app.layout.image_cache.get(&app.layout.modal_file) {
                ui.add(Self::image(image.into(), Vec2::new(512.0, 512.0)));
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.button("Download");
                if ui.button("Close").clicked() {
                    app.layout.modal = None;
                }
            });
        });
    }

    #[rustfmt::skip]
    fn modal_setup(app: &mut App, ui: &mut egui::Ui) {
        egui::Modal::new("modal_setup".into()).show(ui, |ui| {
            ui.set_min_size([1024.0 - 64.0, 768.0 - 64.0].into());

            ui.horizontal(|ui| {
                Self::draw_selectable(ui, &mut app.layout.modal_setup, SetupIndex::Account, Self::IMAGE_ACCOUNT, "Account");
                Self::draw_selectable(ui, &mut app.layout.modal_setup, SetupIndex::Window,  Self::IMAGE_WINDOW,  "Window");
                Self::draw_selectable(ui, &mut app.layout.modal_setup, SetupIndex::Notify,  Self::IMAGE_NOTIFY,  "Notify");
                Self::draw_selectable(ui, &mut app.layout.modal_setup, SetupIndex::Input,   Self::IMAGE_INPUT,   "Input");
            });

            ui.separator();

            let h = ui.available_height() - 56.0;

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(h)
                .show(ui, |ui| match app.layout.modal_setup {
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
            Self::draw_edit_mono(ui, &t!("server.code"), &mut app.layout.modal_code);

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button(t!("general.join")).clicked() {
                    /* TO-DO
                    app.client.client.push(Client::new(
                        app.layout.modal_address.clone(),
                        app.user.identifier.key,
                        app.user.clone().into(),
                    ));
                    */

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

    #[rustfmt::skip]
    fn modal_server_setup(app: &mut App, ui: &mut egui::Ui) {
        egui::Modal::new("modal_server_setup".into()).show(ui, |ui| {
            ui.set_min_size([1024.0 - 64.0, 768.0 - 64.0].into());

            ui.horizontal(|ui| {
                Self::draw_selectable(ui, &mut app.layout.modal_server, ServerIndex::General, Self::IMAGE_SERVER,  "General");
                Self::draw_selectable(ui, &mut app.layout.modal_server, ServerIndex::Account, Self::IMAGE_ACCOUNT, "Account");
                Self::draw_selectable(ui, &mut app.layout.modal_server, ServerIndex::Channel, Self::IMAGE_CHANNEL, "Channel");
                Self::draw_selectable(ui, &mut app.layout.modal_server, ServerIndex::Emote,   Self::IMAGE_EMOTE,   "Emote");
                Self::draw_selectable(ui, &mut app.layout.modal_server, ServerIndex::Stamp, Self::IMAGE_STAMP, "Stamp");
                Self::draw_selectable(ui, &mut app.layout.modal_server, ServerIndex::Role,    Self::IMAGE_ROLE,    "Role");
                Self::draw_selectable(ui, &mut app.layout.modal_server, ServerIndex::Invite,  Self::IMAGE_INVITE,  "Invite");
            });

            ui.separator();

            let h = ui.available_height() - 56.0;

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(h)
                .show(ui, |ui| match app.layout.modal_server {
                    ServerIndex::General => Self::draw_server_setup_general(app, ui),
                    ServerIndex::Account => Self::draw_server_setup_account(app, ui),
                    ServerIndex::Channel => Self::draw_server_setup_channel(app, ui),
                    ServerIndex::Emote   => Self::draw_server_setup_emote(app, ui),
                    ServerIndex::Stamp => Self::draw_server_setup_stamp(app, ui),
                    ServerIndex::Role    => Self::draw_server_setup_role(app, ui),
                    ServerIndex::Invite  => Self::draw_server_setup_invite(app, ui),
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

    //================================================================

    fn get_client(app: &App) -> &Client {
        &app.client.client[app.layout.index_server.unwrap()]
    }

    fn get_client_mutable(app: &mut App) -> &mut Client {
        &mut app.client.client[app.layout.index_server.unwrap()]
    }

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

            cog = Self::draw_button_image(ui, Self::IMAGE_COG).clicked();
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
}
