//! Transient overlay modals: delete confirmation, contact picker, date picker.
//!
//! All overlays are rendered on top of the tab content by `ui::draw_overlays`.
//! Each one clears its rect before drawing to erase the content underneath.

use super::{centered_rect, centered_rect_fixed, theme};
use crate::app::{App, AppMode, Tab};
use chrono::Datelike;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, ListState, Paragraph},
};

/// Dispatches to the appropriate overlay based on the current app mode.
pub fn draw_overlays(f: &mut Frame, app: &App) {
    match app.mode {
        AppMode::DeleteConfirm => draw_delete_confirm(f, app),
        AppMode::ContactPicker {
            selected_contact_index,
            ref search_query,
            ..
        } => draw_contact_picker(f, app, selected_contact_index, search_query),
        AppMode::DatePicker {
            field_index,
            current_date,
            ..
        } => draw_date_picker(f, app, field_index, current_date),
        AppMode::GroupMemberPicker {
            selected_contact_index,
            ref search_query,
            ..
        } => draw_group_member_picker(f, app, selected_contact_index, search_query),
        AppMode::AttachmentPicker {
            selected_attachment_index,
        } => draw_attachment_picker(f, app, selected_attachment_index),
        AppMode::DiscardConfirm { .. } => draw_discard_confirm(f, app),
        _ => {}
    }
}

// ── Discard confirmation ──────────────────────────────────────────────────────

fn draw_discard_confirm(f: &mut Frame, _app: &App) {
    let area = centered_rect(46, 28, f.area());
    f.render_widget(Clear, area);

    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("Discard unsaved changes?", theme::text()))
                .alignment(Alignment::Center),
            Line::from(Span::styled("All edits will be lost.", theme::muted()))
                .alignment(Alignment::Center),
            Line::from(""),
            Line::from(vec![
                Span::styled("  y  ", theme::danger()),
                Span::styled("Discard", theme::muted()),
                Span::styled("     n / Esc  ", theme::accent()),
                Span::styled("Cancel  ", theme::muted()),
            ])
            .alignment(Alignment::Center),
        ])
        .block(theme::modal_danger("  Discard Changes")),
        area,
    );
}

// ── Delete confirmation ───────────────────────────────────────────────────────

fn draw_delete_confirm(f: &mut Frame, app: &App) {
    let area = centered_rect(46, 28, f.area());
    f.render_widget(Clear, area);

    let item_label = match app.active_tab {
        Tab::Journal => "journal entry",
        Tab::Contacts => "contact",
        Tab::Groups => "group",
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

fn draw_contact_picker(f: &mut Frame, app: &App, selected: usize, search_query: &str) {
    let area = centered_rect(60, 56, f.area());
    f.render_widget(Clear, area);

    // Render outer block for the modal
    let block = theme::modal("  Mention  ");
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let [search_area, list_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .areas(inner_area);

    // Search query box
    let display_query = format!(" {}", search_query);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(display_query, theme::text())))
            .block(theme::field("Search", true)),
        search_area,
    );

    // Filter items based on query
    let filtered = app.get_picker_items(search_query);

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|item| {
            ListItem::new(Line::from(Span::styled(
                format!("  {}", item.name),
                theme::text(),
            )))
        })
        .collect();

    let mut state = ListState::default();
    if !filtered.is_empty() {
        state.select(Some(selected));
    }

    f.render_stateful_widget(
        List::new(items)
            .block(theme::panel("Select"))
            .highlight_style(theme::list_highlight()),
        list_area,
        &mut state,
    );
}

fn draw_group_member_picker(f: &mut Frame, app: &App, selected: usize, search_query: &str) {
    let area = centered_rect(60, 56, f.area());
    f.render_widget(Clear, area);

    // Render outer block for the modal
    let block = theme::modal("  Members  ");
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let [search_area, list_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .areas(inner_area);

    // Search query box
    let display_query = format!(" {}", search_query);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(display_query, theme::text())))
            .block(theme::field("Search", true)),
        search_area,
    );

    // Filter contacts based on query
    let contacts = &app.journal.contacts;
    let filtered: Vec<&crate::model::Contact> = if search_query.is_empty() {
        contacts.iter().collect()
    } else {
        let q = search_query.to_lowercase();
        contacts
            .iter()
            .filter(|c| {
                c.full_name().to_lowercase().contains(&q) || c.nickname.to_lowercase().contains(&q)
            })
            .collect()
    };

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|c| {
            let is_checked = app.group_form.selected_member_ids.contains(&c.id);
            let indicator = if is_checked { "[x] " } else { "[ ] " };
            let style = if is_checked {
                theme::accent()
            } else {
                theme::text()
            };
            ListItem::new(Line::from(vec![
                Span::styled(indicator, style),
                Span::styled(c.full_name(), style),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    if !filtered.is_empty() {
        state.select(Some(selected));
    }

    f.render_stateful_widget(
        List::new(items)
            .block(theme::panel("People"))
            .highlight_style(theme::list_highlight()),
        list_area,
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

// ── Attachment picker modal ───────────────────────────────────────────────────

fn draw_attachment_picker(f: &mut Frame, app: &App, selected: usize) {
    let area = centered_rect(68, 48, f.area());
    f.render_widget(Clear, area);

    let real_idx = match app.selected_entry_idx() {
        Some(idx) => idx,
        None => return,
    };
    let entry = &app.journal.entries[real_idx];

    let items: Vec<ListItem> = if entry.attachments.is_empty() {
        vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(Span::styled(
                "    No attachments on this entry.",
                theme::muted(),
            ))),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![
                Span::styled("    Press ", theme::muted()),
                Span::styled("a", theme::accent()),
                Span::styled(" to attach a new file.", theme::muted()),
            ])),
        ]
    } else {
        entry
            .attachments
            .iter()
            .enumerate()
            .map(|(idx, att)| {
                let size_str = if att.size_bytes >= 1024 * 1024 {
                    format!("{:.1} MiB", att.size_bytes as f64 / (1024.0 * 1024.0))
                } else if att.size_bytes >= 1024 {
                    format!("{:.1} KiB", att.size_bytes as f64 / 1024.0)
                } else {
                    format!("{} B", att.size_bytes)
                };

                let prefix = if idx == selected { " ▶ " } else { "   " };
                let style = if idx == selected {
                    theme::accent()
                } else {
                    theme::text()
                };

                ListItem::new(Line::from(vec![
                    Span::styled(prefix, theme::accent()),
                    Span::styled(format!("{:<30} ", att.filename), style),
                    Span::styled(format!("({:<12}) ", att.mime_type), theme::muted()),
                    Span::styled(size_str, theme::muted()),
                ]))
            })
            .collect()
    };

    let mut state = ListState::default();
    if !entry.attachments.is_empty() {
        state.select(Some(selected));
    }

    f.render_stateful_widget(
        List::new(items)
            .block(theme::modal("  Manage Attachments  "))
            .highlight_style(theme::list_highlight()),
        area,
        &mut state,
    );
}
