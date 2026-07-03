//! Stats / Insights tab — four-quadrant analytics dashboard.
//!
//! Displays writing streaks, word-count KPIs, a bar chart of recent word counts,
//! most-mentioned contacts, and a word-frequency cloud.

use crate::app::App;
use crate::ui::theme;
use chrono::NaiveDate;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{BarChart, Paragraph, Wrap},
};
use std::collections::HashMap;

// ── Stop-word list (English + German) ─────────────────────────────────────────

const STOP_WORDS: &[&str] = &[
    // German
    "ich", "du", "er", "sie", "es", "wir", "ihr", "mein", "dein", "sein", "unser", "ein", "eine",
    "einer", "eines", "einem", "einen", "der", "die", "das", "den", "dem", "des", "und", "oder",
    "aber", "so", "ja", "nein", "ist", "sind", "war", "waren", "mit", "von", "zu", "in", "auf",
    "im", "am", "für", "um", "als", "wie", "dass", "mir", "mich", "dir", "dich", "uns", "euch",
    "sich", "nicht", "nur", "auch", "noch", "schon", "jetzt", "dann", "da", "hier", "heute",
    "morgen", "gestern", "mal", "habe", "hat", "haben", "hatte", "wurde", "werden",
    // English
    "i", "you", "he", "she", "it", "we", "they", "my", "your", "his", "her", "its", "our", "their",
    "a", "an", "the", "and", "or", "but", "so", "yes", "no", "is", "are", "was", "were", "with",
    "from", "to", "in", "on", "at", "for", "of", "about", "as", "like", "that", "me", "him", "us",
    "them", "not", "only", "also", "now", "then", "there", "here", "today", "have", "has", "had",
    "been", "would", "could", "should", "will", "can", "this", "these", "those",
];

// ── Public entry point ────────────────────────────────────────────────────────

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;

    let (current_streak, max_streak) = calculate_streaks(entries);
    let total_words: usize = entries
        .iter()
        .map(|e| e.content.split_whitespace().count())
        .sum();
    let avg_words = if entries.is_empty() {
        0
    } else {
        total_words / entries.len()
    };
    let longest = entries
        .iter()
        .map(|e| (e, e.content.split_whitespace().count()))
        .max_by_key(|&(_, n)| n);
    let top_contacts = calculate_top_contacts(app);
    let common_words = calculate_common_words(entries);

    // Two rows, two columns each.
    let [top_row, bottom_row] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .areas(area);

    let [kpi_area, contacts_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .areas(top_row);

    let [chart_area, cloud_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .areas(bottom_row);

    draw_kpi_panel(
        f,
        entries.len(),
        total_words,
        avg_words,
        longest,
        current_streak,
        max_streak,
        kpi_area,
    );
    draw_contacts_panel(f, &top_contacts, contacts_area);
    draw_chart(f, app, chart_area);
    draw_word_cloud(f, &common_words, cloud_area);
}

// ── KPI panel ─────────────────────────────────────────────────────────────────

fn kpi_row<'a>(label: &'a str, value: impl Into<Span<'a>>) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("  {:<18}", label), theme::label()),
        value.into(),
    ])
}

#[allow(clippy::too_many_arguments)]
fn draw_kpi_panel(
    f: &mut Frame,
    entry_count: usize,
    total_words: usize,
    avg_words: usize,
    longest: Option<(&crate::model::JournalEntry, usize)>,
    current_streak: u32,
    max_streak: u32,
    area: Rect,
) {
    let mut lines = vec![Line::from("")];

    lines.push(kpi_row(
        "Total entries",
        Span::styled(entry_count.to_string(), theme::text()),
    ));
    lines.push(kpi_row(
        "Total words",
        Span::styled(total_words.to_string(), theme::text()),
    ));
    lines.push(kpi_row(
        "Avg words/entry",
        Span::styled(avg_words.to_string(), theme::text()),
    ));

    if let Some((entry, count)) = longest {
        let date = entry
            .timestamp
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d")
            .to_string();
        lines.push(kpi_row(
            "Longest entry",
            Span::styled(format!("{} words  ({})", count, date), theme::text()),
        ));
    } else {
        lines.push(kpi_row("Longest entry", Span::styled("—", theme::muted())));
    }

    lines.push(Line::from(""));

    let streak_style = if current_streak > 0 {
        theme::streak()
    } else {
        theme::muted()
    };
    lines.push(kpi_row(
        "Current streak",
        Span::styled(
            format!(
                "{} day{}",
                current_streak,
                if current_streak == 1 { "" } else { "s" }
            ),
            streak_style,
        ),
    ));
    lines.push(kpi_row(
        "Record streak",
        Span::styled(
            format!(
                "{} day{}",
                max_streak,
                if max_streak == 1 { "" } else { "s" }
            ),
            theme::text(),
        ),
    ));

    f.render_widget(
        Paragraph::new(lines)
            .block(theme::panel("Writing Stats"))
            .wrap(Wrap { trim: true }),
        area,
    );
}

// ── Top contacts panel ────────────────────────────────────────────────────────

fn draw_contacts_panel(f: &mut Frame, contacts: &[(&crate::model::Contact, usize)], area: Rect) {
    let mut lines = vec![Line::from("")];

    if contacts.is_empty() {
        lines.push(
            Line::from(Span::styled(
                "No contact mentions found in your journal.",
                theme::muted(),
            ))
            .alignment(ratatui::layout::Alignment::Center),
        );
    } else {
        for (rank, (contact, count)) in contacts.iter().enumerate() {
            let medal = match rank {
                0 => "①",
                1 => "②",
                2 => "③",
                _ => "  ",
            };
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", medal), theme::label()),
                Span::styled(contact.full_name(), theme::text()),
                Span::styled(
                    format!("  {} mention{}", count, if *count == 1 { "" } else { "s" }),
                    theme::muted(),
                ),
            ]));
        }
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(theme::panel("Most Mentioned"))
            .wrap(Wrap { trim: true }),
        area,
    );
}

// ── Word-count bar chart ──────────────────────────────────────────────────────

fn draw_chart(f: &mut Frame, app: &App, area: Rect) {
    let word_data: Vec<(String, u64)> = app
        .filtered_entries()
        .iter()
        .take(7)
        .rev()
        .map(|entry| {
            let date = entry
                .date_for
                .unwrap_or_else(|| entry.timestamp.with_timezone(&chrono::Local).date_naive());
            let label = date.format("%d.%m.").to_string();
            let words = entry.content.split_whitespace().count() as u64;
            (label, words)
        })
        .collect();

    let block = theme::panel("Word Count — Last 7 Entries");

    if word_data.is_empty() {
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled("No entries to display yet.", theme::muted()))
                    .alignment(ratatui::layout::Alignment::Center),
            ])
            .block(block),
            area,
        );
        return;
    }

    let bar_data: Vec<(&str, u64)> = word_data.iter().map(|(s, w)| (s.as_str(), *w)).collect();

    f.render_widget(
        BarChart::default()
            .block(block)
            .data(&bar_data)
            .bar_width(6)
            .bar_gap(2)
            .value_style(theme::text())
            .label_style(theme::muted())
            .bar_style(ratatui::style::Style::default().fg(theme::ACCENT)),
        area,
    );
}

// ── Word cloud ────────────────────────────────────────────────────────────────

fn draw_word_cloud(f: &mut Frame, words: &[(String, usize)], area: Rect) {
    let mut spans: Vec<Span> = Vec::new();

    if words.is_empty() {
        spans.push(Span::styled("Not enough text yet.", theme::muted()));
    } else {
        for (i, (word, count)) in words.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled("  ·  ", theme::dim()));
            }
            let style = match i {
                0..=2 => theme::accent().add_modifier(Modifier::BOLD),
                3..=5 => theme::text().add_modifier(Modifier::BOLD),
                _ => theme::muted(),
            };
            spans.push(Span::styled(format!("{} ({})", word, count), style));
        }
    }

    f.render_widget(
        Paragraph::new(Line::from(spans))
            .block(theme::panel("Frequent Words"))
            .wrap(Wrap { trim: true }),
        area,
    );
}

// ── Calculation helpers ───────────────────────────────────────────────────────

/// Computes (current_streak, all-time_max_streak) from a set of journal entries.
pub(crate) fn calculate_streaks(entries: &[crate::model::JournalEntry]) -> (u32, u32) {
    if entries.is_empty() {
        return (0, 0);
    }

    let mut dates: Vec<NaiveDate> = entries
        .iter()
        .map(|e| {
            e.date_for
                .unwrap_or_else(|| e.timestamp.with_timezone(&chrono::Local).date_naive())
        })
        .collect();
    dates.sort_unstable();
    dates.dedup();

    let mut max_streak = 1u32;
    let mut run = 1u32;

    for pair in dates.windows(2) {
        if pair[1] == pair[0] + chrono::Duration::days(1) {
            run += 1;
            max_streak = max_streak.max(run);
        } else {
            run = 1;
        }
    }

    // Streak is "live" only if the last entry was today or yesterday.
    let today = chrono::Local::now().date_naive();
    let current = if let Some(&last) = dates.last() {
        if last == today || last == today - chrono::Duration::days(1) {
            run
        } else {
            0
        }
    } else {
        0
    };

    (current, max_streak)
}

fn calculate_top_contacts(app: &App) -> Vec<(&crate::model::Contact, usize)> {
    let mut counts: Vec<(&crate::model::Contact, usize)> = app
        .journal
        .contacts
        .iter()
        .filter_map(|c| {
            let tag = c.mention_tag();
            let n: usize = app
                .journal
                .entries
                .iter()
                .map(|e| e.content.matches(&tag).count())
                .sum();
            if n > 0 { Some((c, n)) } else { None }
        })
        .collect();

    counts.sort_by_key(|&(_, n)| std::cmp::Reverse(n));
    counts.truncate(5);
    counts
}

fn calculate_common_words(entries: &[crate::model::JournalEntry]) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for entry in entries {
        for word in entry.content.split(|c: char| !c.is_alphabetic()) {
            if word.len() > 2 {
                let lc = word.to_lowercase();
                if !STOP_WORDS.contains(&lc.as_str()) {
                    *counts.entry(lc).or_insert(0) += 1;
                }
            }
        }
    }

    let mut list: Vec<(String, usize)> = counts.into_iter().collect();
    list.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    list.truncate(10);
    list
}
