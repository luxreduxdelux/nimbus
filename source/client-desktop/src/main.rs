mod app;
mod system;
mod user;

//================================================================

use crate::app::*;

//================================================================

#[tokio::main]
async fn main() -> eframe::Result {
    let option = eframe::NativeOptions {
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
