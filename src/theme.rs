//! Centralized color palette with light / dark variants.
//!
//! Historically every widget hard-coded its own `Color32` literals, which made
//! Dark Mode impossible without touching dozens of call sites. All semantic
//! colors now live in [`Palette`]. Light and dark base palettes are provided by
//! [`Palette::light`] / [`Palette::dark`]; the schema can override the five
//! high-level colors (background / accent / text / tab text / selection) on top
//! of whichever base is active.

use std::cell::Cell;

use egui::Color32;

use crate::schema::{Schema, ThemeMode};

/// Resolved theme variant (after combining the schema preference with the OS).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Light,
    Dark,
}

/// Every semantic color used by the UI. Widgets read from here instead of
/// inlining `Color32` literals so that a single struct swap re-themes the app.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    /// Panel and window background fill.
    pub bg: Color32,
    /// Raised control surfaces on top of `bg` (toggle knobs, selected segment,
    /// checkbox interior, table rows).
    pub surface: Color32,
    /// Primary text color (labels, input text).
    pub text: Color32,
    /// Secondary / inactive text (inactive sub-tabs).
    pub muted_text: Color32,
    /// Color of unselected top-level tab icons and labels.
    pub tab_text: Color32,
    /// Placeholder text inside empty inputs.
    pub placeholder: Color32,
    /// Accent color for selected tabs and interactive highlights.
    pub accent: Color32,
    /// Text / marks drawn on top of an `accent` fill (e.g. checkmark, button text).
    pub on_accent: Color32,
    /// Background fill of the selected-tab highlight.
    pub selection_bg: Color32,
    /// Idle border of input fields.
    pub field_border: Color32,
    /// Track fill of segmented controls and generic hover backgrounds.
    pub control_track: Color32,
    /// Border of a segmented-control track / selected segment.
    pub control_track_border: Color32,
    /// "Off" track of toggles and the rail of sliders.
    pub control_off: Color32,
    /// Thin separator line between field groups.
    pub separator: Color32,
    /// Baseline underline under inactive sub-tabs.
    pub divider: Color32,
    /// Header row background of `key_value_map` tables.
    pub header_bg: Color32,
    /// Strong icon color (hovered close / reveal icons).
    pub icon: Color32,
    /// Idle icon color.
    pub icon_weak: Color32,
    /// Disabled icon color.
    pub icon_disabled: Color32,
    /// Muted icon / secondary glyph color.
    pub icon_muted: Color32,
    /// Error message color.
    pub error: Color32,
    /// Soft drop shadow (premultiplied).
    pub shadow: Color32,
    /// Stronger drop shadow for raised knobs (premultiplied).
    pub shadow_strong: Color32,
}

impl Palette {
    /// Light base palette (matches the original macOS Light Mode appearance).
    pub fn light() -> Self {
        Self {
            bg: Color32::WHITE,
            surface: Color32::WHITE,
            text: Color32::BLACK,
            muted_text: Color32::from_rgb(110, 110, 110),
            tab_text: Color32::from_rgb(128, 128, 128),
            placeholder: Color32::from_rgb(180, 180, 180),
            accent: Color32::from_rgb(0, 155, 255),
            on_accent: Color32::WHITE,
            selection_bg: Color32::TRANSPARENT, // filled in by `finalize`
            field_border: Color32::from_rgb(210, 210, 210),
            control_track: Color32::from_rgb(230, 230, 230),
            control_track_border: Color32::from_rgb(200, 200, 200),
            control_off: Color32::from_rgb(210, 210, 210),
            separator: Color32::from_rgb(243, 243, 243),
            divider: Color32::from_rgb(220, 220, 220),
            header_bg: Color32::from_rgb(246, 246, 246),
            icon: Color32::from_rgb(60, 60, 60),
            icon_weak: Color32::from_rgb(170, 170, 170),
            icon_disabled: Color32::from_rgb(180, 180, 180),
            icon_muted: Color32::from_rgb(150, 150, 150),
            error: Color32::from_rgb(200, 0, 0),
            shadow: Color32::from_rgba_premultiplied(0, 0, 0, 20),
            shadow_strong: Color32::from_rgba_premultiplied(0, 0, 0, 25),
        }
    }

    /// Dark base palette. Values are tuned to mirror the light palette's contrast
    /// relationships rather than to match any specific OS exactly.
    pub fn dark() -> Self {
        Self {
            bg: Color32::from_rgb(30, 30, 30),
            surface: Color32::from_rgb(58, 58, 60),
            text: Color32::from_rgb(228, 228, 228),
            muted_text: Color32::from_rgb(150, 150, 150),
            tab_text: Color32::from_rgb(140, 140, 140),
            placeholder: Color32::from_rgb(110, 110, 110),
            accent: Color32::from_rgb(0, 155, 255),
            on_accent: Color32::WHITE,
            selection_bg: Color32::TRANSPARENT, // filled in by `finalize`
            field_border: Color32::from_rgb(80, 80, 80),
            control_track: Color32::from_rgb(64, 64, 66),
            control_track_border: Color32::from_rgb(96, 96, 98),
            control_off: Color32::from_rgb(80, 80, 82),
            separator: Color32::from_rgb(55, 55, 55),
            divider: Color32::from_rgb(72, 72, 72),
            header_bg: Color32::from_rgb(45, 45, 47),
            icon: Color32::from_rgb(220, 220, 220),
            icon_weak: Color32::from_rgb(120, 120, 120),
            icon_disabled: Color32::from_rgb(90, 90, 90),
            icon_muted: Color32::from_rgb(140, 140, 140),
            error: Color32::from_rgb(255, 110, 110),
            shadow: Color32::from_rgba_premultiplied(0, 0, 0, 60),
            shadow_strong: Color32::from_rgba_premultiplied(0, 0, 0, 90),
        }
    }

    /// Builds the active palette: pick the light or dark base, apply the schema's
    /// high-level color overrides for that variant, then derive `selection_bg`
    /// when the schema does not specify it.
    pub fn from_schema(schema: &Schema, variant: Variant) -> Self {
        let (mut pal, overrides) = match variant {
            Variant::Light => (Self::light(), &schema.colors_light),
            Variant::Dark => (Self::dark(), &schema.colors_dark),
        };

        if let Some(c) = overrides.background_color.as_deref().and_then(parse_hex_color) {
            pal.bg = c;
            pal.surface = c;
        }
        if let Some(c) = overrides.accent_color.as_deref().and_then(parse_hex_color) {
            pal.accent = c;
        }
        if let Some(c) = overrides.text_color.as_deref().and_then(parse_hex_color) {
            pal.text = c;
        }
        if let Some(c) = overrides.tab_text_color.as_deref().and_then(parse_hex_color) {
            pal.tab_text = c;
        }

        // selection_bg: explicit override wins, otherwise derive from the accent.
        pal.selection_bg = overrides
            .selection_bg_color
            .as_deref()
            .and_then(parse_hex_color)
            .unwrap_or_else(|| match variant {
                Variant::Light => pal.accent.gamma_multiply(0.15),
                Variant::Dark => pal.accent.gamma_multiply(0.35),
            });

        pal
    }
}

/// Resolves which [`Variant`] to use given the schema preference and the OS theme.
pub fn resolve_variant(mode: ThemeMode, system: Option<egui::Theme>) -> Variant {
    match mode {
        ThemeMode::Light => Variant::Light,
        ThemeMode::Dark => Variant::Dark,
        ThemeMode::Os => match system.unwrap_or(egui::Theme::Light) {
            egui::Theme::Dark => Variant::Dark,
            egui::Theme::Light => Variant::Light,
        },
    }
}

// ---------------------------------------------------------------------------
// Current palette (thread-local)
//
// The UI runs single-threaded and the palette is fixed for the duration of a
// frame, so the active palette is stashed in a thread-local cell that widgets
// read through `current()`. This keeps every render function free of an extra
// palette parameter.

thread_local! {
    static CURRENT: Cell<Palette> = Cell::new(Palette::light());
}

/// Stores `pal` as the active palette for the current frame. Call once at the
/// top of the render loop, before any widget is drawn.
pub fn set_current(pal: Palette) {
    CURRENT.with(|c| c.set(pal));
}

/// Returns the palette set by the most recent [`set_current`] call.
pub fn current() -> Palette {
    CURRENT.with(|c| c.get())
}


/// Parses a CSS hex color (`#rrggbb` or `#rrggbbaa`) into a [`Color32`].
/// Returns `None` if the string is not a valid hex color.
pub fn parse_hex_color(hex: &str) -> Option<Color32> {
    let h = hex.trim_start_matches('#');
    let byte = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
    match h.len() {
        6 => Some(Color32::from_rgb(byte(0)?, byte(2)?, byte(4)?)),
        8 => Some(Color32::from_rgba_unmultiplied(
            byte(0)?, byte(2)?, byte(4)?, byte(6)?,
        )),
        _ => None,
    }
}
