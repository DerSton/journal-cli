//! Page 6: Contact Insights statistics screen.

use crate::app::App;
use crate::ui::stats_tab::helpers::{calculate_top_contacts, truncate_pad};
use crate::ui::theme;
use chrono::{Datelike, Local};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::Paragraph,
};

/// Renders the contact mention metrics, trends, co-occurrences, and activity timelines.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vertical_chunks[0]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vertical_chunks[1]);

    draw_most_mentioned(f, app, top_chunks[0]);
    draw_mention_trends(f, app, top_chunks[1]);
    draw_cooccurrence_matrix(f, app, bottom_chunks[0]);
    draw_activity_timeline(f, app, bottom_chunks[1]);
}

fn draw_most_mentioned(f: &mut Frame, app: &App, area: Rect) {
    let top_contacts = calculate_top_contacts(app);
    let limit = 10;
    let max_mentions = top_contacts.first().map(|c| c.1).unwrap_or(1).max(1);
    let max_bar_width = area.width.saturating_sub(28) as usize;

    let mut lines = vec![Line::from("")];
    for (i, (contact, count)) in top_contacts.iter().take(limit).enumerate() {
        let medal = match i {
            0 => "① ",
            1 => "② ",
            2 => "③ ",
            _ => "   ",
        };
        let bar_len =
            ((*count as f64 / max_mentions as f64) * max_bar_width as f64).round() as usize;
        let bar_len = bar_len.max(1);
        let bar = "█".repeat(bar_len);

        let style = match i {
            0..=2 => theme::accent(),
            _ => theme::text(),
        };

        lines.push(Line::from(vec![
            Span::styled(medal, theme::label()),
            Span::styled(truncate_pad(&contact.display_name(), 15), theme::text()),
            Span::styled(bar, style),
            Span::styled(format!(" ({})", count), theme::muted()),
        ]));
    }

    if top_contacts.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No contact mentions found.",
            theme::muted(),
        )));
    }

    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Most Mentioned Contacts")),
        area,
    );
}

fn draw_mention_trends(f: &mut Frame, app: &App, area: Rect) {
    let top_contacts = calculate_top_contacts(app);
    let today = Local::now().date_naive();

    let mut lines = vec![Line::from("")];

    let mut header_spans = vec![Span::styled("  Contact         ", theme::dim())];
    for i in (0..6).rev() {
        let date = today.checked_sub_months(chrono::Months::new(i)).unwrap();
        let m_abbrev = crate::ui::stats_tab::helpers::month_abbrev(date.month());
        header_spans.push(Span::styled(format!(" {} ", m_abbrev), theme::muted()));
    }
    lines.push(Line::from(header_spans));

    for (contact, _) in top_contacts.iter().take(5) {
        let tag = contact.mention_tag();
        let mut row_spans = vec![Span::styled(
            format!("  {}", truncate_pad(&contact.display_name(), 16)),
            theme::text(),
        )];

        for i in (0..6).rev() {
            let date = today.checked_sub_months(chrono::Months::new(i)).unwrap();
            let y = date.year();
            let m = date.month();

            let count = app
                .journal
                .entries
                .iter()
                .filter(|e| {
                    let edt = e.timestamp.with_timezone(&Local);
                    edt.year() == y && edt.month() == m && e.content.contains(&tag)
                })
                .count();

            let (cell_char, style) = match count {
                0 => ("░", theme::dim()),
                1..=2 => ("▒", theme::muted()),
                3..=5 => ("▓", theme::label()),
                _ => ("█", theme::accent()),
            };

            row_spans.push(Span::styled(format!("  {}  ", cell_char), style));
        }
        lines.push(Line::from(row_spans));
    }

    if top_contacts.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No contacts found.",
            theme::muted(),
        )));
    }

    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Mention Trends (Last 6 Months)")),
        area,
    );
}

fn draw_cooccurrence_matrix(f: &mut Frame, app: &App, area: Rect) {
    let top_contacts = calculate_top_contacts(app);
    let limit = 6;
    let selected_contacts: Vec<_> = top_contacts.iter().take(limit).map(|c| c.0).collect();

    let mut lines = vec![Line::from("")];

    let mut header_spans = vec![Span::styled("  Colleagues  ", theme::dim())];
    for &c in &selected_contacts {
        let name_raw = if c.nickname.is_empty() {
            &c.last_name
        } else {
            &c.nickname
        };
        header_spans.push(Span::styled(
            format!(" {:^4} ", truncate_pad(name_raw, 4)),
            theme::muted(),
        ));
    }
    lines.push(Line::from(header_spans));

    for (i, &c_i) in selected_contacts.iter().enumerate() {
        let name_raw = if c_i.nickname.is_empty() {
            &c_i.last_name
        } else {
            &c_i.nickname
        };
        let mut row_spans = vec![Span::styled(
            format!("  {}", truncate_pad(name_raw, 12)),
            theme::text(),
        )];

        for (j, &c_j) in selected_contacts.iter().enumerate() {
            if i == j {
                row_spans.push(Span::styled("  -   ", theme::dim()));
            } else {
                let tag_i = c_i.mention_tag();
                let tag_j = c_j.mention_tag();
                let count = app
                    .journal
                    .entries
                    .iter()
                    .filter(|e| e.content.contains(&tag_i) && e.content.contains(&tag_j))
                    .count();

                if count > 0 {
                    row_spans.push(Span::styled(format!("  {:^2}  ", count), theme::accent()));
                } else {
                    row_spans.push(Span::styled("  0   ", theme::muted()));
                }
            }
        }
        lines.push(Line::from(row_spans));
    }

    if selected_contacts.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No co-occurrences.",
            theme::muted(),
        )));
    }

    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Co-occurrence Matrix")),
        area,
    );
}

fn draw_activity_timeline(f: &mut Frame, app: &App, area: Rect) {
    let top_contacts = calculate_top_contacts(app);
    let today = Local::now().date_naive();

    let mut lines = vec![Line::from("")];
    let max_timeline_weeks = area.width.saturating_sub(20) as usize;
    let max_timeline_weeks = max_timeline_weeks.min(26);

    for contact in top_contacts.iter().take(5).map(|x| x.0) {
        let tag = contact.mention_tag();
        let name_raw = if contact.nickname.is_empty() {
            &contact.last_name
        } else {
            &contact.nickname
        };
        let mut row_spans = vec![Span::styled(
            format!("  {}: ", truncate_pad(name_raw, 10)),
            theme::text(),
        )];

        for w_idx in (0..max_timeline_weeks).rev() {
            let week_start = today - chrono::Duration::days(w_idx as i64 * 7 + 6);
            let week_end = today - chrono::Duration::days(w_idx as i64 * 7);

            let has_mention = app.journal.entries.iter().any(|e| {
                let d = e.timestamp.with_timezone(&Local).date_naive();
                d >= week_start && d <= week_end && e.content.contains(&tag)
            });

            let cell = if has_mention { "█" } else { "·" };
            let style = if has_mention {
                theme::accent()
            } else {
                theme::dim()
            };
            row_spans.push(Span::styled(cell, style));
        }
        lines.push(Line::from(row_spans));
    }

    if top_contacts.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No timeline available.",
            theme::muted(),
        )));
    }

    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Contact Activity (Last 26 Weeks)")),
        area,
    );
}
