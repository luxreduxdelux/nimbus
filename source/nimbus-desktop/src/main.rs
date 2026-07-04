mod app;
mod layout;
mod system;
mod user;

//================================================================

rust_i18n::i18n!("asset/locale");
//use nimbus_client::nimbus_common::markdown::Token;

//================================================================

use crate::app::*;

//================================================================

#[tokio::main]
async fn main() -> eframe::Result {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../asset/icon.png")).unwrap();

    let option = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([1024.0, 768.0])
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

/*
fn main() {
    let markdown = Token::parse("#test");
    println!("{markdown:#?}");
}
*/
