//! Page 5: Word Analysis & Language statistics screen.

use crate::app::App;
use crate::ui::stats_tab::helpers::{clean_content, kpi_row, word_frequencies};
use crate::ui::theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};
use std::collections::HashSet;

/// Renders the top word list, word cloud, and vocabulary statistics.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);

    draw_top_words(f, app, chunks[0]);
    draw_word_cloud(f, app, right_chunks[0]);
    draw_vocabulary_richness(f, app, right_chunks[1]);
}

fn draw_top_words(f: &mut Frame, app: &App, area: Rect) {
    let freqs = word_frequencies(&app.journal.entries);
    let limit = 20;
    let max_count = freqs.first().map(|f| f.1).unwrap_or(1).max(1);
    let max_bar_width = area.width.saturating_sub(26) as usize;

    let mut lines = vec![Line::from("")];
    for (i, (word, count)) in freqs.iter().take(limit).enumerate() {
        let bar_len = ((*count as f64 / max_count as f64) * max_bar_width as f64).round() as usize;
        let bar_len = bar_len.max(1);
        let bar = "█".repeat(bar_len);

        let style = match i {
            0..=2 => theme::accent(),
            3..=9 => theme::text(),
            _ => theme::muted(),
        };

        lines.push(Line::from(vec![
            Span::styled(format!("  {:>15}: ", word), theme::muted()),
            Span::styled(bar, style),
            Span::styled(format!(" ({})", count), theme::text()),
        ]));
    }

    if freqs.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No text data yet.",
            theme::muted(),
        )));
    }

    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Most Used Words (Top 20)")),
        area,
    );
}

fn draw_word_cloud(f: &mut Frame, app: &App, area: Rect) {
    let freqs = word_frequencies(&app.journal.entries);
    let limit = 30;

    let mut spans = Vec::new();
    spans.push(Span::styled("  ", theme::dim()));

    for (i, (word, count)) in freqs.iter().take(limit).enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ·  ", theme::dim()));
        }
        let style = match i {
            0..=4 => theme::accent().add_modifier(Modifier::BOLD),
            5..=14 => theme::text().add_modifier(Modifier::BOLD),
            _ => theme::muted(),
        };
        spans.push(Span::styled(format!("{} ({})", word, count), style));
    }

    if freqs.is_empty() {
        spans.push(Span::styled("No text data yet.", theme::muted()));
    }

    f.render_widget(
        Paragraph::new(Line::from(spans))
            .block(theme::panel("Frequent Word Cloud"))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_vocabulary_richness(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;

    let mut total_words = 0;
    let mut unique_words = HashSet::new();
    let mut word_lengths_sum = 0;
    let mut longest_word = String::new();
    let mut hapax_count = 0;

    let mut word_counts = std::collections::HashMap::new();

    for entry in entries {
        let cleaned = clean_content(&entry.content);
        for word in cleaned.split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '-') {
            let w = word.to_lowercase();
            if !w.is_empty() {
                total_words += 1;
                unique_words.insert(w.clone());
                word_lengths_sum += w.len();
                if w.len() > longest_word.len() {
                    longest_word = w.clone();
                }
                *word_counts.entry(w).or_insert(0) += 1;
            }
        }
    }

    for &count in word_counts.values() {
        if count == 1 {
            hapax_count += 1;
        }
    }

    let ttr = if total_words > 0 {
        (unique_words.len() as f64 / total_words as f64) * 100.0
    } else {
        0.0
    };

    let avg_word_len = if total_words > 0 {
        word_lengths_sum as f64 / total_words as f64
    } else {
        0.0
    };

    let mut total_sentences = 0;
    for entry in entries {
        total_sentences += entry.content.split(['.', '!', '?']).count() - 1;
    }
    let avg_sentence_len = if total_sentences > 0 {
        total_words as f64 / total_sentences as f64
    } else {
        0.0
    };

    let mut lines = vec![Line::from("")];
    lines.push(kpi_row(
        "Total Words",
        Span::styled(total_words.to_string(), theme::text()),
    ));
    lines.push(kpi_row(
        "Unique Words",
        Span::styled(unique_words.len().to_string(), theme::text()),
    ));
    lines.push(kpi_row(
        "Type-Token Ratio (TTR)",
        Span::styled(format!("{:.1}%", ttr), theme::accent()),
    ));
    lines.push(kpi_row(
        "Hapax Legomena (Used 1x)",
        Span::styled(hapax_count.to_string(), theme::text()),
    ));
    lines.push(kpi_row(
        "Average Word Length",
        Span::styled(format!("{:.1} chars", avg_word_len), theme::text()),
    ));
    lines.push(kpi_row(
        "Average Sentence Length",
        Span::styled(format!("{:.1} words", avg_sentence_len), theme::text()),
    ));
    lines.push(kpi_row(
        "Longest Word Used",
        Span::styled(longest_word, theme::text()),
    ));

    f.render_widget(
        Paragraph::new(lines)
            .block(theme::panel("Vocabulary Richness"))
            .wrap(Wrap { trim: false }),
        area,
    );
}
