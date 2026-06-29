//! Stats / Insights tab — ten-page modular analytics dashboard.

pub mod helpers;
mod page_1;
mod page_10;
mod page_2;
mod page_3;
mod page_4;
mod page_5;
mod page_6;
mod page_7;
mod page_8;
mod page_9;

use crate::app::{App, STATS_PAGE_COUNT};
use crate::ui::theme;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::Paragraph,
};

/// Page titles displayed in the header.
const PAGE_TITLES: [&str; STATS_PAGE_COUNT] = [
    "Dashboard Overview",
    "Writing Over Time",
    "Time of Day",
    "Streaks & Consistency",
    "Word Analysis",
    "Contact Insights",
    "Entry Length",
    "People Directory",
    "Content Patterns",
    "Year in Review",
];

// ── Public entry point ────────────────────────────────────────────────────────

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let [header_area, content_area] = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Min(0),    // Page Content
    ])
    .areas(area);

    draw_page_header(f, app, header_area);

    // Pass the main content area to the sub-pages.
    draw_page_content(f, app, content_area);
}

// ── Page header with title and navigation indicator ───────────────────────────

fn draw_page_header(f: &mut Frame, app: &App, area: Rect) {
    let page = app.stats_page;
    let title = PAGE_TITLES.get(page).copied().unwrap_or("Unknown");

    let left_arrow = if page > 0 {
        Span::styled("  ◀  ", theme::accent())
    } else {
        Span::styled("     ", theme::dim())
    };

    let right_arrow = if page + 1 < STATS_PAGE_COUNT {
        Span::styled("  ▶  ", theme::accent())
    } else {
        Span::styled("     ", theme::dim())
    };

    let page_indicator = Span::styled(
        format!(" {}/{} ", page + 1, STATS_PAGE_COUNT),
        theme::muted(),
    );

    let title_span = Span::styled(format!(" {} ", title), theme::accent());

    let line = Line::from(vec![left_arrow, page_indicator, title_span, right_arrow]);

    f.render_widget(
        Paragraph::new(line)
            .block(theme::panel(""))
            .alignment(Alignment::Center),
        area,
    );
}

// ── Page content dispatcher ───────────────────────────────────────────────────

fn draw_page_content(f: &mut Frame, app: &App, area: Rect) {
    match app.stats_page {
        0 => page_1::draw(f, app, area),
        1 => page_2::draw(f, app, area),
        2 => page_3::draw(f, app, area),
        3 => page_4::draw(f, app, area),
        4 => page_5::draw(f, app, area),
        5 => page_6::draw(f, app, area),
        6 => page_7::draw(f, app, area),
        7 => page_8::draw(f, app, area),
        8 => page_9::draw(f, app, area),
        9 => page_10::draw(f, app, area),
        _ => {
            let msg = Paragraph::new(Line::from(Span::styled("Unknown page", theme::muted())))
                .alignment(Alignment::Center);
            f.render_widget(msg, area);
        }
    }
}
