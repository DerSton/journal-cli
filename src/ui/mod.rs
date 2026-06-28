mod auth;
mod contacts_tab;
mod journal_tab;
mod modals;
mod settings_tab;
mod stats_tab;
mod theme;

use crate::app::{App, AppMode, Tab};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::Paragraph,
};

/// Main UI entry point: dispatches to a full-screen mode or the tabbed main view.
pub fn draw(f: &mut Frame, app: &mut App) {
    match app.mode {
        AppMode::Login => return auth::draw_login(f, app),
        AppMode::Recovery => return auth::draw_recovery(f, app),
        AppMode::RecoveryReset => return auth::draw_recovery_reset(f, app),
        _ => {}
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    draw_tab_bar(f, app, chunks[0]);

    let main_area = chunks[1];
    if app.active_tab == Tab::Stats {
        stats_tab::draw(f, app, main_area);
    } else {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(main_area);
        let list_area = main_chunks[0];
        let content_area = main_chunks[1];
        let is_searchable_tab = matches!(app.active_tab, Tab::Journal | Tab::Contacts);
        let list_area =
            if is_searchable_tab && (app.mode == AppMode::Search || !app.search_query.is_empty()) {
                let list_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(list_area);

                draw_search_box(f, app, list_layout[0]);

                list_layout[1]
            } else {
                list_area
            };

        match app.active_tab {
            Tab::Journal => journal_tab::draw(f, app, list_area, content_area),
            Tab::Contacts => contacts_tab::draw(f, app, list_area, content_area),
            Tab::Settings => settings_tab::draw(f, app, list_area, content_area),
            Tab::Stats => {}
        }
    }

    draw_status_bar(f, app, chunks[2]);
    modals::draw_overlays(f, app);
}

fn draw_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let tab_span = |label: &str, tab: Tab| {
        Span::styled(
            format!(" {} ", label),
            if app.active_tab == tab {
                theme::title_style()
            } else {
                theme::muted_style()
            },
        )
    };

    let line = Line::from(vec![
        Span::raw(" Journal CLI  "),
        tab_span("Journal [1]", Tab::Journal),
        Span::styled(" | ", theme::muted_style()),
        tab_span("Contacts [2]", Tab::Contacts),
        Span::styled(" | ", theme::muted_style()),
        tab_span("Stats [3]", Tab::Stats),
        Span::styled(" | ", theme::muted_style()),
        tab_span("Settings [4]", Tab::Settings),
    ]);

    f.render_widget(Paragraph::new(line).block(theme::panel_block("")), area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let spans = if let Some(ref err) = app.error_msg {
        vec![Span::styled(err.clone(), theme::danger_style())]
    } else if let Some(ref status) = app.status_msg {
        vec![Span::styled(status.clone(), theme::success_style())]
    } else {
        help_hints(app)
    };

    let block = theme::panel_block("");
    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(
        Paragraph::new(Line::from(spans)).alignment(ratatui::layout::Alignment::Center),
        inner,
    );
}

fn hint(key: &str, action: &str) -> Vec<Span<'static>> {
    vec![
        Span::styled(format!(" {}: ", key), theme::title_style()),
        Span::styled(format!("{} ", action), theme::text_style()),
    ]
}

fn help_hints(app: &App) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    match app.mode {
        AppMode::List => {
            if app.active_tab == Tab::Settings && app.settings_panel_focused {
                spans.extend(hint("Esc", "Back to list"));
                spans.extend(hint("Ctrl+S", "Save"));
            } else {
                match app.active_tab {
                    Tab::Journal => {
                        spans.extend(hint("/", "Search"));
                        spans.extend(hint("n", "New entry"));
                        spans.extend(hint("e", "Edit"));
                        spans.extend(hint("d", "Delete"));
                        spans.extend(hint("PgUp/PgDn", "Scroll preview"));
                    }
                    Tab::Contacts => {
                        spans.extend(hint("/", "Search"));
                        spans.extend(hint("n", "New contact"));
                        spans.extend(hint("e", "Edit"));
                        spans.extend(hint("d", "Delete"));
                    }
                    Tab::Settings => {
                        spans.extend(hint("Up/Down", "Select group"));
                        spans.extend(hint("Enter", "Open"));
                    }
                    Tab::Stats => {}
                }
                spans.extend(hint("q", "Quit"));
            }
        }
        AppMode::Writing { .. } => match app.active_tab {
            Tab::Journal => {
                spans.extend(hint("Alt+P", "Mention contact"));
                spans.extend(hint("Alt+D", "Set date"));
                spans.extend(hint("Ctrl+S", "Save"));
                spans.extend(hint("Esc", "Cancel"));
            }
            Tab::Contacts => {
                spans.extend(hint("Tab/Shift+Tab", "Next/prev field"));
                spans.extend(hint("Ctrl+S", "Save"));
                spans.extend(hint("Esc", "Cancel"));
            }
            Tab::Settings | Tab::Stats => {}
        },
        AppMode::ContactPicker { .. } => {
            spans.extend(hint("Up/Down", "Select contact"));
            spans.extend(hint("Enter", "Mention"));
            spans.extend(hint("Esc", "Cancel"));
        }
        AppMode::DatePicker { .. } => {
            spans.extend(hint("Arrows", "Navigate"));
            spans.extend(hint("PgUp/PgDn", "Month"));
            spans.extend(hint("{ }", "Year"));
            spans.extend(hint("Enter", "Pick"));
            spans.extend(hint("c", "Clear"));
            spans.extend(hint("Esc", "Cancel"));
        }
        AppMode::DeleteConfirm => {
            spans.extend(hint("y", "Yes, delete"));
            spans.extend(hint("n / Esc", "Cancel"));
        }
        AppMode::Login | AppMode::Recovery | AppMode::RecoveryReset => {}
        AppMode::Search => {
            spans.extend(hint("Enter", "Lock search"));
            spans.extend(hint("Esc", "Clear search"));
        }
    }
    spans
}

/// Centers a modal that takes a percentage of the available area.
pub(crate) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

/// Centers a modal with a fixed pixel size.
pub(crate) fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(r.height.saturating_sub(height) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(r.width.saturating_sub(width) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(vertical[1])[1]
}

fn draw_search_box(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.mode == AppMode::Search;
    let block = theme::field_block("Search (Enter to lock, Esc to clear)", focused);

    let display_str = if focused {
        format!("{}_", app.search_query)
    } else {
        app.search_query.clone()
    };

    let p = Paragraph::new(Line::from(Span::styled(display_str, theme::text_style()))).block(block);
    f.render_widget(p, area);
}
