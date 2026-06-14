use i_rs_claw_core::config::Config;
use crossterm::event::{self, Event, KeyCode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use std::io;

// =============================================
// Tools subcommand
// =============================================

pub fn run_tools() -> anyhow::Result<()> {
    let mut cfg = Config::load()?;

    let all_tools: Vec<String> = cfg.i_rs_tools.clone();
    let total = all_tools.len();

    // ── TUI setup ──
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut selection: usize = 0;
    let mut dirty = false;

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> anyhow::Result<()> {
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
                            || cfg.enabled_tools.contains(name);
                        let checkbox = if checked { "[✓]" } else { "[ ]" };
                        let text = format!(" {} {}", checkbox, name);
                        let line = Line::from(Span::styled(
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
                        ));
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

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => selection = selection.saturating_sub(1),
                    KeyCode::Down if selection + 1 < total => selection += 1,
                    KeyCode::Char(' ') => {
                        dirty = true;
                        let name = &all_tools[selection];
                        if cfg.enabled_tools.contains(name) {
                            cfg.enabled_tools.remove(name);
                        } else {
                            cfg.enabled_tools.insert(name.to_string());
                        }
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        dirty = true;
                        cfg.enabled_tools = all_tools.iter().map(|s| s.to_string()).collect();
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
                }
            }
        }
    }));

    // ── TUI teardown (always runs, even on panic) ──
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

    match result {
        Ok(inner) => inner?,
        Err(_) => {
            anyhow::bail!("工具界面发生内部错误，终端已恢复");
        }
    }

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
