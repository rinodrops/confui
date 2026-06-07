//! Runtime schema loading and icon assets embedded at build time.
//!
//! Types, parsing, and validation live in the [`settings_schema`] crate so the
//! settings-schema-checker and `build.rs` can share the same logic.

pub use settings_schema::*;

// build.rs copies the schema into OUT_DIR/schema.toml at compile time.
// Override the source path with: SETTINGS_SCHEMA=/path/to/schema.toml cargo build
const SCHEMA_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/schema.toml"));

/// Load the schema embedded at compile time.
pub fn load() -> Result<Schema, toml::de::Error> {
    let src = std::str::from_utf8(SCHEMA_BYTES).expect("schema.toml must be valid UTF-8");
    parse(src)
}

// ---------------------------------------------------------------------------
// Icon font assets (embedded at build time when `make icons` was run)

/// Raw bytes of the Material Symbols TTF embedded at compile time.
#[cfg(has_icons)]
pub const ICON_FONT: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/icons.ttf"));

/// Contents of the `.codepoints` file: `icon_name HEXCODEPOINT` per line.
#[cfg(has_icons)]
pub const ICON_CODEPOINTS: &str =
    include_str!(concat!(env!("OUT_DIR"), "/icons.codepoints"));
