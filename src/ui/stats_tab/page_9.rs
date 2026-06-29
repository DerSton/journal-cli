//! Page 9: Sentiment & Content Patterns statistics screen.

use crate::app::App;
use crate::ui::stats_tab::helpers::{STOP_WORDS, kpi_row};
use crate::ui::theme;
use chrono::Local;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};
use std::collections::{HashMap, HashSet};

/// Renders the question mark density, sentence punctuation analysis, and top recurring bigram patterns.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    draw_question_density(f, app, top_chunks[0]);
    draw_punctuation_analysis(f, app, top_chunks[1]);
    draw_bigram_analysis(f, app, chunks[1]);
}

fn draw_question_density(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;
    let total_entries = entries.len();
    if total_entries == 0 {
        f.render_widget(
            Paragraph::new("  No entries.").block(theme::panel("Question Density")),
            area,
        );
        return;
    }

    let mut total_questions = 0;
    let mut max_questions = 0;
    let mut max_questions_entry = None;

    for entry in entries {
        let q_count = entry.content.matches('?').count();
        total_questions += q_count;
        if q_count > max_questions {
            max_questions = q_count;
            max_questions_entry = Some(entry);
        }
    }

    let avg_questions = total_questions as f64 / total_entries as f64;

    let mut lines = vec![Line::from("")];
    lines.push(kpi_row(
        "Total Question Marks",
        Span::styled(total_questions.to_string(), theme::text()),
    ));
    lines.push(kpi_row(
        "Average per Entry",
        Span::styled(format!("{:.2}", avg_questions), theme::text()),
    ));
    if let Some(e) = max_questions_entry {
        lines.push(kpi_row(
            "Most Questions/Entry",
            Span::styled(
                format!(
                    "{} (on {})",
                    max_questions,
                    e.timestamp.with_timezone(&Local).format("%Y-%m-%d")
                ),
                theme::text(),
            ),
        ));
    } else {
        lines.push(kpi_row(
            "Most Questions/Entry",
            Span::styled("0", theme::muted()),
        ));
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(theme::panel("Question Density"))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_punctuation_analysis(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;
    let mut dots = 0;
    let mut excls = 0;
    let mut quests = 0;
    let mut ellipses = 0;

    for entry in entries {
        dots += entry.content.matches('.').count();
        excls += entry.content.matches('!').count();
        quests += entry.content.matches('?').count();
        ellipses += entry.content.matches("...").count();
    }

    dots = dots.saturating_sub(ellipses * 3);

    let emotionality_score = if excls + dots > 0 {
        (excls as f64 / (excls + dots) as f64) * 100.0
    } else {
        0.0
    };

    let mut lines = vec![Line::from("")];
    lines.push(kpi_row(
        "Periods (Sentence Ends)",
        Span::styled(dots.to_string(), theme::text()),
    ));
    lines.push(kpi_row(
        "Exclamation Marks (!)",
        Span::styled(excls.to_string(), theme::text()),
    ));
    lines.push(kpi_row(
        "Question Marks (?)",
        Span::styled(quests.to_string(), theme::text()),
    ));
    lines.push(kpi_row(
        "Ellipses (...)",
        Span::styled(ellipses.to_string(), theme::text()),
    ));
    lines.push(kpi_row(
        "Emotionality Score",
        Span::styled(
            format!("{:.1}% (ratio of ! to .)", emotionality_score),
            theme::accent(),
        ),
    ));

    f.render_widget(
        Paragraph::new(lines)
            .block(theme::panel("Punctuation Analysis"))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_bigram_analysis(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;
    let stop: HashSet<&str> = STOP_WORDS.iter().copied().collect();

    let mut bigrams = HashMap::new();

    for entry in entries {
        let tokens: Vec<String> = entry
            .content
            .split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '-')
            .map(|t| t.to_lowercase())
            .filter(|t| t.len() > 2 && !stop.contains(t.as_str()))
            .collect();

        for window in tokens.windows(2) {
            let bigram = format!("{} {}", window[0], window[1]);
            *bigrams.entry(bigram).or_insert(0) += 1;
        }
    }

    let mut bigram_list: Vec<_> = bigrams.into_iter().collect();
    bigram_list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let mut lines = vec![Line::from("")];
    let limit = 15;

    for (i, (bigram, count)) in bigram_list.iter().take(limit).enumerate() {
        let rank_medal = match i {
            0 => "① ",
            1 => "② ",
            2 => "③ ",
            _ => "   ",
        };
        let style = if i < 3 {
            theme::accent()
        } else {
            theme::text()
        };
        lines.push(Line::from(vec![
            Span::styled(rank_medal, theme::label()),
            Span::styled(format!("  {:<26}", bigram), style),
            Span::styled(format!("(used {} times)", count), theme::muted()),
        ]));
    }

    if bigram_list.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Not enough text to extract bigrams.",
            theme::muted(),
        )));
    }

    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Top Recurring Themes (Bigrams)")),
        area,
    );
}
