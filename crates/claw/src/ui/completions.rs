use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::input;
use crate::app::{App, SLASH_COMMANDS};

pub(super) fn render_completions(f: &mut Frame, area: Rect, app: &mut App) {
    if app.overlay.tab_completions.is_empty() {
        return;
    }

    let count = app.overlay.tab_completions.len();
    let popup_height = (count as u16).min(12).saturating_add(2);
    let popup_width = (area.width as f32 * 0.45) as u16;
    let popup_x = area.x + 2;

    let status_height: u16 = 1;
    let input_h = input::input_height(
        &app.input.text,
        area.width,
        &mut app.render_state.cached_input_height,
    );
    let popup_y = area
        .bottom()
        .saturating_sub(status_height + input_h + 1 + popup_height + 1);

    let popup_y = popup_y.max(area.y);

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    let idx = app.overlay.tab_completion_index;
    let theme = &app.config.theme;
    let theme_primary = theme.primary();
    let dim = theme.dim_text();
    let text_color = theme.text();

    let mut items: Vec<ListItem> = Vec::new();

    items.push(ListItem::new(vec![Line::from(Span::styled(
        format!(" Tab 补全 ({} 个)", count),
        Style::default()
            .fg(theme_primary)
            .add_modifier(Modifier::BOLD),
    ))]));
    items.push(ListItem::new(vec![Line::from(Span::styled(
        " ────────────────────────────────────────",
        Style::default().fg(dim),
    ))]));

    for (i, completion) in app.overlay.tab_completions.iter().enumerate() {
        if i >= 12 {
            let remaining = count - 12;
            items.push(ListItem::new(vec![Line::from(Span::styled(
                format!("   ... 还有 {} 个", remaining),
                Style::default().fg(dim),
            ))]));
            break;
        }
        let selected = i == idx;
        let prefix = if selected { " ▶ " } else { "    " };
        let style = if selected {
            Style::default()
                .fg(theme_primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(text_color)
        };
        items.push(ListItem::new(vec![Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(completion.clone(), style),
        ])]));
    }

    items.push(ListItem::new(vec![Line::from(Span::styled(
        " ────────────────────────────────────────",
        Style::default().fg(dim),
    ))]));
    items.push(ListItem::new(vec![Line::from(Span::styled(
        " Tab 选择  Shift+Tab 反向  Esc 关闭",
        Style::default().fg(dim),
    ))]));

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme_primary)),
    );

    f.render_widget(list, popup_area);
}

fn slash_matches_query(cmd: &crate::app::SlashCommand, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let q = query.to_ascii_lowercase();
    cmd.name.starts_with(&q) || (query.len() > 1 && cmd.desc.contains(&query[1..]))
}

pub(super) fn slash_picker_height(app: &App) -> u16 {
    if !app.overlay.slash_visible {
        return 0;
    }
    let query = if app.input.text.starts_with('/') {
        &app.input.text
    } else {
        ""
    };
    let count = SLASH_COMMANDS
        .iter()
        .filter(|cmd| slash_matches_query(cmd, query))
        .count();
    if count == 0 {
        0
    } else {
        (count as u16).min(8).saturating_add(1)
    }
}

pub(super) fn render_slash_panel(f: &mut Frame, area: Rect, app: &App) {
    if !app.overlay.slash_visible || area.height == 0 {
        return;
    }

    let query = if app.input.text.starts_with('/') {
        &app.input.text
    } else {
        ""
    };
    let matches: Vec<&crate::app::SlashCommand> = SLASH_COMMANDS
        .iter()
        .filter(|cmd| slash_matches_query(cmd, query))
        .collect();

    if matches.is_empty() {
        return;
    }

    let count = matches.len();
    let content_lines = (area.height.saturating_sub(1)) as usize;
    let max_visible = content_lines.max(1);
    let idx = app.overlay.slash_index.min(count.saturating_sub(1));

    let primary = app.config.theme.primary();
    let dim = app.config.theme.dim_text();
    let text_color = app.config.theme.text();
    let bg = app.config.theme.background();
    let border_color = app.config.theme.border();

    let scroll_offset = if idx >= max_visible {
        idx - max_visible + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();

    for (i, cmd) in matches.iter().enumerate().skip(scroll_offset) {
        if i - scroll_offset >= max_visible {
            break;
        }
        let selected = i == idx;

        let marker = if selected { " ▌" } else { "  " };
        let name_style = if selected {
            Style::default().fg(primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(text_color)
        };
        let desc_fg = if selected { text_color } else { dim };
        let shortcut_fg = dim;

        let name = format!("/{}", &cmd.name[1..]);
        let shortcut = if cmd.shortcut.is_empty() {
            String::new()
        } else {
            format!("  {}", cmd.shortcut)
        };

        lines.push(Line::from(vec![
            Span::styled(
                marker,
                if selected {
                    Style::default().fg(primary)
                } else {
                    Style::default().fg(dim)
                },
            ),
            Span::styled(name, name_style),
            Span::raw("  "),
            Span::styled(cmd.desc, Style::default().fg(desc_fg)),
            Span::styled(shortcut, Style::default().fg(shortcut_fg)),
        ]));
    }

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(bg));

    let paragraph = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(paragraph, area);
}
