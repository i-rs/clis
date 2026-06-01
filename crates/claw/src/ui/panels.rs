use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, Paragraph},
};
use std::sync::OnceLock;

use crate::app::{App, spinner_char_alt};

static BACKDROP_FILL: OnceLock<String> = OnceLock::new();

fn backdrop_fill(width: u16) -> &'static str {
    let fill = BACKDROP_FILL.get_or_init(|| " ".repeat(512));
    &fill[..(width as usize).min(fill.len())]
}

pub(super) fn render_plan(f: &mut Frame, area: Rect, app: &App) {
    if app.plan_steps.is_empty() {
        return;
    }
    let done_count = app.plan_steps.iter().filter(|s| s.done).count();

    let total = app.plan_steps.len();
    let max_show = total.min(5);

    let mut lines: Vec<Line> = Vec::with_capacity(max_show + 1);

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

pub(super) fn render_processing(f: &mut Frame, area: Rect, app: &App, theme: &crate::theme::Theme) {
    if !app.is_processing() || app.status_text.is_empty() {
        return;
    }
    let spinner = spinner_char_alt(app.spinner_start, &['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷']);

    let label = Line::from(Span::styled(
        format!(" {}  {}", spinner, app.status_text),
        Style::default()
            .fg(theme.primary())
            .add_modifier(Modifier::BOLD),
    ));
    f.render_widget(label, area);
}

pub(super) fn render_help_panel(f: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
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
        ("Ctrl+F", "对回答进行反馈"),
        ("Ctrl+I", "查看配置信息"),
        ("Ctrl+L", "会话列表"),
        ("Ctrl+T", "查看可用工具"),
        ("Ctrl+A", "Agent 管理"),
        ("Ctrl+Shift+U", "Token 用量"),
        ("Ctrl+Shift+P", "插件与技能"),
        ("Ctrl+Shift+C", "复制当前消息"),
        ("Ctrl+E", "导出会话为 Markdown"),
        ("Alt+Enter", "输入换行"),
        ("Fn", "语音输入 (macOS)"),
        ("", ""),
        ("--- 输入编辑 ---", ""),
        ("Ctrl+←/→", "按词移动光标"),
        ("Ctrl+Backspace", "删除前一个词"),
        ("Ctrl+U", "删除到行首"),
        ("Ctrl+K", "删除到行尾"),
        ("Ctrl+Z/Y", "撤销/重做"),
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
            lines.push(Line::from(Span::styled(
                *key,
                Style::default()
                    .fg(theme.secondary())
                    .add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<18}", key),
                    Style::default().fg(theme.dim_text()),
                ),
                Span::styled(desc.to_string(), Style::default().fg(theme.text())),
            ]));
        }
    }

    let list = List::new(lines).block(
        Block::default()
            .title(" ⌨ 快捷键帮助 ")
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary())),
    );
    f.render_widget(list, popup_area);
}

pub(super) fn render_config_panel(
    f: &mut Frame,
    area: Rect,
    app: &App,
    theme: &crate::theme::Theme,
) {
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
        (
            "Execution".to_string(),
            format!("{:?}", config.execution_mode),
        ),
        (String::new(), String::new()),
        (
            "Tools".to_string(),
            if config.enabled_tools.is_empty() {
                "全部启用".to_string()
            } else {
                format!("{} 个", config.enabled_tools.len())
            },
        ),
        (
            "MCP Servers".to_string(),
            format!("{} 个", config.mcp_servers.len()),
        ),
        (
            "Plugins".to_string(),
            if config.plugins_auto_discover {
                "自动发现"
            } else {
                "禁用"
            }
            .to_string(),
        ),
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
                Style::default().fg(theme.dim_text()),
            ),
            Span::styled(value.clone(), Style::default().fg(theme.text())),
        ]));
    }

    let list = List::new(lines).block(
        Block::default()
            .title(" ℹ 配置信息 ")
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary())),
    );
    f.render_widget(list, popup_area);
}

pub(super) fn render_tool_list_panel(
    f: &mut Frame,
    area: Rect,
    _app: &App,
    theme: &crate::theme::Theme,
) {
    static TOOLS: std::sync::OnceLock<Vec<(String, String)>> = std::sync::OnceLock::new();
    let tools = TOOLS.get_or_init(|| {
        crate::tools::ToolRegistry::new()
            .tool_info()
            .into_iter()
            .map(|(n, d)| (n.to_string(), d.to_string()))
            .collect()
    });

    let popup_width = 60u16.min(area.width.saturating_sub(4));
    let popup_height = 20u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    let max_width = (popup_width as usize).saturating_sub(4);

    let mut lines: Vec<Line> = Vec::new();
    for (name, desc) in tools {
        let display_desc = if desc.len() > max_width - 18 {
            format!("{}...", &desc[..(max_width - 21)])
        } else {
            desc.to_string()
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<16}", name),
                Style::default()
                    .fg(theme.dim_text())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(display_desc, Style::default().fg(theme.text())),
        ]));
    }

    let list = List::new(lines).block(
        Block::default()
            .title(" 🔧 可用工具 ")
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary())),
    );
    f.render_widget(list, popup_area);
}

pub(super) fn render_agent_list_panel(
    f: &mut Frame,
    area: Rect,
    app: &App,
    theme: &crate::theme::Theme,
) {
    let popup_width = 70u16.min(area.width.saturating_sub(4));
    let popup_height = 20u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(vec![
        Span::styled(
            "  当前 Agent: ",
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            &app.current_agent,
            Style::default()
                .fg(theme.secondary())
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![Span::raw("")]));

    if app.config.agents.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  (无自定义 Agent，使用默认配置)",
            Style::default().fg(Color::DarkGray),
        )]));
    } else {
        let agent_ids: Vec<&String> = app.config.agents.keys().collect();
        for (i, id) in agent_ids.iter().enumerate() {
            let agent = &app.config.agents[*id];
            let model = agent.model.as_deref().unwrap_or(&app.config.model);
            let provider = agent.provider.as_deref().unwrap_or(&app.config.provider);
            let caps = if agent.capabilities.is_empty() {
                String::new()
            } else {
                format!(" [{}]", agent.capabilities.join(", "))
            };
            let selected = i == app.overlay.agent_picker_index;
            let marker = if **id == app.current_agent {
                if selected { " ▶" } else { " ●" }
            } else if selected {
                " ▶"
            } else {
                "  "
            };
            let id_style = if selected {
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(theme.dim_text())
                    .add_modifier(Modifier::BOLD)
            };
            lines.push(Line::from(vec![
                Span::styled(format!("{} {:<14}", marker, id), id_style),
                Span::styled(
                    format!(" {}@{}", provider, model),
                    Style::default().fg(theme.text()),
                ),
                Span::styled(caps, Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    lines.push(Line::from(vec![Span::raw("")]));
    lines.push(Line::from(vec![Span::styled(
        "  Ctrl+S 切换  |  Ctrl+D 删除",
        Style::default().fg(Color::DarkGray),
    )]));

    let list = List::new(lines).block(
        Block::default()
            .title(" 👤 Agent 管理 ")
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary())),
    );
    f.render_widget(list, popup_area);
}

pub(super) fn render_stats_history_panel(
    f: &mut Frame,
    area: Rect,
    app: &App,
    theme: &crate::theme::Theme,
) {
    let popup_width = 55u16.min(area.width.saturating_sub(4));
    let popup_height = 16u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(vec![
        Span::styled(
            "  今日: ",
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "{} 请求 | {}K tokens",
                app.today_stats.requests,
                app.today_stats.tokens / 1000
            ),
            Style::default().fg(theme.text()),
        ),
    ]));
    if app.today_stats.cost_usd > 0.001 {
        lines.push(Line::from(vec![
            Span::styled(
                "  费用: ",
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("${:.4}", app.today_stats.cost_usd),
                Style::default().fg(theme.secondary()),
            ),
        ]));
    }
    lines.push(Line::from(vec![Span::raw("")]));

    if app.stats_history.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  (暂无历史数据)",
            Style::default().fg(Color::DarkGray),
        )]));
    } else {
        let max_tokens = app
            .stats_history
            .iter()
            .map(|d| d.total_tokens)
            .max()
            .unwrap_or(1);
        let bar_width = (popup_width as usize).saturating_sub(22);

        for day in &app.stats_history {
            let token_k = day.total_tokens / 1000;
            let bar_len = if max_tokens > 0 {
                ((day.total_tokens as f64 / max_tokens as f64) * bar_width as f64) as usize
            } else {
                0
            };
            let bar = "█".repeat(bar_len.max(1));
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:>5}  ", day.date),
                    Style::default().fg(theme.dim_text()),
                ),
                Span::styled(bar, Style::default().fg(theme.primary())),
                Span::styled(
                    format!(" {:>3}K", token_k),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    let list = List::new(lines).block(
        Block::default()
            .title(" 📊 Token 用量 ")
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary())),
    );
    f.render_widget(list, popup_area);
}

pub(super) fn render_plugin_list_panel(
    f: &mut Frame,
    area: Rect,
    app: &App,
    theme: &crate::theme::Theme,
) {
    let popup_width = 65u16.min(area.width.saturating_sub(4));
    let popup_height = 20u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(vec![Span::styled(
        "  技能 ",
        Style::default()
            .fg(theme.accent())
            .add_modifier(Modifier::BOLD),
    )]));

    if app.skill_list.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "    (无已安装技能)",
            Style::default().fg(Color::DarkGray),
        )]));
    } else {
        for skill in &app.skill_list {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    ✦ {:<18}", skill.name),
                    Style::default()
                        .fg(theme.dim_text())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    skill.content.chars().take(30).collect::<String>(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    lines.push(Line::from(vec![Span::raw("")]));
    lines.push(Line::from(vec![Span::styled(
        "  插件 ",
        Style::default()
            .fg(theme.accent())
            .add_modifier(Modifier::BOLD),
    )]));

    if app.plugin_list.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "    (无已安装插件)",
            Style::default().fg(Color::DarkGray),
        )]));
    } else {
        for plugin in &app.plugin_list {
            let status_color = if plugin.enabled {
                theme.secondary()
            } else {
                Color::DarkGray
            };
            let status = if plugin.enabled { "✓" } else { "✗" };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {} {:<16}", status, plugin.name),
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&plugin.description, Style::default().fg(theme.text())),
            ]));
        }
    }

    lines.push(Line::from(vec![Span::raw("")]));
    lines.push(Line::from(vec![Span::styled(
        "  Ctrl+E 切换插件  |  Ctrl+R 删除技能",
        Style::default().fg(Color::DarkGray),
    )]));

    let list = List::new(lines).block(
        Block::default()
            .title(" 🔌 插件与技能 ")
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary())),
    );
    f.render_widget(list, popup_area);
}

pub(super) fn render_backdrop(f: &mut Frame, area: Rect) {
    f.render_widget(Clear, area);
    let fill = backdrop_fill(area.width);
    let lines: Vec<Line> = (0..area.height)
        .map(|_| {
            Line::from(Span::styled(
                fill,
                Style::default().bg(Color::Rgb(8, 8, 15)),
            ))
        })
        .collect();
    f.render_widget(Paragraph::new(lines), area);
}

pub(super) fn render_feedback_prompt(f: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
    let width = 44u16.min(area.width.saturating_sub(4));
    let height = 6u16;
    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;
    let popup = Rect::new(x, y, width, height);

    let block = Block::default()
        .title(" 对本次回答的反馈 ")
        .title_style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent()));

    let inner = block.inner(popup);
    let text = vec![
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled(
                "  [y] ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("👍 满意   ", Style::default().fg(theme.text())),
            Span::styled(
                "  [n] ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled("👎 不满意", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("  [Esc] ", Style::default().fg(theme.text())),
            Span::styled("取消", Style::default().fg(theme.text())),
        ]),
    ];

    f.render_widget(block, popup);
    f.render_widget(Paragraph::new(text).alignment(Alignment::Center), inner);
}
