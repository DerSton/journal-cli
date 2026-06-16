mod auth;
mod contacts_tab;
mod journal_tab;
mod modals;
mod settings_tab;
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

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[1]);
    let list_area = main_chunks[0];
    let content_area = main_chunks[1];

    match app.active_tab {
        Tab::Journal => journal_tab::draw(f, app, list_area, content_area),
        Tab::Contacts => contacts_tab::draw(f, app, list_area, content_area),
        Tab::Settings => settings_tab::draw(f, app, list_area, content_area),
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
        tab_span("Settings [3]", Tab::Settings),
        Span::styled("   (Tab or 1-3 to switch)", theme::muted_style()),
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
                        spans.extend(hint("n", "New entry"));
                        spans.extend(hint("e", "Edit"));
                        spans.extend(hint("d", "Delete"));
                        spans.extend(hint("PgUp/PgDn", "Scroll preview"));
                    }
                    Tab::Contacts => {
                        spans.extend(hint("n", "New contact"));
                        spans.extend(hint("e", "Edit"));
                        spans.extend(hint("d", "Delete"));
                    }
                    Tab::Settings => {
                        spans.extend(hint("Up/Down", "Select group"));
                        spans.extend(hint("Enter", "Open"));
                    }
                }
                spans.extend(hint("Tab", "Switch tab"));
                spans.extend(hint("q", "Quit"));
            }
        }
        AppMode::Writing { .. } => match app.active_tab {
            Tab::Journal => {
                spans.extend(hint("Alt+P", "Mention contact"));
                spans.extend(hint("Ctrl+S", "Save"));
                spans.extend(hint("Esc", "Cancel"));
            }
            Tab::Contacts => {
                spans.extend(hint("Tab/Shift+Tab", "Next/prev field"));
                spans.extend(hint("Ctrl+S", "Save"));
                spans.extend(hint("Esc", "Cancel"));
            }
            Tab::Settings => {}
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
