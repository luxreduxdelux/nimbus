use egui::{
    self, Color32, ColorImage, FontFamily, FontId, ImageSource, IntoAtoms, Response, TextStyle,
    TextureHandle, Vec2,
};

//================================================================

use crate::app::*;
use crate::user::*;
use client::prelude::*;

//================================================================

#[derive(Default, Copy, Clone, PartialEq, Eq)]
enum SetupIndex {
    #[default]
    Account,
    Window,
    Notify,
    Input,
}

#[derive(Default, Clone)]
pub struct Layout {
    modal: Option<fn(&mut App, &mut egui::Ui)>,
    index_server: Option<usize>,
    index_channel: Option<usize>,
    index_account: Option<usize>,
    index_setup: SetupIndex,
    setup_user: User,
    modal_address: String,
}

impl Layout {
    const IMAGE_SEARCH: ImageSource<'_> = egui::include_image!("../asset/search.svg");
    const IMAGE_STAR_MESSAGE: ImageSource<'_> = egui::include_image!("../asset/star_message.svg");
    const IMAGE_USER_SIDE: ImageSource<'_> = egui::include_image!("../asset/user_side.svg");
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
    const IMAGE_WINDOW: ImageSource<'_> = egui::include_image!("../asset/window.svg");
    const IMAGE_NOTIFY: ImageSource<'_> = egui::include_image!("../asset/notify.svg");
    const IMAGE_INPUT: ImageSource<'_> = egui::include_image!("../asset/input.svg");
    const IMAGE_APPLY: ImageSource<'_> = egui::include_image!("../asset/apply.svg");
    const IMAGE_RESET: ImageSource<'_> = egui::include_image!("../asset/reset.svg");
    const IMAGE_LOGO: ImageSource<'_> = egui::include_image!("../asset/logo.svg");
    const IMAGE_DOT: ImageSource<'_> = egui::include_image!("../asset/dot.svg");
    const IMAGE_PLUS: ImageSource<'_> = egui::include_image!("../asset/plus.svg");
    const IMAGE_TEST: ImageSource<'_> = egui::include_image!("../asset/test.png");
    const BUTTON_IMAGE_SCALE: Vec2 = Vec2::new(40.0, 40.0);

    //================================================================

    pub fn draw(app: &mut App, ui: &mut egui::Ui) {
        let height = ui.available_height();

        egui::Panel::left("panel_l").show(ui, |ui| {
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
        });

        egui::CentralPanel::default().show(ui, |ui| {
            if app.layout.index_server.is_none() {
                if app.layout.index_account.is_none() {
                    Self::draw_setup(app, ui);
                } else {
                    Self::draw_chat_account(app, ui);
                }
            } else {
                if app.layout.index_channel.is_none() {
                    Self::draw_chat_channel(app, ui);
                }
            }
        });

        if let Some(modal) = &mut app.layout.modal {
            modal(app, ui);
        }
    }

    fn draw_chat_channel(app: &mut App, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Channel Name");
                ui.label("Channel Info");
            });

            Self::draw_button_image(ui, Self::IMAGE_SEARCH);
            Self::draw_button_image(ui, Self::IMAGE_SEARCH);
            Self::draw_button_image(ui, Self::IMAGE_SEARCH);
            Self::draw_button_image(ui, Self::IMAGE_SEARCH);
        });

        ui.separator();

        egui::ScrollArea::vertical()
            .id_salt("scroll_chat")
            .show(ui, |ui| {
                for x in 0..32 {
                    ui.label(format!("{x}"));
                }
            });
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
        if Self::draw_button_image(ui, Self::IMAGE_LOGO).clicked() {
            app.layout.index_server = None;
        }

        ui.add_sized(
            [Self::BUTTON_IMAGE_SCALE.x + 8.0, 0.0],
            egui::Separator::default(),
        );

        egui::ScrollArea::vertical()
            .id_salt("scroll_picker_l")
            .show(ui, |ui| {
                for (i, server) in app.client.client.iter().enumerate() {
                    if Self::draw_button_image(ui, Self::IMAGE_LOGO).clicked() {
                        app.layout.index_server = Some(i);
                    }
                }

                if Self::draw_button_image(ui, Self::IMAGE_PLUS).clicked() {
                    app.layout.modal = Some(Self::modal_server);
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

        if Self::draw_account(ui).clicked() {
            app.layout.index_account = None;
        }

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

                    if Self::draw_account(ui).clicked() {
                        app.layout.index_account = Some(x);
                    }
                }
            });
    }

    fn draw_picker_r_channel(app: &mut App, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Server Name");
                ui.label("Server Info");
            });

            Self::draw_button_image(ui, Self::IMAGE_SEARCH);
        });

        ui.separator();

        egui::ScrollArea::vertical()
            .id_salt("scroll_picker_r")
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

                    ui.button("channel");
                }
            });
    }

    fn draw_setup(app: &mut App, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            Self::draw_selectable(
                ui,
                &mut app.layout.index_setup,
                SetupIndex::Account,
                Self::IMAGE_USER,
                "Account",
            );
            Self::draw_selectable(
                ui,
                &mut app.layout.index_setup,
                SetupIndex::Window,
                Self::IMAGE_WINDOW,
                "Window",
            );
            Self::draw_selectable(
                ui,
                &mut app.layout.index_setup,
                SetupIndex::Notify,
                Self::IMAGE_NOTIFY,
                "Notify",
            );
            Self::draw_selectable(
                ui,
                &mut app.layout.index_setup,
                SetupIndex::Input,
                Self::IMAGE_INPUT,
                "Input",
            );
        });

        ui.separator();

        let h = ui.available_height() - 56.0;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(h)
            .show(ui, |ui| match app.layout.index_setup {
                SetupIndex::Account => Self::draw_setup_account(app, ui),
                SetupIndex::Window => Self::draw_setup_window(app, ui),
                SetupIndex::Notify => Self::draw_setup_notify(app, ui),
                SetupIndex::Input => Self::draw_setup_input(app, ui),
            });

        ui.separator();

        ui.horizontal(|ui| {
            if Self::draw_button_image_label(ui, Self::IMAGE_APPLY, "Apply").clicked() {
                app.user = app.layout.setup_user.clone();
            }
            if Self::draw_button_image_label(ui, Self::IMAGE_RESET, "Reset").clicked() {
                app.layout.setup_user = app.user.clone();
            }
        });
    }

    fn draw_setup_account(app: &mut App, ui: &mut egui::Ui) {
        Self::draw_edit_mono(ui, "Nick Name", &mut app.layout.setup_user.name_nick);
        Self::draw_edit_mono(ui, "User Name", &mut app.layout.setup_user.name_user);
        Self::draw_edit_multi(ui, "User Info", &mut app.layout.setup_user.info);
    }

    fn draw_setup_window(app: &mut App, ui: &mut egui::Ui) {}

    fn draw_setup_notify(app: &mut App, ui: &mut egui::Ui) {}

    fn draw_setup_input(app: &mut App, ui: &mut egui::Ui) {}

    fn modal_server(app: &mut App, ui: &mut egui::Ui) {
        egui::Modal::new("modal".into()).show(ui, |ui| {
            ui.heading("Join Server");
            ui.separator();

            Self::draw_edit_mono(ui, "IP Address", &mut app.layout.modal_address);

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Join").clicked() {
                    app.client.client.push(Client::new(
                        app.layout.modal_address.clone(),
                        app.user.identifier.key,
                        app.user.clone().into(),
                    ));

                    app.layout.modal = None;
                    app.layout.modal_address.clear();
                }
                if ui.button("Close").clicked() {
                    app.layout.modal = None;
                    app.layout.modal_address.clear();
                }
            });
        });
    }

    //================================================================

    fn draw_account(ui: &mut egui::Ui) -> Response {
        ui.button((
            egui::Image::new(Self::IMAGE_TEST).fit_to_exact_size([32.0, 32.0].into()),
            "lux",
        ))
    }

    fn draw_selectable<V: PartialEq>(
        ui: &mut egui::Ui,
        current: &mut V,
        select: V,
        image: ImageSource,
        text: &str,
    ) {
        let image = egui::Image::new(image).fit_to_exact_size(Self::BUTTON_IMAGE_SCALE);
        ui.selectable_value(current, select, (image, text));
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
        ui.button(egui::Image::new(image).fit_to_exact_size(Self::BUTTON_IMAGE_SCALE))
    }

    fn draw_button_image_label(ui: &mut egui::Ui, image: ImageSource, label: &str) -> Response {
        ui.button((
            egui::Image::new(image).fit_to_exact_size(Self::BUTTON_IMAGE_SCALE),
            label,
        ))
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
