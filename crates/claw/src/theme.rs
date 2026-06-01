use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Theme {
    #[serde(default)]
    pub primary: Option<String>,
    #[serde(default)]
    pub secondary: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub error: Option<String>,
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub text: Option<String>,
    #[serde(default)]
    pub dim_text: Option<String>,
    #[serde(default)]
    pub accent: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub tool_bg: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub code_bg: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub selection_bg: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub border: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub sidebar_bg: Option<String>,
}

pub struct ThemePreset {
    pub name: &'static str,
    pub label: &'static str,
    pub primary: &'static str,
    pub secondary: &'static str,
    pub error: &'static str,
    pub background: &'static str,
    pub text: &'static str,
    pub dim_text: &'static str,
    pub accent: &'static str,
    pub tool_bg: &'static str,
    pub code_bg: &'static str,
    pub selection_bg: &'static str,
    pub border: &'static str,
    pub sidebar_bg: &'static str,
}

pub static BUILT_IN_THEMES: &[ThemePreset] = &[
    ThemePreset {
        name: "midnight",
        label: "午夜",
        primary: "#22d3ee",
        secondary: "#34d399",
        error: "#f87171",
        background: "#0f0f17",
        text: "#fafafa",
        dim_text: "#71717a",
        accent: "#fbbf24",
        tool_bg: "#18181f",
        code_bg: "#12121a",
        selection_bg: "#1e1e2e",
        border: "#27272a",
        sidebar_bg: "#13131a",
    },
    ThemePreset {
        name: "ocean",
        label: "深海",
        primary: "#38bdf8",
        secondary: "#2dd4bf",
        error: "#fb7185",
        background: "#0c1222",
        text: "#e2e8f0",
        dim_text: "#64748b",
        accent: "#fbbf24",
        tool_bg: "#162032",
        code_bg: "#0f172a",
        selection_bg: "#1e3a5f",
        border: "#334155",
        sidebar_bg: "#111b2e",
    },
    ThemePreset {
        name: "forest",
        label: "森林",
        primary: "#4ade80",
        secondary: "#a3e635",
        error: "#f87171",
        background: "#0a110d",
        text: "#e2e8e0",
        dim_text: "#6b7b6e",
        accent: "#facc15",
        tool_bg: "#132118",
        code_bg: "#0e1a13",
        selection_bg: "#1a3325",
        border: "#2d3e32",
        sidebar_bg: "#0f1c14",
    },
    ThemePreset {
        name: "sunset",
        label: "日落",
        primary: "#fb923c",
        secondary: "#f472b6",
        error: "#ef4444",
        background: "#1a0f0f",
        text: "#fef2f2",
        dim_text: "#a8807a",
        accent: "#fbbf24",
        tool_bg: "#261818",
        code_bg: "#1f1212",
        selection_bg: "#3d2020",
        border: "#4a3030",
        sidebar_bg: "#201414",
    },
    ThemePreset {
        name: "rose",
        label: "玫瑰",
        primary: "#f472b6",
        secondary: "#c084fc",
        error: "#f87171",
        background: "#150c14",
        text: "#fdf2f8",
        dim_text: "#9d7a93",
        accent: "#fb7185",
        tool_bg: "#201520",
        code_bg: "#1a1019",
        selection_bg: "#331d30",
        border: "#3d2a3a",
        sidebar_bg: "#1c111b",
    },
    ThemePreset {
        name: "mono",
        label: "极简",
        primary: "#a1a1aa",
        secondary: "#d4d4d8",
        error: "#ef4444",
        background: "#09090b",
        text: "#fafafa",
        dim_text: "#52525b",
        accent: "#e4e4e7",
        tool_bg: "#18181b",
        code_bg: "#111113",
        selection_bg: "#27272a",
        border: "#3f3f46",
        sidebar_bg: "#111113",
    },
];

impl Theme {
    pub fn load(path: &Path) -> Self {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };
        serde_json::from_str(&content).unwrap_or_default()
    }

    pub fn from_preset(name: &str) -> Option<Self> {
        let p = BUILT_IN_THEMES.iter().find(|t| t.name == name)?;
        Some(Self {
            primary: Some(p.primary.to_string()),
            secondary: Some(p.secondary.to_string()),
            error: Some(p.error.to_string()),
            background: Some(p.background.to_string()),
            text: Some(p.text.to_string()),
            dim_text: Some(p.dim_text.to_string()),
            accent: Some(p.accent.to_string()),
            tool_bg: Some(p.tool_bg.to_string()),
            code_bg: Some(p.code_bg.to_string()),
            selection_bg: Some(p.selection_bg.to_string()),
            border: Some(p.border.to_string()),
            sidebar_bg: Some(p.sidebar_bg.to_string()),
        })
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, json)
    }

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

    fn resolve<'a>(opt: &Option<String>, fallback: &'a str) -> Color {
        opt.as_ref()
            .and_then(|s| Self::parse_hex(s))
            .unwrap_or_else(|| Self::parse_hex(fallback).unwrap_or(Color::White))
    }

    fn preset_default(field: &str) -> &'static str {
        BUILT_IN_THEMES
            .iter()
            .find(|t| t.name == "midnight")
            .map(|p| match field {
                "primary" => p.primary,
                "secondary" => p.secondary,
                "error" => p.error,
                "background" => p.background,
                "text" => p.text,
                "dim_text" => p.dim_text,
                "accent" => p.accent,
                "tool_bg" => p.tool_bg,
                "code_bg" => p.code_bg,
                "selection_bg" => p.selection_bg,
                "border" => p.border,
                "sidebar_bg" => p.sidebar_bg,
                _ => p.primary,
            })
            .unwrap_or("#22d3ee")
    }

    pub fn primary(&self) -> Color {
        Self::resolve(&self.primary, Self::preset_default("primary"))
    }

    pub fn secondary(&self) -> Color {
        Self::resolve(&self.secondary, Self::preset_default("secondary"))
    }

    pub fn error(&self) -> Color {
        Self::resolve(&self.error, Self::preset_default("error"))
    }

    pub fn accent(&self) -> Color {
        Self::resolve(&self.accent, Self::preset_default("accent"))
    }

    pub fn dim_text(&self) -> Color {
        Self::resolve(&self.dim_text, Self::preset_default("dim_text"))
    }

    pub fn text(&self) -> Color {
        Self::resolve(&self.text, Self::preset_default("text"))
    }

    pub fn background(&self) -> Color {
        Self::resolve(&self.background, Self::preset_default("background"))
    }

    #[allow(dead_code)]
    pub fn chat_bg(&self) -> Color {
        Color::Rgb(10, 10, 18)
    }

    #[allow(dead_code)]
    pub fn tool_bg(&self) -> Color {
        Self::resolve(&self.tool_bg, Self::preset_default("tool_bg"))
    }

    #[allow(dead_code)]
    pub fn code_bg(&self) -> Color {
        Self::resolve(&self.code_bg, Self::preset_default("code_bg"))
    }

    #[allow(dead_code)]
    pub fn selection_bg(&self) -> Color {
        Self::resolve(&self.selection_bg, Self::preset_default("selection_bg"))
    }

    #[allow(dead_code)]
    pub fn border(&self) -> Color {
        Self::resolve(&self.border, Self::preset_default("border"))
    }

    #[allow(dead_code)]
    pub fn sidebar_bg(&self) -> Color {
        Self::resolve(&self.sidebar_bg, Self::preset_default("sidebar_bg"))
    }

    #[allow(dead_code)]
    pub fn user_colors(&self) -> (Color, Color) {
        (self.secondary(), Color::Rgb(16, 30, 24))
    }

    #[allow(dead_code)]
    pub fn assistant_colors(&self) -> (Color, Color) {
        (self.primary(), Color::Rgb(12, 20, 30))
    }

    #[allow(dead_code)]
    pub fn tool_colors(&self) -> (Color, Color) {
        (self.accent(), self.tool_bg())
    }

    #[allow(dead_code)]
    pub fn error_colors(&self) -> (Color, Color) {
        (self.error(), Color::Rgb(30, 12, 12))
    }
}
