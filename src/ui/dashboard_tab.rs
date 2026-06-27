use crate::app::App;
use crate::ui::stats_tab::calculate_streaks;
use crate::ui::theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let entries = &app.journal.entries;

    // Calculate streaks
    let (current_streak, max_streak) = calculate_streaks(entries);

    // Calculate total words
    let total_words: usize = entries
        .iter()
        .map(|e| e.content.split_whitespace().count())
        .sum();

    // Last entry date
    let last_entry_date = entries
        .first()
        .map(|e| {
            e.timestamp
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d")
                .to_string()
        })
        .unwrap_or_else(|| "N/A".to_string());

    // Split layout horizontally: Left (Simple Stats), Right (Ollama Summary)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left Column: Simple Stats
    let stats_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" Total Entries:  ", theme::title_style()),
            Span::styled(entries.len().to_string(), theme::text_style()),
        ]),
        Line::from(vec![
            Span::styled(" Total Words:    ", theme::title_style()),
            Span::styled(total_words.to_string(), theme::text_style()),
        ]),
        Line::from(vec![
            Span::styled(" Last Entry:     ", theme::title_style()),
            Span::styled(last_entry_date, theme::text_style()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Current Streak: ", theme::title_style()),
            Span::styled(
                format!("{} day(s) 🔥", current_streak),
                if current_streak > 0 {
                    theme::success_style()
                } else {
                    theme::text_style()
                },
            ),
        ]),
        Line::from(vec![
            Span::styled(" Record Streak:  ", theme::title_style()),
            Span::styled(format!("{} day(s) 🏆", max_streak), theme::text_style()),
        ]),
    ];

    let stats_p = Paragraph::new(stats_lines)
        .block(theme::panel_block("Dashboard Stats"))
        .wrap(Wrap { trim: true });
    f.render_widget(stats_p, chunks[0]);

    // Right Column: Ollama Summary
    let mut summary_lines = vec![Line::from("")];

    if !app.journal.settings.ollama_enabled {
        summary_lines.push(Line::from(Span::styled(
            " Ollama summaries are currently disabled.",
            theme::muted_style(),
        )));
        summary_lines.push(Line::from(""));
        summary_lines.push(Line::from(Span::styled(
            " Enable Ollama in Settings [5] and select a model.",
            theme::muted_style(),
        )));
    } else if app.ollama_in_progress {
        summary_lines.push(Line::from(Span::styled(
            " Connecting to Ollama & generating summary... Please wait.",
            theme::title_style(),
        )));
    } else if let Some(ref error) = app.ollama_error {
        summary_lines.push(Line::from(Span::styled(
            format!(" Error: {}", error),
            theme::danger_style(),
        )));
        summary_lines.push(Line::from(""));
        summary_lines.push(Line::from(Span::styled(
            " Make sure Ollama is running locally and the model is pulled.",
            theme::muted_style(),
        )));
        summary_lines.push(Line::from(""));
        summary_lines.push(Line::from(Span::styled(
            " Press 'r' to retry.",
            theme::muted_style(),
        )));
    } else if let Some(ref summary) = app.ollama_summary {
        // Render the summary content split by newlines
        for line in summary.lines() {
            summary_lines.push(Line::from(Span::styled(
                line.to_string(),
                theme::text_style(),
            )));
        }
    } else {
        summary_lines.push(Line::from(Span::styled(
            " No summary cached. Press 'r' to generate.",
            theme::muted_style(),
        )));
    }

    let summary_title = format!("Ollama Summary ({})", app.journal.settings.ollama_model);

    // Calculate wrapped lines to configure scrolling bounds and scrollbar state
    let inner_width = chunks[1].width.saturating_sub(2) as usize;
    let mut total_lines = 0;
    for line in &summary_lines {
        let len = line.width();
        if len == 0 {
            total_lines += 1;
        } else {
            total_lines += len.div_ceil(inner_width);
        }
    }

    let visible_height = chunks[1].height.saturating_sub(2) as usize;
    let max_scroll = total_lines.saturating_sub(visible_height) as u16;
    if app.detail_scroll > max_scroll {
        app.detail_scroll = max_scroll;
    }

    let summary_p = Paragraph::new(summary_lines)
        .block(theme::panel_block(summary_title))
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));
    f.render_widget(summary_p, chunks[1]);

    if total_lines > visible_height {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"));
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(total_lines.saturating_sub(visible_height))
            .position(app.detail_scroll as usize);
        f.render_stateful_widget(
            scrollbar,
            chunks[1].inner(ratatui::layout::Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut scrollbar_state,
        );
    }
}
