//! Transient overlay modals: delete confirmation, contact picker, date picker.
//!
//! All overlays are rendered on top of the tab content by `ui::draw_overlays`.
//! Each one clears its rect before drawing to erase the content underneath.

use super::{centered_rect, centered_rect_fixed, theme};
use crate::app::{App, AppMode, Tab};
use chrono::Datelike;
use ratatui::{
    Frame,
    layout::Alignment,
    text::{Line, Span},
    widgets::{Clear, List, ListItem, ListState, Paragraph},
};

/// Dispatches to the appropriate overlay based on the current app mode.
pub fn draw_overlays(f: &mut Frame, app: &App) {
    match app.mode {
        AppMode::DeleteConfirm => draw_delete_confirm(f, app),
        AppMode::ContactPicker {
            selected_contact_index,
            ..
        } => draw_contact_picker(f, app, selected_contact_index),
        AppMode::DatePicker {
            field_index,
            current_date,
            ..
        } => draw_date_picker(f, app, field_index, current_date),
        _ => {}
    }
}

// ── Delete confirmation ───────────────────────────────────────────────────────

fn draw_delete_confirm(f: &mut Frame, app: &App) {
    let area = centered_rect(46, 28, f.area());
    f.render_widget(Clear, area);

    let item_label = match app.active_tab {
        Tab::Journal => "journal entry",
        Tab::Contacts => "contact",
        Tab::Settings | Tab::Stats => return,
    };

    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("Delete this {}?", item_label),
                theme::text(),
            ))
            .alignment(Alignment::Center),
            Line::from(Span::styled(
                "This action cannot be undone.",
                theme::muted(),
            ))
            .alignment(Alignment::Center),
            Line::from(""),
            Line::from(vec![
                Span::styled("  y  ", theme::danger()),
                Span::styled("Delete", theme::muted()),
                Span::styled("     n / Esc  ", theme::accent()),
                Span::styled("Cancel  ", theme::muted()),
            ])
            .alignment(Alignment::Center),
        ])
        .block(theme::modal_danger("  Confirm Delete")),
        area,
    );
}

// ── Contact picker ────────────────────────────────────────────────────────────

fn draw_contact_picker(f: &mut Frame, app: &App, selected: usize) {
    let area = centered_rect(56, 52, f.area());
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = app
        .journal
        .contacts
        .iter()
        .map(|c| {
            ListItem::new(Line::from(Span::styled(
                format!("  {}", c.full_name()),
                theme::text(),
            )))
        })
        .collect();

    let mut state = ListState::default();
    if !app.journal.contacts.is_empty() {
        state.select(Some(selected));
    }

    f.render_stateful_widget(
        List::new(items)
            .block(theme::modal("  Mention a person"))
            .highlight_style(theme::list_highlight()),
        area,
        &mut state,
    );
}

// ── Date picker (calendar) ────────────────────────────────────────────────────

fn draw_date_picker(f: &mut Frame, app: &App, field_index: usize, current_date: chrono::NaiveDate) {
    // 36 wide × 14 tall comfortably holds 7-column calendar + borders.
    let area = centered_rect_fixed(36, 14, f.area());
    f.render_widget(Clear, area);

    let field_name = if app.active_tab == Tab::Journal {
        "Entry date"
    } else if field_index == 0 {
        "Date of birth"
    } else {
        "Date of death"
    };

    let year = current_date.year();
    let month = current_date.month();

    let month_name = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ][(month - 1) as usize];

    let days_in_month = days_in_month(year, month);
    let first_weekday = (chrono::NaiveDate::from_ymd_opt(year, month, 1)
        .unwrap()
        .weekday()
        .number_from_monday()
        - 1) as usize;

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("{:^32}", format!("{} {}", month_name, year)),
            theme::label(),
        ))
        .alignment(Alignment::Center),
        Line::from(Span::styled(" Mo  Tu  We  Th  Fr  Sa  Su", theme::muted()))
            .alignment(Alignment::Center),
    ];

    let today = chrono::Local::now().date_naive();

    for row in 0..6usize {
        let mut spans = vec![Span::raw(" ")];
        for col in 0..7usize {
            let cell = row * 7 + col;
            if cell < first_weekday || cell >= first_weekday + days_in_month as usize {
                spans.push(Span::raw("    "));
            } else {
                let day = (cell - first_weekday + 1) as u32;
                let date = chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap();
                let text = format!("{:>3} ", day);
                let style = if date == current_date {
                    ratatui::style::Style::default()
                        .bg(theme::CAL_SELECTED_BG)
                        .fg(ratatui::style::Color::White)
                        .add_modifier(ratatui::style::Modifier::BOLD)
                } else if date == today {
                    theme::accent()
                } else {
                    theme::text()
                };
                spans.push(Span::styled(text, style));
            }
        }
        let line = Line::from(spans);
        // Only add rows that have at least one day cell.
        let has_day =
            row * 7 < first_weekday + days_in_month as usize && (row + 1) * 7 > first_weekday;
        if has_day {
            lines.push(line);
        }
    }

    lines.push(Line::from(Span::styled(
        " PgUp/Dn  Month    { }  Year    c  Clear",
        theme::dim(),
    )));

    f.render_widget(
        Paragraph::new(lines).block(theme::modal(format!("  {}", field_name))),
        area,
    );
}

// ── Calendar helper ───────────────────────────────────────────────────────────

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        _ => 30,
    }
}
