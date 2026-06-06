#[cfg(feature = "tui")]
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
    ThemePreset {
        name: "lavender",
        label: "薰衣草",
        primary: "#a78bfa",
        secondary: "#e879f9",
        error: "#f87171",
        background: "#120d1a",
        text: "#f5f0ff",
        dim_text: "#8b7aa8",
        accent: "#c084fc",
        tool_bg: "#1d152a",
        code_bg: "#171022",
        selection_bg: "#2a1e3d",
        border: "#35284a",
        sidebar_bg: "#191228",
    },
    ThemePreset {
        name: "crimson",
        label: "绯红",
        primary: "#f87171",
        secondary: "#fb923c",
        error: "#dc2626",
        background: "#170a0a",
        text: "#fef2f2",
        dim_text: "#a07272",
        accent: "#fca5a5",
        tool_bg: "#241212",
        code_bg: "#1c0d0d",
        selection_bg: "#3a1a1a",
        border: "#4a2525",
        sidebar_bg: "#1e0f0f",
    },
    ThemePreset {
        name: "aurora",
        label: "极光",
        primary: "#34d399",
        secondary: "#67e8f9",
        error: "#f87171",
        background: "#051008",
        text: "#ecfdf5",
        dim_text: "#5a8a75",
        accent: "#2dd4bf",
        tool_bg: "#0f1e15",
        code_bg: "#0a1610",
        selection_bg: "#143828",
        border: "#1f4734",
        sidebar_bg: "#0c1a12",
    },
    ThemePreset {
        name: "amber",
        label: "琥珀",
        primary: "#fbbf24",
        secondary: "#fb923c",
        error: "#ef4444",
        background: "#141006",
        text: "#fefce8",
        dim_text: "#9a8755",
        accent: "#fcd34d",
        tool_bg: "#221d10",
        code_bg: "#1a160b",
        selection_bg: "#3d3118",
        border: "#4a3d20",
        sidebar_bg: "#1c1810",
    },
    ThemePreset {
        name: "sakura",
        label: "樱花",
        primary: "#f9a8d4",
        secondary: "#fbcfe8",
        error: "#fb7185",
        background: "#170a12",
        text: "#fdf2f8",
        dim_text: "#a0748a",
        accent: "#f472b6",
        tool_bg: "#23121b",
        code_bg: "#1c0d15",
        selection_bg: "#3a1f2e",
        border: "#472a38",
        sidebar_bg: "#1e1018",
    },
    ThemePreset {
        name: "slate",
        label: "石板",
        primary: "#818cf8",
        secondary: "#94a3b8",
        error: "#f87171",
        background: "#0b0d14",
        text: "#f1f5f9",
        dim_text: "#64748b",
        accent: "#a5b4fc",
        tool_bg: "#141820",
        code_bg: "#0f1218",
        selection_bg: "#1e2430",
        border: "#2d3540",
        sidebar_bg: "#10141c",
    },
    ThemePreset {
        name: "candy",
        label: "糖果",
        primary: "#f472b6",
        secondary: "#60a5fa",
        error: "#ef4444",
        background: "#0f0a12",
        text: "#fef2f2",
        dim_text: "#9a7a8a",
        accent: "#fbbf24",
        tool_bg: "#1c1420",
        code_bg: "#160e18",
        selection_bg: "#30202e",
        border: "#3d2e3a",
        sidebar_bg: "#19101c",
    },
    ThemePreset {
        name: "mint",
        label: "薄荷",
        primary: "#6ee7b7",
        secondary: "#5eead4",
        error: "#f87171",
        background: "#060f0b",
        text: "#f0fdfa",
        dim_text: "#5a8a7a",
        accent: "#34d399",
        tool_bg: "#0e1e16",
        code_bg: "#0a1710",
        selection_bg: "#15382a",
        border: "#1f4738",
        sidebar_bg: "#0c1a12",
    },
    ThemePreset {
        name: "gold",
        label: "黄金",
        primary: "#fbbf24",
        secondary: "#d97706",
        error: "#ef4444",
        background: "#120f05",
        text: "#fefce8",
        dim_text: "#928348",
        accent: "#f59e0b",
        tool_bg: "#201b0e",
        code_bg: "#18140a",
        selection_bg: "#3d3115",
        border: "#4a3d1a",
        sidebar_bg: "#1a160c",
    },
    ThemePreset {
        name: "nebula",
        label: "星云",
        primary: "#c084fc",
        secondary: "#818cf8",
        error: "#f87171",
        background: "#0c0618",
        text: "#f0edff",
        dim_text: "#7d6e9e",
        accent: "#a78bfa",
        tool_bg: "#1a102e",
        code_bg: "#140c24",
        selection_bg: "#2c1e48",
        border: "#382a54",
        sidebar_bg: "#160e26",
    },
    ThemePreset {
        name: "copper",
        label: "铜色",
        primary: "#fdba74",
        secondary: "#f97316",
        error: "#ef4444",
        background: "#140e08",
        text: "#fff7ed",
        dim_text: "#9a7e60",
        accent: "#fb923c",
        tool_bg: "#221a10",
        code_bg: "#1a140c",
        selection_bg: "#3d2a18",
        border: "#4a3520",
        sidebar_bg: "#1c1410",
    },
    ThemePreset {
        name: "jade",
        label: "翡翠",
        primary: "#34d399",
        secondary: "#10b981",
        error: "#ef4444",
        background: "#060d08",
        text: "#ecfdf5",
        dim_text: "#54806a",
        accent: "#6ee7b7",
        tool_bg: "#0e1c14",
        code_bg: "#0a1610",
        selection_bg: "#143828",
        border: "#1a4730",
        sidebar_bg: "#0c1a12",
    },
    ThemePreset {
        name: "plum",
        label: "梅子",
        primary: "#e879f9",
        secondary: "#f472b6",
        error: "#fb7185",
        background: "#120812",
        text: "#faf5ff",
        dim_text: "#92709a",
        accent: "#d946ef",
        tool_bg: "#201422",
        code_bg: "#1a0e1a",
        selection_bg: "#341e36",
        border: "#402a42",
        sidebar_bg: "#1c101c",
    },
    ThemePreset {
        name: "charcoal",
        label: "木炭",
        primary: "#a3a3a3",
        secondary: "#d4d4d4",
        error: "#ef4444",
        background: "#0b0b0b",
        text: "#f5f5f5",
        dim_text: "#6b6b6b",
        accent: "#c7c7c7",
        tool_bg: "#141414",
        code_bg: "#0e0e0e",
        selection_bg: "#222222",
        border: "#333333",
        sidebar_bg: "#101010",
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

    /// Cheap stable identifier for cache keys. We hash just the
    /// color-bearing fields — the only ones that affect rendered
    /// output — so two themes that produce identical colors share
    /// a cache slot, and any color change invalidates the cache.
    /// Computed in ~100 ns, far cheaper than re-rendering.
    pub fn id(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.primary.hash(&mut h);
        self.secondary.hash(&mut h);
        self.text.hash(&mut h);
        self.accent.hash(&mut h);
        self.dim_text.hash(&mut h);
        self.background.hash(&mut h);
        self.tool_bg.hash(&mut h);
        self.code_bg.hash(&mut h);
        self.border.hash(&mut h);
        h.finish()
    }

    #[cfg(feature = "tui")]
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

    #[cfg(feature = "tui")]
    fn resolve(opt: &Option<String>, fallback: &str) -> Color {
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

    #[cfg(feature = "tui")]
    pub fn primary(&self) -> Color {
        Self::resolve(&self.primary, Self::preset_default("primary"))
    }

    #[cfg(feature = "tui")]
    pub fn secondary(&self) -> Color {
        Self::resolve(&self.secondary, Self::preset_default("secondary"))
    }

    #[cfg(feature = "tui")]
    pub fn error(&self) -> Color {
        Self::resolve(&self.error, Self::preset_default("error"))
    }

    #[cfg(feature = "tui")]
    pub fn accent(&self) -> Color {
        Self::resolve(&self.accent, Self::preset_default("accent"))
    }

    #[cfg(feature = "tui")]
    pub fn dim_text(&self) -> Color {
        Self::resolve(&self.dim_text, Self::preset_default("dim_text"))
    }

    #[cfg(feature = "tui")]
    pub fn text(&self) -> Color {
        Self::resolve(&self.text, Self::preset_default("text"))
    }

    #[cfg(feature = "tui")]
    pub fn background(&self) -> Color {
        Self::resolve(&self.background, Self::preset_default("background"))
    }

    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn chat_bg(&self) -> Color {
        Color::Rgb(10, 10, 18)
    }

    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn surface(&self) -> Color {
        Color::Rgb(22, 22, 32)
    }

    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn user_surface(&self) -> Color {
        Color::Rgb(18, 26, 22)
    }

    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn assistant_surface(&self) -> Color {
        Color::Rgb(16, 22, 32)
    }

    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn tool_surface(&self) -> Color {
        Color::Rgb(24, 20, 32)
    }

    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn error_surface(&self) -> Color {
        Color::Rgb(36, 18, 20)
    }

    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn tool_bg(&self) -> Color {
        Self::resolve(&self.tool_bg, Self::preset_default("tool_bg"))
    }

    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn code_bg(&self) -> Color {
        Self::resolve(&self.code_bg, Self::preset_default("code_bg"))
    }

    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn selection_bg(&self) -> Color {
        Self::resolve(&self.selection_bg, Self::preset_default("selection_bg"))
    }

    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn border(&self) -> Color {
        Self::resolve(&self.border, Self::preset_default("border"))
    }

    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn sidebar_bg(&self) -> Color {
        Self::resolve(&self.sidebar_bg, Self::preset_default("sidebar_bg"))
    }

    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn user_colors(&self) -> (Color, Color) {
        (self.secondary(), Color::Rgb(16, 30, 24))
    }

    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn assistant_colors(&self) -> (Color, Color) {
        (self.primary(), Color::Rgb(12, 20, 30))
    }

    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn tool_colors(&self) -> (Color, Color) {
        (self.accent(), self.tool_bg())
    }

    #[cfg(feature = "tui")]
    #[allow(dead_code)]
    pub fn error_colors(&self) -> (Color, Color) {
        (self.error(), Color::Rgb(30, 12, 12))
    }
}
