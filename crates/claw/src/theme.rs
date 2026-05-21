use ratatui::style::Color;
use serde::Deserialize;
use std::path::Path;

/// Custom color theme loaded from `~/.i-rs-claw/theme.json`.
///
/// All fields are optional — missing fields fall back to built-in defaults.
/// Colors are specified as hex strings like `"#RRGGBB"`.
///
/// # Design Philosophy
/// The default theme follows a "Midnight Command Center" aesthetic:
/// - Deep, rich backgrounds with subtle blue undertones
/// - Electric cyan as the primary accent (like a terminal cursor)
/// - Warm green for user elements (confirms actions, positive feedback)
/// - Amber/gold for warnings and highlights
/// - Carefully calibrated grays to maintain hierarchy without eye strain
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Theme {
    /// Primary accent color (cyan by default)
    /// Used for: assistant messages, active elements, borders
    #[serde(default)]
    pub primary: Option<String>,

    /// Secondary accent color (green by default)
    /// Used for: user messages, success states, positive indicators
    #[serde(default)]
    pub secondary: Option<String>,

    /// Error/warning color (red by default)
    /// Used for: errors, destructive actions
    #[serde(default)]
    #[allow(dead_code)]
    pub error: Option<String>,

    /// Main background color (deep navy by default)
    #[serde(default)]
    pub background: Option<String>,

    /// Primary text color (white by default)
    #[serde(default)]
    #[allow(dead_code)]
    pub text: Option<String>,

    /// Dimmed/muted text color (cool gray by default)
    /// Used for: timestamps, hints, secondary info
    #[serde(default)]
    pub dim_text: Option<String>,

    /// Accent/highlight color (amber/gold by default)
    /// Used for: tool calls, warnings, important highlights
    #[serde(default)]
    pub accent: Option<String>,

    /// Tool call background color
    #[serde(default)]
    #[allow(dead_code)]
    pub tool_bg: Option<String>,

    /// Code block background
    #[serde(default)]
    #[allow(dead_code)]
    pub code_bg: Option<String>,

    /// Selection/highlight background
    #[serde(default)]
    #[allow(dead_code)]
    pub selection_bg: Option<String>,

    /// Border color for inactive elements
    #[serde(default)]
    #[allow(dead_code)]
    pub border: Option<String>,

    /// Sidebar background (slightly different from main bg)
    #[serde(default)]
    #[allow(dead_code)]
    pub sidebar_bg: Option<String>,
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

    /// Primary accent - Electric Cyan (#22d3ee)
    /// Terminal-style accent for active elements
    pub fn primary(&self) -> Color {
        self.primary.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Rgb(34, 211, 238))
    }

    /// Secondary accent - Emerald Green (#34d399)
    /// Warm green for user elements and positive feedback
    pub fn secondary(&self) -> Color {
        self.secondary.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Rgb(52, 211, 153))
    }

    /// Error color - Rose Red (#f87171)
    pub fn error(&self) -> Color {
        self.error.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Rgb(248, 113, 113))
    }

    /// Accent/highlight - Amber (#fbbf24)
    /// For tool calls, warnings, and important elements
    pub fn accent(&self) -> Color {
        self.accent.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Rgb(251, 191, 36))
    }

    /// Dimmed text - Cool Gray (#71717a)
    /// Secondary information, timestamps, hints
    pub fn dim_text(&self) -> Color {
        self.dim_text.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Rgb(113, 113, 122))
    }

    /// Primary text - Near White (#fafafa)
    pub fn text(&self) -> Color {
        self.text.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Rgb(250, 250, 250))
    }

    /// Main background - Deep Navy (#0f0f17)
    pub fn background(&self) -> Color {
        self.background.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Rgb(15, 15, 23))
    }

    /// Chat area background - Slightly darker (#0a0a12)
    #[allow(dead_code)]
    pub fn chat_bg(&self) -> Color {
        Color::Rgb(10, 10, 18)
    }

    /// Tool call background - Dark elevated (#18181f)
    #[allow(dead_code)]
    pub fn tool_bg(&self) -> Color {
        self.tool_bg.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Rgb(24, 24, 31))
    }

    /// Code block background - Very dark (#12121a)
    #[allow(dead_code)]
    pub fn code_bg(&self) -> Color {
        self.code_bg.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Rgb(18, 18, 26))
    }

    /// Selection background - Subtle highlight (#1e1e2e)
    #[allow(dead_code)]
    pub fn selection_bg(&self) -> Color {
        self.selection_bg.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Rgb(30, 30, 46))
    }

    /// Inactive border color - Muted (#27272a)
    #[allow(dead_code)]
    pub fn border(&self) -> Color {
        self.border.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Rgb(39, 39, 42))
    }

    /// Sidebar background - Slightly elevated (#13131a)
    #[allow(dead_code)]
    pub fn sidebar_bg(&self) -> Color {
        self.sidebar_bg.as_ref().and_then(|s| Self::parse_hex(s)).unwrap_or(Color::Rgb(19, 19, 26))
    }

    // ── Convenience helper methods for common style combinations ──

    /// Returns colors suitable for user message bubbles
    #[allow(dead_code)]
    pub fn user_colors(&self) -> (Color, Color) {
        (self.secondary(), Color::Rgb(16, 30, 24))
    }

    /// Returns colors suitable for assistant message bubbles
    #[allow(dead_code)]
    pub fn assistant_colors(&self) -> (Color, Color) {
        (self.primary(), Color::Rgb(12, 20, 30))
    }

    /// Returns colors suitable for tool call blocks
    #[allow(dead_code)]
    pub fn tool_colors(&self) -> (Color, Color) {
        (self.accent(), self.tool_bg())
    }

    /// Returns colors suitable for error messages
    #[allow(dead_code)]
    pub fn error_colors(&self) -> (Color, Color) {
        (self.error(), Color::Rgb(30, 12, 12))
    }
}
