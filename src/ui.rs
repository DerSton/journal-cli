use crate::app::{App, AppMode, Tab};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
};

/// Resolves `{{person|handle}}` tags in text lines, replacing them with highlighted contact full names.
fn render_mentions<'a>(line: &'a str, contacts: &[crate::journal::Contact]) -> Line<'a> {
    let mut spans = Vec::new();
    let mut last_idx = 0;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check if substring starts with "{{person|"
        if i + 9 <= chars.len() && chars[i..i + 9] == ['{', '{', 'p', 'e', 'r', 's', 'o', 'n', '|']
        {
            let start_idx = i;
            i += 9;
            let mut handle = String::new();
            let mut found_closing = false;

            while i < chars.len() {
                if i + 2 <= chars.len() && chars[i..i + 2] == ['}', '}'] {
                    found_closing = true;
                    i += 2;
                    break;
                } else {
                    handle.push(chars[i]);
                    i += 1;
                }
            }

            if found_closing && !handle.is_empty() {
                // Find matching contact by handle (case-insensitive)
                let found_contact = contacts
                    .iter()
                    .find(|c| c.handle.to_lowercase() == handle.to_lowercase());
                if let Some(contact) = found_contact {
                    // Push plain text prior to the handle
                    if start_idx > last_idx {
                        let text: String = chars[last_idx..start_idx].iter().collect();
                        spans.push(Span::styled(text, Style::default().fg(Color::White)));
                    }
                    // Push styled contact full name
                    let full_name = contact.full_name();
                    spans.push(Span::styled(
                        full_name,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::UNDERLINED),
                    ));
                    last_idx = i;
                }
            }
        } else {
            i += 1;
        }
    }

    // Push remaining text on line
    if last_idx < chars.len() {
        let text: String = chars[last_idx..].iter().collect();
        spans.push(Span::styled(text, Style::default().fg(Color::White)));
    }

    Line::from(spans)
}

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
            Span::styled(
                " ● Journal (1) ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("   Contacts (2) ", Style::default().fg(Color::DarkGray)),
            Span::styled("   Settings (3) ", Style::default().fg(Color::DarkGray)),
        ],
        Tab::Contacts => vec![
            Span::styled("   Journal (1) ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                " ● Contacts (2) ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("   Settings (3) ", Style::default().fg(Color::DarkGray)),
        ],
        Tab::Settings => vec![
            Span::styled("   Journal (1) ", Style::default().fg(Color::DarkGray)),
            Span::styled("   Contacts (2) ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                " ● Settings (3) ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ],
    };

    let tab_line = Line::from(vec![
        Span::raw(" NAVIGATION: "),
        tab_titles[0].clone(),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        tab_titles[1].clone(),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        tab_titles[2].clone(),
        Span::styled(
            "  (Press Tab or 1-3 to switch)",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        ),
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
                    let time_str = app.journal.format_timestamp_short(&entry.timestamp);

                    let snippet = entry.content.lines().next().unwrap_or("").trim();
                    let snippet_truncated = if snippet.chars().count() > 30 {
                        let s: String = snippet.chars().take(27).collect();
                        format!("{}...", s)
                    } else {
                        snippet.to_string()
                    };

                    let title_style = if is_selected {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    ListItem::new(vec![
                        Line::from(vec![Span::raw("🗓  "), Span::styled(time_str, title_style)]),
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
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
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

            let entry_list = List::new(items).block(list_block).highlight_style(
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
                        .title(Span::styled(
                            editor_title,
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ));

                    app.textarea.set_block(editor_block);
                    app.textarea
                        .set_cursor_line_style(Style::default().bg(Color::Indexed(235)));
                    f.render_widget(&app.textarea, content_area);
                }
                _ => {
                    if app.journal.entries.is_empty() {
                        let empty_block = Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::DarkGray))
                            .title(Span::styled(
                                " View Entry ",
                                Style::default().fg(Color::White),
                            ));

                        let text = vec![
                            Line::from(""),
                            Line::from("No entries in this journal yet.")
                                .alignment(ratatui::layout::Alignment::Center),
                            Line::from("Press 'n' to write your first entry!")
                                .alignment(ratatui::layout::Alignment::Center),
                        ];
                        let paragraph = Paragraph::new(text)
                            .block(empty_block)
                            .wrap(ratatui::widgets::Wrap { trim: true });
                        f.render_widget(paragraph, content_area);
                    } else {
                        let entry = &app.journal.entries[app.selected_index];
                        let time_str = app.journal.format_timestamp(&entry.timestamp);

                        let detail_title = Span::styled(
                            format!(
                                " Viewing Entry ({} of {}) ",
                                app.selected_index + 1,
                                app.journal.entries.len()
                            ),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        );

                        let detail_block = Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::DarkGray))
                            .title(detail_title);

                        let mut text_lines = vec![
                            Line::from(vec![
                                Span::styled("Date: ", Style::default().fg(Color::Cyan)),
                                Span::styled(
                                    time_str,
                                    Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD),
                                ),
                            ]),
                            Line::from(Span::styled(
                                "━".repeat((content_area.width as usize).saturating_sub(4)),
                                Style::default().fg(Color::DarkGray),
                            )),
                            Line::from(""),
                        ];

                        // Render lines resolving mentions to full contact names
                        for line in entry.content.lines() {
                            text_lines.push(render_mentions(line, &app.journal.contacts));
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
                                content_area.inner(ratatui::layout::Margin {
                                    horizontal: 0,
                                    vertical: 1,
                                }),
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
                    let display_name = contact.display_name();

                    let title_style = if is_selected {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
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
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
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

            let contact_list = List::new(items).block(list_block).highlight_style(
                Style::default()
                    .bg(Color::Indexed(236))
                    .add_modifier(Modifier::BOLD),
            );

            f.render_stateful_widget(contact_list, list_area, &mut list_state);

            // --- DRAW CONTACT DETAILS PREVIEW OR 5-FIELD WRITER ---
            match app.mode {
                AppMode::Writing { is_edit } => {
                    let form_title = if is_edit {
                        " ✏️  Edit Contact "
                    } else {
                        " ➕  New Contact "
                    };

                    let frame_block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Cyan))
                        .title(Span::styled(
                            form_title,
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ));

                    f.render_widget(frame_block, content_area);

                    let inner_area = content_area.inner(ratatui::layout::Margin {
                        horizontal: 2,
                        vertical: 2,
                    });
                    let form_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3), // First Name
                            Constraint::Length(3), // Middle Name
                            Constraint::Length(3), // Last Name
                            Constraint::Length(3), // Handle
                            Constraint::Length(5), // Notes
                            Constraint::Min(0),    // Hints / Navigation instructions
                        ])
                        .split(inner_area);

                    // Configure and render individual form fields
                    let block_first = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(if app.active_field_index == 0 {
                            Color::Cyan
                        } else {
                            Color::DarkGray
                        }))
                        .title(" First Name ");
                    app.contact_first_name.set_block(block_first);
                    app.contact_first_name
                        .set_cursor_line_style(Style::default());
                    f.render_widget(&app.contact_first_name, form_chunks[0]);

                    let block_middle = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(if app.active_field_index == 1 {
                            Color::Cyan
                        } else {
                            Color::DarkGray
                        }))
                        .title(" Middle Name ");
                    app.contact_middle_name.set_block(block_middle);
                    app.contact_middle_name
                        .set_cursor_line_style(Style::default());
                    f.render_widget(&app.contact_middle_name, form_chunks[1]);

                    let block_last = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(if app.active_field_index == 2 {
                            Color::Cyan
                        } else {
                            Color::DarkGray
                        }))
                        .title(" Last Name ");
                    app.contact_last_name.set_block(block_last);
                    app.contact_last_name
                        .set_cursor_line_style(Style::default());
                    f.render_widget(&app.contact_last_name, form_chunks[2]);

                    let block_handle = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(if app.active_field_index == 3 {
                            Color::Cyan
                        } else {
                            Color::DarkGray
                        }))
                        .title(" Handle (for @mentions) ");
                    app.contact_handle.set_block(block_handle);
                    app.contact_handle.set_cursor_line_style(Style::default());
                    f.render_widget(&app.contact_handle, form_chunks[3]);

                    let block_notes = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(if app.active_field_index == 4 {
                            Color::Cyan
                        } else {
                            Color::DarkGray
                        }))
                        .title(" Notes ");
                    app.contact_notes.set_block(block_notes);
                    app.contact_notes
                        .set_cursor_line_style(Style::default().bg(Color::Indexed(235)));
                    f.render_widget(&app.contact_notes, form_chunks[4]);

                    // Render hints & helpers
                    let hints = vec![
                        Line::from("Form Controls:").alignment(ratatui::layout::Alignment::Center),
                        Line::from(vec![
                            Span::styled(" Tab / Down arrow ", Style::default().fg(Color::Cyan)),
                            Span::raw("Next Field   "),
                            Span::styled(
                                " Shift+Tab / Up arrow ",
                                Style::default().fg(Color::Cyan),
                            ),
                            Span::raw("Prev Field"),
                        ])
                        .alignment(ratatui::layout::Alignment::Center),
                        Line::from(vec![
                            Span::styled(
                                " Ctrl + S ",
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw("Save Contact   "),
                            Span::styled(" Esc ", Style::default().fg(Color::Red)),
                            Span::raw("Cancel"),
                        ])
                        .alignment(ratatui::layout::Alignment::Center),
                    ];
                    f.render_widget(Paragraph::new(hints), form_chunks[5]);
                }
                _ => {
                    if app.journal.contacts.is_empty() {
                        let empty_block = Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::DarkGray))
                            .title(Span::styled(
                                " View Contact ",
                                Style::default().fg(Color::White),
                            ));

                        let text = vec![
                            Line::from(""),
                            Line::from("No contacts found in database.")
                                .alignment(ratatui::layout::Alignment::Center),
                            Line::from("Press 'n' to add a new contact!")
                                .alignment(ratatui::layout::Alignment::Center),
                        ];
                        let paragraph = Paragraph::new(text)
                            .block(empty_block)
                            .wrap(ratatui::widgets::Wrap { trim: true });
                        f.render_widget(paragraph, content_area);
                    } else {
                        let contact = &app.journal.contacts[app.selected_index];
                        let initials = format!(" {} ", contact.initials());

                        let splits = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([
                                Constraint::Percentage(60), // Contact Info & Notes
                                Constraint::Percentage(40), // Mention History List
                            ])
                            .split(content_area);

                        // 1. Draw Profile & Notes Card
                        let detail_block = Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::DarkGray))
                            .title(Span::styled(
                                " Contact Profile ",
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            ));

                        let mut profile_text = vec![
                            Line::from(""),
                            Line::from(vec![
                                Span::styled("  [", Style::default().fg(Color::DarkGray)),
                                Span::styled(
                                    initials,
                                    Style::default()
                                        .fg(Color::Cyan)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled("]  ", Style::default().fg(Color::DarkGray)),
                                Span::styled(
                                    contact.full_name(),
                                    Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD),
                                ),
                            ]),
                            Line::from(Span::styled(
                                format!(
                                    "  {}",
                                    "━".repeat((splits[0].width as usize).saturating_sub(6))
                                ),
                                Style::default().fg(Color::DarkGray),
                            )),
                            Line::from(vec![
                                Span::styled("  First Name:  ", Style::default().fg(Color::Cyan)),
                                Span::styled(
                                    &contact.first_name,
                                    Style::default().fg(Color::White),
                                ),
                            ]),
                            Line::from(vec![
                                Span::styled("  Middle Name: ", Style::default().fg(Color::Cyan)),
                                Span::styled(
                                    if contact.middle_name.is_empty() {
                                        "-"
                                    } else {
                                        &contact.middle_name
                                    },
                                    Style::default().fg(Color::White),
                                ),
                            ]),
                            Line::from(vec![
                                Span::styled("  Last Name:   ", Style::default().fg(Color::Cyan)),
                                Span::styled(&contact.last_name, Style::default().fg(Color::White)),
                            ]),
                            Line::from(vec![
                                Span::styled("  Handle:      ", Style::default().fg(Color::Cyan)),
                                Span::styled(
                                    format!("@{}", contact.handle),
                                    Style::default().fg(Color::White),
                                ),
                            ]),
                            Line::from(""),
                            Line::from(Span::styled("  Notes:", Style::default().fg(Color::Cyan))),
                        ];

                        if contact.notes.is_empty() {
                            profile_text.push(Line::from(Span::styled(
                                "  -",
                                Style::default().fg(Color::DarkGray),
                            )));
                        } else {
                            for note_line in contact.notes.lines() {
                                profile_text.push(Line::from(Span::styled(
                                    format!("  {}", note_line),
                                    Style::default().fg(Color::White),
                                )));
                            }
                        }

                        f.render_widget(
                            Paragraph::new(profile_text)
                                .block(detail_block)
                                .wrap(ratatui::widgets::Wrap { trim: false }),
                            splits[0],
                        );

                        // 2. Draw Mention History List
                        let mentions = app.get_mentions_for_contact(&contact.handle);
                        let mentions_block = Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::DarkGray))
                            .title(Span::styled(
                                " Mentions in Journal ",
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            ));

                        if mentions.is_empty() {
                            let no_mentions = vec![
                                Line::from(""),
                                Line::from(format!(
                                    "No mentions of @{} found in your journal.",
                                    contact.handle
                                ))
                                .alignment(ratatui::layout::Alignment::Center)
                                .fg(Color::DarkGray),
                            ];
                            f.render_widget(
                                Paragraph::new(no_mentions).block(mentions_block),
                                splits[1],
                            );
                        } else {
                            let mut mention_items = Vec::new();
                            for entry in mentions {
                                let date_str = app.journal.format_date_short(&entry.timestamp);

                                let snippet = entry.content.lines().next().unwrap_or("").trim();
                                let snippet_truncated = if snippet.chars().count() > 45 {
                                    let s: String = snippet.chars().take(42).collect();
                                    format!("{}...", s)
                                } else {
                                    snippet.to_string()
                                };

                                mention_items.push(ListItem::new(vec![Line::from(vec![
                                    Span::styled(
                                        format!(" • {}: ", date_str),
                                        Style::default().fg(Color::Cyan),
                                    ),
                                    Span::styled(
                                        snippet_truncated,
                                        Style::default().fg(Color::White),
                                    ),
                                ])]));
                            }
                            f.render_widget(
                                List::new(mention_items).block(mentions_block),
                                splits[1],
                            );
                        }
                    }
                }
            }
        }
        Tab::Settings => {
            // --- DRAW SETTINGS LIST (Left Pane) ---
            let settings_groups = vec![
                "🔑  Change Password",
                "🌐  Language & Locale",
                "🕒  Timezone Offset",
            ];

            let list_items: Vec<ListItem> = settings_groups
                .iter()
                .enumerate()
                .map(|(i, group)| {
                    let is_selected = i == app.selected_index;
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(Line::from(vec![
                        Span::raw(if is_selected { "➔ " } else { "  " }),
                        Span::styled(*group, style),
                    ]))
                })
                .collect();

            let list_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Span::styled(
                    " Settings Menu ",
                    Style::default().fg(Color::White),
                ));

            let list_widget = List::new(list_items)
                .block(list_block)
                .highlight_style(Style::default().bg(Color::Indexed(236)));

            let mut list_state = ratatui::widgets::ListState::default();
            list_state.select(Some(app.selected_index));
            f.render_stateful_widget(list_widget, list_area, &mut list_state);

            // --- DRAW SETTINGS EDITOR/SELECTOR (Right Pane) ---
            match app.selected_index {
                0 => {
                    // CHANGE PASSWORD EDITOR
                    let is_editing_password = match app.mode {
                        AppMode::Writing { .. } => true,
                        _ => false,
                    };

                    let frame_block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(if is_editing_password {
                            Color::Cyan
                        } else {
                            Color::DarkGray
                        }))
                        .title(Span::styled(
                            " Change Master Password ",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ));

                    f.render_widget(frame_block, content_area);

                    let inner_area = content_area.inner(ratatui::layout::Margin {
                        horizontal: 2,
                        vertical: 2,
                    });
                    let form_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3), // New Password
                            Constraint::Length(3), // Confirm Password
                            Constraint::Min(0),    // Instructions / Status
                        ])
                        .split(inner_area);

                    let block_new = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(
                            if is_editing_password && app.settings_active_field == 0 {
                                Color::Cyan
                            } else {
                                Color::DarkGray
                            },
                        ))
                        .title(" New Master Password ");
                    app.settings_password_new.set_block(block_new);
                    app.settings_password_new
                        .set_cursor_line_style(Style::default());
                    f.render_widget(&app.settings_password_new, form_chunks[0]);

                    let block_confirm = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(
                            if is_editing_password && app.settings_active_field == 1 {
                                Color::Cyan
                            } else {
                                Color::DarkGray
                            },
                        ))
                        .title(" Confirm New Password ");
                    app.settings_password_confirm.set_block(block_confirm);
                    app.settings_password_confirm
                        .set_cursor_line_style(Style::default());
                    f.render_widget(&app.settings_password_confirm, form_chunks[1]);

                    // Render Instructions
                    let mut instructions = vec![
                        Line::from(""),
                        Line::from(vec![
                            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
                            Span::styled(
                                "Enter",
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(" or ", Style::default().fg(Color::DarkGray)),
                            Span::styled(
                                "e",
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                " to start typing new password.",
                                Style::default().fg(Color::DarkGray),
                            ),
                        ])
                        .alignment(ratatui::layout::Alignment::Center),
                    ];

                    if is_editing_password {
                        instructions = vec![
                            Line::from(""),
                            Line::from(vec![
                                Span::styled(
                                    " Tab / Down arrow ",
                                    Style::default().fg(Color::Cyan),
                                ),
                                Span::raw("Next Field   "),
                                Span::styled(
                                    " Shift+Tab / Up arrow ",
                                    Style::default().fg(Color::Cyan),
                                ),
                                Span::raw("Prev Field"),
                            ])
                            .alignment(ratatui::layout::Alignment::Center),
                            Line::from(vec![
                                Span::styled(
                                    " Ctrl + S ",
                                    Style::default()
                                        .fg(Color::Green)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::raw("Save & Re-encrypt   "),
                                Span::styled(" Esc ", Style::default().fg(Color::Red)),
                                Span::raw("Cancel"),
                            ])
                            .alignment(ratatui::layout::Alignment::Center),
                        ];
                    }

                    f.render_widget(Paragraph::new(instructions), form_chunks[2]);
                }
                1 => {
                    // LANGUAGE & LOCALE SELECTOR
                    let is_active = match app.mode {
                        AppMode::Writing { .. } => true,
                        _ => false,
                    };

                    let picker_block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(if is_active {
                            Color::Cyan
                        } else {
                            Color::DarkGray
                        }))
                        .title(Span::styled(
                            " Select Language & Locale ",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ));

                    // List of locale options
                    let locales = vec![
                        ("en_US", "English (United States)"),
                        ("de_DE", "Deutsch (Deutschland)"),
                        ("fr_FR", "Français (France)"),
                        ("es_ES", "Español (España)"),
                        ("it_IT", "Italiano (Italia)"),
                        ("ja_JP", "日本語 (日本)"),
                    ];

                    let list_items: Vec<ListItem> = locales
                        .iter()
                        .enumerate()
                        .map(|(idx, (code, name))| {
                            let is_current = code == &app.journal.settings.locale;
                            let is_highlighted = idx == app.settings_selected_option && is_active;

                            let mut spans = vec![];
                            if is_highlighted {
                                spans.push(Span::styled("➔ ", Style::default().fg(Color::Cyan)));
                            } else {
                                spans.push(Span::raw("  "));
                            }

                            let style = if is_highlighted {
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::White)
                            };

                            spans.push(Span::styled(format!("{} - {}", code, name), style));

                            if is_current {
                                spans.push(Span::styled(
                                    " (Active)",
                                    Style::default()
                                        .fg(Color::Green)
                                        .add_modifier(Modifier::ITALIC),
                                ));
                            }

                            ListItem::new(Line::from(spans))
                        })
                        .collect();

                    let list_widget = List::new(list_items)
                        .block(picker_block)
                        .highlight_style(Style::default().bg(Color::Indexed(236)));

                    let mut picker_state = ratatui::widgets::ListState::default();
                    if is_active {
                        picker_state.select(Some(app.settings_selected_option));
                    }

                    f.render_stateful_widget(list_widget, content_area, &mut picker_state);
                }
                2 => {
                    // TIMEZONE OFFSET SELECTOR
                    let is_active = match app.mode {
                        AppMode::Writing { .. } => true,
                        _ => false,
                    };

                    let picker_block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(if is_active {
                            Color::Cyan
                        } else {
                            Color::DarkGray
                        }))
                        .title(Span::styled(
                            " Select Timezone Offset ",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ));

                    // List of offsets (offset in mins, name)
                    let offsets = vec![
                        (-720, "UTC-12:00"),
                        (-660, "UTC-11:00"),
                        (-600, "UTC-10:00 (Hawaii)"),
                        (-540, "UTC-09:00 (Alaska)"),
                        (-480, "UTC-08:00 (Pacific Time)"),
                        (-420, "UTC-07:00 (Mountain Time)"),
                        (-360, "UTC-06:00 (Central Time)"),
                        (-300, "UTC-05:00 (Eastern Time)"),
                        (-240, "UTC-04:00 (Atlantic Time)"),
                        (-180, "UTC-03:00 (Argentina/Brazil)"),
                        (-120, "UTC-02:00"),
                        (-60, "UTC-01:00 (Azores)"),
                        (0, "UTC+00:00 (GMT / Greenwich Time)"),
                        (60, "UTC+01:00 (CET - Europe/Berlin, Paris, Rome)"),
                        (120, "UTC+02:00 (EET - Helsinki, Kyiv, Cairo)"),
                        (180, "UTC+03:00 (Moscow, Nairobi, Baghdad)"),
                        (240, "UTC+04:00 (Dubai)"),
                        (300, "UTC+05:00 (Karachi, Tashkent)"),
                        (330, "UTC+05:30 (IST - India)"),
                        (360, "UTC+06:00 (Dhaka)"),
                        (420, "UTC+07:00 (Bangkok, Jakarta)"),
                        (480, "UTC+08:00 (SGT - Singapore, Beijing, Perth)"),
                        (540, "UTC+09:00 (JST - Tokyo, Seoul)"),
                        (600, "UTC+10:00 (AEST - Sydney, Melbourne, Vladivostok)"),
                        (660, "UTC+11:00"),
                        (720, "UTC+12:00 (Auckland, Fiji)"),
                        (780, "UTC+13:00"),
                        (840, "UTC+14:00"),
                    ];

                    let list_items: Vec<ListItem> = offsets
                        .iter()
                        .enumerate()
                        .map(|(idx, (mins, name))| {
                            let is_current = *mins == app.journal.settings.timezone_offset_mins;
                            let is_highlighted = idx == app.settings_selected_option && is_active;

                            let mut spans = vec![];
                            if is_highlighted {
                                spans.push(Span::styled("➔ ", Style::default().fg(Color::Cyan)));
                            } else {
                                spans.push(Span::raw("  "));
                            }

                            let style = if is_highlighted {
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::White)
                            };

                            spans.push(Span::styled(*name, style));

                            if is_current {
                                spans.push(Span::styled(
                                    " (Active)",
                                    Style::default()
                                        .fg(Color::Green)
                                        .add_modifier(Modifier::ITALIC),
                                ));
                            }

                            ListItem::new(Line::from(spans))
                        })
                        .collect();

                    let list_widget = List::new(list_items)
                        .block(picker_block)
                        .highlight_style(Style::default().bg(Color::Indexed(236)));

                    let mut picker_state = ratatui::widgets::ListState::default();
                    if is_active {
                        picker_state.select(Some(app.settings_selected_option));
                    }

                    f.render_stateful_widget(list_widget, content_area, &mut picker_state);
                }
                _ => {}
            }
        }
    }

    // --- DRAW STATUS & ACTION FOOTER ---
    let status_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let help_spans = match app.mode {
        AppMode::List => {
            let mut spans = vec![];
            match app.active_tab {
                Tab::Journal => {
                    spans.push(Span::styled(" n: ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled(
                        "New Entry ",
                        Style::default().fg(Color::White),
                    ));
                    spans.push(Span::styled(" e: ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled("Edit ", Style::default().fg(Color::White)));
                    spans.push(Span::styled(" d: ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled("Delete ", Style::default().fg(Color::White)));
                    spans.push(Span::styled(
                        " PgUp/PgDn: ",
                        Style::default().fg(Color::Cyan),
                    ));
                    spans.push(Span::styled(
                        "Scroll Preview ",
                        Style::default().fg(Color::White),
                    ));
                }
                Tab::Contacts => {
                    spans.push(Span::styled(" n: ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled(
                        "New Contact ",
                        Style::default().fg(Color::White),
                    ));
                    spans.push(Span::styled(" e: ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled("Edit ", Style::default().fg(Color::White)));
                    spans.push(Span::styled(" d: ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled("Delete ", Style::default().fg(Color::White)));
                }
                Tab::Settings => {
                    spans.push(Span::styled(" Up/Down: ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled(
                        "Select Group ",
                        Style::default().fg(Color::White),
                    ));
                    spans.push(Span::styled(" Enter/e: ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled(
                        "Configure ",
                        Style::default().fg(Color::White),
                    ));
                }
            }
            spans.push(Span::styled(" Tab: ", Style::default().fg(Color::Cyan)));
            spans.push(Span::styled(
                "Switch Tab ",
                Style::default().fg(Color::White),
            ));
            spans.push(Span::styled(" q: ", Style::default().fg(Color::Cyan)));
            spans.push(Span::styled("Quit ", Style::default().fg(Color::White)));
            spans
        }
        AppMode::Writing { .. } => match app.active_tab {
            Tab::Journal => vec![
                Span::styled(" Alt+P: ", Style::default().fg(Color::Cyan)),
                Span::styled("Mention Contact ", Style::default().fg(Color::White)),
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
            Tab::Settings => match app.selected_index {
                0 => vec![
                    Span::styled(" Tab/Down: ", Style::default().fg(Color::Cyan)),
                    Span::styled("Next Field ", Style::default().fg(Color::White)),
                    Span::styled(" Shift+Tab/Up: ", Style::default().fg(Color::Cyan)),
                    Span::styled("Prev Field ", Style::default().fg(Color::White)),
                    Span::styled(" Ctrl+S: ", Style::default().fg(Color::Cyan)),
                    Span::styled("Re-encrypt ", Style::default().fg(Color::White)),
                    Span::styled(" Esc: ", Style::default().fg(Color::Cyan)),
                    Span::styled("Cancel ", Style::default().fg(Color::White)),
                ],
                _ => vec![
                    Span::styled(" Up/Down / j/k: ", Style::default().fg(Color::Cyan)),
                    Span::styled("Select Option ", Style::default().fg(Color::White)),
                    Span::styled(" Enter / Ctrl+S: ", Style::default().fg(Color::Cyan)),
                    Span::styled("Select & Save ", Style::default().fg(Color::White)),
                    Span::styled(" Esc: ", Style::default().fg(Color::Cyan)),
                    Span::styled("Cancel ", Style::default().fg(Color::White)),
                ],
            },
        },
        AppMode::ContactPicker { .. } => vec![
            Span::styled(" Up/Down / j/k: ", Style::default().fg(Color::Cyan)),
            Span::styled("Select Contact ", Style::default().fg(Color::White)),
            Span::styled(" Enter: ", Style::default().fg(Color::Cyan)),
            Span::styled("Mention ", Style::default().fg(Color::White)),
            Span::styled(" Esc: ", Style::default().fg(Color::Cyan)),
            Span::styled("Cancel ", Style::default().fg(Color::White)),
        ],
        AppMode::DeleteConfirm => vec![
            Span::styled(
                " Confirm Delete? ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" y: ", Style::default().fg(Color::Cyan)),
            Span::styled("Yes, Delete ", Style::default().fg(Color::White)),
            Span::styled(" n/Esc: ", Style::default().fg(Color::Cyan)),
            Span::styled("Cancel ", Style::default().fg(Color::White)),
        ],
    };

    let inner_status_area = status_block.inner(status_area);
    f.render_widget(status_block, status_area);
    f.render_widget(
        Paragraph::new(Line::from(help_spans)).alignment(ratatui::layout::Alignment::Center),
        inner_status_area,
    );

    // --- DRAW MODAL OVERLAY FOR DELETE CONFIRMATION ---
    if app.mode == AppMode::DeleteConfirm {
        let modal_area = centered_rect(50, 25, f.area());
        f.render_widget(Clear, modal_area);

        let confirm_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Red))
            .title(Span::styled(
                " WARNING ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ));

        let item_type = match app.active_tab {
            Tab::Journal => "journal entry",
            Tab::Contacts => "contact details",
            Tab::Settings => "settings",
        };

        let confirm_text = vec![
            Line::from(""),
            Line::from(format!(
                "Are you sure you want to delete this {}?",
                item_type
            ))
            .alignment(ratatui::layout::Alignment::Center),
            Line::from("This action is permanent and cannot be undone.")
                .alignment(ratatui::layout::Alignment::Center),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    " [y] Yes, Delete ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw("   "),
                Span::styled(" [n/Esc] Cancel ", Style::default().fg(Color::White)),
            ])
            .alignment(ratatui::layout::Alignment::Center),
        ];

        let confirm_para = Paragraph::new(confirm_text)
            .block(confirm_block)
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(confirm_para, modal_area);
    }

    // --- DRAW OVERLAY FOR CONTACT PICKER MENTIONS DIALOG ---
    if let AppMode::ContactPicker {
        selected_contact_index,
        ..
    } = app.mode
    {
        let modal_area = centered_rect(60, 50, f.area());
        f.render_widget(Clear, modal_area);

        let picker_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                " Select Contact to Mention [Enter: Pick, Esc: Cancel] ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));

        let items: Vec<ListItem> = app
            .journal
            .contacts
            .iter()
            .enumerate()
            .map(|(i, contact)| {
                let is_selected = i == selected_contact_index;
                let display = format!("{} (@{})", contact.full_name(), contact.handle);
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(vec![
                    Span::raw(if is_selected { "➔  " } else { "   " }),
                    Span::styled(display, style),
                ]))
            })
            .collect();

        let mut list_state = ratatui::widgets::ListState::default();
        if !app.journal.contacts.is_empty() {
            list_state.select(Some(selected_contact_index));
        }

        let list_widget = List::new(items)
            .block(picker_block)
            .highlight_style(Style::default().bg(Color::Indexed(236)));

        f.render_stateful_widget(list_widget, modal_area, &mut list_state);
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
