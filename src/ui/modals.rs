use super::{centered_rect, centered_rect_fixed, theme};
use crate::app::{App, AppMode, Tab};
use chrono::Datelike;
use ratatui::{
    Frame,
    layout::Alignment,
    text::{Line, Span},
    widgets::{BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

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

fn draw_delete_confirm(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 25, f.area());
    f.render_widget(Clear, area);

    let item = match app.active_tab {
        Tab::Dashboard => "dashboard item",
        Tab::Journal => "journal entry",
        Tab::Contacts => "contact",
        Tab::Settings => "setting",
        Tab::Stats => "stat",
    };

    let block = ratatui::widgets::Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme::danger_style())
        .title(" Confirm Delete ");

    let text = vec![
        Line::from(""),
        Line::from(format!("Delete this {}?", item)).alignment(Alignment::Center),
        Line::from("This cannot be undone.").alignment(Alignment::Center),
        Line::from(""),
        Line::from(vec![
            Span::styled(" y ", theme::danger_style()),
            Span::raw("Delete   "),
            Span::styled(" n / Esc ", theme::title_style()),
            Span::raw("Cancel"),
        ])
        .alignment(Alignment::Center),
    ];

    f.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_contact_picker(f: &mut Frame, app: &App, selected_contact_index: usize) {
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);

    let block = ratatui::widgets::Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme::border_style(true))
        .title(" Mention Contact (Enter: Pick, Esc: Cancel) ");

    let items: Vec<ListItem> = app
        .journal
        .contacts
        .iter()
        .map(|c| ListItem::new(Line::from(Span::raw(c.full_name()))))
        .collect();

    let mut state = ListState::default();
    if !app.journal.contacts.is_empty() {
        state.select(Some(selected_contact_index));
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(theme::list_highlight_style());
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_date_picker(f: &mut Frame, app: &App, field_index: usize, current_date: chrono::NaiveDate) {
    let area = centered_rect_fixed(34, 13, f.area());
    f.render_widget(Clear, area);

    let field_name = if app.active_tab == Tab::Journal {
        "Entry Date"
    } else if field_index == 0 {
        "Birthdate"
    } else {
        "Date of Death"
    };
    let block = ratatui::widgets::Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme::border_style(true))
        .title(format!(" Select {} ", field_name));

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

    let mut lines = vec![
        Line::from(""),
        Line::from(format!("  {:^28}  ", format!("{} {}", month_name, year)))
            .style(theme::title_style()),
        Line::from("   Mo  Tu  We  Th  Fr  Sa  Su   ").style(theme::muted_style()),
        Line::from("  ----------------------------  ").style(theme::muted_style()),
    ];

    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        _ => 30,
    };
    let first_day = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let weekday_offset = (first_day.weekday().number_from_monday() - 1) as usize;

    for row in 0..6 {
        let mut spans = vec![Span::raw("   ")];
        for col in 0..7 {
            let cell_idx = row * 7 + col;
            if cell_idx < weekday_offset || cell_idx >= weekday_offset + days_in_month as usize {
                spans.push(Span::raw("    "));
            } else {
                let day = (cell_idx - weekday_offset + 1) as u32;
                let cell_date = chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap();
                let text = if day < 10 {
                    format!("  {} ", day)
                } else {
                    format!(" {} ", day)
                };
                let style = if cell_date == current_date {
                    ratatui::style::Style::default()
                        .bg(theme::ACCENT)
                        .fg(ratatui::style::Color::Black)
                        .add_modifier(ratatui::style::Modifier::BOLD)
                } else {
                    theme::text_style()
                };
                spans.push(Span::styled(text, style));
            }
        }
        spans.push(Span::raw(" "));
        lines.push(Line::from(spans));
    }

    lines.push(Line::from("   PgUp/Dn: Month   { }: Year   ").style(theme::muted_style()));

    f.render_widget(Paragraph::new(lines).block(block), area);
}
