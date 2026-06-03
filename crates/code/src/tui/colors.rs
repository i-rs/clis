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

/// Darcula — JetBrains IDE dark theme
pub const THEME_DARCULA: Theme = Theme {
    id: "darcula",
    display_name: "Darcula",
    bg: Color::Rgb(43, 43, 43),
    bg_surface: Color::Rgb(50, 50, 50),
    bg_input: Color::Rgb(50, 50, 50),
    bg_title: Color::Rgb(47, 47, 47),
    border: Color::Rgb(81, 81, 81),
    border_active: Color::Rgb(75, 110, 175),
    text: Color::Rgb(169, 183, 198),
    dim: Color::Rgb(153, 153, 153),
    muted: Color::Rgb(127, 127, 127),
    label: Color::Rgb(110, 110, 110),
    accent: Color::Rgb(75, 110, 175),
    green: Color::Rgb(98, 151, 85),
    orange: Color::Rgb(207, 142, 109),
    red: Color::Rgb(204, 102, 102),
    purple: Color::Rgb(152, 118, 170),
    yellow: Color::Rgb(230, 192, 117),
    cyan: Color::Rgb(107, 153, 184),
    tool_output: Color::Rgb(187, 188, 187),
    file_edit: Color::Rgb(152, 118, 170),
    summary: Color::Rgb(98, 151, 85),
    diff_green: Color::Rgb(106, 171, 115),
    diff_red: Color::Rgb(204, 102, 102),
    diff_hunk: Color::Rgb(107, 153, 184),
    bg_user: Color::Rgb(60, 63, 65),
    bg_ai: Color::Rgb(50, 53, 55),
    bg_tool: Color::Rgb(54, 57, 59),
    bg_system: Color::Rgb(48, 50, 52),
    bg_file: Color::Rgb(60, 53, 65),
    bg_sidebar: Color::Rgb(47, 50, 52),
    status_idle: Color::Rgb(98, 151, 85),
    status_busy: Color::Rgb(230, 192, 117),
};

/// Nord — Arctic, north-bluish clean palette
pub const THEME_NORD: Theme = Theme {
    id: "nord",
    display_name: "Nord",
    bg: Color::Rgb(46, 52, 64),
    bg_surface: Color::Rgb(59, 66, 82),
    bg_input: Color::Rgb(59, 66, 82),
    bg_title: Color::Rgb(53, 60, 72),
    border: Color::Rgb(76, 86, 106),
    border_active: Color::Rgb(136, 192, 208),
    text: Color::Rgb(216, 222, 233),
    dim: Color::Rgb(192, 197, 209),
    muted: Color::Rgb(148, 156, 173),
    label: Color::Rgb(129, 138, 154),
    accent: Color::Rgb(136, 192, 208),
    green: Color::Rgb(163, 190, 140),
    orange: Color::Rgb(208, 135, 112),
    red: Color::Rgb(191, 97, 106),
    purple: Color::Rgb(180, 142, 173),
    yellow: Color::Rgb(235, 203, 139),
    cyan: Color::Rgb(143, 188, 187),
    tool_output: Color::Rgb(229, 233, 240),
    file_edit: Color::Rgb(180, 142, 173),
    summary: Color::Rgb(163, 190, 140),
    diff_green: Color::Rgb(163, 190, 140),
    diff_red: Color::Rgb(191, 97, 106),
    diff_hunk: Color::Rgb(143, 188, 187),
    bg_user: Color::Rgb(67, 76, 94),
    bg_ai: Color::Rgb(59, 66, 82),
    bg_tool: Color::Rgb(63, 70, 86),
    bg_system: Color::Rgb(53, 60, 72),
    bg_file: Color::Rgb(70, 65, 84),
    bg_sidebar: Color::Rgb(53, 60, 72),
    status_idle: Color::Rgb(163, 190, 140),
    status_busy: Color::Rgb(235, 203, 139),
};

/// Dracula — iconic purple-tinted dark theme
pub const THEME_DRACULA: Theme = Theme {
    id: "dracula",
    display_name: "Dracula",
    bg: Color::Rgb(40, 42, 54),
    bg_surface: Color::Rgb(52, 54, 68),
    bg_input: Color::Rgb(52, 54, 68),
    bg_title: Color::Rgb(46, 48, 60),
    border: Color::Rgb(78, 80, 96),
    border_active: Color::Rgb(139, 233, 253),
    text: Color::Rgb(248, 248, 242),
    dim: Color::Rgb(189, 192, 198),
    muted: Color::Rgb(141, 144, 152),
    label: Color::Rgb(122, 125, 134),
    accent: Color::Rgb(139, 233, 253),
    green: Color::Rgb(80, 250, 123),
    orange: Color::Rgb(255, 184, 108),
    red: Color::Rgb(255, 85, 85),
    purple: Color::Rgb(189, 147, 249),
    yellow: Color::Rgb(241, 250, 140),
    cyan: Color::Rgb(139, 233, 253),
    tool_output: Color::Rgb(232, 234, 246),
    file_edit: Color::Rgb(189, 147, 249),
    summary: Color::Rgb(80, 250, 123),
    diff_green: Color::Rgb(80, 250, 123),
    diff_red: Color::Rgb(255, 85, 85),
    diff_hunk: Color::Rgb(139, 233, 253),
    bg_user: Color::Rgb(60, 62, 76),
    bg_ai: Color::Rgb(52, 54, 68),
    bg_tool: Color::Rgb(56, 58, 72),
    bg_system: Color::Rgb(46, 48, 60),
    bg_file: Color::Rgb(64, 56, 78),
    bg_sidebar: Color::Rgb(46, 48, 60),
    status_idle: Color::Rgb(80, 250, 123),
    status_busy: Color::Rgb(241, 250, 140),
};

/// Monokai — classic warm dark theme
pub const THEME_MONOKAI: Theme = Theme {
    id: "monokai",
    display_name: "Monokai",
    bg: Color::Rgb(39, 40, 34),
    bg_surface: Color::Rgb(49, 50, 44),
    bg_input: Color::Rgb(49, 50, 44),
    bg_title: Color::Rgb(43, 44, 38),
    border: Color::Rgb(73, 72, 65),
    border_active: Color::Rgb(102, 217, 239),
    text: Color::Rgb(248, 248, 242),
    dim: Color::Rgb(196, 196, 191),
    muted: Color::Rgb(144, 144, 139),
    label: Color::Rgb(124, 124, 119),
    accent: Color::Rgb(102, 217, 239),
    green: Color::Rgb(166, 226, 46),
    orange: Color::Rgb(253, 151, 31),
    red: Color::Rgb(249, 38, 114),
    purple: Color::Rgb(174, 129, 255),
    yellow: Color::Rgb(230, 219, 116),
    cyan: Color::Rgb(102, 217, 239),
    tool_output: Color::Rgb(228, 228, 222),
    file_edit: Color::Rgb(174, 129, 255),
    summary: Color::Rgb(166, 226, 46),
    diff_green: Color::Rgb(166, 226, 46),
    diff_red: Color::Rgb(249, 38, 114),
    diff_hunk: Color::Rgb(102, 217, 239),
    bg_user: Color::Rgb(58, 60, 52),
    bg_ai: Color::Rgb(49, 50, 44),
    bg_tool: Color::Rgb(53, 54, 48),
    bg_system: Color::Rgb(43, 44, 38),
    bg_file: Color::Rgb(62, 52, 64),
    bg_sidebar: Color::Rgb(43, 44, 38),
    status_idle: Color::Rgb(166, 226, 46),
    status_busy: Color::Rgb(230, 219, 116),
};

/// Ayu — modern, warm and bright dark theme
pub const THEME_AYU: Theme = Theme {
    id: "ayu",
    display_name: "Ayu Dark",
    bg: Color::Rgb(15, 20, 31),
    bg_surface: Color::Rgb(24, 30, 42),
    bg_input: Color::Rgb(24, 30, 42),
    bg_title: Color::Rgb(19, 25, 36),
    border: Color::Rgb(56, 64, 79),
    border_active: Color::Rgb(89, 184, 255),
    text: Color::Rgb(204, 207, 210),
    dim: Color::Rgb(166, 168, 173),
    muted: Color::Rgb(120, 124, 134),
    label: Color::Rgb(101, 108, 119),
    accent: Color::Rgb(89, 184, 255),
    green: Color::Rgb(186, 222, 81),
    orange: Color::Rgb(255, 174, 26),
    red: Color::Rgb(255, 51, 51),
    purple: Color::Rgb(208, 153, 245),
    yellow: Color::Rgb(255, 220, 92),
    cyan: Color::Rgb(90, 222, 255),
    tool_output: Color::Rgb(198, 202, 211),
    file_edit: Color::Rgb(208, 153, 245),
    summary: Color::Rgb(186, 222, 81),
    diff_green: Color::Rgb(186, 222, 81),
    diff_red: Color::Rgb(255, 51, 51),
    diff_hunk: Color::Rgb(90, 222, 255),
    bg_user: Color::Rgb(30, 38, 52),
    bg_ai: Color::Rgb(24, 30, 42),
    bg_tool: Color::Rgb(26, 34, 46),
    bg_system: Color::Rgb(20, 26, 38),
    bg_file: Color::Rgb(36, 30, 50),
    bg_sidebar: Color::Rgb(19, 25, 36),
    status_idle: Color::Rgb(186, 222, 81),
    status_busy: Color::Rgb(255, 220, 92),
};

/// Kanagawa — inspired by traditional Japanese colors
pub const THEME_KANAGAWA: Theme = Theme {
    id: "kanagawa",
    display_name: "Kanagawa",
    bg: Color::Rgb(22, 22, 29),
    bg_surface: Color::Rgb(32, 32, 39),
    bg_input: Color::Rgb(32, 32, 39),
    bg_title: Color::Rgb(26, 26, 33),
    border: Color::Rgb(54, 54, 66),
    border_active: Color::Rgb(127, 180, 202),
    text: Color::Rgb(220, 213, 198),
    dim: Color::Rgb(178, 173, 162),
    muted: Color::Rgb(132, 128, 121),
    label: Color::Rgb(114, 110, 102),
    accent: Color::Rgb(127, 180, 202),
    green: Color::Rgb(152, 187, 108),
    orange: Color::Rgb(255, 174, 66),
    red: Color::Rgb(224, 64, 64),
    purple: Color::Rgb(187, 154, 174),
    yellow: Color::Rgb(225, 183, 102),
    cyan: Color::Rgb(109, 169, 184),
    tool_output: Color::Rgb(216, 210, 196),
    file_edit: Color::Rgb(187, 154, 174),
    summary: Color::Rgb(152, 187, 108),
    diff_green: Color::Rgb(152, 187, 108),
    diff_red: Color::Rgb(224, 64, 64),
    diff_hunk: Color::Rgb(109, 169, 184),
    bg_user: Color::Rgb(40, 40, 48),
    bg_ai: Color::Rgb(32, 32, 39),
    bg_tool: Color::Rgb(36, 36, 43),
    bg_system: Color::Rgb(28, 28, 35),
    bg_file: Color::Rgb(44, 36, 46),
    bg_sidebar: Color::Rgb(26, 26, 33),
    status_idle: Color::Rgb(152, 187, 108),
    status_busy: Color::Rgb(225, 183, 102),
};

/// Night Owl — Sarah Drasner's popular theme
pub const THEME_NIGHT_OWL: Theme = Theme {
    id: "nightowl",
    display_name: "Night Owl",
    bg: Color::Rgb(1, 22, 39),
    bg_surface: Color::Rgb(11, 35, 56),
    bg_input: Color::Rgb(11, 35, 56),
    bg_title: Color::Rgb(5, 27, 47),
    border: Color::Rgb(28, 64, 92),
    border_active: Color::Rgb(130, 170, 255),
    text: Color::Rgb(199, 218, 238),
    dim: Color::Rgb(168, 184, 200),
    muted: Color::Rgb(123, 145, 165),
    label: Color::Rgb(99, 119, 139),
    accent: Color::Rgb(130, 170, 255),
    green: Color::Rgb(173, 219, 103),
    orange: Color::Rgb(255, 203, 107),
    red: Color::Rgb(255, 110, 103),
    purple: Color::Rgb(199, 148, 234),
    yellow: Color::Rgb(255, 215, 109),
    cyan: Color::Rgb(137, 221, 255),
    tool_output: Color::Rgb(208, 222, 235),
    file_edit: Color::Rgb(199, 148, 234),
    summary: Color::Rgb(173, 219, 103),
    diff_green: Color::Rgb(173, 219, 103),
    diff_red: Color::Rgb(255, 110, 103),
    diff_hunk: Color::Rgb(137, 221, 255),
    bg_user: Color::Rgb(15, 40, 64),
    bg_ai: Color::Rgb(11, 35, 56),
    bg_tool: Color::Rgb(13, 38, 60),
    bg_system: Color::Rgb(7, 30, 50),
    bg_file: Color::Rgb(25, 32, 64),
    bg_sidebar: Color::Rgb(5, 27, 47),
    status_idle: Color::Rgb(173, 219, 103),
    status_busy: Color::Rgb(255, 215, 109),
};

/// Rosé Pine — soothing pastel dark theme
pub const THEME_ROSE_PINE: Theme = Theme {
    id: "rosepine",
    display_name: "Rosé Pine",
    bg: Color::Rgb(25, 23, 36),
    bg_surface: Color::Rgb(38, 35, 53),
    bg_input: Color::Rgb(38, 35, 53),
    bg_title: Color::Rgb(31, 29, 44),
    border: Color::Rgb(64, 61, 82),
    border_active: Color::Rgb(196, 167, 231),
    text: Color::Rgb(224, 222, 244),
    dim: Color::Rgb(186, 184, 200),
    muted: Color::Rgb(144, 142, 158),
    label: Color::Rgb(122, 119, 138),
    accent: Color::Rgb(196, 167, 231),
    green: Color::Rgb(156, 207, 216),
    orange: Color::Rgb(235, 188, 186),
    red: Color::Rgb(235, 111, 146),
    purple: Color::Rgb(196, 167, 231),
    yellow: Color::Rgb(246, 193, 119),
    cyan: Color::Rgb(156, 207, 216),
    tool_output: Color::Rgb(216, 213, 232),
    file_edit: Color::Rgb(196, 167, 231),
    summary: Color::Rgb(156, 207, 216),
    diff_green: Color::Rgb(156, 207, 216),
    diff_red: Color::Rgb(235, 111, 146),
    diff_hunk: Color::Rgb(246, 193, 119),
    bg_user: Color::Rgb(46, 42, 64),
    bg_ai: Color::Rgb(38, 35, 53),
    bg_tool: Color::Rgb(42, 38, 58),
    bg_system: Color::Rgb(32, 30, 48),
    bg_file: Color::Rgb(50, 42, 64),
    bg_sidebar: Color::Rgb(31, 29, 44),
    status_idle: Color::Rgb(156, 207, 216),
    status_busy: Color::Rgb(246, 193, 119),
};

/// Synthwave — retro neon 80s vibe
pub const THEME_SYNTHWAVE: Theme = Theme {
    id: "synthwave",
    display_name: "Synthwave '84",
    bg: Color::Rgb(34, 24, 64),
    bg_surface: Color::Rgb(46, 32, 86),
    bg_input: Color::Rgb(46, 32, 86),
    bg_title: Color::Rgb(40, 28, 75),
    border: Color::Rgb(76, 50, 132),
    border_active: Color::Rgb(255, 92, 184),
    text: Color::Rgb(240, 234, 250),
    dim: Color::Rgb(204, 196, 222),
    muted: Color::Rgb(160, 152, 184),
    label: Color::Rgb(140, 132, 168),
    accent: Color::Rgb(255, 92, 184),
    green: Color::Rgb(72, 248, 130),
    orange: Color::Rgb(255, 156, 84),
    red: Color::Rgb(255, 88, 122),
    purple: Color::Rgb(196, 132, 252),
    yellow: Color::Rgb(255, 222, 100),
    cyan: Color::Rgb(96, 240, 252),
    tool_output: Color::Rgb(232, 224, 248),
    file_edit: Color::Rgb(196, 132, 252),
    summary: Color::Rgb(72, 248, 130),
    diff_green: Color::Rgb(72, 248, 130),
    diff_red: Color::Rgb(255, 88, 122),
    diff_hunk: Color::Rgb(96, 240, 252),
    bg_user: Color::Rgb(56, 38, 100),
    bg_ai: Color::Rgb(46, 32, 86),
    bg_tool: Color::Rgb(50, 36, 92),
    bg_system: Color::Rgb(40, 28, 75),
    bg_file: Color::Rgb(64, 42, 100),
    bg_sidebar: Color::Rgb(40, 28, 75),
    status_idle: Color::Rgb(72, 248, 130),
    status_busy: Color::Rgb(255, 222, 100),
};

/// Solarized Light — classic light version
pub const THEME_SOLARIZED_LIGHT: Theme = Theme {
    id: "solarized-light",
    display_name: "Solarized Light",
    bg: Color::Rgb(253, 246, 227),
    bg_surface: Color::Rgb(238, 232, 213),
    bg_input: Color::Rgb(238, 232, 213),
    bg_title: Color::Rgb(245, 240, 222),
    border: Color::Rgb(196, 188, 165),
    border_active: Color::Rgb(38, 139, 210),
    text: Color::Rgb(101, 123, 131),
    dim: Color::Rgb(133, 153, 153),
    muted: Color::Rgb(147, 161, 161),
    label: Color::Rgb(165, 175, 178),
    accent: Color::Rgb(38, 139, 210),
    green: Color::Rgb(133, 153, 51),
    orange: Color::Rgb(203, 75, 22),
    red: Color::Rgb(220, 50, 47),
    purple: Color::Rgb(108, 113, 196),
    yellow: Color::Rgb(181, 137, 0),
    cyan: Color::Rgb(42, 161, 152),
    tool_output: Color::Rgb(88, 110, 117),
    file_edit: Color::Rgb(108, 113, 196),
    summary: Color::Rgb(133, 153, 51),
    diff_green: Color::Rgb(133, 153, 51),
    diff_red: Color::Rgb(220, 50, 47),
    diff_hunk: Color::Rgb(42, 161, 152),
    bg_user: Color::Rgb(228, 222, 198),
    bg_ai: Color::Rgb(245, 240, 222),
    bg_tool: Color::Rgb(232, 226, 205),
    bg_system: Color::Rgb(238, 232, 213),
    bg_file: Color::Rgb(232, 220, 215),
    bg_sidebar: Color::Rgb(238, 232, 213),
    status_idle: Color::Rgb(133, 153, 51),
    status_busy: Color::Rgb(181, 137, 0),
};

/// GitHub Light — clean light theme
pub const THEME_GITHUB_LIGHT: Theme = Theme {
    id: "github-light",
    display_name: "GitHub Light",
    bg: Color::Rgb(255, 255, 255),
    bg_surface: Color::Rgb(246, 248, 250),
    bg_input: Color::Rgb(246, 248, 250),
    bg_title: Color::Rgb(250, 250, 250),
    border: Color::Rgb(208, 215, 222),
    border_active: Color::Rgb(9, 105, 218),
    text: Color::Rgb(31, 35, 40),
    dim: Color::Rgb(87, 96, 106),
    muted: Color::Rgb(125, 133, 144),
    label: Color::Rgb(149, 157, 165),
    accent: Color::Rgb(9, 105, 218),
    green: Color::Rgb(26, 127, 55),
    orange: Color::Rgb(154, 103, 0),
    red: Color::Rgb(207, 34, 46),
    purple: Color::Rgb(130, 80, 223),
    yellow: Color::Rgb(191, 135, 0),
    cyan: Color::Rgb(5, 122, 152),
    tool_output: Color::Rgb(68, 76, 86),
    file_edit: Color::Rgb(130, 80, 223),
    summary: Color::Rgb(26, 127, 55),
    diff_green: Color::Rgb(26, 127, 55),
    diff_red: Color::Rgb(207, 34, 46),
    diff_hunk: Color::Rgb(5, 122, 152),
    bg_user: Color::Rgb(232, 240, 250),
    bg_ai: Color::Rgb(250, 250, 250),
    bg_tool: Color::Rgb(240, 244, 248),
    bg_system: Color::Rgb(244, 246, 248),
    bg_file: Color::Rgb(242, 240, 250),
    bg_sidebar: Color::Rgb(246, 248, 250),
    status_idle: Color::Rgb(26, 127, 55),
    status_busy: Color::Rgb(191, 135, 0),
};

/// One Light — Atom's light counterpart
pub const THEME_ONE_LIGHT: Theme = Theme {
    id: "one-light",
    display_name: "One Light",
    bg: Color::Rgb(250, 250, 250),
    bg_surface: Color::Rgb(240, 240, 240),
    bg_input: Color::Rgb(240, 240, 240),
    bg_title: Color::Rgb(245, 245, 245),
    border: Color::Rgb(218, 218, 222),
    border_active: Color::Rgb(64, 120, 192),
    text: Color::Rgb(56, 58, 66),
    dim: Color::Rgb(101, 105, 119),
    muted: Color::Rgb(135, 140, 152),
    label: Color::Rgb(150, 155, 165),
    accent: Color::Rgb(64, 120, 192),
    green: Color::Rgb(80, 161, 79),
    orange: Color::Rgb(229, 152, 31),
    red: Color::Rgb(222, 75, 79),
    purple: Color::Rgb(170, 99, 200),
    yellow: Color::Rgb(229, 182, 31),
    cyan: Color::Rgb(8, 165, 192),
    tool_output: Color::Rgb(72, 75, 85),
    file_edit: Color::Rgb(170, 99, 200),
    summary: Color::Rgb(80, 161, 79),
    diff_green: Color::Rgb(80, 161, 79),
    diff_red: Color::Rgb(222, 75, 79),
    diff_hunk: Color::Rgb(8, 165, 192),
    bg_user: Color::Rgb(232, 234, 238),
    bg_ai: Color::Rgb(245, 245, 245),
    bg_tool: Color::Rgb(238, 238, 240),
    bg_system: Color::Rgb(242, 242, 244),
    bg_file: Color::Rgb(242, 238, 244),
    bg_sidebar: Color::Rgb(242, 242, 244),
    status_idle: Color::Rgb(80, 161, 79),
    status_busy: Color::Rgb(229, 182, 31),
};

/// Catppuccin Latte — light variant of the popular theme
pub const THEME_CATPPUCCIN_LATTE: Theme = Theme {
    id: "catppuccin-latte",
    display_name: "Catppuccin Latte",
    bg: Color::Rgb(239, 241, 245),
    bg_surface: Color::Rgb(230, 233, 239),
    bg_input: Color::Rgb(230, 233, 239),
    bg_title: Color::Rgb(235, 237, 243),
    border: Color::Rgb(204, 208, 218),
    border_active: Color::Rgb(30, 102, 245),
    text: Color::Rgb(76, 79, 105),
    dim: Color::Rgb(110, 115, 141),
    muted: Color::Rgb(138, 143, 168),
    label: Color::Rgb(156, 160, 180),
    accent: Color::Rgb(30, 102, 245),
    green: Color::Rgb(64, 160, 43),
    orange: Color::Rgb(255, 100, 64),
    red: Color::Rgb(210, 15, 57),
    purple: Color::Rgb(136, 57, 239),
    yellow: Color::Rgb(223, 142, 29),
    cyan: Color::Rgb(4, 165, 229),
    tool_output: Color::Rgb(92, 95, 119),
    file_edit: Color::Rgb(136, 57, 239),
    summary: Color::Rgb(64, 160, 43),
    diff_green: Color::Rgb(64, 160, 43),
    diff_red: Color::Rgb(210, 15, 57),
    diff_hunk: Color::Rgb(4, 165, 229),
    bg_user: Color::Rgb(220, 224, 232),
    bg_ai: Color::Rgb(235, 237, 243),
    bg_tool: Color::Rgb(225, 228, 235),
    bg_system: Color::Rgb(228, 231, 238),
    bg_file: Color::Rgb(232, 226, 238),
    bg_sidebar: Color::Rgb(228, 231, 238),
    status_idle: Color::Rgb(64, 160, 43),
    status_busy: Color::Rgb(223, 142, 29),
};

/// All available themes in display order
pub const THEMES: &[&Theme] = &[
    // ─── Dark themes ────────────────────────────────────────────
    &THEME_OPENCODE,
    &THEME_TOKYO,
    &THEME_CATPPUCCIN,
    &THEME_GRUVBOX,
    &THEME_SOLARIZED,
    &THEME_ONEDARK,
    &THEME_DARCULA,
    &THEME_NORD,
    &THEME_DRACULA,
    &THEME_MONOKAI,
    &THEME_AYU,
    &THEME_KANAGAWA,
    &THEME_NIGHT_OWL,
    &THEME_ROSE_PINE,
    &THEME_SYNTHWAVE,
    // ─── Light themes ───────────────────────────────────────────
    &THEME_LIGHT,
    &THEME_SOLARIZED_LIGHT,
    &THEME_GITHUB_LIGHT,
    &THEME_ONE_LIGHT,
    &THEME_CATPPUCCIN_LATTE,
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
