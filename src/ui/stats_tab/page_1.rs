//! Page 1: Dashboard Overview statistics screen.

use crate::app::App;
use crate::ui::stats_tab::helpers::{compute_streaks, kpi_row, sorted_unique_dates};
use crate::ui::theme;
use chrono::{Datelike, Local, NaiveDate, Timelike};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::collections::HashMap;

/// Renders the overview dashboard page.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(10),
            Constraint::Length(9),
        ])
        .split(area);

    draw_hero_kpis(f, app, chunks[0]);
    draw_heatmap(f, app, chunks[1]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);
    draw_writing_habits(f, app, bottom_chunks[0]);
    draw_time_patterns(f, app, bottom_chunks[1]);
}

fn draw_hero_kpis(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;
    let dates = sorted_unique_dates(entries);
    let streaks = compute_streaks(&dates);

    let total_entries = entries.len();
    let total_words: usize = entries
        .iter()
        .map(|e| e.content.split_whitespace().count())
        .sum();

    let journal_age = if let (Some(first), Some(last)) = (entries.last(), entries.first()) {
        let duration = last.timestamp.signed_duration_since(first.timestamp);
        let days = duration.num_days();
        if days == 0 {
            "1 day".to_string()
        } else if days < 30 {
            format!("{} days", days)
        } else if days < 365 {
            format!("{} months, {} days", days / 30, days % 30)
        } else {
            format!("{} years, {} months", days / 365, (days % 365) / 30)
        }
    } else {
        "0 days".to_string()
    };

    let today = Local::now().date_naive();
    let yesterday = today - chrono::Duration::days(1);
    let current_streak = streaks
        .iter()
        .find(|s| s.1 == today || s.1 == yesterday)
        .map(|s| s.2)
        .unwrap_or(0);
    let max_streak = streaks.first().map(|s| s.2).unwrap_or(0);

    let this_month = Local::now().month();
    let this_year = Local::now().year();
    let entries_this_month = entries
        .iter()
        .filter(|e| {
            let local_date = e.timestamp.with_timezone(&Local);
            local_date.month() == this_month && local_date.year() == this_year
        })
        .count();

    let mut lines = vec![Line::from("")];
    lines.push(kpi_row(
        "Total Entries",
        Span::styled(total_entries.to_string(), theme::text()),
    ));
    lines.push(kpi_row(
        "Total Words Written",
        Span::styled(total_words.to_string(), theme::text()),
    ));
    lines.push(kpi_row(
        "Journal Age",
        Span::styled(journal_age, theme::text()),
    ));
    lines.push(kpi_row(
        "Current Streak 🔥",
        Span::styled(format!("{} days", current_streak), theme::streak()),
    ));
    lines.push(kpi_row(
        "Record Streak 🏆",
        Span::styled(format!("{} days", max_streak), theme::accent()),
    ));
    lines.push(kpi_row(
        "Entries This Month",
        Span::styled(entries_this_month.to_string(), theme::text()),
    ));

    f.render_widget(Paragraph::new(lines).block(theme::panel("Hero KPIs")), area);
}

fn draw_heatmap(f: &mut Frame, app: &App, area: Rect) {
    let today = Local::now().date_naive();
    let start_date = today - chrono::Duration::days(364);

    let mut entry_counts = HashMap::new();
    for entry in &app.journal.entries {
        let d = entry.timestamp.with_timezone(&Local).date_naive();
        *entry_counts.entry(d).or_insert(0) += 1;
    }

    let mut lines = Vec::new();
    lines.push(Line::from(""));

    let mut month_labels = vec![Span::styled("      ", theme::dim())];
    let mut current_month = 0;
    for day_idx in 0..52 {
        let col_date = start_date + chrono::Duration::days(day_idx * 7);
        let m = col_date.month();
        if m != current_month && col_date.day() <= 7 {
            current_month = m;
            let month_str = crate::ui::stats_tab::helpers::month_abbrev(m);
            month_labels.push(Span::styled(format!("{:<4}", month_str), theme::muted()));
        } else if month_labels.len() < (day_idx + 1) as usize {
            month_labels.push(Span::styled(" ", theme::dim()));
        }
    }
    lines.push(Line::from(month_labels));

    let weekday_labels = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    for (wday, &label) in weekday_labels.iter().enumerate() {
        let mut row_spans = vec![Span::styled(format!("  {} ", label), theme::muted())];
        for col in 0..52 {
            let col_date = start_date + chrono::Duration::days(col * 7 + wday as i64);
            if col_date > today {
                row_spans.push(Span::styled(" ", theme::dim()));
                continue;
            }
            let count = entry_counts.get(&col_date).copied().unwrap_or(0);
            let (cell_char, style) = match count {
                0 => ("░", theme::dim()),
                1 => ("▒", theme::muted()),
                2..=3 => ("▓", theme::label()),
                _ => ("█", theme::accent()),
            };
            row_spans.push(Span::styled(cell_char, style));
        }
        lines.push(Line::from(row_spans));
    }

    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Writing Activity Heatmap (Last Year)")),
        area,
    );
}

fn draw_writing_habits(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;
    let total_entries = entries.len();
    if total_entries == 0 {
        f.render_widget(
            Paragraph::new("  No data available.").block(theme::panel("Writing Habits")),
            area,
        );
        return;
    }

    let mut word_counts: Vec<usize> = entries
        .iter()
        .map(|e| e.content.split_whitespace().count())
        .collect();
    word_counts.sort();

    let total_words: usize = word_counts.iter().sum();
    let avg_words = total_words / total_entries;
    let median_words = if total_entries.is_multiple_of(2) {
        (word_counts[total_entries / 2 - 1] + word_counts[total_entries / 2]) / 2
    } else {
        word_counts[total_entries / 2]
    };

    let longest = entries
        .iter()
        .map(|e| (e, e.content.split_whitespace().count()))
        .max_by_key(|&(_, c)| c);
    let shortest = entries
        .iter()
        .map(|e| (e, e.content.split_whitespace().count()))
        .min_by_key(|&(_, c)| c);

    let total_sentences: usize = entries
        .iter()
        .map(|e| e.content.split(['.', '!', '?']).count() - 1)
        .sum();

    let mut lines = vec![Line::from("")];
    lines.push(kpi_row(
        "Avg Words/Entry",
        Span::styled(avg_words.to_string(), theme::text()),
    ));
    lines.push(kpi_row(
        "Median Words/Entry",
        Span::styled(median_words.to_string(), theme::text()),
    ));
    if let Some((e, c)) = longest {
        lines.push(kpi_row(
            "Longest Entry",
            Span::styled(
                format!(
                    "{} w ({})",
                    c,
                    e.timestamp.with_timezone(&Local).format("%Y-%m-%d")
                ),
                theme::text(),
            ),
        ));
    }
    if let Some((e, c)) = shortest {
        lines.push(kpi_row(
            "Shortest Entry",
            Span::styled(
                format!(
                    "{} w ({})",
                    c,
                    e.timestamp.with_timezone(&Local).format("%Y-%m-%d")
                ),
                theme::text(),
            ),
        ));
    }
    lines.push(kpi_row(
        "Total Sentences",
        Span::styled(total_sentences.to_string(), theme::text()),
    ));

    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Writing Habits")),
        area,
    );
}

fn draw_time_patterns(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;
    if entries.is_empty() {
        f.render_widget(
            Paragraph::new("  No data available.").block(theme::panel("Time Patterns")),
            area,
        );
        return;
    }

    let mut weekday_counts = [0; 7];
    let mut hour_counts = [0; 24];
    let mut date_words: HashMap<NaiveDate, usize> = HashMap::new();

    for entry in entries {
        let local_dt = entry.timestamp.with_timezone(&Local);
        let wday = local_dt.weekday().num_days_from_monday() as usize;
        let hour = local_dt.hour() as usize;

        if wday < 7 {
            weekday_counts[wday] += 1;
        }
        if hour < 24 {
            hour_counts[hour] += 1;
        }

        let date = local_dt.date_naive();
        let words = entry.content.split_whitespace().count();
        *date_words.entry(date).or_insert(0) += words;
    }

    let weekdays = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ];
    let max_wday_idx = weekday_counts
        .iter()
        .enumerate()
        .max_by_key(|&(_, c)| c)
        .map(|(i, _)| i)
        .unwrap_or(0);
    let max_hour = hour_counts
        .iter()
        .enumerate()
        .max_by_key(|&(_, c)| c)
        .map(|(i, _)| i)
        .unwrap_or(0);
    let max_productive_day = date_words.iter().max_by_key(|&(_, &w)| w);

    let mut lines = vec![Line::from("")];
    lines.push(kpi_row(
        "Most Active Weekday",
        Span::styled(weekdays[max_wday_idx], theme::text()),
    ));
    lines.push(kpi_row(
        "Most Active Hour",
        Span::styled(format!("{:02}:00", max_hour), theme::text()),
    ));
    if let Some((d, w)) = max_productive_day {
        lines.push(kpi_row(
            "Most Productive Day",
            Span::styled(
                format!("{} ({} words)", d.format("%Y-%m-%d"), w),
                theme::text(),
            ),
        ));
    }

    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Time Patterns")),
        area,
    );
}
