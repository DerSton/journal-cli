//! Page 8: People Directory Stats statistics screen.

use crate::app::App;
use crate::ui::stats_tab::helpers::{kpi_row, truncate_pad};
use crate::ui::theme;
use chrono::{Datelike, Local, NaiveDate};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{BarChart, Paragraph, Wrap},
};
use std::collections::HashMap;

/// Renders the contacts KPIs, age distribution bar chart, and upcoming birthday notifications.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(10)])
        .split(area);

    draw_contact_kpis(f, app, chunks[0]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    draw_age_distribution(f, app, bottom_chunks[0]);
    draw_upcoming_birthdays(f, app, bottom_chunks[1]);
}

fn draw_contact_kpis(f: &mut Frame, app: &App, area: Rect) {
    let contacts = &app.journal.contacts;
    let entries = &app.journal.entries;

    let total_contacts = contacts.len();
    let with_birthdate = contacts.iter().filter(|c| c.birthdate.is_some()).count();

    let alive_ages: Vec<u32> = contacts
        .iter()
        .filter(|c| c.date_of_death.is_none())
        .filter_map(|c| c.calculate_age())
        .collect();
    let avg_age = if !alive_ages.is_empty() {
        alive_ages.iter().sum::<u32>() as f64 / alive_ages.len() as f64
    } else {
        0.0
    };

    let mut genders = HashMap::new();
    for c in contacts {
        if !c.gender.trim().is_empty() {
            *genders.entry(c.gender.clone()).or_insert(0) += 1;
        }
    }
    let mut gender_list: Vec<_> = genders.into_iter().collect();
    gender_list.sort_by(|a, b| b.1.cmp(&a.1));
    let gender_summary = gender_list
        .iter()
        .take(2)
        .map(|(g, count)| format!("{}: {}", g, count))
        .collect::<Vec<_>>()
        .join(", ");

    let mut mentioned_ids = std::collections::HashSet::new();
    for entry in entries {
        for c in contacts {
            if entry.content.contains(&c.mention_tag()) {
                mentioned_ids.insert(c.id.clone());
            }
        }
    }
    let never_mentioned = total_contacts - mentioned_ids.len();

    let mut lines = vec![Line::from("")];
    lines.push(kpi_row(
        "Total Contacts",
        Span::styled(total_contacts.to_string(), theme::text()),
    ));
    lines.push(kpi_row(
        "Contacts with Birthday",
        Span::styled(with_birthdate.to_string(), theme::text()),
    ));
    lines.push(kpi_row(
        "Average Age (Alive)",
        Span::styled(format!("{:.1} yrs", avg_age), theme::text()),
    ));
    lines.push(kpi_row(
        "Gender Distribution",
        Span::styled(
            if gender_summary.is_empty() {
                "N/A".to_string()
            } else {
                gender_summary
            },
            theme::text(),
        ),
    ));
    lines.push(kpi_row(
        "Never Mentioned",
        Span::styled(never_mentioned.to_string(), theme::text()),
    ));

    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Contact Database KPIs")),
        area,
    );
}

fn draw_age_distribution(f: &mut Frame, app: &App, area: Rect) {
    let contacts = &app.journal.contacts;
    let mut buckets = [0u64; 5];

    for c in contacts {
        if let Some(age) = c.calculate_age().filter(|_| c.date_of_death.is_none()) {
            match age {
                0..=18 => buckets[0] += 1,
                19..=30 => buckets[1] += 1,
                31..=50 => buckets[2] += 1,
                51..=70 => buckets[3] += 1,
                _ => buckets[4] += 1,
            }
        }
    }

    let temp_data = [
        ("0-18".to_string(), buckets[0]),
        ("19-30".to_string(), buckets[1]),
        ("31-50".to_string(), buckets[2]),
        ("51-70".to_string(), buckets[3]),
        ("70+".to_string(), buckets[4]),
    ];

    let bar_data: Vec<(&str, u64)> = temp_data.iter().map(|(l, c)| (l.as_str(), *c)).collect();

    let block = theme::panel("Age Distribution (Alive)");
    let bar_width =
        (((area.width.saturating_sub(6) as usize) / 5).saturating_sub(2)).clamp(2, 6) as u16;

    let chart = BarChart::default()
        .block(block)
        .data(&bar_data)
        .bar_width(bar_width)
        .bar_gap(2)
        .value_style(theme::text())
        .label_style(theme::muted())
        .bar_style(Style::default().fg(theme::ACCENT));

    f.render_widget(chart, area);
}

fn draw_upcoming_birthdays(f: &mut Frame, app: &App, area: Rect) {
    let contacts = &app.journal.contacts;
    let today = Local::now().date_naive();
    let current_year = today.year();

    let mut upcoming = Vec::new();

    for c in contacts {
        if c.date_of_death.is_some() {
            continue;
        }
        if let Some(birth) = c.birthdate {
            let mut next_bday = NaiveDate::from_ymd_opt(current_year, birth.month(), birth.day())
                .unwrap_or_else(|| {
                    NaiveDate::from_ymd_opt(current_year, birth.month(), 28).unwrap()
                });

            if next_bday < today {
                next_bday = NaiveDate::from_ymd_opt(current_year + 1, birth.month(), birth.day())
                    .unwrap_or_else(|| {
                        NaiveDate::from_ymd_opt(current_year + 1, birth.month(), 28).unwrap()
                    });
            }

            let days_until = (next_bday - today).num_days();
            let age_turning = next_bday.year() - birth.year();
            upcoming.push((c, next_bday, days_until, age_turning));
        }
    }

    upcoming.sort_by_key(|x| x.2);

    let mut lines = vec![Line::from("")];
    for (contact, bday, days, age) in upcoming.iter().take(10) {
        let style = if *days <= 7 {
            theme::streak()
        } else {
            theme::text()
        };

        let label = if *days == 0 { " 🎂 TODAY!" } else { "" };

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} ", truncate_pad(&contact.display_name(), 15)),
                theme::text(),
            ),
            Span::styled(
                format!("{} (turns {})", bday.format("%b %d"), age),
                theme::muted(),
            ),
            Span::styled(format!(" - in {} days{}", days, label), style),
        ]));
    }

    if upcoming.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No upcoming birthdays found.",
            theme::muted(),
        )));
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(theme::panel("Upcoming Birthdays (Top 10)"))
            .wrap(Wrap { trim: false }),
        area,
    );
}
