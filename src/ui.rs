use crate::app::{App, AppMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

/// Main UI rendering entry point.
pub fn draw(f: &mut Frame, app: &mut App) {
    // 1. Create vertical layout split: main area + bottom status/help bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    let main_area = chunks[0];
    let status_area = chunks[1];

    // 2. Split main area: Left List (35%) + Right Preview/Editor (65%)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(main_area);

    let list_area = main_chunks[0];
    let content_area = main_chunks[1];

    // --- DRAW LEFT SIDEBAR (ENTRIES LIST) ---
    let items: Vec<ListItem> = app
        .journal
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i == app.selected_index;
            let local_time = entry.timestamp.with_timezone(&chrono::Local);
            let time_str = local_time.format("%Y-%m-%d %H:%M:%S").to_string();

            // Create snippet (first line, truncated)
            let snippet = entry.content.lines().next().unwrap_or("").trim();
            let snippet_truncated = if snippet.chars().count() > 30 {
                let s: String = snippet.chars().take(27).collect();
                format!("{}...", s)
            } else {
                snippet.to_string()
            };

            let title_style = if is_selected {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::raw("🗓  "),
                    Span::styled(time_str, title_style),
                ]),
                Line::from(vec![
                    Span::raw("   "),
                    Span::styled(snippet_truncated, Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(""), // separator line
            ])
        })
        .collect();

    let list_title = Span::styled(
        format!(" Entries ({}) ", app.journal.entries.len()),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    );

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(list_title);

    let mut list_state = ratatui::widgets::ListState::default();
    if !app.journal.entries.is_empty() {
        list_state.select(Some(app.selected_index));
    }

    let entry_list = List::new(items)
        .block(list_block)
        .highlight_style(
            Style::default()
                .bg(Color::Indexed(236)) // subtle background highlighting
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(entry_list, list_area, &mut list_state);

    // --- DRAW RIGHT PANE (DETAIL PREVIEW OR EDITOR) ---
    match app.mode {
        AppMode::Writing { is_edit } => {
            let editor_title = if is_edit {
                " ✏️  Edit Entry [Ctrl+S: Save, Esc: Cancel] "
            } else {
                " ➕  New Entry [Ctrl+S: Save, Esc: Cancel] "
            };

            let editor_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(editor_title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));

            app.textarea.set_block(editor_block);
            app.textarea.set_cursor_line_style(Style::default().bg(Color::Indexed(235)));
            f.render_widget(&app.textarea, content_area);
        }
        _ => {
            if app.journal.entries.is_empty() {
                let empty_block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(Span::styled(" View Entry ", Style::default().fg(Color::White)));

                let text = vec![
                    Line::from(""),
                    Line::from("No entries in this journal yet.").alignment(ratatui::layout::Alignment::Center),
                    Line::from("Press 'n' to write your first entry!").alignment(ratatui::layout::Alignment::Center),
                ];
                let paragraph = Paragraph::new(text).block(empty_block).wrap(ratatui::widgets::Wrap { trim: true });
                f.render_widget(paragraph, content_area);
            } else {
                let entry = &app.journal.entries[app.selected_index];
                let local_time = entry.timestamp.with_timezone(&chrono::Local);
                let time_str = local_time.format("%A, %B %d, %Y - %H:%M:%S").to_string();

                let detail_title = Span::styled(
                    format!(" Viewing Entry ({} of {}) ", app.selected_index + 1, app.journal.entries.len()),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                );

                let detail_block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(detail_title);

                // Build full entry display
                let mut text_lines = vec![
                    Line::from(vec![
                        Span::styled("Date: ", Style::default().fg(Color::Cyan)),
                        Span::styled(time_str, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(Span::styled("━".repeat(content_area.width as usize - 4), Style::default().fg(Color::DarkGray))),
                    Line::from(""),
                ];

                for line in entry.content.lines() {
                    text_lines.push(Line::from(Span::styled(line, Style::default().fg(Color::White))));
                }

                let total_text_lines = text_lines.len();

                let paragraph = Paragraph::new(text_lines)
                    .block(detail_block)
                    .wrap(ratatui::widgets::Wrap { trim: false })
                    .scroll((app.detail_scroll, 0));

                f.render_widget(paragraph, content_area);

                // Add scrollbar indicator if the entry content runs off screen
                let content_height = content_area.height.saturating_sub(2) as usize;
                if total_text_lines > content_height {
                    let scrollbar = Scrollbar::default()
                        .orientation(ScrollbarOrientation::VerticalRight)
                        .begin_symbol(Some("▲"))
                        .end_symbol(Some("▼"));
                    let mut scrollbar_state = ScrollbarState::default()
                        .content_length(total_text_lines.saturating_sub(content_height))
                        .position(app.detail_scroll as usize);
                    f.render_stateful_widget(
                        scrollbar,
                        content_area.inner(ratatui::layout::Margin { horizontal: 0, vertical: 1 }),
                        &mut scrollbar_state,
                    );
                }
            }
        }
    }

    // --- DRAW STATUS & ACTION FOOTER ---
    let status_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let status_span = if let Some(err) = &app.error_msg {
        Span::styled(err, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
    } else if let Some(status) = &app.status_msg {
        Span::styled(status, Style::default().fg(Color::Green))
    } else {
        Span::styled("Securely encrypted with Argon2id + ChaCha20Poly1305", Style::default().fg(Color::DarkGray))
    };

    let help_spans = match app.mode {
        AppMode::List => vec![
            Span::styled(" n: ", Style::default().fg(Color::Cyan)),
            Span::styled("New ", Style::default().fg(Color::White)),
            Span::styled(" e: ", Style::default().fg(Color::Cyan)),
            Span::styled("Edit ", Style::default().fg(Color::White)),
            Span::styled(" d: ", Style::default().fg(Color::Cyan)),
            Span::styled("Delete ", Style::default().fg(Color::White)),
            Span::styled(" PgUp/PgDn: ", Style::default().fg(Color::Cyan)),
            Span::styled("Scroll Preview ", Style::default().fg(Color::White)),
            Span::styled(" q: ", Style::default().fg(Color::Cyan)),
            Span::styled("Quit ", Style::default().fg(Color::White)),
        ],
        AppMode::Writing { .. } => vec![
            Span::styled(" Ctrl+S: ", Style::default().fg(Color::Cyan)),
            Span::styled("Save ", Style::default().fg(Color::White)),
            Span::styled(" Esc: ", Style::default().fg(Color::Cyan)),
            Span::styled("Cancel ", Style::default().fg(Color::White)),
        ],
        AppMode::DeleteConfirm => vec![
            Span::styled(" Confirm Delete? ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" y: ", Style::default().fg(Color::Cyan)),
            Span::styled("Yes, Delete ", Style::default().fg(Color::White)),
            Span::styled(" n/Esc: ", Style::default().fg(Color::Cyan)),
            Span::styled("Cancel ", Style::default().fg(Color::White)),
        ],
    };

    let status_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(status_block.inner(status_area));

    f.render_widget(status_block, status_area);
    f.render_widget(Paragraph::new(Line::from(vec![Span::raw(" STATUS: "), status_span])), status_layout[0]);
    f.render_widget(Paragraph::new(Line::from(help_spans)).alignment(ratatui::layout::Alignment::Right), status_layout[1]);

    // --- DRAW MODAL OVERLAY FOR DELETE CONFIRMATION ---
    if app.mode == AppMode::DeleteConfirm {
        let modal_area = centered_rect(50, 25, f.area());
        f.render_widget(Clear, modal_area);

        let confirm_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Red))
            .title(Span::styled(" WARNING ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));

        let confirm_text = vec![
            Line::from(""),
            Line::from("Are you sure you want to delete this entry?").alignment(ratatui::layout::Alignment::Center),
            Line::from("This action is permanent and cannot be undone.").alignment(ratatui::layout::Alignment::Center),
            Line::from(""),
            Line::from(vec![
                Span::styled(" [y] Yes, Delete ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw("   "),
                Span::styled(" [n/Esc] Cancel ", Style::default().fg(Color::White)),
            ]).alignment(ratatui::layout::Alignment::Center),
        ];

        let confirm_para = Paragraph::new(confirm_text)
            .block(confirm_block)
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(confirm_para, modal_area);
    }
}

/// Helper function to center a modal window on screen
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
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
        .split(popup_layout[1])[1]
}
