//! The overlay theme: a preset plus optional imported token overrides.
//!
//! The overlays inline the Night Broadcast tokens as their stylesheet
//! defaults. A different theme arrives as a `theme` message carrying the CSS
//! custom properties to override, so switching a theme is a live message and
//! never needs a rebuild.

use std::collections::BTreeMap;

use fgc_core::{DisplaySettings, ThemePreset};
use serde::{Deserialize, Serialize};

use crate::Message;

/// The message type the overlays listen for.
pub const THEME_MESSAGE: &str = "theme";

/// The built-in Light theme's token overrides.
///
/// Only the colour tokens change. Spacing, radii, durations and easing stay
/// with the stylesheet, so a theme never moves a box.
pub const LIGHT_THEME_TOKENS: &[(&str, &str)] = &[
    ("--fgc-surface", "#f4f6fb"),
    ("--fgc-surface-raised", "#ffffff"),
    ("--fgc-surface-inverse", "#11151d"),
    ("--fgc-surface-muted", "#e4e8f1"),
    ("--fgc-line", "#d3d9e4"),
    ("--fgc-line-strong", "#b4bdcd"),
    ("--fgc-line-on-inverse", "#eef2f8"),
    ("--fgc-text", "#11151d"),
    ("--fgc-text-muted", "#586274"),
    ("--fgc-text-faint", "#7c8697"),
    ("--fgc-text-inverse", "#eef2f8"),
    ("--fgc-accent", "#c2410c"),
    ("--fgc-accent-strong", "#e2610a"),
    ("--fgc-accent-soft", "rgb(194 65 12 / 14%)"),
    ("--fgc-on-accent", "#ffffff"),
    ("--fgc-success", "#1f8a5b"),
    ("--fgc-warning", "#a86f12"),
    ("--fgc-danger", "#c93a3a"),
];

/// The `theme` payload.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeDto {
    /// The built-in preset's wire name.
    pub preset: String,
    /// The CSS custom properties to override.
    ///
    /// This is the complete set: an overlay removes any property it applied
    /// earlier and that is missing here, so an empty map restores the
    /// stylesheet defaults.
    pub tokens: BTreeMap<String, String>,
}

/// Builds the effective token overrides for the current display settings.
///
/// The built-in preset contributes its tokens and an imported theme file
/// overrides them, so a file may set only the tokens it cares about.
#[must_use]
pub fn effective_tokens(display: &DisplaySettings) -> BTreeMap<String, String> {
    let mut tokens = BTreeMap::new();
    if display.theme == ThemePreset::Light {
        for (name, value) in LIGHT_THEME_TOKENS {
            tokens.insert((*name).to_owned(), (*value).to_owned());
        }
    }
    if let Some(imported) = &display.theme_tokens {
        for (name, value) in imported {
            tokens.insert(name.clone(), value.clone());
        }
    }
    tokens
}

/// Builds the `theme` payload for the current display settings.
#[must_use]
pub fn theme_dto(display: &DisplaySettings) -> ThemeDto {
    ThemeDto {
        preset: display.theme.as_str().to_owned(),
        tokens: effective_tokens(display),
    }
}

/// Builds the `theme` message for the current display settings.
#[must_use]
pub fn theme_message(display: &DisplaySettings) -> Message<ThemeDto> {
    Message::new(THEME_MESSAGE, theme_dto(display))
}

/// Why a theme file was rejected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThemeError {
    /// The file was not valid JSON.
    Json(String),
    /// The JSON did not hold an object of token names and values.
    Shape(String),
    /// A token name was not a CSS custom property.
    Name(String),
    /// A token value was not a safe CSS value.
    Value(String),
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(message) => write!(formatter, "the theme file is not valid JSON: {message}"),
            Self::Shape(message) => write!(formatter, "{message}"),
            Self::Name(name) => write!(formatter, "'{name}' is not a CSS custom property name"),
            Self::Value(value) => write!(formatter, "'{value}' is not a safe CSS value"),
        }
    }
}

impl std::error::Error for ThemeError {}

/// Parses an imported theme file.
///
/// The file holds either a flat object of token names and values, or an
/// object with a `tokens` object and an optional `name`. Only custom property
/// names and plain colour values are accepted, so a theme file can never pull
/// in a remote resource or run script.
pub fn parse_theme_json(text: &str) -> Result<BTreeMap<String, String>, ThemeError> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|source| ThemeError::Json(source.to_string()))?;

    let object = value
        .as_object()
        .ok_or_else(|| ThemeError::Shape("the theme file must hold a JSON object".to_owned()))?;

    let tokens = match object.get("tokens") {
        Some(tokens) => tokens
            .as_object()
            .ok_or_else(|| ThemeError::Shape("'tokens' must hold a JSON object".to_owned()))?,
        None => object,
    };

    if tokens.is_empty() {
        return Err(ThemeError::Shape(
            "the theme file does not set any tokens".to_owned(),
        ));
    }

    let mut parsed = BTreeMap::new();
    for (name, value) in tokens {
        if !is_custom_property(name) {
            return Err(ThemeError::Name(name.clone()));
        }
        let text = value
            .as_str()
            .ok_or_else(|| ThemeError::Value(name.clone()))?;
        if !is_safe_value(text) {
            return Err(ThemeError::Value(text.to_owned()));
        }
        parsed.insert(name.clone(), text.to_owned());
    }
    Ok(parsed)
}

/// Whether the name is a CSS custom property.
fn is_custom_property(name: &str) -> bool {
    name.starts_with("--")
        && name.len() > 2
        && name[2..]
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
}

/// Whether the value is a plain CSS value that cannot load anything remote.
fn is_safe_value(value: &str) -> bool {
    if value.is_empty() || value.len() > 64 {
        return false;
    }
    let lowered = value.to_ascii_lowercase();
    if lowered.contains("url") || lowered.contains("expression") || lowered.contains("javascript") {
        return false;
    }
    value.chars().all(|character| {
        character.is_ascii_alphanumeric()
            || matches!(
                character,
                '#' | '%' | ',' | '.' | '(' | ')' | '/' | '-' | '_' | ' '
            )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn light_display() -> DisplaySettings {
        DisplaySettings {
            theme: ThemePreset::Light,
            ..DisplaySettings::default()
        }
    }

    #[test]
    fn night_broadcast_has_no_overrides() {
        let tokens = effective_tokens(&DisplaySettings::default());
        assert!(tokens.is_empty());
    }

    #[test]
    fn the_light_theme_overrides_the_colour_tokens() {
        let tokens = effective_tokens(&light_display());
        assert_eq!(
            tokens.get("--fgc-surface").map(String::as_str),
            Some("#f4f6fb")
        );
        assert_eq!(
            tokens.get("--fgc-text").map(String::as_str),
            Some("#11151d")
        );
        assert!(!tokens.contains_key("--fgc-space-1"));
    }

    #[test]
    fn an_imported_theme_overrides_the_preset() {
        let mut display = light_display();
        let mut imported = BTreeMap::new();
        imported.insert("--fgc-surface".to_owned(), "#010203".to_owned());
        display.theme_tokens = Some(imported);

        let tokens = effective_tokens(&display);
        assert_eq!(
            tokens.get("--fgc-surface").map(String::as_str),
            Some("#010203")
        );
        assert_eq!(
            tokens.get("--fgc-text").map(String::as_str),
            Some("#11151d")
        );
    }

    #[test]
    fn the_theme_message_names_the_preset() {
        let message = theme_message(&light_display());
        assert_eq!(message.kind, "theme");
        assert_eq!(message.data.preset, "light");
    }

    #[test]
    fn a_flat_theme_file_parses() {
        let parsed =
            parse_theme_json(r##"{"--fgc-surface": "#ffffff"}"##).expect("flat theme file");
        assert_eq!(
            parsed.get("--fgc-surface").map(String::as_str),
            Some("#ffffff")
        );
    }

    #[test]
    fn a_nested_theme_file_parses() {
        let parsed =
            parse_theme_json(r##"{"name": "Daylight", "tokens": {"--fgc-text": "#11151d"}}"##)
                .expect("nested theme file");
        assert_eq!(
            parsed.get("--fgc-text").map(String::as_str),
            Some("#11151d")
        );
    }

    #[test]
    fn unsafe_theme_files_are_rejected() {
        assert!(parse_theme_json("not json").is_err());
        assert!(parse_theme_json("[]").is_err());
        assert!(parse_theme_json("{}").is_err());
        assert!(parse_theme_json(r##"{"color": "#fff"}"##).is_err());
        assert!(
            parse_theme_json(r##"{"--fgc-surface": "url(https://example.com/x.png)"}"##).is_err()
        );
        assert!(parse_theme_json(r#"{"--fgc-surface": 12}"#).is_err());
    }
}
