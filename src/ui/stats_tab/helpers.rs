//! Common helpers and utilities for stats panels.

use crate::app::App;
use crate::model::{Contact, JournalEntry};
use crate::ui::theme;
use chrono::NaiveDate;
use ratatui::text::{Line, Span};
use std::collections::HashMap;

/// List of common German and English stop words to filter out during word frequency analysis.
pub const STOP_WORDS: &[&str] = &[
    "ich", "du", "er", "sie", "es", "wir", "ihr", "mein", "dein", "sein", "unser", "ein", "eine",
    "einer", "eines", "einem", "einen", "der", "die", "das", "den", "dem", "des", "und", "oder",
    "aber", "so", "ja", "nein", "ist", "sind", "war", "waren", "mit", "von", "zu", "in", "auf",
    "im", "am", "für", "um", "als", "wie", "dass", "mir", "mich", "dir", "dich", "uns", "euch",
    "sich", "nicht", "nur", "auch", "noch", "schon", "jetzt", "dann", "da", "hier", "heute",
    "morgen", "gestern", "mal", "habe", "hat", "haben", "hatte", "wurde", "werden", "i", "you",
    "he", "she", "it", "we", "they", "my", "your", "his", "her", "its", "our", "their", "a", "an",
    "the", "and", "or", "but", "so", "yes", "no", "is", "are", "was", "were", "with", "from", "to",
    "in", "on", "at", "for", "of", "about", "as", "like", "that", "me", "him", "us", "them", "not",
    "only", "also", "now", "then", "there", "here", "today", "have", "has", "had", "been", "would",
    "could", "should", "will", "can", "this", "these", "those", "wenn", "weil", "denn", "nach",
    "vor", "über", "unter", "durch", "gegen", "ohne", "beim", "zum", "zur", "vom", "ins", "ans",
];

/// Helper function to render a uniform KPI row in lists.
pub fn kpi_row<'a>(label: &'a str, value: impl Into<Span<'a>>) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("  {:<24}", label), theme::label()),
        value.into(),
    ])
}

/// Extracts a sorted, de-duplicated list of local dates when entries were written.
pub fn sorted_unique_dates(entries: &[JournalEntry]) -> Vec<NaiveDate> {
    let mut dates: Vec<NaiveDate> = entries
        .iter()
        .map(|e| e.timestamp.with_timezone(&chrono::Local).date_naive())
        .collect();
    dates.sort();
    dates.dedup();
    dates
}

/// Computes all consecutive writing streaks, returning a list of (start, end, duration) tuples.
/// The list is sorted by streak duration in descending order.
pub fn compute_streaks(dates: &[NaiveDate]) -> Vec<(NaiveDate, NaiveDate, u32)> {
    if dates.is_empty() {
        return Vec::new();
    }
    let mut streaks: Vec<(NaiveDate, NaiveDate, u32)> = Vec::new();
    let mut start = dates[0];
    let mut end = dates[0];
    for &d in &dates[1..] {
        if d == end + chrono::Duration::days(1) {
            end = d;
        } else {
            let len = (end - start).num_days() as u32 + 1;
            streaks.push((start, end, len));
            start = d;
            end = d;
        }
    }
    let len = (end - start).num_days() as u32 + 1;
    streaks.push((start, end, len));
    streaks.sort_by(|a, b| b.2.cmp(&a.2));
    streaks
}

/// Calculates the number of days in a specific month of a year.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap()
    .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
    .num_days() as u32
}

/// Returns a three-letter abbreviation for the given month index.
pub fn month_abbrev(month: u32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    }
}

/// Performs frequency analysis on all words in the journal entries, filtering out stop words.
pub fn word_frequencies(entries: &[JournalEntry]) -> Vec<(String, usize)> {
    let stop: std::collections::HashSet<&str> = STOP_WORDS.iter().copied().collect();
    let mut freq: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        for word in entry
            .content
            .split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '-')
        {
            let w = word.to_lowercase();
            if w.len() > 2 && !stop.contains(w.as_str()) {
                *freq.entry(w).or_insert(0) += 1;
            }
        }
    }
    let mut sorted: Vec<(String, usize)> = freq.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    sorted
}

/// Counts how many times each contact has been mentioned across all journal entries.
pub fn calculate_top_contacts(app: &App) -> Vec<(&Contact, usize)> {
    let mut counts: Vec<(&Contact, usize)> = app
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

    counts.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| a.0.last_name.cmp(&b.0.last_name))
    });
    counts
}
