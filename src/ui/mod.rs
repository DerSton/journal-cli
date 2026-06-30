//! Root UI dispatcher.
//!
//! Handles the top-level layout split (tab bar → main content → status bar),
//! dispatches to per-tab renderers, and draws transient overlays last.

mod auth;
mod contacts_tab;
mod journal_tab;
mod modals;
mod settings_tab;
mod stats_tab;
pub mod theme;

use crate::app::{App, AppMode, Tab};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::Paragraph,
};

/// Entry point called once per frame by the event loop.
pub fn draw(f: &mut Frame, app: &mut App) {
    // Full-screen modes bypass the tab shell entirely.
    match app.mode {
        AppMode::Login => return auth::draw_login(f, app),
        AppMode::Recovery => return auth::draw_recovery(f, app),
        AppMode::RecoveryReset => return auth::draw_recovery_reset(f, app),
        _ => {}
    }

    let [tab_area, main_area, status_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(f.area());

    draw_tab_bar(f, app, tab_area);
    draw_main(f, app, main_area);
    draw_status_bar(f, app, status_area);

    // Overlays are always painted on top.
    modals::draw_overlays(f, app);
}

// ── Tab bar ───────────────────────────────────────────────────────────────────

fn draw_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let tabs = [
        ("  Journal ", Tab::Journal),
        ("  People  ", Tab::Contacts),
        ("  Insights", Tab::Stats),
        ("  Settings", Tab::Settings),
    ];
    let hints = ["[1]", "[2]", "[3]", "[4]"];

    let mut spans: Vec<Span> = Vec::new();

    for (i, ((label, tab), hint)) in tabs.iter().zip(hints).enumerate() {
        let active = app.active_tab == *tab;

        let label_style = if active {
            theme::accent()
        } else {
            theme::muted()
        };
        let hint_style = theme::dim();

        if i > 0 {
            spans.push(Span::styled("  ", theme::muted()));
        }

        spans.push(Span::styled(*label, label_style));
        spans.push(Span::styled(format!(" {} ", hint), hint_style));
    }

    let bar = Paragraph::new(Line::from(spans)).block(theme::panel(""));
    f.render_widget(bar, area);
}

// ── Main content ──────────────────────────────────────────────────────────────

fn draw_main(f: &mut Frame, app: &mut App, area: Rect) {
    // Stats tab uses the entire area as a dashboard — no list/detail split.
    if app.active_tab == Tab::Stats {
        return stats_tab::draw(f, app, area);
    }

    let [list_area, content_area] =
        Layout::horizontal([Constraint::Percentage(33), Constraint::Percentage(67)]).areas(area);

    // Search box consumes the top of the list panel when active.
    let list_area = if matches!(app.active_tab, Tab::Journal | Tab::Contacts)
        && (app.mode == AppMode::Search || !app.search_query.is_empty())
    {
        let [search_area, remainder] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(list_area);
        draw_search_box(f, app, search_area);
        remainder
    } else {
        list_area
    };

    match app.active_tab {
        Tab::Journal => journal_tab::draw(f, app, list_area, content_area),
        Tab::Contacts => contacts_tab::draw(f, app, list_area, content_area),
        Tab::Settings => settings_tab::draw(f, app, list_area, content_area),
        Tab::Stats => {} // handled above
    }
}

// ── Status bar ────────────────────────────────────────────────────────────────

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let block = theme::panel("");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let content = if let Some(ref err) = app.error_msg {
        Line::from(Span::styled(format!("  ✕  {}", err), theme::danger()))
    } else if let Some(ref msg) = app.status_msg {
        Line::from(Span::styled(format!("  ✓  {}", msg), theme::success()))
    } else {
        Line::from(build_hint_spans(app))
    };

    f.render_widget(
        Paragraph::new(content).alignment(ratatui::layout::Alignment::Center),
        inner,
    );
}

fn key(k: &str) -> Span<'static> {
    Span::styled(format!(" {} ", k), theme::accent())
}

fn sep() -> Span<'static> {
    Span::styled("  ", theme::muted())
}

fn action(a: &str) -> Span<'static> {
    Span::styled(format!("{} ", a), theme::muted())
}

fn hint_pair(k: &str, a: &str) -> [Span<'static>; 3] {
    [key(k), action(a), sep()]
}

fn build_hint_spans(app: &App) -> Vec<Span<'static>> {
    let mut v: Vec<Span> = Vec::new();

    match app.mode {
        AppMode::List => {
            if app.active_tab == Tab::Settings && app.settings_panel_focused {
                v.extend(hint_pair("Ctrl+S", "Save"));
                v.extend(hint_pair("Esc", "Back"));
            } else {
                match app.active_tab {
                    Tab::Journal => {
                        v.extend(hint_pair("/", "Search"));
                        v.extend(hint_pair("n", "New entry"));
                        v.extend(hint_pair("e", "Edit"));
                        v.extend(hint_pair("d", "Delete"));
                        v.extend(hint_pair("a", "Attach"));
                        v.extend(hint_pair("x", "Export .md"));
                        v.extend(hint_pair("PgUp/Dn", "Scroll"));
                    }
                    Tab::Contacts => {
                        v.extend(hint_pair("/", "Search"));
                        v.extend(hint_pair("n", "New contact"));
                        v.extend(hint_pair("e", "Edit"));
                        v.extend(hint_pair("d", "Delete"));
                    }
                    Tab::Settings => {
                        v.extend(hint_pair("↑↓", "Navigate"));
                        v.extend(hint_pair("Enter", "Open"));
                    }
                    Tab::Stats => {}
                }
                v.extend(hint_pair("q", "Quit"));
            }
        }
        AppMode::Writing { .. } => match app.active_tab {
            Tab::Journal => {
                v.extend(hint_pair("Alt+P", "Mention person"));
                v.extend(hint_pair("Alt+D", "Set date"));
                v.extend(hint_pair("Ctrl+S", "Save"));
                v.extend(hint_pair("Esc", "Cancel"));
            }
            Tab::Contacts => {
                v.extend(hint_pair("Tab/⇧Tab", "Next/prev field"));
                v.extend(hint_pair("Ctrl+S", "Save"));
                v.extend(hint_pair("Esc", "Cancel"));
            }
            Tab::Settings | Tab::Stats => {}
        },
        AppMode::ContactPicker { .. } => {
            v.extend(hint_pair("↑↓", "Select"));
            v.extend(hint_pair("Enter", "Insert mention"));
            v.extend(hint_pair("Esc", "Cancel"));
        }
        AppMode::DatePicker { .. } => {
            v.extend(hint_pair("←→↑↓", "Navigate"));
            v.extend(hint_pair("PgUp/Dn", "Month"));
            v.extend(hint_pair("{ }", "Year"));
            v.extend(hint_pair("Enter", "Confirm"));
            v.extend(hint_pair("c", "Clear"));
            v.extend(hint_pair("Esc", "Cancel"));
        }
        AppMode::DeleteConfirm => {
            v.extend(hint_pair("y", "Confirm delete"));
            v.extend(hint_pair("n / Esc", "Cancel"));
        }
        AppMode::Search => {
            v.extend(hint_pair("Enter", "Lock filter"));
            v.extend(hint_pair("Esc", "Clear"));
        }
        AppMode::Login | AppMode::Recovery | AppMode::RecoveryReset => {}
    }

    v
}

// ── Search box ────────────────────────────────────────────────────────────────

fn draw_search_box(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.mode == AppMode::Search;
    let display = if focused {
        format!("{}_", app.search_query)
    } else {
        app.search_query.clone()
    };

    let label = if focused {
        "Search (Enter to lock · Esc to clear)"
    } else {
        "Search"
    };

    let p = Paragraph::new(Line::from(Span::styled(
        format!(" {}", display),
        theme::text(),
    )))
    .block(theme::field(label, focused));
    f.render_widget(p, area);
}

// ── Layout helpers ────────────────────────────────────────────────────────────

/// Returns a [`Rect`] centred within `r`, taking a percentage of each dimension.
pub(crate) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let margin_v = (100 - percent_y) / 2;
    let margin_h = (100 - percent_x) / 2;

    let [_, mid, _] = Layout::vertical([
        Constraint::Percentage(margin_v),
        Constraint::Percentage(percent_y),
        Constraint::Percentage(margin_v),
    ])
    .areas(r);

    let [_, center, _] = Layout::horizontal([
        Constraint::Percentage(margin_h),
        Constraint::Percentage(percent_x),
        Constraint::Percentage(margin_h),
    ])
    .areas(mid);

    center
}

/// Returns a [`Rect`] centred within `r` with fixed pixel dimensions.
pub(crate) fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let [_, mid, _] = Layout::vertical([
        Constraint::Length(r.height.saturating_sub(height) / 2),
        Constraint::Length(height),
        Constraint::Min(0),
    ])
    .areas(r);

    let [_, center, _] = Layout::horizontal([
        Constraint::Length(r.width.saturating_sub(width) / 2),
        Constraint::Length(width),
        Constraint::Min(0),
    ])
    .areas(mid);

    center
}
