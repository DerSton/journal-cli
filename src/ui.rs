use crate::app::{App, AppMode, Tab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

/// Main UI rendering entry point.
pub fn draw(f: &mut Frame, app: &mut App) {
    // 1. Create vertical layout split: Tab bar (3 lines) + Main Body + Bottom status/help bar (3 lines)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Navigation Tabs
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status / Help
        ])
        .split(f.area());

    let tabs_area = chunks[0];
    let main_area = chunks[1];
    let status_area = chunks[2];

    // --- DRAW TOP TAB BAR ---
    let tab_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let tab_titles = match app.active_tab {
        Tab::Journal => vec![
            Span::styled(" ● Journal (1) ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("   Contacts (2) ", Style::default().fg(Color::DarkGray)),
        ],
        Tab::Contacts => vec![
            Span::styled("   Journal (1) ", Style::default().fg(Color::DarkGray)),
            Span::styled(" ● Contacts (2) ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ],
    };

    let tab_line = Line::from(vec![
        Span::raw(" NAVIGATION: "),
        tab_titles[0].clone(),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        tab_titles[1].clone(),
        Span::styled("  (Press Tab to switch)", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
    ]);
    f.render_widget(Paragraph::new(tab_line).block(tab_block), tabs_area);

    // 2. Split main area: Left List (35%) + Right Preview/Editor (65%)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(main_area);

    let list_area = main_chunks[0];
    let content_area = main_chunks[1];

    // --- DRAW VIEWS DEPENDING ON ACTIVE TAB ---
    match app.active_tab {
        Tab::Journal => {
            // --- DRAW JOURNAL LIST ---
            let items: Vec<ListItem> = app
                .journal
                .entries
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let is_selected = i == app.selected_index;
                    let local_time = entry.timestamp.with_timezone(&chrono::Local);
                    let time_str = local_time.format("%Y-%m-%d %H:%M:%S").to_string();

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
                        Line::from(""),
                    ])
                })
                .collect();

            let list_title = Span::styled(
                format!(" Journal Entries ({}) ", app.journal.entries.len()),
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
                        .bg(Color::Indexed(236))
                        .add_modifier(Modifier::BOLD),
                );

            f.render_stateful_widget(entry_list, list_area, &mut list_state);

            // --- DRAW JOURNAL PREVIEW OR WRITER ---
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
        }
        Tab::Contacts => {
            // --- DRAW CONTACTS LIST ---
            let items: Vec<ListItem> = app
                .journal
                .contacts
                .iter()
                .enumerate()
                .map(|(i, contact)| {
                    let is_selected = i == app.selected_index;
                    let display_name = format!("{}, {} {}", contact.last_name, contact.first_name, contact.middle_name)
                        .trim()
                        .to_string();

                    let title_style = if is_selected {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    ListItem::new(vec![
                        Line::from(vec![
                            Span::raw("👤  "),
                            Span::styled(display_name, title_style),
                        ]),
                        Line::from(""),
                    ])
                })
                .collect();

            let list_title = Span::styled(
                format!(" Contacts ({}) ", app.journal.contacts.len()),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            );

            let list_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(list_title);

            let mut list_state = ratatui::widgets::ListState::default();
            if !app.journal.contacts.is_empty() {
                list_state.select(Some(app.selected_index));
            }

            let contact_list = List::new(items)
                .block(list_block)
                .highlight_style(
                    Style::default()
                        .bg(Color::Indexed(236))
                        .add_modifier(Modifier::BOLD),
                );

            f.render_stateful_widget(contact_list, list_area, &mut list_state);

            // --- DRAW CONTACT DETAILS PREVIEW OR MULTI-FIELD WRITER ---
            match app.mode {
                AppMode::Writing { is_edit } => {
                    let form_title = if is_edit { " ✏️  Edit Contact " } else { " ➕  New Contact " };
                    
                    let frame_block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Cyan))
                        .title(Span::styled(form_title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
                    
                    f.render_widget(frame_block, content_area);

                    let inner_area = content_area.inner(ratatui::layout::Margin { horizontal: 2, vertical: 2 });
                    let form_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3), // First Name
                            Constraint::Length(3), // Middle Name
                            Constraint::Length(3), // Last Name
                            Constraint::Min(0),    // Hints / Navigation instructions
                        ])
                        .split(inner_area);

                    // Configure and render individual form fields
                    let block_first = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(if app.active_field_index == 0 { Color::Cyan } else { Color::DarkGray }))
                        .title(" First Name ");
                    app.contact_first_name.set_block(block_first);
                    app.contact_first_name.set_cursor_line_style(Style::default());
                    f.render_widget(&app.contact_first_name, form_chunks[0]);

                    let block_middle = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(if app.active_field_index == 1 { Color::Cyan } else { Color::DarkGray }))
                        .title(" Middle Name ");
                    app.contact_middle_name.set_block(block_middle);
                    app.contact_middle_name.set_cursor_line_style(Style::default());
                    f.render_widget(&app.contact_middle_name, form_chunks[1]);

                    let block_last = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(if app.active_field_index == 2 { Color::Cyan } else { Color::DarkGray }))
                        .title(" Last Name ");
                    app.contact_last_name.set_block(block_last);
                    app.contact_last_name.set_cursor_line_style(Style::default());
                    f.render_widget(&app.contact_last_name, form_chunks[2]);

                    // Render hints & helpers
                    let hints = vec![
                        Line::from(""),
                        Line::from("Form Controls:").alignment(ratatui::layout::Alignment::Center),
                        Line::from(vec![
                            Span::styled(" Tab / Down arrow ", Style::default().fg(Color::Cyan)),
                            Span::raw("Next Field   "),
                            Span::styled(" Shift+Tab / Up arrow ", Style::default().fg(Color::Cyan)),
                            Span::raw("Prev Field"),
                        ]).alignment(ratatui::layout::Alignment::Center),
                        Line::from(""),
                        Line::from(vec![
                            Span::styled(" Ctrl + S ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                            Span::raw("Save Contact   "),
                            Span::styled(" Esc ", Style::default().fg(Color::Red)),
                            Span::raw("Cancel"),
                        ]).alignment(ratatui::layout::Alignment::Center),
                    ];
                    f.render_widget(Paragraph::new(hints), form_chunks[3]);
                }
                _ => {
                    if app.journal.contacts.is_empty() {
                        let empty_block = Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::DarkGray))
                            .title(Span::styled(" View Contact ", Style::default().fg(Color::White)));

                        let text = vec![
                            Line::from(""),
                            Line::from("No contacts found in database.").alignment(ratatui::layout::Alignment::Center),
                            Line::from("Press 'n' to add a new contact!").alignment(ratatui::layout::Alignment::Center),
                        ];
                        let paragraph = Paragraph::new(text).block(empty_block).wrap(ratatui::widgets::Wrap { trim: true });
                        f.render_widget(paragraph, content_area);
                    } else {
                        let contact = &app.journal.contacts[app.selected_index];
                        let first_initial = contact.first_name.chars().next().unwrap_or('?').to_uppercase().to_string();
                        let last_initial = contact.last_name.chars().next().unwrap_or('?').to_uppercase().to_string();
                        let initials = format!(" {}{} ", first_initial, last_initial);

                        let detail_block = Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::DarkGray))
                            .title(Span::styled(" Contact Details ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));

                        let text = vec![
                            Line::from(""),
                            Line::from(vec![
                                Span::styled("  [", Style::default().fg(Color::DarkGray)),
                                Span::styled(initials, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                                Span::styled("]  ", Style::default().fg(Color::DarkGray)),
                                Span::styled(
                                    format!("{} {} {}", contact.first_name, contact.middle_name, contact.last_name).trim().to_string(),
                                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                                ),
                            ]),
                            Line::from(""),
                            Line::from(Span::styled("  ━".repeat(content_area.width as usize - 6), Style::default().fg(Color::DarkGray))),
                            Line::from(""),
                            Line::from(vec![
                                Span::styled("  First Name:  ", Style::default().fg(Color::Cyan)),
                                Span::styled(&contact.first_name, Style::default().fg(Color::White)),
                            ]),
                            Line::from(""),
                            Line::from(vec![
                                Span::styled("  Middle Name: ", Style::default().fg(Color::Cyan)),
                                Span::styled(if contact.middle_name.is_empty() { "-" } else { &contact.middle_name }, Style::default().fg(Color::White)),
                            ]),
                            Line::from(""),
                            Line::from(vec![
                                Span::styled("  Last Name:   ", Style::default().fg(Color::Cyan)),
                                Span::styled(&contact.last_name, Style::default().fg(Color::White)),
                            ]),
                        ];
                        f.render_widget(Paragraph::new(text).block(detail_block), content_area);
                    }
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
        AppMode::List => {
            let mut spans = vec![];
            match app.active_tab {
                Tab::Journal => {
                    spans.push(Span::styled(" n: ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled("New Entry ", Style::default().fg(Color::White)));
                    spans.push(Span::styled(" e: ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled("Edit ", Style::default().fg(Color::White)));
                    spans.push(Span::styled(" d: ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled("Delete ", Style::default().fg(Color::White)));
                    spans.push(Span::styled(" PgUp/PgDn: ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled("Scroll Preview ", Style::default().fg(Color::White)));
                }
                Tab::Contacts => {
                    spans.push(Span::styled(" n: ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled("New Contact ", Style::default().fg(Color::White)));
                    spans.push(Span::styled(" e: ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled("Edit ", Style::default().fg(Color::White)));
                    spans.push(Span::styled(" d: ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled("Delete ", Style::default().fg(Color::White)));
                }
            }
            spans.push(Span::styled(" Tab: ", Style::default().fg(Color::Cyan)));
            spans.push(Span::styled("Switch Tab ", Style::default().fg(Color::White)));
            spans.push(Span::styled(" q: ", Style::default().fg(Color::Cyan)));
            spans.push(Span::styled("Quit ", Style::default().fg(Color::White)));
            spans
        }
        AppMode::Writing { .. } => match app.active_tab {
            Tab::Journal => vec![
                Span::styled(" Ctrl+S: ", Style::default().fg(Color::Cyan)),
                Span::styled("Save Entry ", Style::default().fg(Color::White)),
                Span::styled(" Esc: ", Style::default().fg(Color::Cyan)),
                Span::styled("Cancel ", Style::default().fg(Color::White)),
            ],
            Tab::Contacts => vec![
                Span::styled(" Tab/Down: ", Style::default().fg(Color::Cyan)),
                Span::styled("Next Field ", Style::default().fg(Color::White)),
                Span::styled(" Shift+Tab/Up: ", Style::default().fg(Color::Cyan)),
                Span::styled("Prev Field ", Style::default().fg(Color::White)),
                Span::styled(" Ctrl+S: ", Style::default().fg(Color::Cyan)),
                Span::styled("Save Contact ", Style::default().fg(Color::White)),
                Span::styled(" Esc: ", Style::default().fg(Color::Cyan)),
                Span::styled("Cancel ", Style::default().fg(Color::White)),
            ],
        },
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

        let item_type = match app.active_tab {
            Tab::Journal => "journal entry",
            Tab::Contacts => "contact details",
        };

        let confirm_text = vec![
            Line::from(""),
            Line::from(format!("Are you sure you want to delete this {}?", item_type)).alignment(ratatui::layout::Alignment::Center),
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
