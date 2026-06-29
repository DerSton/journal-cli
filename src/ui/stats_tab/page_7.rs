//! Page 7: Entry Length Analysis statistics screen.

use crate::app::App;
use crate::ui::stats_tab::helpers::{kpi_row, month_abbrev};
use crate::ui::theme;
use chrono::{Datelike, Local};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{BarChart, Paragraph, Wrap},
};
use std::collections::HashMap;

/// Renders the entry length distribution, monthly average length, and extremes records.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[1]);

    draw_length_distribution(f, app, chunks[0]);
    draw_monthly_avg_length(f, app, bottom_chunks[0]);
    draw_records_extremes(f, app, bottom_chunks[1]);
}

fn draw_length_distribution(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;
    let mut buckets = [0u64; 5];

    for entry in entries {
        let words = entry.content.split_whitespace().count();
        match words {
            0..=50 => buckets[0] += 1,
            51..=100 => buckets[1] += 1,
            101..=200 => buckets[2] += 1,
            201..=500 => buckets[3] += 1,
            _ => buckets[4] += 1,
        }
    }

    let temp_data = [
        ("0-50".to_string(), buckets[0]),
        ("51-100".to_string(), buckets[1]),
        ("101-200".to_string(), buckets[2]),
        ("201-500".to_string(), buckets[3]),
        ("500+".to_string(), buckets[4]),
    ];

    let bar_data: Vec<(&str, u64)> = temp_data.iter().map(|(l, c)| (l.as_str(), *c)).collect();

    let block = theme::panel("Entry Length Distribution (Word Count Buckets)");
    let bar_width =
        (((area.width.saturating_sub(6) as usize) / 5).saturating_sub(2)).clamp(2, 8) as u16;

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

fn draw_monthly_avg_length(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;
    let today = Local::now().date_naive();

    let mut monthly_totals = HashMap::new();
    let mut monthly_counts = HashMap::new();

    for entry in entries {
        let local_dt = entry.timestamp.with_timezone(&Local);
        let year = local_dt.year();
        let month = local_dt.month();
        let words = entry.content.split_whitespace().count();

        *monthly_totals.entry((year, month)).or_insert(0) += words;
        *monthly_counts.entry((year, month)).or_insert(0) += 1;
    }

    let mut temp_data = Vec::new();
    for i in (0..12).rev() {
        let date = today.checked_sub_months(chrono::Months::new(i)).unwrap();
        let year = date.year();
        let month = date.month();

        let total = monthly_totals.get(&(year, month)).copied().unwrap_or(0);
        let count = monthly_counts.get(&(year, month)).copied().unwrap_or(0);
        let avg = if count > 0 {
            total as u64 / count as u64
        } else {
            0
        };

        let label = format!("{}{}", month_abbrev(month), year % 100);
        temp_data.push((label, avg));
    }

    let bar_data: Vec<(&str, u64)> = temp_data.iter().map(|(l, c)| (l.as_str(), *c)).collect();

    let block = theme::panel("Average Entry Length per Month");
    let bar_width =
        (((area.width.saturating_sub(4) as usize) / 12).saturating_sub(1)).clamp(1, 4) as u16;

    let chart = BarChart::default()
        .block(block)
        .data(&bar_data)
        .bar_width(bar_width)
        .bar_gap(1)
        .value_style(theme::text())
        .label_style(theme::muted())
        .bar_style(Style::default().fg(theme::LABEL));

    f.render_widget(chart, area);
}

fn draw_records_extremes(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;

    let mut longest_entry = None;
    let mut shortest_entry = None;
    let mut entries_per_day = HashMap::new();
    let mut words_per_day = HashMap::new();
    let mut most_mentions = None;

    for entry in entries {
        let local_dt = entry.timestamp.with_timezone(&Local);
        let date = local_dt.date_naive();
        let words = entry.content.split_whitespace().count();

        if longest_entry.map(|(_, w)| words > w).unwrap_or(true) {
            longest_entry = Some((entry, words));
        }
        if shortest_entry.map(|(_, w)| words < w).unwrap_or(true) {
            shortest_entry = Some((entry, words));
        }

        *entries_per_day.entry(date).or_insert(0) += 1;
        *words_per_day.entry(date).or_insert(0) += words;

        let mentions = entry.content.matches("{{person|").count();
        if most_mentions.map(|(_, m)| mentions > m).unwrap_or(true) {
            most_mentions = Some((entry, mentions));
        }
    }

    let max_entries_day = entries_per_day.iter().max_by_key(|&(_, &c)| c);
    let max_words_day = words_per_day.iter().max_by_key(|&(_, &w)| w);

    let mut lines = vec![Line::from("")];

    if let Some((e, w)) = longest_entry {
        lines.push(kpi_row(
            "Longest Entry",
            Span::styled(
                format!(
                    "{} w ({})",
                    w,
                    e.timestamp.with_timezone(&Local).format("%Y-%m-%d")
                ),
                theme::text(),
            ),
        ));
    } else {
        lines.push(kpi_row(
            "Longest Entry",
            Span::styled("N/A", theme::muted()),
        ));
    }

    if let Some((e, w)) = shortest_entry {
        lines.push(kpi_row(
            "Shortest Entry",
            Span::styled(
                format!(
                    "{} w ({})",
                    w,
                    e.timestamp.with_timezone(&Local).format("%Y-%m-%d")
                ),
                theme::text(),
            ),
        ));
    } else {
        lines.push(kpi_row(
            "Shortest Entry",
            Span::styled("N/A", theme::muted()),
        ));
    }

    if let Some((d, c)) = max_entries_day {
        lines.push(kpi_row(
            "Most Entries/Day",
            Span::styled(
                format!("{} entries ({})", c, d.format("%Y-%m-%d")),
                theme::text(),
            ),
        ));
    } else {
        lines.push(kpi_row(
            "Most Entries/Day",
            Span::styled("N/A", theme::muted()),
        ));
    }

    if let Some((d, w)) = max_words_day {
        lines.push(kpi_row(
            "Most Words/Day",
            Span::styled(
                format!("{} words ({})", w, d.format("%Y-%m-%d")),
                theme::text(),
            ),
        ));
    } else {
        lines.push(kpi_row(
            "Most Words/Day",
            Span::styled("N/A", theme::muted()),
        ));
    }

    if let Some((e, m)) = most_mentions {
        lines.push(kpi_row(
            "Most Mentions/Entry",
            Span::styled(
                format!(
                    "{} mentions ({})",
                    m,
                    e.timestamp.with_timezone(&Local).format("%Y-%m-%d")
                ),
                theme::text(),
            ),
        ));
    } else {
        lines.push(kpi_row(
            "Most Mentions/Entry",
            Span::styled("N/A", theme::muted()),
        ));
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(theme::panel("Records & Extremes"))
            .wrap(Wrap { trim: false }),
        area,
    );
}
