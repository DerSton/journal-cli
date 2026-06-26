use crate::app::App;
use crate::ui::theme;
use chrono::NaiveDate;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{BarChart, Paragraph, Wrap},
};
use std::collections::HashMap;

const STOP_WORDS: &[&str] = &[
    // Deutsch
    "ich", "du", "er", "sie", "es", "wir", "ihr", "mein", "dein", "sein", "ihr", "unser", "ein",
    "eine", "einer", "eines", "einem", "einen", "der", "die", "das", "den", "dem", "des", "und",
    "oder", "aber", "so", "ja", "nein", "ist", "sind", "war", "waren", "mit", "von", "zu", "in",
    "auf", "im", "am", "für", "um", "als", "wie", "dass", "mir", "mich", "dir", "dich", "uns",
    "euch", "sich", "nicht", "nur", "auch", "noch", "schon", "jetzt", "dann", "da", "hier",
    "heute", "morgen", "gestern", "mal", "habe", "hat", "haben", "hatte", "wurde", "werden",
    // Englisch
    "i", "you", "he", "she", "it", "we", "they", "my", "your", "his", "her", "its", "our", "their",
    "a", "an", "the", "and", "or", "but", "so", "yes", "no", "is", "are", "was", "were", "with",
    "from", "to", "in", "on", "at", "for", "of", "about", "as", "like", "that", "me", "him", "us",
    "them", "not", "only", "also", "now", "then", "there", "here", "today", "have", "has", "had",
    "been", "would", "could", "should", "will", "can", "this", "these", "those",
];

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;

    // Streaks berechnen
    let (current_streak, max_streak) = calculate_streaks(entries);

    // Gesamte Wortanzahl und Durchschnitt berechnen
    let total_words: usize = entries
        .iter()
        .map(|e| e.content.split_whitespace().count())
        .collect::<Vec<usize>>()
        .iter()
        .sum();
    let avg_words = if !entries.is_empty() {
        total_words / entries.len()
    } else {
        0
    };

    // Längster Eintrag
    let longest_entry = entries
        .iter()
        .map(|e| (e, e.content.split_whitespace().count()))
        .max_by_key(|&(_, count)| count);

    // Top-Kontakte berechnen
    let top_contacts = calculate_top_contacts(app);

    // Häufigste Wörter berechnen
    let common_words = calculate_common_words(entries);

    // Layout erstellen
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Spalte 1: Allgemeine KPIs
    let mut kpi_lines = vec![
        Line::from(vec![
            Span::styled(" Total Entries: ", theme::title_style()),
            Span::styled(entries.len().to_string(), theme::text_style()),
        ]),
        Line::from(vec![
            Span::styled(" Total Words:   ", theme::title_style()),
            Span::styled(total_words.to_string(), theme::text_style()),
        ]),
        Line::from(vec![
            Span::styled(" Avg Words/Day: ", theme::title_style()),
            Span::styled(avg_words.to_string(), theme::text_style()),
        ]),
    ];

    if let Some((entry, count)) = longest_entry {
        let date_str = entry
            .timestamp
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d")
            .to_string();
        kpi_lines.push(Line::from(vec![
            Span::styled(" Longest Entry: ", theme::title_style()),
            Span::styled(
                format!("{} words (on {})", count, date_str),
                theme::text_style(),
            ),
        ]));
    } else {
        kpi_lines.push(Line::from(vec![
            Span::styled(" Longest Entry: ", theme::title_style()),
            Span::styled("N/A", theme::muted_style()),
        ]));
    }

    kpi_lines.push(Line::from(""));
    kpi_lines.push(Line::from(vec![
        Span::styled(" Current Streak: ", theme::title_style()),
        Span::styled(
            format!("{} day(s) 🔥", current_streak),
            if current_streak > 0 {
                theme::success_style()
            } else {
                theme::text_style()
            },
        ),
    ]));
    kpi_lines.push(Line::from(vec![
        Span::styled(" Record Streak:  ", theme::title_style()),
        Span::styled(format!("{} day(s) 🏆", max_streak), theme::text_style()),
    ]));

    let kpi_paragraph = Paragraph::new(kpi_lines)
        .block(theme::panel_block("General Statistics"))
        .wrap(Wrap { trim: true });
    f.render_widget(kpi_paragraph, top_chunks[0]);

    // Spalte 2: Top-Kontakte
    let mut contact_lines = vec![];
    if top_contacts.is_empty() {
        contact_lines.push(Line::from(""));
        contact_lines.push(Line::from(Span::styled(
            " No contact mentions found yet.",
            theme::muted_style(),
        )));
    } else {
        for (i, (contact, count)) in top_contacts.iter().enumerate() {
            contact_lines.push(Line::from(vec![
                Span::styled(format!(" {}. ", i + 1), theme::title_style()),
                Span::styled(contact.full_name(), theme::text_style()),
                Span::styled(format!(" ({} mentions)", count), theme::muted_style()),
            ]));
        }
    }

    let contact_paragraph = Paragraph::new(contact_lines)
        .block(theme::panel_block("Top Mentions"))
        .wrap(Wrap { trim: true });
    f.render_widget(contact_paragraph, top_chunks[1]);

    // Untere Reihe links: Wortanzahl-Diagramm der letzten 7 Einträge
    let last_entries: Vec<_> = app.filtered_entries().iter().take(7).cloned().collect();
    let mut word_data = Vec::new();
    for entry in last_entries.iter().rev() {
        let date_str = entry
            .timestamp
            .with_timezone(&chrono::Local)
            .format("%d.%m.")
            .to_string();
        let words = entry.content.split_whitespace().count() as u64;
        word_data.push((date_str, words));
    }

    let bar_data: Vec<(&str, u64)> = word_data.iter().map(|(s, w)| (s.as_str(), *w)).collect();

    let chart_block = theme::panel_block("Word Count of Last 7 Entries");
    if bar_data.is_empty() {
        let empty_p = Paragraph::new(vec![
            Line::from(""),
            Line::from(" No entries to display chart.")
                .alignment(ratatui::layout::Alignment::Center),
        ])
        .block(chart_block);
        f.render_widget(empty_p, bottom_chunks[0]);
    } else {
        let chart = BarChart::default()
            .block(chart_block)
            .data(&bar_data)
            .bar_width(6)
            .bar_gap(2)
            .value_style(theme::text_style())
            .label_style(theme::title_style())
            .bar_style(ratatui::style::Style::default().fg(theme::ACCENT));
        f.render_widget(chart, bottom_chunks[0]);
    }

    // Untere Reihe rechts: Wortwolke
    let mut cloud_spans = Vec::new();
    if common_words.is_empty() {
        cloud_spans.push(Span::styled(" No words found yet.", theme::muted_style()));
    } else {
        for (i, (word, count)) in common_words.iter().enumerate() {
            if i > 0 {
                cloud_spans.push(Span::styled("  •  ", theme::muted_style()));
            }

            let style = if i < 3 {
                theme::title_style().add_modifier(ratatui::style::Modifier::UNDERLINED)
            } else if i < 6 {
                theme::text_style().add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                theme::muted_style()
            };

            cloud_spans.push(Span::styled(format!("{} ({})", word, count), style));
        }
    }

    let cloud_paragraph = Paragraph::new(Line::from(cloud_spans))
        .block(theme::panel_block("Common Words (Cloud)"))
        .wrap(Wrap { trim: true });
    f.render_widget(cloud_paragraph, bottom_chunks[1]);
}

fn calculate_streaks(entries: &[crate::model::JournalEntry]) -> (u32, u32) {
    if entries.is_empty() {
        return (0, 0);
    }

    let mut dates: Vec<NaiveDate> = entries
        .iter()
        .map(|e| e.timestamp.with_timezone(&chrono::Local).date_naive())
        .collect();
    dates.sort();
    dates.dedup();

    let mut max_streak = 0;
    let mut current_streak = 0;
    let mut last_date: Option<NaiveDate> = None;

    for date in &dates {
        match last_date {
            Some(last) => {
                if *date == last + chrono::Duration::days(1) {
                    current_streak += 1;
                } else if *date > last + chrono::Duration::days(1) {
                    if current_streak > max_streak {
                        max_streak = current_streak;
                    }
                    current_streak = 1;
                }
            }
            None => {
                current_streak = 1;
            }
        }
        last_date = Some(*date);
    }

    if current_streak > max_streak {
        max_streak = current_streak;
    }

    let today = chrono::Local::now().date_naive();
    let is_active = if let Some(last) = dates.last() {
        *last == today || *last == today - chrono::Duration::days(1)
    } else {
        false
    };

    let current = if is_active { current_streak } else { 0 };
    (current, max_streak)
}

fn calculate_top_contacts(app: &App) -> Vec<(&crate::model::Contact, usize)> {
    let mut counts = Vec::new();
    for contact in &app.journal.contacts {
        let tag = contact.mention_tag();
        let mut count = 0;
        for entry in &app.journal.entries {
            count += entry.content.matches(&tag).count();
        }
        if count > 0 {
            counts.push((contact, count));
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    counts.into_iter().take(5).collect()
}

fn calculate_common_words(entries: &[crate::model::JournalEntry]) -> Vec<(String, usize)> {
    let mut counts = HashMap::new();
    for entry in entries {
        let cleaned: String = entry
            .content
            .chars()
            .map(|c| {
                if c.is_alphabetic() {
                    c.to_lowercase().to_string()
                } else if c.is_whitespace() {
                    " ".to_string()
                } else {
                    "".to_string()
                }
            })
            .collect();

        for word in cleaned.split_whitespace() {
            if word.len() > 2 && !STOP_WORDS.contains(&word) {
                *counts.entry(word.to_string()).or_insert(0) += 1;
            }
        }
    }

    let mut list: Vec<(String, usize)> = counts.into_iter().collect();
    list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    list.truncate(10);
    list
}
