//! Journal tab — entry list (left) and entry preview / editor (right).

use super::theme;
use crate::app::{App, AppMode};
use crate::model::Contact;
use ratatui::{
    Frame,
    layout::{Alignment, Margin, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{
        List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
};

pub fn draw(f: &mut Frame, app: &mut App, list_area: Rect, content_area: Rect) {
    draw_list(f, app, list_area);

    match app.mode {
        AppMode::Writing { is_edit } => draw_editor(f, app, content_area, is_edit),
        _ => draw_preview(f, app, content_area),
    }
}

// ── Entry list ────────────────────────────────────────────────────────────────

fn draw_list(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_entries();
    let count = filtered.len();

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let date_str = if entry.date_for.is_some() {
                app.journal.format_date_short(&entry.sort_timestamp())
            } else {
                app.journal.format_timestamp_short(&entry.timestamp)
            };
            let snippet = first_line_truncated(&entry.content, 34);
            let selected = i == app.selected_index;

            let date_style = if selected {
                theme::accent()
            } else {
                theme::label()
            };
            let snippet_style = if selected {
                theme::text()
            } else {
                theme::muted()
            };

            ListItem::new(vec![
                Line::from(Span::styled(format!(" {}", date_str), date_style)),
                Line::from(Span::styled(format!("  {}", snippet), snippet_style)),
                Line::from(""),
            ])
        })
        .collect();

    let title = if count == 0 {
        "Journal".to_string()
    } else {
        format!(
            "Journal  {} entr{}",
            count,
            if count == 1 { "y" } else { "ies" }
        )
    };

    let mut state = ListState::default();
    if !filtered.is_empty() {
        state.select(Some(app.selected_index));
    }

    f.render_stateful_widget(
        List::new(items)
            .block(theme::panel(title))
            .highlight_style(theme::list_highlight()),
        area,
        &mut state,
    );
}

// ── Editor ────────────────────────────────────────────────────────────────────

fn draw_editor(f: &mut Frame, app: &mut App, area: Rect, is_edit: bool) {
    let date_hint = app
        .entry_date_for
        .map(|d| format!("  ·  {}", d.format("%Y-%m-%d")))
        .unwrap_or_default();

    let title = format!(
        "{}{}",
        if is_edit { "Edit Entry" } else { "New Entry" },
        date_hint
    );

    app.textarea.set_block(theme::field(title, true));
    app.textarea
        .set_cursor_line_style(theme::editor_cursor_line());
    f.render_widget(&app.textarea, area);
}

// ── Preview ───────────────────────────────────────────────────────────────────

fn draw_preview(f: &mut Frame, app: &mut App, area: Rect) {
    let filtered = app.filtered_entries();

    if filtered.is_empty() {
        let msg = if !app.search_query.is_empty() {
            "No entries match the current search."
        } else {
            "Your journal is empty.  Press  n  to write your first entry."
        };
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(""),
                Line::from(Span::styled(msg, theme::muted())).alignment(Alignment::Center),
            ])
            .block(theme::panel("Entry")),
            area,
        );
        return;
    }

    let entry = filtered[app.selected_index];

    let date_label = if entry.date_for.is_some() {
        app.journal.format_date(&entry.sort_timestamp())
    } else {
        app.journal.format_timestamp(&entry.timestamp)
    };

    let title = format!("Entry {} of {}", app.selected_index + 1, filtered.len());

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Date     ", theme::label()),
            Span::styled(date_label, theme::text()),
        ]),
    ];

    if entry.date_for.is_some() {
        let created = app.journal.format_timestamp(&entry.timestamp);
        lines.push(Line::from(vec![
            Span::styled("  Created  ", theme::muted()),
            Span::styled(created, theme::muted()),
        ]));
    }

    lines.push(Line::from(Span::styled(
        format!("  {}", "─".repeat((area.width as usize).saturating_sub(4))),
        theme::dim(),
    )));
    lines.push(Line::from(""));

    for line in entry.content.lines() {
        lines.push(render_mentions(line, &app.journal.contacts));
    }

    let total_lines = lines.len();
    let scrollbar_needed = total_lines > area.height.saturating_sub(2) as usize;

    f.render_widget(
        Paragraph::new(lines)
            .block(theme::panel(title))
            .wrap(Wrap { trim: false })
            .scroll((app.detail_scroll, 0)),
        area,
    );

    if scrollbar_needed {
        let visible = area.height.saturating_sub(2) as usize;
        let mut sb_state = ScrollbarState::default()
            .content_length(total_lines.saturating_sub(visible))
            .position(app.detail_scroll as usize);
        f.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼")),
            area.inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut sb_state,
        );
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Takes the first non-empty line of `text`, truncates to `max_chars` with an ellipsis.
fn first_line_truncated(text: &str, max_chars: usize) -> String {
    let line = text
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim();
    if line.chars().count() > max_chars {
        let head: String = line.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{}…", head)
    } else {
        line.to_string()
    }
}

/// Resolves `{{person|id}}` mention tags into highlighted contact names.
pub fn render_mentions<'a>(line: &'a str, contacts: &[Contact]) -> Line<'a> {
    const TAG_PREFIX: &str = "{{person|";
    const TAG_PREFIX_LEN: usize = TAG_PREFIX.len();

    let mut spans: Vec<Span<'a>> = Vec::new();
    let mut rest = line;

    while let Some(start) = rest.find(TAG_PREFIX) {
        // Push plain text before the tag.
        if start > 0 {
            spans.push(Span::styled(&rest[..start], theme::text()));
        }

        let after_prefix = &rest[start + TAG_PREFIX_LEN..];
        if let Some(end) = after_prefix.find("}}") {
            let id = &after_prefix[..end];
            if let Some(contact) = contacts.iter().find(|c| c.id == id) {
                spans.push(Span::styled(
                    contact.full_name(),
                    theme::mention().add_modifier(Modifier::UNDERLINED),
                ));
            } else {
                // Unknown id — render the raw tag.
                let raw_len = TAG_PREFIX_LEN + end + 2;
                spans.push(Span::styled(&rest[start..start + raw_len], theme::muted()));
            }
            rest = &rest[start + TAG_PREFIX_LEN + end + 2..];
        } else {
            // Malformed tag — emit the remainder as plain text.
            spans.push(Span::styled(rest, theme::text()));
            return Line::from(spans);
        }
    }

    // Remaining plain text.
    if !rest.is_empty() {
        spans.push(Span::styled(rest, theme::text()));
    }

    Line::from(spans)
}
