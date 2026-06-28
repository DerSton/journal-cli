use super::theme;
use crate::app::{App, AppMode, ContactField};
use crate::model::{BLOOD_TYPE_OPTIONS, Contact, MARITAL_STATUS_OPTIONS};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
};
use ratatui_textarea::TextArea;

pub fn draw(f: &mut Frame, app: &mut App, list_area: Rect, content_area: Rect) {
    draw_list(f, app, list_area);

    match app.mode {
        AppMode::Writing { is_edit } => draw_form(f, app, content_area, is_edit),
        _ => draw_profile(f, app, content_area),
    }
}

fn draw_list(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_contacts();
    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, contact)| {
            let style = if i == app.selected_index {
                theme::title_style()
            } else {
                theme::text_style()
            };
            ListItem::new(vec![
                Line::from(Span::styled(contact.display_name(), style)),
                Line::from(""),
            ])
        })
        .collect();

    let block = theme::panel_block(format!("Contacts ({})", filtered.len()));
    let list = List::new(items)
        .block(block)
        .highlight_style(theme::list_highlight_style());

    let mut state = ListState::default();
    if !filtered.is_empty() {
        state.select(Some(app.selected_index));
    }
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_profile(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_contacts();
    if filtered.is_empty() {
        let msg = if !app.search_query.is_empty() {
            "No contacts found matching search"
        } else {
            "No contacts yet. Press 'n' to add one"
        };
        let text = vec![Line::from(""), Line::from(msg).alignment(Alignment::Center)];
        let paragraph = Paragraph::new(text)
            .block(theme::panel_block("Contact"))
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
        return;
    }

    let contact = filtered[app.selected_index];
    let splits = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    draw_profile_card(f, contact, splits[0]);
    draw_mentions_panel(f, app, contact, splits[1]);
}

fn draw_profile_card(f: &mut Frame, contact: &Contact, area: Rect) {
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", contact.full_name()),
            theme::text_style().add_modifier(ratatui::style::Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {}", "-".repeat((area.width as usize).saturating_sub(2))),
            theme::muted_style(),
        )),
    ];

    let mut field = |label: &str, value: &str| {
        if !value.is_empty() {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<16}", label), theme::title_style()),
                Span::styled(value.to_string(), theme::text_style()),
            ]));
        }
    };

    if !contact.nickname.is_empty() {
        field("Nickname", &contact.nickname);
    }
    if !contact.preferred_name.is_empty() {
        field("Preferred Name", &contact.preferred_name);
    }
    if !contact.maiden_name.is_empty() {
        field("Maiden Name", &contact.maiden_name);
    }

    if let Some(birth) = contact.birthdate {
        let age = if contact.date_of_death.is_none() {
            contact
                .calculate_age()
                .map(|a| format!(" (Age {})", a))
                .unwrap_or_default()
        } else {
            String::new()
        };
        field(
            "Born",
            &format!("{}{}", crate::app::format_localized_date(birth), age),
        );
    }
    if let Some(death) = contact.date_of_death {
        let age = contact
            .calculate_age()
            .map(|a| format!(" (Aged {})", a))
            .unwrap_or_default();
        field(
            "Deceased",
            &format!("{}{}", crate::app::format_localized_date(death), age),
        );
    }

    field("Gender", &contact.gender);
    field("Pronouns", &contact.pronouns);
    field("Nationalities", &contact.nationalities.join(", "));
    field("Languages", &contact.languages.join(", "));
    if contact.marital_status != "N/A" {
        field("Marital Status", &contact.marital_status);
    }
    field("Religion", &contact.religion);
    if contact.blood_type != "N/A" {
        field("Blood Type", &contact.blood_type);
    }
    field("Eye Color", &contact.eye_color);
    field("Hair Color", &contact.hair_color);
    if let Some(h) = contact.height {
        field("Height", &format!("{} cm", h));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  Notes", theme::title_style())));
    if contact.notes.is_empty() {
        lines.push(Line::from(Span::styled("  -", theme::muted_style())));
    } else {
        for note_line in contact.notes.lines() {
            lines.push(Line::from(Span::styled(
                format!("  {}", note_line),
                theme::text_style(),
            )));
        }
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(theme::panel_block("Contact Profile"))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_mentions_panel(f: &mut Frame, app: &App, contact: &Contact, area: Rect) {
    let mentions = app.get_mentions_for_contact(&contact.id);
    let block = theme::panel_block("Mentions in Journal");

    if mentions.is_empty() {
        let text = vec![
            Line::from(""),
            Line::from(format!("No mentions of {} found", contact.full_name()))
                .alignment(Alignment::Center)
                .style(theme::muted_style()),
        ];
        f.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let items: Vec<ListItem> = mentions
        .iter()
        .map(|entry| {
            let date = app.journal.format_date_short(&entry.timestamp);
            let snippet = entry.content.lines().next().unwrap_or("").trim();
            let snippet = if snippet.chars().count() > 45 {
                let s: String = snippet.chars().take(42).collect();
                format!("{}...", s)
            } else {
                snippet.to_string()
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", date), theme::title_style()),
                Span::styled(snippet, theme::text_style()),
            ]))
        })
        .collect();

    f.render_widget(List::new(items).block(block), area);
}

/// Height in rows the field's input box takes (matches `theme::field_block` + 1-line content).
fn field_height(field: &ContactField) -> u16 {
    match field {
        ContactField::Notes => 6,
        _ => 3,
    }
}

fn field_label(field: ContactField) -> String {
    match field {
        ContactField::Title => "Title".to_string(),
        ContactField::FirstName(0) => "First Name".to_string(),
        ContactField::FirstName(i) => format!("Additional First Name {}", i + 1),
        ContactField::LastName => "Last Name".to_string(),
        ContactField::Nickname => "Nickname".to_string(),
        ContactField::PreferredName => "Preferred Name".to_string(),
        ContactField::MaidenName => "Maiden Name".to_string(),
        ContactField::Suffix => "Suffix".to_string(),
        ContactField::Birthdate => {
            let (placeholder, _) = crate::app::get_date_format_info();
            format!("Date of Birth ({})", placeholder)
        }
        ContactField::DateOfDeath => {
            let (placeholder, _) = crate::app::get_date_format_info();
            format!("Date of Death ({})", placeholder)
        }
        ContactField::Gender => "Gender".to_string(),
        ContactField::Pronouns => "Pronouns".to_string(),
        ContactField::Nationality(i) => format!("Nationality {}", i + 1),
        ContactField::Language(i) => format!("Language {}", i + 1),
        ContactField::MaritalStatus => "Marital Status".to_string(),
        ContactField::Religion => "Religion".to_string(),
        ContactField::BloodType => "Blood Type".to_string(),
        ContactField::EyeColor => "Eye Color".to_string(),
        ContactField::HairColor => "Hair Color".to_string(),
        ContactField::Height => "Height (cm)".to_string(),
        ContactField::Notes => "Notes".to_string(),
    }
}

fn draw_form(f: &mut Frame, app: &mut App, area: Rect, is_edit: bool) {
    let title = if is_edit {
        "Edit Contact"
    } else {
        "New Contact"
    };
    f.render_widget(theme::field_block(title, true), area);
    let inner = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    let fields = app.contact_form.field_order();
    let heights: Vec<u16> = fields.iter().map(field_height).collect();
    let total_height: u16 = heights.iter().sum();
    let viewport = inner.height;

    let mut focus_top = 0u16;
    let mut focus_bottom = 0u16;
    let mut cursor = 0u16;
    for (i, h) in heights.iter().enumerate() {
        if i == app.contact_form.active_field {
            focus_top = cursor;
            focus_bottom = cursor + h;
        }
        cursor += h;
    }

    if focus_top < app.contact_form.scroll {
        app.contact_form.scroll = focus_top;
    } else if focus_bottom > app.contact_form.scroll + viewport {
        app.contact_form.scroll = focus_bottom.saturating_sub(viewport);
    }
    app.contact_form.scroll = app
        .contact_form
        .scroll
        .min(total_height.saturating_sub(viewport));
    let scroll = app.contact_form.scroll;

    let mut cursor = 0u16;
    for (i, field) in fields.iter().enumerate() {
        let h = heights[i];
        let field_top = cursor;
        let field_bottom = cursor + h;
        cursor += h;

        let visible_top = field_top.max(scroll);
        let visible_bottom = field_bottom.min(scroll + viewport);

        if visible_bottom <= visible_top {
            continue;
        }

        let rect = Rect {
            x: inner.x,
            y: inner.y + (visible_top - scroll),
            width: inner.width,
            height: visible_bottom - visible_top,
        };
        let focused = i == app.contact_form.active_field;
        render_field(f, app, *field, rect, focused);
    }
}

fn render_text(
    f: &mut Frame,
    area: Rect,
    label: String,
    focused: bool,
    ta: &mut TextArea<'static>,
) {
    ta.set_block(theme::field_block(label, focused));
    f.render_widget(&*ta, area);
}

fn render_selector(f: &mut Frame, area: Rect, label: String, focused: bool, value: &str) {
    let display = if focused {
        format!("<  {}  >", value)
    } else {
        format!("   {}   ", value)
    };
    let p = Paragraph::new(Line::from(Span::styled(display, theme::text_style())))
        .alignment(Alignment::Center)
        .block(theme::field_block(label, focused));
    f.render_widget(p, area);
}

fn render_field(f: &mut Frame, app: &mut App, field: ContactField, area: Rect, focused: bool) {
    let label = field_label(field);
    let form = &mut app.contact_form;
    match field {
        ContactField::Title => render_text(f, area, label, focused, &mut form.title),
        ContactField::FirstName(i) => {
            render_text(f, area, label, focused, &mut form.first_names.boxes[i])
        }
        ContactField::LastName => render_text(f, area, label, focused, &mut form.last_name),
        ContactField::Nickname => render_text(f, area, label, focused, &mut form.nickname),
        ContactField::PreferredName => {
            render_text(f, area, label, focused, &mut form.preferred_name)
        }
        ContactField::MaidenName => render_text(f, area, label, focused, &mut form.maiden_name),
        ContactField::Suffix => render_text(f, area, label, focused, &mut form.suffix),
        ContactField::Birthdate => render_text(f, area, label, focused, &mut form.birthdate),
        ContactField::DateOfDeath => render_text(f, area, label, focused, &mut form.date_of_death),
        ContactField::Gender => render_text(f, area, label, focused, &mut form.gender),
        ContactField::Pronouns => render_text(f, area, label, focused, &mut form.pronouns),
        ContactField::Nationality(i) => {
            render_text(f, area, label, focused, &mut form.nationalities.boxes[i])
        }
        ContactField::Language(i) => {
            render_text(f, area, label, focused, &mut form.languages.boxes[i])
        }
        ContactField::MaritalStatus => render_selector(
            f,
            area,
            label,
            focused,
            MARITAL_STATUS_OPTIONS[form.marital_status_idx],
        ),
        ContactField::Religion => render_text(f, area, label, focused, &mut form.religion),
        ContactField::BloodType => render_selector(
            f,
            area,
            label,
            focused,
            BLOOD_TYPE_OPTIONS[form.blood_type_idx],
        ),
        ContactField::EyeColor => render_text(f, area, label, focused, &mut form.eye_color),
        ContactField::HairColor => render_text(f, area, label, focused, &mut form.hair_color),
        ContactField::Height => render_text(f, area, label, focused, &mut form.height),
        ContactField::Notes => render_text(f, area, label, focused, &mut form.notes),
    }
}
