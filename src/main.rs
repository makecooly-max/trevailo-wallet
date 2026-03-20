mod app;
mod node_client;
mod keystore_manager;
mod ui;

use app::TrevailoWallet;
use tracing_subscriber::EnvFilter;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Trevailo Wallet")
            .with_inner_size([1080.0, 640.0])
            .with_min_inner_size([900.0, 520.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Trevailo Wallet",
        options,
        Box::new(|cc| Box::new(TrevailoWallet::new(cc))),
    )
}

/// Загружает иконку из PNG, встроенного в бинарник через include_bytes!.
/// Работает одинаково на Linux и Windows — egui сам передаёт её в заголовок окна.
fn load_icon() -> egui::IconData {
    // PNG встраивается в бинарник на этапе компиляции — никаких внешних файлов не нужно
    let png_bytes: &[u8] = include_bytes!("../assets/icon.png");

    let image = image::load_from_memory(png_bytes)
        .expect("Failed to load icon.png — проверь путь assets/icon.png")
        .into_rgba8();

    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    egui::IconData { rgba, width, height }
}