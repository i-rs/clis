use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::app::App;

mod chat;
mod completions;
mod input;
mod panels;
mod sidebar;
mod status;
mod title;
mod utils;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let plan_height: u16 = if !app.plan_steps.is_empty() && app.is_processing() {
        (app.plan_steps.len() as u16).min(5)
    } else {
        0
    };

    // Show a single-line status bar during processing.
    // Reasoning content is now rendered inline in the chat message area.
    let processing_height: u16 = 1;

    let mut constraints = vec![Constraint::Length(1), Constraint::Min(1)];

    if plan_height > 0 {
        constraints.push(Constraint::Length(1));
    }

    constraints.extend(vec![
        Constraint::Length(processing_height),
        Constraint::Length(input::input_height(&app.input.text)),
        Constraint::Length(1),
    ]);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut idx = 0;
    title::render_title(f, layout[idx], app);
    idx += 1;
    if app.overlay.show_sidebar && !app.is_processing() {
        let chat_side = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Percentage(35)])
            .split(layout[idx]);
        chat::render_chat(f, chat_side[0], app);
        sidebar::render_sidebar(f, chat_side[1], app);
    } else {
        chat::render_chat(f, layout[idx], app);
    }
    idx += 1;
    if plan_height > 0 {
        panels::render_plan(f, layout[idx], app);
        idx += 1;
    }
    panels::render_processing(f, layout[idx], app, &app.config.theme);
    idx += 1;
    input::render_input(f, layout[idx], app);
    idx += 1;
    status::render_status(f, layout[idx], app);

    if app.overlay.show_session_list {
        panels::render_backdrop(f, area);
        sidebar::render_session_list(f, area, app);
    }

    if app.overlay.show_agent_picker {
        panels::render_backdrop(f, area);
        sidebar::render_agent_picker(f, area, app);
    }

    let theme = &app.config.theme;

    if app.overlay.show_help {
        panels::render_backdrop(f, area);
        panels::render_help_panel(f, area, theme);
    }

    if app.overlay.show_config {
        panels::render_backdrop(f, area);
        panels::render_config_panel(f, area, app, theme);
    }

    if app.overlay.show_tool_list {
        panels::render_backdrop(f, area);
        panels::render_tool_list_panel(f, area, app, theme);
    }

    if app.overlay.show_agent_list {
        panels::render_backdrop(f, area);
        panels::render_agent_list_panel(f, area, app, theme);
    }

    if app.overlay.show_stats_history {
        panels::render_backdrop(f, area);
        panels::render_stats_history_panel(f, area, app, theme);
    }

    if app.overlay.show_plugin_list {
        panels::render_backdrop(f, area);
        panels::render_plugin_list_panel(f, area, app, theme);
    }

    if app.overlay.show_feedback {
        panels::render_backdrop(f, area);
        panels::render_feedback_prompt(f, area, theme);
    }

    if !app.overlay.tab_completions.is_empty() {
        completions::render_completions(f, area, app);
    }

    if let Some(idx) = app.overlay.sidebar_body_idx
        && let Some(log) = app.http_logs.get(idx)
    {
        sidebar::render_request_body(
            f,
            area,
            &log.request_body,
            idx,
            app.http_logs.len(),
            app.overlay.sidebar_body_scroll,
        );
    }
}
