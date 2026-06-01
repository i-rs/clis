use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use super::input;
use crate::app::App;

pub(super) fn render_completions(f: &mut Frame, area: Rect, app: &App) {
    if app.overlay.tab_completions.is_empty() {
        return;
    }

    let count = app.overlay.tab_completions.len();
    let popup_height = (count as u16).min(12).saturating_add(2);
    let popup_width = (area.width as f32 * 0.45) as u16;
    let popup_x = area.x + 2;

    let status_height: u16 = 1;
    let input_h = input::input_height(&app.input.text);
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
