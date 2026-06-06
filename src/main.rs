mod app;
mod config;
mod i18n;
mod schema;
mod theme;

use std::path::PathBuf;

fn main() {
    let config_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            // Standalone dev fallback; pairs with the demo/schema.toml build.rs default.
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("demo/config.toml")
        });

    let schema = schema::load().expect("Failed to parse schema.toml");
    let config = config::ConfigStore::load(&config_path)
        .unwrap_or_else(|e| panic!("Failed to load config {config_path:?}: {e}"));

    // Resolve the UI language once at startup; it does not change at runtime.
    // When `lang = "os"`, prefer the parent application's own language setting
    // (read from the config file at the schema's `lang_key`) so the settings
    // window matches the parent, then fall back to native OS detection.
    let parent_lang = schema
        .lang_key
        .as_deref()
        .and_then(|key| config.get_str(key))
        .map(str::to_owned);
    i18n::init(i18n::Lang::resolve(&schema.lang, parent_lang.as_deref()), &schema.ui_strings);

    let viewport = egui::ViewportBuilder::default()
        .with_title(i18n::t().window_title)
        .with_inner_size(app::compute_window_size(&schema));
    // macOS: the dock icon comes from the .app bundle; with_icon() is a no-op.
    #[cfg(not(target_os = "macos"))]
    let viewport = viewport.with_icon(std::sync::Arc::new(
        eframe::icon_data::from_png_bytes(include_bytes!("../assets/appicon.png"))
            .expect("failed to load app icon"),
    ));

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "confui",
        options,
        Box::new(|cc| Ok(Box::new(app::ConfUiApp::new(cc, schema, config)))),
    )
    .expect("eframe failed");
}
