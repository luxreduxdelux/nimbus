mod app;
mod layout;
mod system;
mod user;

//================================================================

use crate::app::*;

//================================================================

#[tokio::main]
async fn main() -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../asset/icon.png")).unwrap();

    let option = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([640.0, 480.0])
            .with_icon(std::sync::Arc::new(icon)),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Nimbus",
        option,
        Box::new(|context| {
            egui_extras::install_image_loaders(&context.egui_ctx);
            Ok(Box::new(App::new(&context.egui_ctx)))
        }),
    )
}
