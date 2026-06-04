use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::app::{App, Overlay};

mod chat;
mod completions;
mod input;
mod panels;
mod sidebar;
mod status;
mod title;
mod utils;

pub(crate) mod chat_api {
    //! Re-exports of the bits the App needs from the chat module.
    //!
    //! `chat` itself stays a private `mod`, but the App
    //! (`crate::app`) and tests have to reach into it to build the
    //! per-message render components and the scroller hit-region
    //! type. We funnel those types through this tiny facade so the
    //! privacy boundary is clear and documented.
    pub(crate) use crate::ui::chat::components::{build_component_for, ComponentOp};
    pub(crate) use crate::ui::chat::scroller::ComponentCell;
    pub(crate) use ratatui_interact::traits::ClickRegionRegistry;
}

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let plan_height: u16 = if !app.plan_steps.is_empty() && app.is_processing() {
        let steps = (app.plan_steps.len() as u16).min(5);
        let more = if app.plan_steps.len() > 5 { 1 } else { 0 };
        steps + 1 + more
    } else {
        0
    };

    let processing_height: u16 = 1;
    let slash_height = completions::slash_picker_height(app);

    let mut constraints = Vec::with_capacity(7);
    constraints.push(Constraint::Length(1));
    constraints.push(Constraint::Min(1));

    if plan_height > 0 {
        constraints.push(Constraint::Length(plan_height));
    }

    constraints.push(Constraint::Length(processing_height));

    if slash_height > 0 {
        constraints.push(Constraint::Length(slash_height));
    }

    constraints.push(Constraint::Length(input::input_height(&app.input.text, area.width)));
    constraints.push(Constraint::Length(1));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut idx = 0;
    title::render_title(f, layout[idx], app);
    idx += 1;
    if app.overlay.is_overlay(Overlay::Sidebar) && !app.is_processing() {
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
    if slash_height > 0 {
        completions::render_slash_panel(f, layout[idx], app);
        idx += 1;
    }
    input::render_input(f, layout[idx], app);
    idx += 1;
    status::render_status(f, layout[idx], app);

    match app.overlay.current {
        Some(Overlay::SessionList) => {
            panels::render_backdrop(f, area);
            sidebar::render_session_list(f, area, app);
        }
        Some(Overlay::AgentPicker) => {
            panels::render_backdrop(f, area);
            sidebar::render_agent_picker(f, area, app);
        }
        Some(Overlay::Help) => {
            panels::render_backdrop(f, area);
            panels::render_help_panel(f, area, &app.config.theme);
        }
        Some(Overlay::Config) => {
            panels::render_backdrop(f, area);
            panels::render_config_panel(f, area, app, &app.config.theme);
        }
        Some(Overlay::ToolList) => {
            panels::render_backdrop(f, area);
            panels::render_tool_list_panel(f, area, app, &app.config.theme);
        }
        Some(Overlay::AgentList) => {
            panels::render_backdrop(f, area);
            panels::render_agent_list_panel(f, area, app, &app.config.theme);
        }
        Some(Overlay::StatsHistory) => {
            panels::render_backdrop(f, area);
            panels::render_stats_history_panel(f, area, app, &app.config.theme);
        }
        Some(Overlay::PluginList) => {
            panels::render_backdrop(f, area);
            panels::render_plugin_list_panel(f, area, app, &app.config.theme);
        }
        Some(Overlay::InfoPanel) => {
            panels::render_backdrop(f, area);
            panels::render_info_panel(f, area, app, &app.config.theme);
        }
        Some(Overlay::Feedback) => {
            panels::render_backdrop(f, area);
            panels::render_feedback_prompt(f, area, &app.config.theme);
        }
        Some(Overlay::ThemePicker) => {
            panels::render_backdrop(f, area);
            panels::render_theme_picker(f, area, app);
        }
        Some(Overlay::Sidebar) => {}
        None => {}
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
            &mut app.overlay.sidebar_formatted_json,
        );
    }
}
