use ratatui::style::Color;

// ═══════════════════════════════════════════════════════════════════════════════
// i-rs-code TUI — Theme System
// Switchable themes via /theme slash command
// ═══════════════════════════════════════════════════════════════════════════════

/// Complete color theme for the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub id: &'static str,
    pub display_name: &'static str,

    // ─── Base ────────────────────────────────────────────────────────────
    pub bg: Color,
    pub bg_surface: Color,
    pub bg_input: Color,
    pub bg_title: Color,
    pub border: Color,
    pub border_active: Color,

    // ─── Text ────────────────────────────────────────────────────────────
    pub text: Color,
    pub dim: Color,
    pub muted: Color,
    pub label: Color,

    // ─── Accents ─────────────────────────────────────────────────────────
    pub accent: Color,
    pub green: Color,
    pub orange: Color,
    pub red: Color,
    pub purple: Color,
    pub yellow: Color,
    pub cyan: Color,

    // ─── Semantic ────────────────────────────────────────────────────────
    pub tool_output: Color,
    pub file_edit: Color,
    pub summary: Color,

    // ─── Diff ────────────────────────────────────────────────────────────
    pub diff_green: Color,
    pub diff_red: Color,
    pub diff_hunk: Color,

    // ─── Message Backgrounds ─────────────────────────────────────────────
    pub bg_user: Color,
    pub bg_ai: Color,
    pub bg_tool: Color,
    pub bg_system: Color,
    pub bg_file: Color,
    pub bg_sidebar: Color,

    // ─── Status ──────────────────────────────────────────────────────────
    pub status_idle: Color,
    pub status_busy: Color,
}

impl Default for Theme {
    fn default() -> Self {
        THEME_OPENCODE
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Theme Presets
// ═══════════════════════════════════════════════════════════════════════════════

/// OpenCode-inspired minimal dark theme
pub const THEME_OPENCODE: Theme = Theme {
    id: "opencode",
    display_name: "OpenCode",

    bg: Color::Rgb(18, 18, 22),
    bg_surface: Color::Rgb(26, 27, 32),
    bg_input: Color::Rgb(26, 27, 32),
    bg_title: Color::Rgb(22, 22, 26),
    border: Color::Rgb(50, 52, 60),
    border_active: Color::Rgb(110, 130, 170),

    text: Color::Rgb(220, 220, 225),
    dim: Color::Rgb(160, 162, 175),
    muted: Color::Rgb(118, 120, 135),
    label: Color::Rgb(95, 98, 115),

    accent: Color::Rgb(130, 160, 230),
    green: Color::Rgb(150, 200, 130),
    orange: Color::Rgb(225, 195, 120),
    red: Color::Rgb(230, 130, 150),
    purple: Color::Rgb(195, 170, 230),
    yellow: Color::Rgb(225, 200, 110),
    cyan: Color::Rgb(140, 210, 230),

    tool_output: Color::Rgb(200, 205, 215),
    file_edit: Color::Rgb(195, 170, 230),
    summary: Color::Rgb(150, 200, 130),

    diff_green: Color::Rgb(160, 215, 160),
    diff_red: Color::Rgb(230, 150, 165),
    diff_hunk: Color::Rgb(140, 210, 230),

    bg_user: Color::Rgb(24, 26, 32),
    bg_ai: Color::Rgb(20, 22, 28),
    bg_tool: Color::Rgb(22, 24, 30),
    bg_system: Color::Rgb(20, 22, 26),
    bg_file: Color::Rgb(26, 24, 32),
    bg_sidebar: Color::Rgb(20, 21, 26),

    status_idle: Color::Rgb(150, 200, 130),
    status_busy: Color::Rgb(225, 200, 110),
};

/// Tokyo Night inspired dark theme
pub const THEME_TOKYO: Theme = Theme {
    id: "tokyo",
    display_name: "Tokyo Night",

    bg: Color::Rgb(26, 27, 38),
    bg_surface: Color::Rgb(36, 40, 59),
    bg_input: Color::Rgb(36, 40, 59),
    bg_title: Color::Rgb(30, 33, 54),
    border: Color::Rgb(65, 72, 104),
    border_active: Color::Rgb(122, 162, 247),

    text: Color::Rgb(192, 202, 245),
    dim: Color::Rgb(147, 153, 178),
    muted: Color::Rgb(98, 104, 128),
    label: Color::Rgb(86, 94, 122),

    accent: Color::Rgb(122, 162, 247),
    green: Color::Rgb(158, 206, 106),
    orange: Color::Rgb(224, 175, 104),
    red: Color::Rgb(247, 118, 142),
    purple: Color::Rgb(203, 166, 247),
    yellow: Color::Rgb(229, 200, 104),
    cyan: Color::Rgb(136, 231, 255),

    tool_output: Color::Rgb(200, 210, 200),
    file_edit: Color::Rgb(203, 166, 247),
    summary: Color::Rgb(158, 206, 106),

    diff_green: Color::Rgb(166, 227, 161),
    diff_red: Color::Rgb(247, 118, 142),
    diff_hunk: Color::Rgb(136, 231, 255),

    bg_user: Color::Rgb(48, 54, 78),
    bg_ai: Color::Rgb(36, 40, 59),
    bg_tool: Color::Rgb(40, 45, 65),
    bg_system: Color::Rgb(40, 44, 54),
    bg_file: Color::Rgb(52, 44, 68),
    bg_sidebar: Color::Rgb(30, 33, 54),

    status_idle: Color::Rgb(158, 206, 106),
    status_busy: Color::Rgb(229, 200, 104),
};

/// Catppuccin Mocha — warm pastel dark theme
pub const THEME_CATPPUCCIN: Theme = Theme {
    id: "catppuccin",
    display_name: "Catppuccin Mocha",

    bg: Color::Rgb(30, 30, 46),
    bg_surface: Color::Rgb(49, 50, 68),
    bg_input: Color::Rgb(49, 50, 68),
    bg_title: Color::Rgb(36, 39, 58),
    border: Color::Rgb(69, 71, 90),
    border_active: Color::Rgb(137, 180, 250),

    text: Color::Rgb(205, 214, 244),
    dim: Color::Rgb(166, 173, 200),
    muted: Color::Rgb(127, 132, 156),
    label: Color::Rgb(108, 112, 134),

    accent: Color::Rgb(137, 180, 250),
    green: Color::Rgb(166, 227, 161),
    orange: Color::Rgb(250, 179, 135),
    red: Color::Rgb(243, 139, 168),
    purple: Color::Rgb(203, 166, 247),
    yellow: Color::Rgb(249, 226, 175),
    cyan: Color::Rgb(148, 226, 213),

    tool_output: Color::Rgb(186, 194, 222),
    file_edit: Color::Rgb(203, 166, 247),
    summary: Color::Rgb(166, 227, 161),

    diff_green: Color::Rgb(166, 227, 161),
    diff_red: Color::Rgb(243, 139, 168),
    diff_hunk: Color::Rgb(148, 226, 213),

    bg_user: Color::Rgb(54, 58, 79),
    bg_ai: Color::Rgb(42, 44, 60),
    bg_tool: Color::Rgb(48, 50, 66),
    bg_system: Color::Rgb(40, 42, 56),
    bg_file: Color::Rgb(56, 50, 70),
    bg_sidebar: Color::Rgb(36, 39, 58),

    status_idle: Color::Rgb(166, 227, 161),
    status_busy: Color::Rgb(249, 226, 175),
};

/// Gruvbox dark — warm retro dark theme
pub const THEME_GRUVBOX: Theme = Theme {
    id: "gruvbox",
    display_name: "Gruvbox Dark",

    bg: Color::Rgb(40, 40, 40),
    bg_surface: Color::Rgb(50, 48, 47),
    bg_input: Color::Rgb(50, 48, 47),
    bg_title: Color::Rgb(45, 45, 45),
    border: Color::Rgb(80, 73, 69),
    border_active: Color::Rgb(215, 153, 33),

    text: Color::Rgb(235, 219, 178),
    dim: Color::Rgb(189, 174, 147),
    muted: Color::Rgb(146, 131, 116),
    label: Color::Rgb(124, 111, 100),

    accent: Color::Rgb(215, 153, 33),
    green: Color::Rgb(142, 192, 124),
    orange: Color::Rgb(254, 128, 25),
    red: Color::Rgb(251, 73, 52),
    purple: Color::Rgb(211, 134, 155),
    yellow: Color::Rgb(250, 189, 47),
    cyan: Color::Rgb(131, 165, 152),

    tool_output: Color::Rgb(213, 196, 161),
    file_edit: Color::Rgb(211, 134, 155),
    summary: Color::Rgb(184, 187, 38),

    diff_green: Color::Rgb(142, 192, 124),
    diff_red: Color::Rgb(251, 73, 52),
    diff_hunk: Color::Rgb(131, 165, 152),

    bg_user: Color::Rgb(64, 60, 56),
    bg_ai: Color::Rgb(50, 48, 47),
    bg_tool: Color::Rgb(56, 54, 52),
    bg_system: Color::Rgb(45, 43, 42),
    bg_file: Color::Rgb(72, 56, 60),
    bg_sidebar: Color::Rgb(45, 43, 42),

    status_idle: Color::Rgb(142, 192, 124),
    status_busy: Color::Rgb(250, 189, 47),
};

/// Solarized Dark — classic balanced dark theme
pub const THEME_SOLARIZED: Theme = Theme {
    id: "solarized",
    display_name: "Solarized Dark",

    bg: Color::Rgb(0, 43, 54),
    bg_surface: Color::Rgb(7, 54, 66),
    bg_input: Color::Rgb(7, 54, 66),
    bg_title: Color::Rgb(4, 48, 60),
    border: Color::Rgb(88, 110, 117),
    border_active: Color::Rgb(38, 139, 210),

    text: Color::Rgb(147, 161, 161),
    dim: Color::Rgb(133, 153, 153),
    muted: Color::Rgb(101, 123, 131),
    label: Color::Rgb(88, 110, 117),

    accent: Color::Rgb(38, 139, 210),
    green: Color::Rgb(133, 153, 51),
    orange: Color::Rgb(203, 75, 22),
    red: Color::Rgb(220, 50, 47),
    purple: Color::Rgb(108, 113, 196),
    yellow: Color::Rgb(181, 137, 0),
    cyan: Color::Rgb(42, 161, 152),

    tool_output: Color::Rgb(147, 161, 161),
    file_edit: Color::Rgb(108, 113, 196),
    summary: Color::Rgb(133, 153, 51),

    diff_green: Color::Rgb(133, 153, 51),
    diff_red: Color::Rgb(220, 50, 47),
    diff_hunk: Color::Rgb(42, 161, 152),

    bg_user: Color::Rgb(18, 60, 72),
    bg_ai: Color::Rgb(7, 54, 66),
    bg_tool: Color::Rgb(14, 64, 76),
    bg_system: Color::Rgb(4, 48, 60),
    bg_file: Color::Rgb(28, 56, 80),
    bg_sidebar: Color::Rgb(4, 48, 60),

    status_idle: Color::Rgb(133, 153, 51),
    status_busy: Color::Rgb(181, 137, 0),
};

/// One Dark — popular Atom-inspired theme
pub const THEME_ONEDARK: Theme = Theme {
    id: "onedark",
    display_name: "One Dark",

    bg: Color::Rgb(40, 44, 52),
    bg_surface: Color::Rgb(50, 56, 66),
    bg_input: Color::Rgb(50, 56, 66),
    bg_title: Color::Rgb(45, 50, 60),
    border: Color::Rgb(78, 86, 102),
    border_active: Color::Rgb(97, 175, 239),

    text: Color::Rgb(220, 223, 228),
    dim: Color::Rgb(171, 178, 191),
    muted: Color::Rgb(127, 134, 148),
    label: Color::Rgb(110, 118, 130),

    accent: Color::Rgb(97, 175, 239),
    green: Color::Rgb(152, 195, 121),
    orange: Color::Rgb(229, 192, 123),
    red: Color::Rgb(224, 108, 117),
    purple: Color::Rgb(198, 120, 221),
    yellow: Color::Rgb(229, 192, 123),
    cyan: Color::Rgb(86, 182, 194),

    tool_output: Color::Rgb(192, 198, 211),
    file_edit: Color::Rgb(198, 120, 221),
    summary: Color::Rgb(152, 195, 121),

    diff_green: Color::Rgb(152, 195, 121),
    diff_red: Color::Rgb(224, 108, 117),
    diff_hunk: Color::Rgb(86, 182, 194),

    bg_user: Color::Rgb(56, 64, 75),
    bg_ai: Color::Rgb(50, 56, 66),
    bg_tool: Color::Rgb(54, 60, 72),
    bg_system: Color::Rgb(44, 50, 60),
    bg_file: Color::Rgb(60, 54, 72),
    bg_sidebar: Color::Rgb(45, 50, 60),

    status_idle: Color::Rgb(152, 195, 121),
    status_busy: Color::Rgb(229, 192, 123),
};

/// Light — clean light theme for daytime use
pub const THEME_LIGHT: Theme = Theme {
    id: "light",
    display_name: "Light",

    bg: Color::Rgb(252, 252, 252),
    bg_surface: Color::Rgb(244, 244, 246),
    bg_input: Color::Rgb(244, 244, 246),
    bg_title: Color::Rgb(248, 248, 250),
    border: Color::Rgb(210, 212, 218),
    border_active: Color::Rgb(70, 110, 200),

    text: Color::Rgb(40, 44, 52),
    dim: Color::Rgb(110, 115, 130),
    muted: Color::Rgb(150, 154, 168),
    label: Color::Rgb(170, 174, 184),

    accent: Color::Rgb(70, 110, 200),
    green: Color::Rgb(60, 145, 70),
    orange: Color::Rgb(200, 130, 30),
    red: Color::Rgb(200, 60, 70),
    purple: Color::Rgb(150, 90, 180),
    yellow: Color::Rgb(195, 155, 30),
    cyan: Color::Rgb(40, 150, 170),

    tool_output: Color::Rgb(70, 76, 90),
    file_edit: Color::Rgb(150, 90, 180),
    summary: Color::Rgb(60, 145, 70),

    diff_green: Color::Rgb(45, 135, 65),
    diff_red: Color::Rgb(190, 60, 75),
    diff_hunk: Color::Rgb(40, 150, 170),

    bg_user: Color::Rgb(238, 240, 245),
    bg_ai: Color::Rgb(248, 248, 250),
    bg_tool: Color::Rgb(242, 242, 246),
    bg_system: Color::Rgb(244, 244, 248),
    bg_file: Color::Rgb(242, 238, 246),
    bg_sidebar: Color::Rgb(246, 246, 250),

    status_idle: Color::Rgb(60, 145, 70),
    status_busy: Color::Rgb(195, 155, 30),
};

/// All available themes in display order
pub const THEMES: &[&Theme] = &[
    &THEME_OPENCODE,
    &THEME_TOKYO,
    &THEME_CATPPUCCIN,
    &THEME_GRUVBOX,
    &THEME_SOLARIZED,
    &THEME_ONEDARK,
    &THEME_LIGHT,
];

/// Find a theme by id or display name (case-insensitive)
pub fn find_theme(name: &str) -> Option<&'static Theme> {
    let lower = name.to_lowercase();
    THEMES
        .iter()
        .find(|t| t.id == lower || t.display_name.to_lowercase() == lower)
        .copied()
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Active Theme Holder (sync + safe for async tasks)
// ═══════════════════════════════════════════════════════════════════════════════

use std::sync::Mutex;

static ACTIVE_THEME: Mutex<Option<&'static Theme>> = Mutex::new(None);

/// Set the active theme.
pub fn set_active(theme: &'static Theme) {
    let mut g = ACTIVE_THEME.lock().expect("theme lock poisoned");
    *g = Some(theme);
}

/// Get the active theme (or default)
pub fn active() -> &'static Theme {
    let g = ACTIVE_THEME.lock().expect("theme lock poisoned");
    g.unwrap_or(&THEME_OPENCODE)
}

/// Run `f` with a temporary active theme. The previous active theme is
/// restored when the closure returns, even on panic. Used for live preview
/// in the theme picker overlay.
pub fn with_preview<F, R>(theme: &'static Theme, f: F) -> R
where
    F: FnOnce() -> R,
{
    struct Guard(Option<&'static Theme>);
    impl Drop for Guard {
        fn drop(&mut self) {
            if let Some(t) = self.0 {
                set_active(t);
            }
        }
    }
    // Save current and apply preview
    let prev = {
        let g = ACTIVE_THEME.lock().expect("theme lock poisoned");
        *g
    };
    set_active(theme);
    let _guard = Guard(prev);
    f()
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Color aliases — read from active theme
// (kept for backward compatibility with existing code)
// ═══════════════════════════════════════════════════════════════════════════════

macro_rules! themed {
    ($field:ident) => {
        #[allow(dead_code)]
        pub fn $field() -> Color {
            active().$field
        }
    };
}

// Base
themed!(bg);
themed!(bg_surface);
themed!(bg_input);
themed!(bg_title);
themed!(border);
themed!(border_active);

// Text
themed!(text);
themed!(dim);
themed!(muted);
themed!(label);

// Accents
themed!(accent);
themed!(green);
themed!(orange);
themed!(red);
themed!(purple);
themed!(yellow);
themed!(cyan);

// Semantic
themed!(tool_output);
themed!(file_edit);
themed!(summary);

// Diff
themed!(diff_green);
themed!(diff_red);
themed!(diff_hunk);

// Message backgrounds
themed!(bg_user);
themed!(bg_ai);
themed!(bg_tool);
themed!(bg_system);
themed!(bg_file);
themed!(bg_sidebar);

// Status
themed!(status_idle);
themed!(status_busy);

// ═══════════════════════════════════════════════════════════════════════════════
//  Backward-compatible constants → functions (read from active theme)
// ═══════════════════════════════════════════════════════════════════════════════

macro_rules! themed_const {
    ($name:ident, $field:ident) => {
        pub fn $name() -> Color {
            active().$field
        }
    };
}

themed_const!(c_bg, bg);
themed_const!(c_bg_surface, bg_surface);
themed_const!(c_bg_input, bg_input);
themed_const!(c_bg_title, bg_title);
themed_const!(c_border, border);
themed_const!(c_border_active, border_active);

themed_const!(c_text, text);
themed_const!(c_dim, dim);
themed_const!(c_muted, muted);
themed_const!(c_label, label);

themed_const!(c_accent, accent);
themed_const!(c_green, green);
themed_const!(c_orange, orange);
themed_const!(c_red, red);
themed_const!(c_purple, purple);
themed_const!(c_yellow, yellow);
themed_const!(c_cyan, cyan);

themed_const!(c_tool_output, tool_output);
themed_const!(c_file_edit, file_edit);
themed_const!(c_summary, summary);

themed_const!(c_diff_green, diff_green);
themed_const!(c_diff_red, diff_red);
themed_const!(c_diff_hunk, diff_hunk);

themed_const!(c_bg_user, bg_user);
themed_const!(c_bg_ai, bg_ai);
themed_const!(c_bg_tool, bg_tool);
themed_const!(c_bg_system, bg_system);
themed_const!(c_bg_file, bg_file);
themed_const!(c_bg_sidebar, bg_sidebar);

themed_const!(c_status_idle, status_idle);
themed_const!(c_status_busy, status_busy);
