use owo_colors::OwoColorize;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    #[serde(default = "default_error")]
    pub error: String,
    #[serde(default = "default_success")]
    pub success: String,
    #[serde(default = "default_header")]
    pub header: String,
    #[serde(default = "default_warning")]
    pub warning: String,
    #[serde(default = "default_table_border")]
    pub table_border: String,
    #[serde(default = "default_table_header")]
    pub table_header: String,
    #[serde(default = "default_table_row")]
    pub table_row: String,
    #[serde(default = "default_dimmed")]
    pub dimmed: String,
}

fn default_error() -> String {
    "bold red".into()
}
fn default_success() -> String {
    "green".into()
}
fn default_header() -> String {
    "bold cyan".into()
}
fn default_warning() -> String {
    "yellow".into()
}
fn default_table_border() -> String {
    "cyan".into()
}
fn default_table_header() -> String {
    "bold cyan".into()
}
fn default_table_row() -> String {
    "green".into()
}
fn default_dimmed() -> String {
    "bright black".into()
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            error: default_error(),
            success: default_success(),
            header: default_header(),
            warning: default_warning(),
            table_border: default_table_border(),
            table_header: default_table_header(),
            table_row: default_table_row(),
            dimmed: default_dimmed(),
        }
    }
}

impl Theme {
    fn config_path() -> PathBuf {
        if let Ok(dir) = std::env::var("CONFIG_DIR") {
            PathBuf::from(dir).join("theme.json")
        } else {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("i-rs")
                .join("theme.json")
        }
    }

    #[must_use]
    pub fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }
}

pub fn get_theme() -> &'static Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(Theme::load)
}

#[must_use]
pub fn apply(text: &str, spec: &str) -> String {
    let parts: Vec<&str> = spec.split_whitespace().collect();
    let has_bold = parts.contains(&"bold");
    let has_dim = parts.contains(&"dim") || parts.contains(&"dimmed");
    let is_bright = parts.contains(&"bright");
    let color = parts
        .iter()
        .find(|p| !["bold", "dim", "dimmed", "italic", "underline", "bright"].contains(p))
        .copied();

    macro_rules! c {
        ($e:expr) => {{
            let mut s = $e.to_string();
            if has_bold {
                s = s.bold().to_string();
            }
            if has_dim {
                s = s.dimmed().to_string();
            }
            s
        }};
    }

    match (is_bright, color) {
        (false, Some("red")) => c!(text.red()),
        (false, Some("green")) => c!(text.green()),
        (false, Some("blue")) => c!(text.blue()),
        (false, Some("cyan")) => c!(text.cyan()),
        (false, Some("magenta" | "purple")) => c!(text.magenta()),
        (false, Some("yellow")) => c!(text.yellow()),
        (false, Some("white")) => c!(text.white()),
        (false, Some("black")) => c!(text.black()),
        (true, Some("red")) => c!(text.bright_red()),
        (true, Some("green")) => c!(text.bright_green()),
        (true, Some("blue")) => c!(text.bright_blue()),
        (true, Some("cyan")) => c!(text.bright_cyan()),
        (true, Some("magenta" | "purple")) => c!(text.bright_magenta()),
        (true, Some("yellow")) => c!(text.bright_yellow()),
        (true, Some("white")) => c!(text.bright_white()),
        (true, Some("black")) => c!(text.bright_black()),
        _ => {
            let mut s = text.to_string();
            if has_bold {
                s = s.bold().to_string();
            }
            if has_dim {
                s = s.dimmed().to_string();
            }
            s
        }
    }
}

pub fn print_error(msg: &str) {
    let theme = get_theme();
    eprintln!("{}", apply(&format!("Error: {msg}"), &theme.error));
}

pub fn print_success(msg: &str) {
    let theme = get_theme();
    println!("{}", apply(msg, &theme.success));
}

pub fn print_header(msg: &str) {
    let theme = get_theme();
    println!("{}", apply(msg, &theme.header));
}

pub fn print_warning(msg: &str) {
    let theme = get_theme();
    println!("{}", apply(msg, &theme.warning));
}

pub fn println_dimmed(msg: &str) {
    let theme = get_theme();
    println!("{}", apply(msg, &theme.dimmed));
}

use tabled::settings::Color;

#[must_use]
pub fn table_border_color() -> Color {
    table_color(&get_theme().table_border)
}

#[must_use]
pub fn table_header_style() -> Color {
    table_color(&get_theme().table_header) | Color::BOLD
}

#[must_use]
pub fn table_row_style() -> Color {
    table_color(&get_theme().table_row)
}

fn table_color(spec: &str) -> Color {
    let parts: Vec<&str> = spec.split_whitespace().collect();
    let color = parts
        .iter()
        .find(|p| !["bold", "dim", "dimmed", "italic", "underline", "bright"].contains(p))
        .copied();

    match color {
        Some("red") => Color::FG_RED,
        Some("green") => Color::FG_GREEN,
        Some("blue") => Color::FG_BLUE,
        Some("cyan") => Color::FG_CYAN,
        Some("magenta" | "purple") => Color::FG_MAGENTA,
        Some("yellow") => Color::FG_YELLOW,
        Some("white") => Color::FG_WHITE,
        Some("black") => Color::FG_BLACK,
        _ => Color::FG_CYAN,
    }
}
