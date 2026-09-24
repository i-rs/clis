use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, Paragraph},
};

use crate::app::{App, spinner_char_alt};

/// Compute a centered popup rect within `area`, clamped to `pref_w × pref_h`.
fn centered_popup(area: Rect, pref_w: u16, pref_h: u16) -> Rect {
    let w = pref_w.min(area.width.saturating_sub(4));
    let h = pref_h.min(area.height.saturating_sub(4));
    let x = (area.width - w) / 2;
    let y = (area.height - h) / 2;
    Rect::new(x, y, w, h)
}

/// Build a rounded, centered-title list widget with theme-colored borders.
fn themed_list<'a>(
    title: &str,
    lines: Vec<Line<'a>>,
    theme: &i_rs_claw_core::theme::Theme,
) -> List<'a> {
    List::new(lines).block(
        Block::default()
            .title(title.to_string())
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(theme.primary())),
    )
}

pub(super) fn render_plan(f: &mut Frame, area: Rect, app: &App) {
    if app.chat.plan_steps.is_empty() {
        return;
    }
    let done_count = app.chat.plan_steps.iter().filter(|s| s.done).count();

    let total = app.chat.plan_steps.len();
    let max_show = total.min(5);

    let mut lines: Vec<Line> = Vec::with_capacity(max_show + 1);

    lines.push(Line::from(Span::styled(
        format!(" 📋 计划 ({}/{}) ", done_count, total),
        Style::default()
            .fg(app.config.theme.primary())
            .add_modifier(Modifier::BOLD),
    )));

    for (i, step) in app.chat.plan_steps.iter().take(max_show).enumerate() {
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

pub(super) fn render_processing(
    f: &mut Frame,
    area: Rect,
    app: &App,
    theme: &i_rs_claw_core::theme::Theme,
) {
    if !app.is_processing() || app.llm.status_text.is_empty() {
        return;
    }
    let spinner = spinner_char_alt(app.spinner_start, &['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷']);

    let label = Line::from(Span::styled(
        format!(" {}  {}", spinner, app.llm.status_text),
        Style::default()
            .fg(theme.primary())
            .add_modifier(Modifier::BOLD),
    ));
    f.render_widget(label, area);
}

pub(super) fn render_help_panel(f: &mut Frame, area: Rect, theme: &i_rs_claw_core::theme::Theme) {
    let popup_area = centered_popup(area, 50, 29);

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
        ("Ctrl+Shift+I", "Claw 状态面板"),
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

    let list = themed_list(" ⌨ 快捷键帮助 ", lines, theme);
    f.render_widget(list, popup_area);
}

pub(super) fn render_config_panel(
    f: &mut Frame,
    area: Rect,
    app: &App,
    theme: &i_rs_claw_core::theme::Theme,
) {
    let popup_area = centered_popup(area, 52, 16);

    let config = &app.config;
    let stats = &app.today_stats;
    let info: Vec<(String, String)> = vec![
        ("Provider".to_string(), config.provider.to_string()),
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
            Span::styled(value.as_str(), Style::default().fg(theme.text())),
        ]));
    }

    let list = themed_list(" ℹ 配置信息 ", lines, theme);
    f.render_widget(list, popup_area);
}

pub(super) fn render_tool_list_panel(
    f: &mut Frame,
    area: Rect,
    _app: &App,
    theme: &i_rs_claw_core::theme::Theme,
) {
    static TOOLS: std::sync::OnceLock<Vec<(String, String)>> = std::sync::OnceLock::new();
    let tools = TOOLS.get_or_init(|| {
        i_rs_claw_core::tools::ToolRegistry::new()
            .tool_info()
            .into_iter()
            .map(|(n, d)| (n.to_string(), d.to_string()))
            .collect()
    });

    let popup_area = centered_popup(area, 60, 20);

    let max_width = (popup_area.width as usize).saturating_sub(4);

    let mut lines: Vec<Line> = Vec::new();
    for (name, desc) in tools {
        let max_desc_chars = max_width.saturating_sub(18);
        let display_desc = if desc.chars().count() > max_desc_chars {
            let truncated: String = desc
                .chars()
                .take(max_desc_chars.saturating_sub(3))
                .collect();
            format!("{}...", truncated)
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

    let list = themed_list(" 🔧 可用工具 ", lines, theme);
    f.render_widget(list, popup_area);
}

pub(super) fn render_agent_list_panel(
    f: &mut Frame,
    area: Rect,
    app: &App,
    theme: &i_rs_claw_core::theme::Theme,
) {
    let popup_area = centered_popup(area, 70, 20);

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
            Style::default().fg(theme.dim_text()),
        )]));
    } else {
        let agent_ids: Vec<&String> = app.config.agents.keys().collect();
        for (i, id) in agent_ids.iter().enumerate() {
            let agent = &app.config.agents[*id];
            let model = agent.model.as_deref().unwrap_or(&app.config.model);
            let provider = agent.provider.unwrap_or(app.config.provider);
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
                Span::styled(caps, Style::default().fg(theme.dim_text())),
            ]));
        }
    }

    lines.push(Line::from(vec![Span::raw("")]));
    lines.push(Line::from(vec![Span::styled(
        "  Ctrl+S 切换  |  Ctrl+D 删除",
        Style::default().fg(theme.dim_text()),
    )]));

    let list = themed_list(" 👤 Agent 管理 ", lines, theme);
    f.render_widget(list, popup_area);
}

pub(super) fn render_stats_history_panel(
    f: &mut Frame,
    area: Rect,
    app: &App,
    theme: &i_rs_claw_core::theme::Theme,
) {
    let popup_area = centered_popup(area, 55, 16);

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
            Style::default().fg(theme.dim_text()),
        )]));
    } else {
        let max_tokens = app
            .stats_history
            .iter()
            .map(|d| d.total_tokens)
            .max()
            .unwrap_or(1);
        let bar_width = (popup_area.width as usize).saturating_sub(22);

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
                    Style::default().fg(theme.dim_text()),
                ),
            ]));
        }
    }

    let list = themed_list(" 📊 Token 用量 ", lines, theme);
    f.render_widget(list, popup_area);
}

pub(super) fn render_plugin_list_panel(
    f: &mut Frame,
    area: Rect,
    app: &App,
    theme: &i_rs_claw_core::theme::Theme,
) {
    let popup_area = centered_popup(area, 65, 20);

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
            Style::default().fg(theme.dim_text()),
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
                    Style::default().fg(theme.dim_text()),
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
            Style::default().fg(theme.dim_text()),
        )]));
    } else {
        for plugin in &app.plugin_list {
            let status_color = if plugin.enabled {
                theme.secondary()
            } else {
                theme.dim_text()
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
        Style::default().fg(theme.dim_text()),
    )]));

    let list = themed_list(" 🔌 插件与技能 ", lines, theme);
    f.render_widget(list, popup_area);
}

pub(super) fn render_backdrop(f: &mut Frame, area: Rect, theme: &i_rs_claw_core::theme::Theme) {
    f.render_widget(Clear, area);
    f.render_widget(
        Block::default().style(Style::default().bg(theme.background())),
        area,
    );
}

pub(super) fn render_feedback_prompt(
    f: &mut Frame,
    area: Rect,
    theme: &i_rs_claw_core::theme::Theme,
) {
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
                    .fg(theme.secondary())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("👍 满意   ", Style::default().fg(theme.text())),
            Span::styled(
                "  [n] ",
                Style::default()
                    .fg(theme.error())
                    .add_modifier(Modifier::BOLD),
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

pub(super) fn render_info_panel(
    f: &mut Frame,
    area: Rect,
    app: &App,
    theme: &i_rs_claw_core::theme::Theme,
) {
    let popup_area = centered_popup(area, 48, 22);

    let primary = theme.primary();
    let accent = theme.accent();
    let secondary = theme.secondary();
    let dim = theme.dim_text();
    let text = theme.text();

    let mut lines: Vec<Line> = Vec::new();

    // Header
    lines.push(Line::from(vec![
        Span::styled(
            "  ✦ ",
            Style::default().fg(primary).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            &app.current_agent,
            Style::default().fg(primary).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("  @{}", app.config.model), Style::default().fg(dim)),
    ]));
    lines.push(Line::from(Span::styled(
        format!(
            "  {}  |  {:?}",
            app.config.provider, app.config.execution_mode
        ),
        Style::default().fg(dim),
    )));
    lines.push(Line::from(Span::raw("")));

    // ── Usage ──
    lines.push(Line::from(Span::styled(
        " ── 今日用量 ──",
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![Span::styled(
        format!("  请求  {:>6} 次", app.today_stats.requests),
        Style::default().fg(text),
    )]));
    lines.push(Line::from(vec![Span::styled(
        format!("  Token {:>5} K", app.today_stats.tokens / 1000),
        Style::default().fg(text),
    )]));
    if app.today_stats.cost_usd > 0.0001 {
        lines.push(Line::from(vec![Span::styled(
            format!("  费用  ${:.4}", app.today_stats.cost_usd),
            Style::default().fg(secondary),
        )]));
    }
    lines.push(Line::from(Span::raw("")));

    // ── Session ──
    lines.push(Line::from(Span::styled(
        " ── 当前会话 ──",
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![Span::styled(
        format!("  消息  {:>6} 条", app.chat.messages.len()),
        Style::default().fg(text),
    )]));
    lines.push(Line::from(vec![Span::styled(
        format!("  工具调用 {:>3} 次", app.chat.tool_call_count),
        Style::default().fg(text),
    )]));
    if let Some(ref usage) = app.llm.token_usage {
        lines.push(Line::from(vec![Span::styled(
            format!(
                "  最后请求 {:>4} in + {:>4} out",
                usage.prompt_tokens, usage.completion_tokens
            ),
            Style::default().fg(dim),
        )]));
    }
    if !app.http_logs.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            format!("  HTTP 日志 {:>3} 条", app.http_logs.len()),
            Style::default().fg(dim),
        )]));
    }
    lines.push(Line::from(Span::raw("")));

    // ── System ──
    lines.push(Line::from(Span::styled(
        " ── 系统 ──",
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    )));
    let agent_count = app.config.agents.len() + 1; // +1 for default
    let mcp_count = app.config.mcp_servers.len();
    let plugin_count = app.plugin_list.len();
    let skill_count = app.skill_list.len();
    lines.push(Line::from(vec![Span::styled(
        format!("  Agent  {:>4}  ·  MCP {:>4}", agent_count, mcp_count),
        Style::default().fg(text),
    )]));
    lines.push(Line::from(vec![Span::styled(
        format!("  插件  {:>4}  ·  技能 {:>4}", plugin_count, skill_count),
        Style::default().fg(text),
    )]));
    let tools_text = if app.config.enabled_tools.is_empty() {
        "全部".to_string()
    } else {
        format!("{} 个", app.config.enabled_tools.len())
    };
    lines.push(Line::from(vec![Span::styled(
        format!("  已启用工具  {}", tools_text),
        Style::default().fg(text),
    )]));
    lines.push(Line::from(vec![Span::styled(
        format!(
            "  插件发现  {}",
            if app.config.plugins_auto_discover {
                "✓"
            } else {
                "✗"
            }
        ),
        Style::default().fg(text),
    )]));
    lines.push(Line::from(Span::raw("")));

    // ── Footer ──
    lines.push(Line::from(Span::styled(
        " ─────────────────────────────────────────────",
        Style::default().fg(dim),
    )));
    lines.push(Line::from(Span::styled(
        "  Esc 关闭  |  Ctrl+H 帮助  |  Ctrl+I 配置",
        Style::default().fg(dim),
    )));

    let info_list = themed_list(" ℹ Claw 状态 ", lines, theme);
    f.render_widget(info_list, popup_area);
}

pub(super) fn render_theme_picker(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.config.theme;
    let primary = theme.primary();
    let dim = theme.dim_text();
    let bg = theme.background();

    let themes = i_rs_claw_core::theme::BUILT_IN_THEMES;
    let idx = app.overlay.theme_index.min(themes.len().saturating_sub(1));

    let popup_height = (themes.len() as u16).saturating_add(2);
    let popup_width = 44u16.min(area.width.saturating_sub(8));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    f.render_widget(Clear, popup_area);

    let mut lines: Vec<Line> = Vec::new();

    for (i, preset) in themes.iter().enumerate() {
        let selected = i == idx;
        let preset_theme =
            i_rs_claw_core::theme::Theme::from_preset(preset.name).unwrap_or_default();
        let p_color = preset_theme.primary();
        let s_color = preset_theme.secondary();
        let a_color = preset_theme.accent();
        let bg_color = preset_theme.background();

        let sel_bg = if selected { theme.selection_bg() } else { bg };

        let name_fg = if selected { theme.text() } else { dim };
        let label_fg = if selected { p_color } else { dim };

        let prefix = if selected { " > " } else { "   " };

        lines.push(Line::from(vec![
            Span::styled(prefix.to_string(), Style::default().fg(name_fg).bg(sel_bg)),
            Span::styled(
                format!("{:<10}", preset.name),
                Style::default()
                    .fg(name_fg)
                    .bg(sel_bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ".to_string(), Style::default().bg(sel_bg)),
            Span::styled(
                format!("{:<6}", preset.label),
                Style::default().fg(label_fg).bg(sel_bg),
            ),
            Span::styled("  ".to_string(), Style::default().bg(sel_bg)),
            Span::styled("██".to_string(), Style::default().fg(p_color).bg(sel_bg)),
            Span::styled(" ".to_string(), Style::default().bg(sel_bg)),
            Span::styled("██".to_string(), Style::default().fg(s_color).bg(sel_bg)),
            Span::styled(" ".to_string(), Style::default().bg(sel_bg)),
            Span::styled("██".to_string(), Style::default().fg(a_color).bg(sel_bg)),
            Span::styled(" ".to_string(), Style::default().bg(sel_bg)),
            Span::styled("██".to_string(), Style::default().fg(bg_color).bg(sel_bg)),
        ]));
    }

    lines.push(Line::from(Span::styled("", Style::default().bg(bg))));
    lines.push(Line::from(Span::styled(
        " ↑↓ 预览  Enter 确认  Esc 取消",
        Style::default().fg(dim).bg(bg),
    )));

    let list = List::new(lines).block(
        Block::default()
            .title(" 🎨 主题 ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(primary))
            .style(Style::default().bg(bg)),
    );

    f.render_widget(list, popup_area);
}
