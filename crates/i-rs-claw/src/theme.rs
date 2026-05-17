use ratatui::style::Color;
use serde::Deserialize;
use std::path::Path;

/// Custom color theme loaded from `~/.i-rs-claw/theme.json`.
///
/// All fields are optional — missing fields fall back to built-in defaults.
/// Colors are specified as hex strings like `"#RRGGBB"`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Theme {
    #[serde(default)]
    pub primary: Option<String>,    // Cyan equivalent
    #[serde(default)]
    pub secondary: Option<String>,  // Green equivalent
    #[serde(default)]
    #[allow(dead_code)]
    pub error: Option<String>,      // Red equivalent
    #[serde(default)]
    pub background: Option<String>, // Dark background
    #[serde(default)]
    #[allow(dead_code)]
    pub text: Option<String>,       // White equivalent
    #[serde(default)]
    pub dim_text: Option<String>,   // Gray equivalent
    #[serde(default)]
    pub accent: Option<String>,     // Yellow equivalent
}

impl Theme {
    /// Load theme from JSON file. Returns Default (empty) on failure.
    pub fn load(path: &Path) -> Self {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };
        serde_json::from_str(&content).unwrap_or_default()
    }

    /// Parse a hex color string like `"#RRGGBB"` into ratatui Color.
    fn parse_hex(s: &str) -> Option<Color> {
        let s = s.trim_start_matches('#');
        if s.len() == 6 {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(Color::Rgb(r, g, b))
        } else {
            None
        }
    }

    pub fn primary(&self) -> Color {
        self.primary.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Cyan)
    }
    pub fn secondary(&self) -> Color {
        self.secondary.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Green)
    }
    #[allow(dead_code)]
    pub fn error(&self) -> Color {
        self.error.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Red)
    }
    pub fn accent(&self) -> Color {
        self.accent.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Yellow)
    }
    pub fn dim_text(&self) -> Color {
        self.dim_text.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Rgb(80, 80, 100))
    }
    #[allow(dead_code)]
    pub fn text(&self) -> Color {
        self.text.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::White)
    }
    pub fn background(&self) -> Color {
        self.background.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Rgb(25, 25, 42))
    }
    /// Background color for chat area (slightly different from title background).
    #[allow(dead_code)]
    pub fn chat_bg(&self) -> Color {
        Color::Rgb(12, 12, 20)
    }
}
