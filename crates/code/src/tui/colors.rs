use ratatui::style::Color;

// ═══════════════════════════════════════════════════════════════════════════════
// i-rs-code TUI — OpenCode-inspired Color Palette
// Minimal contrast, soft borders, no harsh color blocks
// ═══════════════════════════════════════════════════════════════════════════════

// ─── Base Colors ─────────────────────────────────────────────────────────────

/// Main background - near-black with subtle warm tint
pub const C_BG: Color = Color::Rgb(18, 18, 22);

/// Elevated surfaces (sidebar, input, overlays)
pub const C_BG_SURFACE: Color = Color::Rgb(26, 27, 32);

/// Input area background
pub const C_BG_INPUT: Color = Color::Rgb(26, 27, 32);

/// Title bar / header background
pub const C_BG_TITLE: Color = Color::Rgb(22, 22, 26);

/// Subtle borders and dividers
pub const C_BORDER: Color = Color::Rgb(50, 52, 60);

/// Prominent borders (active elements)
pub const C_BORDER_ACTIVE: Color = Color::Rgb(110, 130, 170);

// ─── Text Colors ──────────────────────────────────────────────────────────────

/// Primary text - soft white
pub const C_TEXT: Color = Color::Rgb(220, 220, 225);

/// Secondary / muted text
pub const C_DIM: Color = Color::Rgb(160, 162, 175);

/// Tertiary / very muted (timestamps, hints)
pub const C_MUTED: Color = Color::Rgb(118, 120, 135);

/// Labels in sidebar
pub const C_LABEL: Color = Color::Rgb(95, 98, 115);

// ─── Accent Colors ───────────────────────────────────────────────────────────

/// Primary accent - soft blue (links, highlights, selection)
pub const C_ACCENT: Color = Color::Rgb(130, 160, 230);

/// Success / positive - soft green
pub const C_GREEN: Color = Color::Rgb(150, 200, 130);

/// Warning / caution - soft orange/yellow
pub const C_ORANGE: Color = Color::Rgb(225, 195, 120);

/// Error / danger - soft red/pink
pub const C_RED: Color = Color::Rgb(230, 130, 150);

/// Purple accent (for special elements)
pub const C_PURPLE: Color = Color::Rgb(195, 170, 230);

/// Yellow accent (highlights, reasoning)
pub const C_YELLOW: Color = Color::Rgb(225, 200, 110);

/// Cyan accent (info, links)
pub const C_CYAN: Color = Color::Rgb(140, 210, 230);

// ─── Semantic Colors ─────────────────────────────────────────────────────────

/// Tool output text
pub const C_TOOL_OUTPUT: Color = Color::Rgb(200, 205, 215);

/// File edit indicators
pub const C_FILE_EDIT: Color = Color::Rgb(195, 170, 230);

/// Summary / success messages
pub const C_SUMMARY: Color = Color::Rgb(150, 200, 130);

// ─── Diff Colors ────────────────────────────────────────────────────────────

/// Added lines in diffs
pub const C_DIFF_GREEN: Color = Color::Rgb(160, 215, 160);

/// Removed lines in diffs
pub const C_DIFF_RED: Color = Color::Rgb(230, 150, 165);

/// Diff markers (hunks)
pub const C_DIFF_HUNK: Color = Color::Rgb(140, 210, 230);

// ─── Message Backgrounds (subtle, no harsh blocks) ──────────────────────────

/// User message bubble
pub const C_BG_USER: Color = Color::Rgb(24, 26, 32);

/// AI / Assistant message bubble
pub const C_BG_AI: Color = Color::Rgb(20, 22, 28);

/// Tool call indicator
pub const C_BG_TOOL: Color = Color::Rgb(22, 24, 30);

/// System messages
pub const C_BG_SYSTEM: Color = Color::Rgb(20, 22, 26);

/// File edit notifications
pub const C_BG_FILE: Color = Color::Rgb(26, 24, 32);

// ─── Sidebar ─────────────────────────────────────────────────────────────────

/// Sidebar background
pub const C_BG_SIDEBAR: Color = Color::Rgb(20, 21, 26);

// ─── Status Indicators ─────────────────────────────────────────────────────

/// Idle state (ready)
pub const C_STATUS_IDLE: Color = Color::Rgb(150, 200, 130);

/// Busy / waiting state
pub const C_STATUS_BUSY: Color = Color::Rgb(225, 200, 110);
