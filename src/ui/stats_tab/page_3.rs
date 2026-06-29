//! Page 3: Time of Day Analysis statistics screen.

use crate::app::App;
use crate::ui::theme;
use chrono::{Datelike, Local, Timelike};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{BarChart, Paragraph},
};

/// Renders the hourly activity charts and weekday heatmap.
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_hourly_chart(f, app, chunks[0]);
    draw_day_hour_heatmap(f, app, chunks[1]);
}

fn draw_hourly_chart(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;
    let mut hour_counts = [0u64; 24];
    for entry in entries {
        let hour = entry.timestamp.with_timezone(&Local).hour() as usize;
        if hour < 24 {
            hour_counts[hour] += 1;
        }
    }

    let mut temp_data = Vec::new();
    for (h, &count) in hour_counts.iter().enumerate() {
        let label = format!("{:02}", h);
        temp_data.push((label, count));
    }

    let bar_data: Vec<(&str, u64)> = temp_data.iter().map(|(l, c)| (l.as_str(), *c)).collect();

    let block = theme::panel("Hourly Activity (0:00 - 23:00)");
    let chart = BarChart::default()
        .block(block)
        .data(&bar_data)
        .bar_width(1)
        .bar_gap(1)
        .value_style(theme::text())
        .label_style(theme::muted())
        .bar_style(Style::default().fg(theme::ACCENT));

    f.render_widget(chart, area);
}

fn draw_day_hour_heatmap(f: &mut Frame, app: &App, area: Rect) {
    let entries = &app.journal.entries;

    let mut grid = [[0; 24]; 7];
    for entry in entries {
        let local_dt = entry.timestamp.with_timezone(&Local);
        let wday = local_dt.weekday().num_days_from_monday() as usize;
        let hour = local_dt.hour() as usize;
        if wday < 7 && hour < 24 {
            grid[wday][hour] += 1;
        }
    }

    let weekdays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let mut lines = Vec::new();
    lines.push(Line::from(""));

    let mut header_spans = vec![Span::styled("       ", theme::dim())];
    for h in 0..24 {
        if h % 3 == 0 {
            header_spans.push(Span::styled(format!("{:<3}", h), theme::muted()));
        } else {
            header_spans.push(Span::styled(" ", theme::dim()));
        }
    }
    lines.push(Line::from(header_spans));

    for (wday, &wday_label) in weekdays.iter().enumerate() {
        let mut row_spans = vec![Span::styled(format!("  {}  ", wday_label), theme::muted())];
        for &count in &grid[wday] {
            let (cell_char, style) = match count {
                0 => ("·", theme::dim()),
                1 => ("░", theme::muted()),
                2..=3 => ("▒", theme::label()),
                _ => ("▓", theme::accent()),
            };
            row_spans.push(Span::styled(cell_char, style));
            row_spans.push(Span::styled(" ", theme::dim()));
        }
        lines.push(Line::from(row_spans));
    }

    f.render_widget(
        Paragraph::new(lines).block(theme::panel("Weekday × Hour Activity Grid")),
        area,
    );
}
