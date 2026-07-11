use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tray_icon::menu::MenuEvent;
use tray_icon::{Icon, TrayIconBuilder};

//================================================================

use crate::user::*;

//================================================================

pub struct System {
    icon: Arc<Mutex<Option<u8>>>,
    exit: Arc<Mutex<bool>>,
    pub tray: bool,
}

impl System {
    const ICON_NORMAL: &[u8] = include_bytes!("../asset/icon_a.png");
    const ICON_NOTIFY_A: &[u8] = include_bytes!("../asset/icon_b.png");
    const ICON_NOTIFY_B: &[u8] = include_bytes!("../asset/icon_c.png");

    //================================================================

    pub fn new(user: &User, ui: &egui::Context) -> Self {
        let ui = ui.clone();
        let icon_m = Arc::new(Mutex::new(None));
        let icon_t = icon_m.clone();
        let exit_m = Arc::new(Mutex::new(false));
        let exit_t = exit_m.clone();
        let mut tray = false;

        #[cfg(not(target_os = "windows"))]
        if user.tray_show {
            tray = true;

            std::thread::spawn(move || {
                gtk::glib::set_application_name("Nimbus");
                gtk::init().unwrap();

                let icon_normal = image::load_from_memory(Self::ICON_NORMAL).unwrap();
                let icon_notify_a = image::load_from_memory(Self::ICON_NOTIFY_A).unwrap();
                let icon_notify_b = image::load_from_memory(Self::ICON_NOTIFY_B).unwrap();

                let icon_normal = Icon::from_rgba(icon_normal.to_rgba8().to_vec(), 64, 64).unwrap();
                let icon_notify_a =
                    Icon::from_rgba(icon_notify_a.to_rgba8().to_vec(), 64, 64).unwrap();
                let icon_notify_b =
                    Icon::from_rgba(icon_notify_b.to_rgba8().to_vec(), 64, 64).unwrap();

                let tray_menu = tray_icon::menu::Menu::with_items(&[
                    &tray_icon::menu::MenuItemBuilder::new()
                        .text("Show Nimbus")
                        .enabled(true)
                        .build(),
                    &tray_icon::menu::MenuItemBuilder::new()
                        .text("Exit Nimbus")
                        .enabled(true)
                        .build(),
                ])
                .expect("System::new(): Couldn't create tray menu.");
                let tray_icon = TrayIconBuilder::new()
                    .with_tooltip("Nimbus")
                    .with_menu(Box::new(tray_menu))
                    .with_icon(icon_normal.clone())
                    .build()
                    .unwrap();

                tray_icon::menu::MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
                    match event.id.0.as_str() {
                        "1" => {
                            ui.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        }
                        "2" => {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                            *exit_t.lock().unwrap() = true;
                        }
                        _ => {}
                    }
                }));

                gtk::glib::timeout_add_local(Duration::from_millis(250), move || {
                    let icon = *icon_t.lock().unwrap();

                    if let Some(icon) = icon {
                        match icon {
                            0 => tray_icon.set_icon(Some(icon_normal.clone())),
                            1 => tray_icon.set_icon(Some(icon_notify_a.clone())),
                            _ => tray_icon.set_icon(Some(icon_notify_b.clone())),
                        };

                        *icon_t.lock().unwrap() = None;
                    }

                    gtk::glib::ControlFlow::Continue
                });

                gtk::main();
            });
        }

        Self {
            exit: exit_m,
            icon: icon_m,
            tray,
        }
    }

    pub fn set_icon_normal(&mut self) {
        *self.icon.lock().unwrap() = Some(0);
    }

    pub fn set_icon_notify_a(&mut self) {
        *self.icon.lock().unwrap() = Some(1);
    }

    pub fn set_icon_notify_b(&mut self) {
        *self.icon.lock().unwrap() = Some(2);
    }

    pub fn push_notification(&self, name: String, text: String) {
        use notify_rust::Notification;

        std::thread::spawn(move || {
            let icon = image::load_from_memory(Self::ICON_NORMAL).unwrap();
            let icon = notify_rust::Image::from_rgba(64, 64, icon.to_rgba8().to_vec()).unwrap();

            let handle = Notification::new()
                .appname("nimbus")
                .summary(&name)
                .body(&text)
                // TO-DO use user avatar
                .image_data(icon)
                // TO-DO open Nimbus if hidden
                //.action("open", "Open Chat")
                //.action("read", "Mark Read")
                .show()
                .unwrap();

            handle.wait_for_action(|event| println!("{event:?}"));
        });
    }

    pub fn exit(&self) -> bool {
        *self.exit.lock().unwrap()
    }
}
