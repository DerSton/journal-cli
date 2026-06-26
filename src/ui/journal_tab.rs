use super::theme;
use crate::app::{App, AppMode};
use crate::model::Contact;
use ratatui::{
    Frame,
    layout::Rect,
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

fn draw_list(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_entries();
    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let time_str = app.journal.format_timestamp_short(&entry.timestamp);
            let snippet = truncate(entry.content.lines().next().unwrap_or("").trim(), 30);

            let title_style = if i == app.selected_index {
                theme::title_style()
            } else {
                theme::text_style()
            };

            ListItem::new(vec![
                Line::from(Span::styled(time_str, title_style)),
                Line::from(Span::styled(snippet, theme::muted_style())),
                Line::from(""),
            ])
        })
        .collect();

    let block = theme::panel_block(format!("Journal Entries ({})", filtered.len()));
    let list = List::new(items)
        .block(block)
        .highlight_style(theme::list_highlight_style());

    let mut state = ListState::default();
    if !filtered.is_empty() {
        state.select(Some(app.selected_index));
    }
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_editor(f: &mut Frame, app: &mut App, area: Rect, is_edit: bool) {
    let title = if is_edit { "Edit Entry" } else { "New Entry" };
    app.textarea.set_block(theme::field_block(title, true));
    app.textarea.set_cursor_line_style(
        ratatui::style::Style::default().bg(ratatui::style::Color::Indexed(235)),
    );
    f.render_widget(&app.textarea, area);
}

fn draw_preview(f: &mut Frame, app: &mut App, area: Rect) {
    let filtered = app.filtered_entries();
    if filtered.is_empty() {
        let msg = if !app.search_query.is_empty() {
            "No entries found matching search."
        } else {
            "No entries yet. Press 'n' to write one."
        };
        let text = vec![
            Line::from(""),
            Line::from(msg).alignment(ratatui::layout::Alignment::Center),
        ];
        let paragraph = Paragraph::new(text)
            .block(theme::panel_block("Entry"))
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
        return;
    }

    let entry = filtered[app.selected_index];
    let time_str = app.journal.format_timestamp(&entry.timestamp);

    let title = format!("Entry {} of {}", app.selected_index + 1, filtered.len());

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Date: ", theme::title_style()),
            Span::styled(time_str, theme::text_style()),
        ]),
        Line::from(Span::styled(
            "-".repeat((area.width as usize).saturating_sub(4)),
            theme::muted_style(),
        )),
        Line::from(""),
    ];
    for line in entry.content.lines() {
        lines.push(render_mentions(line, &app.journal.contacts));
    }

    let total_lines = lines.len();
    let paragraph = Paragraph::new(lines)
        .block(theme::panel_block(title))
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));
    f.render_widget(paragraph, area);

    let visible_height = area.height.saturating_sub(2) as usize;
    if total_lines > visible_height {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"));
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(total_lines.saturating_sub(visible_height))
            .position(app.detail_scroll as usize);
        f.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut scrollbar_state,
        );
    }
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        let head: String = s.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{}...", head)
    } else {
        s.to_string()
    }
}

/// Resolves `{{person|id}}` tags in a line of entry text into highlighted contact names.
pub fn render_mentions<'a>(line: &'a str, contacts: &[Contact]) -> Line<'a> {
    let mut spans = Vec::new();
    let mut last_idx = 0;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 9 <= chars.len() && chars[i..i + 9] == ['{', '{', 'p', 'e', 'r', 's', 'o', 'n', '|']
        {
            let start_idx = i;
            i += 9;
            let mut id = String::new();
            let mut found_closing = false;

            while i < chars.len() {
                if i + 2 <= chars.len() && chars[i..i + 2] == ['}', '}'] {
                    found_closing = true;
                    i += 2;
                    break;
                }
                id.push(chars[i]);
                i += 1;
            }

            if found_closing && let Some(contact) = contacts.iter().find(|c| c.id == id) {
                if start_idx > last_idx {
                    let text: String = chars[last_idx..start_idx].iter().collect();
                    spans.push(Span::styled(text, theme::text_style()));
                }
                spans.push(Span::styled(
                    contact.full_name(),
                    theme::title_style().add_modifier(ratatui::style::Modifier::UNDERLINED),
                ));
                last_idx = i;
            }
        } else {
            i += 1;
        }
    }

    if last_idx < chars.len() {
        let text: String = chars[last_idx..].iter().collect();
        spans.push(Span::styled(text, theme::text_style()));
    }

    Line::from(spans)
}
