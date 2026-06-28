//! Contacts tab — contact list (left) and profile / form (right).

use super::theme;
use crate::app::{App, AppMode, ContactField};
use crate::model::{BLOOD_TYPE_OPTIONS, Contact, MARITAL_STATUS_OPTIONS};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::Modifier,
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

// ── Contact list ──────────────────────────────────────────────────────────────

fn draw_list(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_contacts();
    let count = filtered.len();

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let selected = i == app.selected_index;
            let name_style = if selected {
                theme::accent()
            } else {
                theme::text()
            };
            let meta_style = if selected {
                theme::muted()
            } else {
                theme::dim()
            };

            // Build a one-line subtitle (nickname or birth year if available).
            let subtitle = if !c.nickname.is_empty() {
                format!("  {}", c.nickname)
            } else if let Some(y) = c.birthdate.map(|d| d.format("%Y").to_string()) {
                format!("  b. {}", y)
            } else {
                String::new()
            };

            let mut item_lines = vec![Line::from(Span::styled(
                format!(" {}", c.display_name()),
                name_style,
            ))];
            if !subtitle.is_empty() {
                item_lines.push(Line::from(Span::styled(subtitle, meta_style)));
            }
            item_lines.push(Line::from(""));
            ListItem::new(item_lines)
        })
        .collect();

    let title = if count == 0 {
        "People".to_string()
    } else {
        format!(
            "People  {} contact{}",
            count,
            if count == 1 { "" } else { "s" }
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

// ── Profile card ──────────────────────────────────────────────────────────────

fn draw_profile(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_contacts();

    if filtered.is_empty() {
        let msg = if !app.search_query.is_empty() {
            "No contacts match the current search."
        } else {
            "No contacts yet.  Press  n  to add the first one."
        };
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(""),
                Line::from(Span::styled(msg, theme::muted())).alignment(Alignment::Center),
            ])
            .block(theme::panel("Profile")),
            area,
        );
        return;
    }

    let contact = filtered[app.selected_index];

    // Split into profile card (top 60 %) and mentions panel (bottom 40 %).
    let [card_area, mentions_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .areas(area);

    draw_profile_card(f, contact, card_area);
    draw_mentions_panel(f, app, contact, mentions_area);
}

fn draw_profile_card(f: &mut Frame, c: &Contact, area: Rect) {
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", c.full_name()),
            theme::text().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {}", "─".repeat((area.width as usize).saturating_sub(4))),
            theme::dim(),
        )),
        Line::from(""),
    ];

    let mut row = |label: &str, value: &str| {
        if !value.is_empty() {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<14}", label), theme::label()),
                Span::styled(value.to_string(), theme::text()),
            ]));
        }
    };

    if !c.nickname.is_empty() {
        row("Nickname", &c.nickname);
    }
    if !c.preferred_name.is_empty() {
        row("Preferred", &c.preferred_name);
    }
    if !c.maiden_name.is_empty() {
        row("Maiden name", &c.maiden_name);
    }

    if let Some(birth) = c.birthdate {
        let age_suffix = if c.date_of_death.is_none() {
            c.calculate_age()
                .map(|a| format!("  (age {})", a))
                .unwrap_or_default()
        } else {
            String::new()
        };
        row(
            "Born",
            &format!("{}{}", crate::app::format_localized_date(birth), age_suffix),
        );
    }
    if let Some(death) = c.date_of_death {
        let age_suffix = c
            .calculate_age()
            .map(|a| format!("  (aged {})", a))
            .unwrap_or_default();
        row(
            "Deceased",
            &format!("{}{}", crate::app::format_localized_date(death), age_suffix),
        );
    }

    if !c.gender.is_empty() {
        row("Gender", &c.gender);
    }
    if !c.pronouns.is_empty() {
        row("Pronouns", &c.pronouns);
    }
    if !c.nationalities.is_empty() {
        row("Nationality", &c.nationalities.join(", "));
    }
    if !c.languages.is_empty() {
        row("Languages", &c.languages.join(", "));
    }
    if c.marital_status != "N/A" {
        row("Marital", &c.marital_status);
    }
    if !c.religion.is_empty() {
        row("Religion", &c.religion);
    }
    if c.blood_type != "N/A" {
        row("Blood type", &c.blood_type);
    }
    if !c.eye_color.is_empty() {
        row("Eyes", &c.eye_color);
    }
    if !c.hair_color.is_empty() {
        row("Hair", &c.hair_color);
    }
    if let Some(h) = c.height {
        row("Height", &format!("{} cm", h));
    }

    if !c.notes.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("  Notes", theme::label())));
        for note_line in c.notes.lines() {
            lines.push(Line::from(Span::styled(
                format!("    {}", note_line),
                theme::text(),
            )));
        }
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(theme::panel("Profile"))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_mentions_panel(f: &mut Frame, app: &App, contact: &Contact, area: Rect) {
    let mentions = app.get_mentions_for_contact(&contact.id);
    let block = theme::panel("Journal mentions");

    if mentions.is_empty() {
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("No mentions of {} yet", contact.display_name()),
                    theme::muted(),
                ))
                .alignment(Alignment::Center),
            ])
            .block(block),
            area,
        );
        return;
    }

    let items: Vec<ListItem> = mentions
        .iter()
        .map(|entry| {
            let date = app.journal.format_date_short(&entry.timestamp);
            let snippet: String = entry
                .content
                .lines()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("")
                .trim()
                .chars()
                .take(50)
                .collect();
            let snippet = if snippet.len() >= 50 {
                format!("{}…", snippet.trim_end())
            } else {
                snippet
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", date), theme::label()),
                Span::styled(snippet, theme::text()),
            ]))
        })
        .collect();

    f.render_widget(List::new(items).block(block), area);
}

// ── Contact form ──────────────────────────────────────────────────────────────

/// Row height for a given field type.
fn field_height(field: &ContactField) -> u16 {
    match field {
        ContactField::Notes => 6,
        _ => 3,
    }
}

fn field_label(field: ContactField) -> String {
    let (placeholder, _) = crate::app::get_date_format_info();
    match field {
        ContactField::Title => "Title".into(),
        ContactField::FirstName(0) => "First name".into(),
        ContactField::FirstName(i) => format!("Additional first name {}", i + 1),
        ContactField::LastName => "Last name".into(),
        ContactField::Nickname => "Nickname".into(),
        ContactField::PreferredName => "Preferred name".into(),
        ContactField::MaidenName => "Maiden name".into(),
        ContactField::Suffix => "Suffix".into(),
        ContactField::Birthdate => format!("Date of birth  ({})", placeholder),
        ContactField::DateOfDeath => format!("Date of death  ({})", placeholder),
        ContactField::Gender => "Gender".into(),
        ContactField::Pronouns => "Pronouns".into(),
        ContactField::Nationality(i) => format!("Nationality {}", i + 1),
        ContactField::Language(i) => format!("Language {}", i + 1),
        ContactField::MaritalStatus => "Marital status".into(),
        ContactField::Religion => "Religion".into(),
        ContactField::BloodType => "Blood type".into(),
        ContactField::EyeColor => "Eye colour".into(),
        ContactField::HairColor => "Hair colour".into(),
        ContactField::Height => "Height (cm)".into(),
        ContactField::Notes => "Notes".into(),
    }
}

fn draw_form(f: &mut Frame, app: &mut App, area: Rect, is_edit: bool) {
    let title = if is_edit {
        "Edit Contact"
    } else {
        "New Contact"
    };
    f.render_widget(theme::field(title, true), area);

    let inner = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });
    let fields = app.contact_form.field_order();
    let heights: Vec<u16> = fields.iter().map(field_height).collect();
    let total: u16 = heights.iter().sum();
    let viewport = inner.height;

    // Keep focused field visible by adjusting the scroll offset.
    let mut focus_top = 0u16;
    let mut focus_bottom = 0u16;
    let mut cursor = 0u16;
    for (i, &h) in heights.iter().enumerate() {
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
    app.contact_form.scroll = app.contact_form.scroll.min(total.saturating_sub(viewport));

    let scroll = app.contact_form.scroll;
    let mut cursor = 0u16;

    for (i, field) in fields.iter().enumerate() {
        let h = heights[i];
        let field_top = cursor;
        let field_bottom = cursor + h;
        cursor += h;

        let vis_top = field_top.max(scroll);
        let vis_bottom = field_bottom.min(scroll + viewport);
        if vis_bottom <= vis_top {
            continue;
        }

        let rect = Rect {
            x: inner.x,
            y: inner.y + (vis_top - scroll),
            width: inner.width,
            height: vis_bottom - vis_top,
        };
        render_field(f, app, *field, rect, i == app.contact_form.active_field);
    }
}

fn render_text(
    f: &mut Frame,
    area: Rect,
    label: String,
    focused: bool,
    ta: &mut TextArea<'static>,
) {
    ta.set_block(theme::field(label, focused));
    f.render_widget(&*ta, area);
}

fn render_selector(f: &mut Frame, area: Rect, label: String, focused: bool, value: &str) {
    let display = if focused {
        format!("  ‹  {}  ›", value)
    } else {
        format!("     {}   ", value)
    };
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(display, theme::text())))
            .alignment(Alignment::Center)
            .block(theme::field(label, focused)),
        area,
    );
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
