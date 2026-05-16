use crate::config::Config;
use crate::session::SessionManager;
use crossterm::event::{self, Event, KeyCode};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Terminal;
use std::io::{self, Write};

// =============================================
// Interactive Config Wizard
// =============================================

pub fn run_config() -> anyhow::Result<()> {
    let config_path = claw_dir().join("config.toml");
    let mut cfg = if config_path.exists() {
        Config::load().unwrap_or_else(|_| Config::new())
    } else {
        println!("未发现配置文件，开始交互式设置...\n");
        Config::new()
    };

    // ── API Key ──
    let current = if cfg.api_key.is_empty() {
        String::new()
    } else {
        format!(
            " [{}...{}]",
            &cfg.api_key[..4.min(cfg.api_key.len())],
            &cfg.api_key[cfg.api_key.len().saturating_sub(4)..]
        )
    };
    print!("API Key{}: ", current);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        cfg.api_key = trimmed;
    }

    // ── Base URL ──
    print!("Base URL [{}]: ", cfg.base_url);
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        cfg.base_url = trimmed;
    }

    // ── Model ──
    print!("Model [{}]: ", cfg.model);
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        cfg.model = trimmed;
    }

    // ── Save ──
    if cfg.api_key.is_empty() {
        anyhow::bail!("API Key 不能为空，配置未保存");
    }

    cfg.save()?;
    let total_tools = crate::tools::TOOL_INDEX.len();
    let enabled_count = if cfg.enabled_tools.is_empty() {
        total_tools
    } else {
        cfg.enabled_tools.len()
    };

    println!("\n配置摘要：");
    println!(
        "  API Key: {}...{}",
        &cfg.api_key[..4.min(cfg.api_key.len())],
        &cfg.api_key[cfg.api_key.len().saturating_sub(4)..]
    );
    println!("  Base URL: {}", cfg.base_url);
    println!("  Model: {}", cfg.model);
    println!(
        "  工具: {} ({} 个 / 总 {} 个)",
        if cfg.enabled_tools.is_empty() {
            "全部启用"
        } else {
            "部分启用"
        },
        enabled_count,
        total_tools,
    );
    println!("  运行 `i-rs-claw tools` 管理工具开关");

    Ok(())
}

// =============================================
// Tools subcommand
// =============================================

pub fn run_tools() -> anyhow::Result<()> {
    let mut cfg = Config::load()?;

    // If no explicit tool selection exists (empty = all enabled in runtime),
    // seed the UI with DEFAULT_TOOLS so only those 10 show as checked initially
    if cfg.enabled_tools.is_empty() {
        cfg.enabled_tools = crate::config::DEFAULT_TOOLS
            .iter()
            .map(|s| s.to_string())
            .collect();
    }

    let all_tools: Vec<&str> = crate::tools::TOOL_INDEX
        .iter()
        .map(|(n, _)| *n)
        .collect();
    let total = all_tools.len();

    // ── TUI setup ──
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut selection: usize = 0;
    let mut dirty = false;

    let result: anyhow::Result<()> = (|| {
        loop {
            terminal.draw(|f| {
                let area = f.area();

                // Layout: title + list + footer
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Min(1),
                        Constraint::Length(1),
                    ])
                    .split(area);

                // Title bar
                let checked_count = if cfg.enabled_tools.is_empty() {
                    total
                } else {
                    cfg.enabled_tools.len()
                };
                let title = Line::from(Span::styled(
                    format!(
                        " ✦ 工具管理  [{}✓ / {}总]  ↑↓选择  Space切换  a全选  n清空  Enter保存  Esc取消",
                        checked_count, total
                    ),
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ));
                f.render_widget(title, chunks[0]);

                // Tool list
                let items: Vec<ListItem> = all_tools
                    .iter()
                    .map(|name| {
                        let checked = cfg.enabled_tools.is_empty()
                            || cfg.enabled_tools.contains(*name);
                        let checkbox = if checked { "[✓]" } else { "[ ]" };
                        let desc = crate::tools::TOOL_INDEX
                            .iter()
                            .find(|(n, _)| n == name)
                            .map(|(_, d)| *d)
                            .unwrap_or("");
                        let text = format!(" {} {}", checkbox, name);
                        let line = Line::from(vec![
                            Span::styled(
                                text,
                                Style::default().fg(if checked {
                                    Color::Green
                                } else {
                                    Color::DarkGray
                                })
                                .add_modifier(if checked {
                                    Modifier::BOLD
                                } else {
                                    Modifier::empty()
                                }),
                            ),
                            Span::styled(
                                format!("  — {}", desc),
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]);
                        ListItem::new(line)
                    })
                    .collect();

                let list = List::new(items)
                    .highlight_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD | Modifier::REVERSED),
                    )
                    .block(Block::default().borders(Borders::NONE));

                f.render_stateful_widget(
                    list,
                    chunks[1],
                    &mut ratatui::widgets::ListState::default()
                        .with_selected(Some(selection)),
                );
            })?;

            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Up => selection = selection.saturating_sub(1),
                    KeyCode::Down if selection + 1 < total => selection += 1,
                    KeyCode::Char(' ') => {
                        dirty = true;
                        let name = all_tools[selection];
                        if cfg.enabled_tools.is_empty() {
                            cfg.enabled_tools =
                                all_tools.iter().map(|s| s.to_string()).collect();
                        }
                        if cfg.enabled_tools.contains(name) {
                            cfg.enabled_tools.remove(name);
                        } else {
                            cfg.enabled_tools.insert(name.to_string());
                        }
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        dirty = true;
                        cfg.enabled_tools =
                            all_tools.iter().map(|s| s.to_string()).collect();
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        dirty = true;
                        cfg.enabled_tools.clear();
                    }
                    KeyCode::Enter => break Ok(()),
                    KeyCode::Esc | KeyCode::Char('q') => {
                        dirty = false;
                        break Ok(());
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    })();

    // ── TUI teardown ──
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

    result?;

    // Save if modified
    if dirty {
        if cfg.enabled_tools.len() == total {
            cfg.enabled_tools.clear();
        }
        if cfg.enabled_tools.is_empty() {
            println!("✓ 已启用全部 {} 个工具", total);
        } else {
            println!(
                "✓ 已启用 {} 个工具 (停用 {} 个)",
                cfg.enabled_tools.len(),
                total - cfg.enabled_tools.len()
            );
        }
        cfg.save()?;
    } else {
        println!("✓ 未做修改");
    }

    Ok(())
}

// =============================================
// Session subcommand
// =============================================

pub fn run_session_list() -> anyhow::Result<()> {
    let session_mgr = SessionManager::new(claw_dir().join("claw"));

    let sessions = session_mgr.sessions();
    if sessions.is_empty() {
        println!("暂无会话");
        return Ok(());
    }

    println!("会话列表 ({} 个):\n", sessions.len());
    for (i, s) in sessions.iter().enumerate() {
        let time = chrono::DateTime::from_timestamp(s.created_at, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default();
        println!("  {}. {}", i + 1, s.title);
        println!("     {} 条消息 | {}", s.message_count, time);
    }
    Ok(())
}

// =============================================
// Helpers
// =============================================

fn claw_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .expect("无法获取用户主目录")
        .join(".i-rs-claw")
}
