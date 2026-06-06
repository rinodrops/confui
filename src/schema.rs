use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer};

// build.rs copies the schema into OUT_DIR/schema.toml at compile time.
// Override the source path with: CONFUI_SCHEMA=/path/to/schema.toml cargo build
const SCHEMA_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/schema.toml"));

pub fn load() -> Result<Schema, toml::de::Error> {
    let src = std::str::from_utf8(SCHEMA_BYTES).expect("schema.toml must be valid UTF-8");
    toml::from_str(src)
}

// ---------------------------------------------------------------------------
// LocalizedString

/// A user-facing string in the schema that may be authored either as a single
/// bare value (used regardless of language) or as a per-language table, e.g.
///
/// ```toml
/// label = "Name"                       # language-agnostic
/// label = { en = "Name", ja = "名前" } # localized
/// ```
///
/// The active UI language is resolved once at startup in [`crate::i18n`]; this
/// type queries it at render time via [`LocalizedString::get`], so the same
/// schema serves every supported language without duplicating the structure.
#[derive(Debug, Clone, Default)]
pub struct LocalizedString {
    /// Value returned when the active language has no dedicated entry. Equals the
    /// bare string when the field was authored without a per-language table.
    fallback: String,
    /// Per-language variants keyed by language code (e.g. `"en"`, `"ja"`).
    variants: BTreeMap<String, String>,
}

impl LocalizedString {
    /// Returns the variant for the currently active UI language, falling back to
    /// the language-agnostic value (or the `en` entry) when no exact match exists.
    pub fn get(&self) -> &str {
        let code = crate::i18n::active_lang_code();
        self.variants
            .get(code)
            .map(String::as_str)
            .unwrap_or(&self.fallback)
    }
}

impl<'de> Deserialize<'de> for LocalizedString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Accept either a bare string or a `{ <lang> = "..." }` table.
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Plain(String),
            Map(BTreeMap<String, String>),
        }

        Ok(match Raw::deserialize(deserializer)? {
            Raw::Plain(s) => LocalizedString {
                fallback: s,
                variants: BTreeMap::new(),
            },
            Raw::Map(map) => {
                // Prefer the English entry as the fallback, else the first entry.
                let fallback = map
                    .get("en")
                    .or_else(|| map.values().next())
                    .cloned()
                    .unwrap_or_default();
                LocalizedString {
                    fallback,
                    variants: map,
                }
            }
        })
    }
}

// ---------------------------------------------------------------------------
// SaveButtonMode

/// Controls whether a Save button is shown or changes are written automatically.
#[derive(Debug, Deserialize, Default, PartialEq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SaveButtonMode {
    /// Platform default: show a Save button on Windows/Linux, auto-save on macOS.
    #[default]
    Os,
    /// Always show a Save button; changes are written only when it is pressed.
    Show,
    /// Never show a Save button; changes are written immediately on every edit.
    Hide,
}

// ---------------------------------------------------------------------------
// Top-level

#[derive(Debug, Deserialize)]
pub struct Schema {
    pub tabs: Vec<Tab>,
    /// Icon variant to use for Material Symbols ("rounded" | "outlined" | "sharp").
    /// Only informational for the schema consumer; the actual font is selected at
    /// build time via `make icons [ICON_STYLE=…]`. Parsed only to validate and
    /// document the schema; never read at runtime.
    #[serde(default)]
    #[allow(dead_code)]
    pub icon_style: Option<String>,
    /// Which color variant to display.
    /// `"os"` (default, follow the OS) | `"light"` | `"dark"`
    #[serde(default)]
    pub theme: ThemeMode,
    /// UI chrome language.
    /// `"os"` (default, follow the OS locale) | BCP-47 code (e.g. `"en"`, `"ja"`, `"de"`)
    #[serde(default)]
    pub lang: LangMode,
    /// Dot-separated path to the config key that stores the parent application's
    /// language (e.g. `"display.language"`). When `lang = "os"`, ConfUI reads this
    /// value from the config file and matches it, so the settings window follows
    /// the same language as the parent app. Falls back to native OS detection when
    /// the key is absent or unset.
    #[serde(default)]
    pub lang_key: Option<String>,
    /// Light-variant color overrides. These flat top-level fields
    /// (`background_color`, `accent_color`, …) override the built-in light
    /// palette and are also used in Light Mode when `theme` follows the OS.
    #[serde(flatten)]
    pub colors_light: ColorOverrides,
    /// Dark-variant color overrides, supplied via a `[dark]` table. Override the
    /// built-in dark palette; ignored entirely in Light Mode.
    #[serde(default, rename = "dark")]
    pub colors_dark: ColorOverrides,
    /// Save button visibility / auto-save behavior.
    /// `"os"` (default) | `"show"` | `"hide"`
    #[serde(default)]
    pub save_button: SaveButtonMode,
    /// Optional overrides for built-in UI chrome strings (`[ui_strings]` table).
    #[serde(default)]
    pub ui_strings: UiStrings,
    /// Width of the content area in logical pixels (excludes title bar, tab bar,
    /// and bottom button bar). Defaults to 700 when omitted.
    #[serde(default)]
    pub content_width: Option<f32>,
    /// Height of the content area in logical pixels (excludes title bar, tab bar,
    /// and bottom button bar). Defaults to 370 when omitted.
    #[serde(default)]
    pub content_height: Option<f32>,
}

// ---------------------------------------------------------------------------
// Theme / language preferences

/// Which color variant the UI should display.
#[derive(Debug, Deserialize, Default, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    /// Follow the operating system's light/dark setting (default).
    #[default]
    Os,
    Light,
    Dark,
}

/// Which language the built-in UI chrome should use.
///
/// `"os"` (default) follows the OS locale. Any BCP-47 base language code
/// (e.g. `"en"`, `"ja"`, `"de"`) selects that language explicitly; codes
/// without a built-in translation fall back to English unless `[ui_strings]`
/// supplies overrides.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum LangMode {
    /// Follow the operating system locale (default).
    #[default]
    Os,
    /// Use the specified BCP-47 language code (e.g. `"en"`, `"ja"`, `"de"`).
    Fixed(String),
}

impl<'de> Deserialize<'de> for LangMode {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(if s.eq_ignore_ascii_case("os") {
            LangMode::Os
        } else {
            LangMode::Fixed(s)
        })
    }
}

/// Optional overrides for built-in UI chrome strings, supplied via a
/// `[ui_strings]` table in the schema. Each field replaces the corresponding
/// built-in string for the active language. Omitted fields keep the built-in
/// translation (or English when no built-in translation exists for the language).
#[derive(Debug, Deserialize, Default)]
pub struct UiStrings {
    pub ok: Option<String>,
    pub apply: Option<String>,
    pub no_sections: Option<String>,
    pub add_section: Option<String>,
    pub section_name_label: Option<String>,
    pub add: Option<String>,
    pub cancel: Option<String>,
    pub enter_name: Option<String>,
    pub delete: Option<String>,
    /// Template for the delete confirmation dialog; use `{}` as a placeholder
    /// for the item name (e.g. `"Delete \"{}\"?"`).
    pub delete_confirm: Option<String>,
    pub browse: Option<String>,
    pub all_files: Option<String>,
    pub click_to_input: Option<String>,
    pub press_key: Option<String>,
    pub clear: Option<String>,
    pub show: Option<String>,
    pub hide: Option<String>,
    /// Body text of the external-change conflict dialog.
    pub file_changed: Option<String>,
    /// "Reload" button label in the conflict dialog.
    pub reload: Option<String>,
    /// "Keep Editing" button label in the conflict dialog.
    pub keep_editing: Option<String>,
    /// Window title bar text.
    pub window_title: Option<String>,
}

/// High-level color overrides shared by the light and dark variants. Each field
/// is an optional CSS hex string; `None` falls back to the built-in palette.
#[derive(Debug, Deserialize, Default)]
pub struct ColorOverrides {
    /// Panel and window background color (CSS hex, e.g. `"#ffffff"`).
    #[serde(default)]
    pub background_color: Option<String>,
    /// Accent color for the selected tab and interactive highlights (CSS hex).
    #[serde(default)]
    pub accent_color: Option<String>,
    /// Primary text color for labels and input text (CSS hex).
    #[serde(default)]
    pub text_color: Option<String>,
    /// Color of unselected tab icons and labels (CSS hex).
    #[serde(default)]
    pub tab_text_color: Option<String>,
    /// Background fill of the selected-tab highlight (CSS hex).
    /// Defaults to the accent color at reduced opacity when omitted.
    #[serde(default)]
    pub selection_bg_color: Option<String>,
}

// ---------------------------------------------------------------------------
// Tab

#[derive(Debug, Deserialize)]
pub struct Tab {
    /// Stable tab identifier. Parsed only to enforce its presence and document
    /// the schema; tabs are addressed by index at runtime, so it is never read.
    #[allow(dead_code)]
    pub id: String,
    pub label: LocalizedString,
    /// Material Symbol icon name (e.g. `"settings"`, `"translate"`).
    /// Shown in the tab bar when the icon font was embedded at build time.
    #[serde(default)]
    pub icon: Option<String>,
    /// Per-tab content width override in logical pixels.
    /// Falls back to `Schema::content_width` when omitted.
    #[serde(default)]
    pub content_width: Option<f32>,
    /// Per-tab content height override in logical pixels.
    /// Falls back to `Schema::content_height` when omitted.
    #[serde(default)]
    pub content_height: Option<f32>,
    /// Flat field list (mutually exclusive with `section_map`).
    pub fields: Option<Vec<Field>>,
    /// Sub-section map (mutually exclusive with `fields`).
    pub section_map: Option<SectionMap>,
}

// ---------------------------------------------------------------------------
// SectionMap

/// Visual style of the sub-section tab bar inside a `section_map`.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum SectionTabStyle {
    /// Underline style: accent-colored thick underline on active tab,
    /// thin grey underline on inactive tabs (default).
    #[default]
    Underline,
    /// Segmented-control style: all tabs rendered as a single pill-based
    /// segmented control (best for a small, fixed number of sections).
    Segmented,
}

#[derive(Debug, Deserialize)]
pub struct SectionMap {
    pub key_prefix: String,
    pub allow_add_remove: bool,
    pub fields: Vec<Field>,
    /// Visual style of the sub-tab bar. Defaults to `"underline"`.
    #[serde(default)]
    pub tab_style: SectionTabStyle,
    /// Maximum width of the segmented control (px). Has no effect on underline style.
    pub max_width: Option<f32>,
}

// ---------------------------------------------------------------------------
// Field

#[derive(Debug, Deserialize)]
pub struct Field {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub label: LocalizedString,
    /// Declared TOML value type. Parsed (and validated against `FieldType`) to
    /// catch typos in the schema, but rendering is driven by `widget`, so the
    /// value itself is never read at runtime.
    #[serde(rename = "type", default)]
    #[allow(dead_code)]
    pub field_type: FieldType,
    pub widget: WidgetKind,
    /// Generic one-line hint shown below the widget.
    pub hint: Option<LocalizedString>,
    /// Hint shown for the "direct" variant of an exclusive_radio field.
    pub hint_direct: Option<LocalizedString>,
    /// Hint shown for the "env" variant of an exclusive_radio field.
    pub hint_env: Option<LocalizedString>,
    /// Static option list for `select` widgets.
    pub options: Option<Vec<String>>,
    /// Reference to a section_map whose keys become the option list at render time.
    pub options_from: Option<String>,
    /// Inline description placed to the right of the widget (non-bold, same font size).
    /// Intended for checkbox / toggle rows where the description belongs beside the control.
    /// Distinct from `hint`, which is rendered below the widget on a separate line.
    pub sublabel: Option<LocalizedString>,
    /// Minimum widget width (logical px). Applies to `text_input`, `multiline`,
    /// `segmented_control`, and similar widgets. No constraint when omitted.
    pub min_width: Option<f32>,
    /// Maximum widget width (logical px). Applies to `text_input`, `multiline`,
    /// `segmented_control`, and similar widgets. Full available width when omitted.
    pub max_width: Option<f32>,
    /// Number of visible rows for `multiline` widgets. Defaults to 4 when omitted.
    pub rows: Option<usize>,
    /// Minimum numeric value for `slider` and `drag_value` widgets.
    pub min: Option<f64>,
    /// Maximum numeric value for `slider` and `drag_value` widgets.
    pub max: Option<f64>,
    /// Step increment for `slider` and `drag_value` widgets. Defaults to 1 when omitted.
    pub step: Option<f64>,
    /// Suffix string appended to the displayed number (e.g. `" px"`, `" pt"`).
    pub suffix: Option<String>,
    /// Present only when `widget = "exclusive_radio"`.
    pub exclusive: Option<ExclusiveConfig>,

    // --- file_path ---
    /// If `true`, the file dialog picks a directory instead of a file.
    #[serde(default)]
    pub is_directory: bool,
    /// Filter label shown in the file dialog (e.g. `"TOML files"`).
    /// Only consumed when building the dialog filter, which is compiled out on
    /// macOS (NSOpenPanel has no usable filter dropdown), so it is dead there.
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    pub file_filter: Option<LocalizedString>,
    /// File extensions shown by the filter (e.g. `["toml", "txt"]`).
    /// Unused on macOS for the same reason as `file_filter`.
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    pub file_extensions: Option<Vec<String>>,

    // --- key_value_map ---
    /// Header text for the key column. Defaults to `"Key"`.
    pub key_label: Option<LocalizedString>,
    /// Header text for the value column. Defaults to `"Value"`.
    pub value_label: Option<LocalizedString>,
}

// ---------------------------------------------------------------------------
// Enums

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum WidgetKind {
    TextInput,
    SecretInput,
    Multiline,
    Checkbox,
    Toggle,
    Select,
    SegmentedControl,
    ExclusiveRadio,
    Hotkey,
    /// Horizontal slider with numeric value display.
    Slider,
    /// Drag-to-change numeric input (click to type directly).
    DragValue,
    /// A full-width horizontal rule. No key, label, or type needed in the schema.
    Separator,
    /// Text input + "Browse…" button that opens a native file/directory dialog.
    FilePath,
    /// Color swatch that opens an inline color picker popup. Stored as `"#rrggbb"` in TOML.
    ColorPicker,
    /// Editable table of string key → string value pairs.
    KeyValueMap,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    #[default]
    String,
    Bool,
    Number,
}

// ---------------------------------------------------------------------------
// ExclusiveConfig / ExclusiveVariant

#[derive(Debug, Deserialize, Clone)]
pub struct ExclusiveConfig {
    pub mode_key: String,
    pub mode_default: String,
    pub variants: Vec<ExclusiveVariant>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExclusiveVariant {
    pub value: String,
    pub label: LocalizedString,
    pub field_key: String,
    pub widget: WidgetKind,
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
