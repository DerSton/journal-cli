//! Page 4: Streaks & Consistency statistics screen.

use crate::app::App;
use crate::ui::stats_tab::helpers::{
    compute_streaks, days_in_month, kpi_row, month_abbrev, sorted_unique_dates,
};
use crate::ui::theme;
use chrono::{Datelike, Local, NaiveDate};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

/// Renders the streak tracking, monthly writing consistency, and entry gaps panels.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    draw_streak_overview(f, app, chunks[0]);
    draw_monthly_consistency(f, app, bottom_chunks[0]);
    draw_gaps_analysis(f, app, bottom_chunks[1]);
}

fn draw_streak_overview(f: &mut Frame, app: &App, area: Rect) {
    let dates = sorted_unique_dates(&app.journal.entries);
    let streaks = compute_streaks(&dates);

    let today = Local::now().date_naive();
    let yesterday = today - chrono::Duration::days(1);
    let max_len = streaks.first().map(|s| s.2).unwrap_or(1).max(1);
    let max_bar_width = area.width.saturating_sub(42) as usize;

    let mut lines = vec![Line::from("")];
    for (i, &(start, end, len)) in streaks.iter().take(5).enumerate() {
        let is_live = end == today || end == yesterday;
        let bar_len = ((len as f64 / max_len as f64) * max_bar_width as f64).round() as usize;
        let bar_len = bar_len.max(1);
        let bar = "█".repeat(bar_len);

        let style = if is_live {
            theme::streak()
        } else if i < 3 {
            theme::accent()
        } else {
            theme::text()
        };

        let label = if is_live { " ★ LIVE" } else { "" };
        lines.push(Line::from(vec![
            Span::styled(
                format!(
                    "  {} - {} ",
                    start.format("%Y-%m-%d"),
                    end.format("%Y-%m-%d")
                ),
                theme::muted(),
            ),
            Span::styled(bar, style),
            Span::styled(format!(" ({} days){}", len, label), style),
        ]));
    }

    if streaks.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No streaks recorded yet.",
            theme::muted(),
        )));
    }

    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Streak Overview (Top 5)")),
        area,
    );
}

fn draw_monthly_consistency(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;
    let today = Local::now().date_naive();

    let mut entry_dates = std::collections::HashSet::new();
    for e in entries {
        entry_dates.insert(e.timestamp.with_timezone(&Local).date_naive());
    }

    let mut lines = vec![Line::from("")];
    let max_bar_width = area.width.saturating_sub(22) as usize;

    for i in (0..6).rev() {
        let month_date = today.checked_sub_months(chrono::Months::new(i)).unwrap();
        let year = month_date.year();
        let month = month_date.month();
        let total_days = days_in_month(year, month);

        let mut written_days = 0;
        for day in 1..=total_days {
            if NaiveDate::from_ymd_opt(year, month, day).is_some_and(|d| entry_dates.contains(&d)) {
                written_days += 1;
            }
        }

        let pct = (written_days as f64 / total_days as f64) * 100.0;
        let bar_len =
            ((written_days as f64 / total_days as f64) * max_bar_width as f64).round() as usize;
        let bar = "█".repeat(bar_len);

        let style = if pct < 25.0 {
            theme::danger()
        } else if pct < 50.0 {
            theme::streak()
        } else if pct < 75.0 {
            theme::success()
        } else {
            theme::accent()
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:>3} {:02}: ", month_abbrev(month), year % 100),
                theme::muted(),
            ),
            Span::styled(bar, style),
            Span::styled(format!(" {:>3.0}%", pct), theme::text()),
        ]));
    }

    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Monthly Consistency (Last 6 Months)")),
        area,
    );
}

fn draw_gaps_analysis(f: &mut Frame, app: &App, area: Rect) {
    let dates = sorted_unique_dates(&app.journal.entries);

    let mut longest_gap = 0;
    let mut longest_gap_range = None;
    let mut total_gaps = 0;
    let mut sum_gaps = 0;
    let mut gaps = Vec::new();
    let mut gaps_over_7 = 0;
    let mut gaps_over_30 = 0;

    for pair in dates.windows(2) {
        let diff = (pair[1] - pair[0]).num_days() - 1;
        if diff > 0 {
            gaps.push(diff);
            total_gaps += 1;
            sum_gaps += diff;
            if diff > longest_gap {
                longest_gap = diff;
                longest_gap_range = Some((pair[0], pair[1]));
            }
            if diff > 7 {
                gaps_over_7 += 1;
            }
            if diff > 30 {
                gaps_over_30 += 1;
            }
        }
    }

    let avg_gap = if total_gaps > 0 {
        sum_gaps as f64 / total_gaps as f64
    } else {
        0.0
    };

    gaps.sort();
    let median_gap = if gaps.is_empty() {
        0
    } else if gaps.len() % 2 == 0 {
        (gaps[gaps.len() / 2 - 1] + gaps[gaps.len() / 2]) / 2
    } else {
        gaps[gaps.len() / 2]
    };

    let longest_gap_str = if let Some((start, end)) = longest_gap_range {
        format!(
            "{} days ({} to {})",
            longest_gap,
            start.format("%Y-%m-%d"),
            end.format("%Y-%m-%d")
        )
    } else {
        "0 days".to_string()
    };

    let mut lines = vec![Line::from("")];
    lines.push(kpi_row(
        "Longest Gap",
        Span::styled(longest_gap_str, theme::text()),
    ));
    lines.push(kpi_row(
        "Average Gap",
        Span::styled(format!("{:.1} days", avg_gap), theme::text()),
    ));
    lines.push(kpi_row(
        "Median Gap",
        Span::styled(format!("{} days", median_gap), theme::text()),
    ));
    lines.push(kpi_row(
        "Gaps > 7 Days",
        Span::styled(gaps_over_7.to_string(), theme::text()),
    ));
    lines.push(kpi_row(
        "Gaps > 30 Days",
        Span::styled(gaps_over_30.to_string(), theme::text()),
    ));

    f.render_widget(
        Paragraph::new(lines)
            .block(theme::panel("Gaps Analysis"))
            .wrap(Wrap { trim: false }),
        area,
    );
}
