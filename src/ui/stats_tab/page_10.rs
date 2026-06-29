//! Page 10: Year in Review statistics screen.

use crate::app::App;
use crate::ui::stats_tab::helpers::compute_streaks;
use crate::ui::theme;
use chrono::{Datelike, Local, NaiveDate};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};
use std::collections::HashMap;

/// Renders the year-over-year performance summary and the multi-year monthly comparison heatmap.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    draw_yoy_table(f, app, chunks[0]);
    draw_monthly_cross_year(f, app, chunks[1]);
}

fn draw_yoy_table(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;

    let mut year_entries: HashMap<i32, Vec<_>> = HashMap::new();
    for entry in entries {
        let y = entry.timestamp.with_timezone(&Local).year();
        year_entries.entry(y).or_default().push(entry);
    }

    let mut years: Vec<_> = year_entries.keys().copied().collect();
    years.sort_by(|a, b| b.cmp(a));

    let mut lines = vec![Line::from("")];

    lines.push(Line::from(vec![
        Span::styled("  Year ", theme::label()),
        Span::styled("  Entries ", theme::label()),
        Span::styled("   Words     ", theme::label()),
        Span::styled("  Avg Words   ", theme::label()),
        Span::styled("  Active Days  ", theme::label()),
        Span::styled("  Best Streak     ", theme::label()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  ─────", theme::dim()),
        Span::styled("  ────────", theme::dim()),
        Span::styled("   ──────────", theme::dim()),
        Span::styled("  ────────────", theme::dim()),
        Span::styled("  ─────────────", theme::dim()),
        Span::styled("  ───────────────", theme::dim()),
    ]));

    for y in years {
        let y_entries = &year_entries[&y];
        let total_entries = y_entries.len();
        let total_words: usize = y_entries
            .iter()
            .map(|e| e.content.split_whitespace().count())
            .sum();
        let avg_words = if total_entries > 0 {
            total_words / total_entries
        } else {
            0
        };

        let mut y_dates: Vec<NaiveDate> = y_entries
            .iter()
            .map(|e| e.timestamp.with_timezone(&Local).date_naive())
            .collect();
        y_dates.sort();
        y_dates.dedup();
        let active_days = y_dates.len();
        let y_streaks = compute_streaks(&y_dates);
        let best_streak = y_streaks.first().map(|s| s.2).unwrap_or(0);

        lines.push(Line::from(vec![
            Span::styled(format!("  {:<5}", y), theme::accent()),
            Span::styled(format!("  {:>7} ", total_entries), theme::text()),
            Span::styled(format!("   {:>9} ", total_words), theme::text()),
            Span::styled(format!("  {:>11} ", avg_words), theme::text()),
            Span::styled(format!("  {:>11}  ", active_days), theme::text()),
            Span::styled(format!("  {:>11} days", best_streak), theme::streak()),
        ]));
    }

    if year_entries.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No data available.",
            theme::muted(),
        )));
    }

    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Year-over-Year Performance")),
        area,
    );
}

fn draw_monthly_cross_year(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;

    let mut monthly_activity = HashMap::new();
    for entry in entries {
        let local_dt = entry.timestamp.with_timezone(&Local);
        let year = local_dt.year();
        let month = local_dt.month();
        *monthly_activity.entry((year, month)).or_insert(0) += 1;
    }

    let years: std::collections::BTreeSet<i32> = entries
        .iter()
        .map(|e| e.timestamp.with_timezone(&Local).year())
        .collect();
    let years_vec: Vec<_> = years.iter().copied().rev().collect();

    let mut lines = vec![Line::from("")];

    lines.push(Line::from(vec![
        Span::styled("  Year   ", theme::dim()),
        Span::styled(" J  F  M  A  M  J  J  A  S  O  N  D", theme::muted()),
    ]));

    for y in years_vec {
        let mut row_spans = vec![Span::styled(format!("  {:<5}  ", y), theme::accent())];
        for m in 1..=12 {
            let count = monthly_activity.get(&(y, m)).copied().unwrap_or(0);
            let (cell, style) = match count {
                0 => ("·", theme::dim()),
                1..=5 => ("░", theme::muted()),
                6..=15 => ("▒", theme::label()),
                _ => ("▓", theme::accent()),
            };
            row_spans.push(Span::styled(format!(" {}", cell), style));
            row_spans.push(Span::styled(" ", theme::dim()));
        }
        lines.push(Line::from(row_spans));
    }

    if years.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No data available.",
            theme::muted(),
        )));
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(theme::panel("Monthly Activity Comparison (Jan - Dec)"))
            .wrap(Wrap { trim: false }),
        area,
    );
}
