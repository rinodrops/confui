#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod config;
mod i18n;
mod migrate;
mod schema;
mod single_instance;
mod theme;
mod validation;

use std::path::PathBuf;

/// Default config path when no CLI argument is given.
///
/// Packaged builds: beside the executable (Windows/Linux) or inside the
/// `.app` bundle's `Resources/` (macOS). Development: `demo/config.toml`.
fn default_config_path() -> PathBuf {
    if let Some(bundled) = bundled_config_path() {
        return bundled;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("demo/config.toml")
}

#[cfg(target_os = "macos")]
fn bundled_config_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let macos_dir = exe.parent()?;
    if macos_dir.file_name().is_some_and(|n| n == "MacOS") {
        return Some(macos_dir.parent()?.join("Resources/config.toml"));
    }
    None
}

#[cfg(not(target_os = "macos"))]
fn bundled_config_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let path = exe.parent()?.join("config.toml");
    path.exists().then_some(path)
}

fn main() {
    let config_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_config_path);

    if !single_instance::acquire(&config_path) {
        return;
    }

    let schema = schema::load().expect("Failed to parse schema.toml");
    let mut config = config::ConfigStore::load(&config_path)
        .unwrap_or_else(|e| panic!("Failed to load config {config_path:?}: {e}"));

    // Migrate the config to the schema's target version before the window and
    // file watcher start. Runs before single-instance-owned edits so the file
    // is written once, up front.
    if let Err(e) = migrate::run(&schema, &mut config) {
        eprintln!("warning: config migration failed: {e}");
    }

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
        "settings",
        options,
        Box::new(|cc| Ok(Box::new(app::SettingsApp::new(cc, schema, config)))),
    )
    .expect("eframe failed");
}
