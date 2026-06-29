//! Page 2: Writing Over Time statistics screen.

use crate::app::App;
use crate::ui::stats_tab::helpers::month_abbrev;
use crate::ui::theme;
use chrono::{Datelike, Local};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{BarChart, Paragraph},
};
use std::collections::HashMap;

/// Renders the writing activity trend charts.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_word_trend(f, app, chunks[0]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[1]);
    draw_monthly_activity(f, app, bottom_chunks[0]);
    draw_weekday_activity(f, app, bottom_chunks[1]);
}

fn draw_word_trend(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;
    let limit = 14;
    let trend_entries: Vec<_> = entries.iter().take(limit).rev().collect();

    let mut temp_data = Vec::new();
    for entry in &trend_entries {
        let label = entry
            .timestamp
            .with_timezone(&Local)
            .format("%d.%m.")
            .to_string();
        let words = entry.content.split_whitespace().count() as u64;
        temp_data.push((label, words));
    }

    let bar_data: Vec<(&str, u64)> = temp_data.iter().map(|(l, w)| (l.as_str(), *w)).collect();

    let block = theme::panel("Word Count — Last 14 Entries");
    if bar_data.is_empty() {
        f.render_widget(
            Paragraph::new("  No entries to display.").block(block),
            area,
        );
        return;
    }

    let bar_width =
        (((area.width.saturating_sub(4) as usize) / limit).saturating_sub(1)).clamp(1, 4) as u16;

    let chart = BarChart::default()
        .block(block)
        .data(&bar_data)
        .bar_width(bar_width)
        .bar_gap(1)
        .value_style(theme::text())
        .label_style(theme::muted())
        .bar_style(Style::default().fg(theme::ACCENT));

    f.render_widget(chart, area);
}

fn draw_monthly_activity(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;
    let today = Local::now().date_naive();

    let mut monthly_counts = HashMap::new();
    for entry in entries {
        let local_dt = entry.timestamp.with_timezone(&Local);
        let year = local_dt.year();
        let month = local_dt.month();
        *monthly_counts.entry((year, month)).or_insert(0) += 1;
    }

    let mut temp_data = Vec::new();
    for i in (0..12).rev() {
        let date = today.checked_sub_months(chrono::Months::new(i)).unwrap();
        let year = date.year();
        let month = date.month();
        let count = monthly_counts.get(&(year, month)).copied().unwrap_or(0);
        let label = format!("{}{}", month_abbrev(month), year % 100);
        temp_data.push((label, count as u64));
    }

    let bar_data: Vec<(&str, u64)> = temp_data.iter().map(|(l, c)| (l.as_str(), *c)).collect();

    let block = theme::panel("Entries per Month (Last Year)");
    if bar_data.is_empty() {
        f.render_widget(
            Paragraph::new("  No entries to display.").block(block),
            area,
        );
        return;
    }

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

fn draw_weekday_activity(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;
    let mut weekday_counts = [0; 7];
    for entry in entries {
        let wday = entry
            .timestamp
            .with_timezone(&Local)
            .weekday()
            .num_days_from_monday() as usize;
        if wday < 7 {
            weekday_counts[wday] += 1;
        }
    }

    let weekdays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let max_count = weekday_counts.iter().copied().max().unwrap_or(1).max(1);
    let max_bar_width = area.width.saturating_sub(18) as usize;

    let mut lines = vec![Line::from("")];
    for i in 0..7 {
        let count = weekday_counts[i];
        let bar_len = ((count as f64 / max_count as f64) * max_bar_width as f64).round() as usize;
        let bar = "█".repeat(bar_len);
        lines.push(Line::from(vec![
            Span::styled(format!("  {:>3}: ", weekdays[i]), theme::muted()),
            Span::styled(bar, theme::accent()),
            Span::styled(format!(" ({})", count), theme::text()),
        ]));
    }

    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Entries by Weekday")),
        area,
    );
}
