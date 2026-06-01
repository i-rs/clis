use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
};

use super::input;
use crate::app::{App, SLASH_COMMANDS};

pub(super) fn render_completions(f: &mut Frame, area: Rect, app: &App) {
    if app.overlay.tab_completions.is_empty() {
        return;
    }

    let count = app.overlay.tab_completions.len();
    let popup_height = (count as u16).min(12).saturating_add(2);
    let popup_width = (area.width as f32 * 0.45) as u16;
    let popup_x = area.x + 2;

    let status_height: u16 = 1;
    let input_h = input::input_height(&app.input.text, area.width);
    let popup_y = area
        .bottom()
        .saturating_sub(status_height + input_h + 1 + popup_height + 1);

    let popup_y = popup_y.max(area.y);

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    let idx = app.overlay.tab_completion_index;
    let theme_primary = app.config.theme.primary();

    let mut items: Vec<ListItem> = Vec::new();

    items.push(ListItem::new(vec![Line::from(Span::styled(
        format!(" Tab 补全 ({} 个)", count),
        Style::default()
            .fg(theme_primary)
            .add_modifier(Modifier::BOLD),
    ))]));
    items.push(ListItem::new(vec![Line::from(Span::styled(
        " ────────────────────────────────────────",
        Style::default().fg(Color::DarkGray),
    ))]));

    for (i, completion) in app.overlay.tab_completions.iter().enumerate() {
        if i >= 12 {
            let remaining = count - 12;
            items.push(ListItem::new(vec![Line::from(Span::styled(
                format!("   ... 还有 {} 个", remaining),
                Style::default().fg(Color::DarkGray),
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
            Style::default().fg(Color::White)
        };
        items.push(ListItem::new(vec![Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(completion.clone(), style),
        ])]));
    }

    items.push(ListItem::new(vec![Line::from(Span::styled(
        " ────────────────────────────────────────",
        Style::default().fg(Color::DarkGray),
    ))]));
    items.push(ListItem::new(vec![Line::from(Span::styled(
        " Tab 选择  Shift+Tab 反向  Esc 关闭",
        Style::default().fg(Color::DarkGray),
    ))]));

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme_primary)),
    );

    f.render_widget(list, popup_area);
}

pub(super) fn render_slash_panel(f: &mut Frame, area: Rect, app: &App) {
    if !app.overlay.slash_visible {
        return;
    }

    let query = if app.input.text.starts_with('/') {
        &app.input.text
    } else {
        ""
    };
    let matches: Vec<&crate::app::SlashCommand> = SLASH_COMMANDS
        .iter()
        .filter(|cmd| {
            if query.is_empty() {
                return true;
            }
            let q = query.to_lowercase();
            cmd.name.starts_with(&q)
                || (query.len() > 1 && cmd.desc.contains(&query[1..]))
        })
        .collect();

    if matches.is_empty() {
        return;
    }

    let count = matches.len();
    let max_visible = 8;
    let visible = (count as u16).min(max_visible);
    let popup_height = visible;
    let popup_width = 52u16.min(area.width.saturating_sub(4));
    let popup_x = area.x + 2;

    let status_height: u16 = 1;
    let input_h = input::input_height(&app.input.text, area.width);
    let popup_y = area
        .bottom()
        .saturating_sub(status_height + input_h + 1 + popup_height);
    let popup_y = popup_y.max(area.y);

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);
    f.render_widget(Clear, popup_area);

    let idx = app.overlay.slash_index.min(count.saturating_sub(1));

    let bg = app.config.theme.background();
    let primary = app.config.theme.primary();
    let dim = app.config.theme.dim_text();
    let selection_bg = Color::Rgb(30, 30, 46);

    let scroll_offset = if idx >= max_visible as usize {
        idx - max_visible as usize + 1
    } else {
        0
    };

    let mut items: Vec<ListItem> = Vec::new();

    for (i, cmd) in matches.iter().enumerate().skip(scroll_offset) {
        if i - scroll_offset >= max_visible as usize {
            break;
        }
        let selected = i == idx;

        let row_bg = if selected { selection_bg } else { bg };
        let name_fg = if selected { primary } else { primary };
        let desc_fg = if selected { Color::White } else { dim };
        let shortcut_fg = if selected { dim } else { Color::DarkGray };

        let name_mod = if selected {
            Modifier::BOLD
        } else {
            Modifier::empty()
        };

        let name = format!(" {:<12}", cmd.name);
        let desc = cmd.desc;
        let shortcut = if cmd.shortcut.is_empty() {
            String::new()
        } else {
            format!("  {}", cmd.shortcut)
        };

        items.push(ListItem::new(vec![Line::from(vec![
            Span::styled(name, Style::default().fg(name_fg).bg(row_bg).add_modifier(name_mod)),
            Span::styled(desc, Style::default().fg(desc_fg).bg(row_bg)),
            Span::styled(shortcut, Style::default().fg(shortcut_fg).bg(row_bg)),
        ])]));
    }

    let list = List::new(items).block(
        Block::default()
            .style(Style::default().bg(bg)),
    );

    f.render_widget(list, popup_area);
}
