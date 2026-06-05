//! Workspace settings exposed to the GUI (app-level prefs, bypass Operation/apply).
use serde::{Deserialize, Serialize};
use specta::Type;

/// The user's theme preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum ThemeChoice {
    System,
    Light,
    Dark,
}

impl ThemeChoice {
    /// The persisted string form (matches the `workspace_settings.value` column).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ThemeChoice::System => "system",
            ThemeChoice::Light => "light",
            ThemeChoice::Dark => "dark",
        }
    }

    /// Parse the persisted string form, returning `None` for unknown values.
    #[must_use]
    pub fn parse(s: &str) -> Option<ThemeChoice> {
        match s {
            "system" => Some(ThemeChoice::System),
            "light" => Some(ThemeChoice::Light),
            "dark" => Some(ThemeChoice::Dark),
            _ => None,
        }
    }
}

/// App-level settings surfaced to the GUI.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct Settings {
    pub theme: ThemeChoice,
}
