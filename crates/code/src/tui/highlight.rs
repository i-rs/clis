use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use std::sync::LazyLock;
use std::sync::Mutex;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style as SyntectStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

static HIGHLIGHTER: LazyLock<Mutex<Highlighter>> = LazyLock::new(|| Mutex::new(Highlighter::new()));

struct Highlighter {
    ss: SyntaxSet,
    ts: ThemeSet,
}

impl Highlighter {
    fn new() -> Self {
        Self {
            ss: SyntaxSet::load_defaults_newlines(),
            ts: ThemeSet::load_defaults(),
        }
    }

    fn highlight(&self, code: &str, lang: Option<&str>) -> Vec<Vec<Span<'static>>> {
        let syntax = lang
            .and_then(|l| self.ss.find_syntax_by_token(l))
            .unwrap_or_else(|| self.ss.find_syntax_plain_text());
        let theme = &self.ts.themes["base16-ocean.dark"];
        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut result = Vec::new();

        for line in LinesWithEndings::from(code) {
            let Ok(ranges) = highlighter.highlight_line(line, &self.ss) else {
                continue;
            };
            let mut spans: Vec<Span<'static>> = Vec::new();
            for (style, text) in ranges {
                let text = text.strip_suffix('\n').unwrap_or(text).to_string();
                spans.push(Span::styled(text, syntect_style_to_ratatui(&style)));
            }
            result.push(spans);
        }
        result
    }
}

fn syntect_style_to_ratatui(style: &SyntectStyle) -> Style {
    let mut s = Style::default().fg(Color::Rgb(
        style.foreground.r,
        style.foreground.g,
        style.foreground.b,
    ));
    if style.font_style.contains(FontStyle::BOLD) {
        s = s.add_modifier(Modifier::BOLD);
    }
    if style.font_style.contains(FontStyle::ITALIC) {
        s = s.add_modifier(Modifier::ITALIC);
    }
    if style.font_style.contains(FontStyle::UNDERLINE) {
        s = s.add_modifier(Modifier::UNDERLINED);
    }
    s
}

/// Highlight a code block with optional language hint.
/// Returns one `Vec<Span>` per line, suitable for direct use in ratatui `Line` widgets.
pub fn highlight_code_block(code: &str, lang: Option<&str>) -> Vec<Vec<Span<'static>>> {
    let Ok(highlighter) = HIGHLIGHTER.lock() else {
        return code
            .lines()
            .map(|l| vec![Span::raw(l.to_string())])
            .collect();
    };
    highlighter.highlight(code, lang)
}
