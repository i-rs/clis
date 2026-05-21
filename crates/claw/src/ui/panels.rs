use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, Paragraph},
    Frame,
};

use crate::app::App;
use super::utils;

pub(super) fn render_plan(f: &mut Frame, area: Rect, app: &App) {
    if app.plan_steps.is_empty() {
        return;
    }
    let done_count = app.plan_steps.iter().filter(|s| s.done).count();

    let total = app.plan_steps.len();
    let max_show = total.min(5);

    let mut lines: Vec<Line> = Vec::with_capacity(max_show + 1);

    // Header with summary
    lines.push(Line::from(Span::styled(
        format!(" 📋 计划 ({}/{}) ", done_count, total),
        Style::default()
            .fg(app.config.theme.primary())
            .add_modifier(Modifier::BOLD),
    )));

    for (i, step) in app.plan_steps.iter().take(max_show).enumerate() {
        let (icon, color) = if step.done {
            ("✓", app.config.theme.secondary())
        } else if i == done_count || (done_count == 0 && i == 0) {
            ("⏳", app.config.theme.accent())
        } else {
            ("○", app.config.theme.dim_text())
        };
        lines.push(Line::from(Span::styled(
            format!("   {} {}", icon, step.description),
            Style::default().fg(color),
        )));
    }

    if total > max_show {
        lines.push(Line::from(Span::styled(
            format!("   ... 还有 {} 步", total - max_show),
            Style::default().fg(app.config.theme.dim_text()),
        )));
    }

    f.render_widget(Paragraph::new(lines), area);
}

pub(super) fn render_processing(f: &mut Frame, area: Rect, app: &App) {
    if !app.is_processing() || app.status_text.is_empty() {
        return;
    }
    let dots = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
    let frame = (app.messages.len() + app.tool_call_count) % dots.len();
    let spinner = dots[frame];

    // Split area: first line for spinner+status, rest for reasoning
    let (status_area, reason_area) = if area.height > 1 {
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([ratatui::layout::Constraint::Length(1), ratatui::layout::Constraint::Min(1)])
            .split(area);
        (chunks[0], chunks[1])
    } else {
        (area, Rect::default())
    };

    let label = Line::from(Span::styled(
        format!(" {}  {}", spinner, app.status_text),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    f.render_widget(label, status_area);

    // Render reasoning content below the processing line if available
    if !app.current_reasoning.is_empty() && reason_area.height >= 1 {
        let mut reason_lines: Vec<Line> = app
            .current_reasoning
            .lines()
            .take(reason_area.height as usize)
            .map(|line| {
                let display = utils::truncate_str(line, (area.width as usize).saturating_sub(4).max(20));
                Line::from(Span::styled(
                    format!(" 🤔 {}", display),
                    Style::default().fg(Color::Rgb(120, 120, 140)),
                ))
            })
            .collect();

        // Fill remaining lines
        while reason_lines.len() < reason_area.height as usize {
            reason_lines.push(Line::from(Span::raw("")));
        }

        f.render_widget(Paragraph::new(reason_lines), reason_area);
    }
}

pub(super) fn render_help_panel(f: &mut Frame, area: Rect) {
    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = 28u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    let shortcuts = [
        ("Ctrl+Q", "退出程序"),
        ("Ctrl+N", "新建会话"),
        ("Ctrl+S", "消息选择模式"),
        ("Ctrl+H", "显示/隐藏帮助"),
        ("Ctrl+P", "Agent 切换器"),
        ("Ctrl+R", "HTTP 调试面板"),
        ("Ctrl+I", "查看配置信息"),
        ("Ctrl+L", "会话列表"),
        ("Ctrl+Shift+C", "复制当前消息"),
        ("Alt+Enter", "输入换行"),
        ("Fn", "语音输入 (macOS)"),
        ("", ""),
        ("--- 选择模式 ---", ""),
        ("↑/↓", "切换选中消息"),
        ("Space", "展开/折叠工具调用"),
        ("Ctrl+D", "删除选中消息"),
        ("Esc/q", "退出选择模式"),
        ("", ""),
        ("--- 会话列表 ---", ""),
        ("Ctrl+F", "搜索会话"),
        ("Enter", "切换到会话"),
        ("Ctrl+D", "删除会话"),
        ("Ctrl+R", "重命名会话"),
    ];

    let mut lines: Vec<Line> = Vec::new();
    for (key, desc) in &shortcuts {
        if key.is_empty() {
            lines.push(Line::from(Span::raw("")));
            continue;
        }
        if desc.is_empty() {
            // Section header
            lines.push(Line::from(Span::styled(
                *key,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<18}", key),
                    Style::default().fg(Color::Rgb(180, 180, 120)),
                ),
                Span::styled(
                    desc.to_string(),
                    Style::default().fg(Color::White),
                ),
            ]));
        }
    }

    let list = List::new(lines).block(
        Block::default()
            .title(" ⌨ 快捷键帮助 ")
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(list, popup_area);
}

pub(super) fn render_config_panel(f: &mut Frame, area: Rect, app: &App) {
    let popup_width = 52u16.min(area.width.saturating_sub(4));
    let popup_height = 16u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    let config = &app.config;
    let stats = &app.today_stats;
    let info: Vec<(String, String)> = vec![
        ("Provider".to_string(), config.provider.clone()),
        ("Model".to_string(), config.model.clone()),
        ("Base URL".to_string(), config.base_url.clone()),
        ("Execution".to_string(), format!("{:?}", config.execution_mode)),
        (String::new(), String::new()),
        ("Tools".to_string(), if config.enabled_tools.is_empty() {
            "全部启用".to_string()
        } else {
            format!("{} 个", config.enabled_tools.len())
        }),
        ("MCP Servers".to_string(), format!("{} 个", config.mcp_servers.len())),
        ("Plugins".to_string(), if config.plugins_auto_discover { "自动发现" } else { "禁用" }.to_string()),
        (String::new(), String::new()),
        ("今日请求".to_string(), format!("{} 次", stats.requests)),
        ("今日 Token".to_string(), format!("{} tok", stats.tokens)),
        ("今日费用".to_string(), format!("${:.4}", stats.cost_usd)),
    ];

    let mut lines: Vec<Line> = Vec::new();
    for (label, value) in &info {
        if label.is_empty() {
            lines.push(Line::from(Span::raw("")));
            continue;
        }
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<16}", label),
                Style::default().fg(Color::Rgb(180, 180, 120)),
            ),
            Span::styled(
                value.clone(),
                Style::default().fg(Color::White),
            ),
        ]));
    }

    let list = List::new(lines).block(
        Block::default()
            .title(" ℹ 配置信息 ")
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(list, popup_area);
}

pub(super) fn render_backdrop(f: &mut Frame, area: Rect) {
    f.render_widget(
        Block::default()
            .style(Style::default().bg(Color::Rgb(20, 20, 30))),
        area,
    );
}
