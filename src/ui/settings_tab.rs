//! Settings tab — navigation list (left) and context-sensitive control panel (right).

use super::theme;
use crate::app::{App, SETTINGS_GROUPS};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    text::{Line, Span},
    widgets::{
        List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
};

pub fn draw(f: &mut Frame, app: &mut App, list_area: Rect, content_area: Rect) {
    draw_list(f, app, list_area);

    f.render_widget(theme::field("", app.settings_panel_focused), content_area);
    let inner = content_area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    match app.selected_index {
        0 => draw_password_panel(f, app, inner),
        1 => draw_timeout_panel(f, app, inner),
        2 => draw_lock_panel(f, app, inner),
        3 => draw_recovery_panel(f, app, inner),
        _ => {}
    }
}

// ── Settings list ─────────────────────────────────────────────────────────────

fn draw_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = SETTINGS_GROUPS
        .iter()
        .map(|label| {
            ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(*label, theme::text()),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected_index));

    f.render_stateful_widget(
        List::new(items)
            .block(theme::panel("Settings"))
            .highlight_style(theme::list_highlight()),
        area,
        &mut state,
    );
}

// ── Change password ───────────────────────────────────────────────────────────

fn draw_password_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let [field0, field1, instructions_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .areas(area);

    let f0_focused = app.settings_panel_focused && app.settings_active_field == 0;
    let f1_focused = app.settings_panel_focused && app.settings_active_field == 1;

    app.settings_password_new
        .set_block(theme::field("New master password", f0_focused));
    f.render_widget(&app.settings_password_new, field0);

    app.settings_password_confirm
        .set_block(theme::field("Confirm new password", f1_focused));
    f.render_widget(&app.settings_password_confirm, field1);

    let instruction_text = if app.settings_panel_focused {
        "Tab  Next field    Ctrl+S  Re-encrypt & save    Esc  Back"
    } else {
        "Press  Enter  to open"
    };
    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(instruction_text, theme::muted())).alignment(Alignment::Center),
        ]),
        instructions_area,
    );
}

// ── Inactivity timeout ────────────────────────────────────────────────────────

fn draw_timeout_panel(f: &mut Frame, app: &App, area: Rect) {
    let mins = app.journal.settings.autolock_timeout_mins;
    let value = if mins == 0 {
        "Disabled".to_string()
    } else {
        format!("{} minute{}", mins, if mins == 1 { "" } else { "s" })
    };

    let hint = if app.settings_panel_focused {
        "←/→  Adjust    Esc  Back"
    } else {
        "Press  Enter  to open"
    };

    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Auto-lock after ", theme::muted()),
                Span::styled(format!(" ‹  {}  › ", value), theme::text()),
            ])
            .alignment(Alignment::Center),
            Line::from(""),
            Line::from(Span::styled(hint, theme::muted())).alignment(Alignment::Center),
        ]),
        area,
    );
}

// ── Lock-on-suspend ───────────────────────────────────────────────────────────

fn draw_lock_panel(f: &mut Frame, app: &App, area: Rect) {
    let (value, value_style) = if app.journal.settings.lock_on_suspend {
        ("Enabled", theme::success())
    } else {
        ("Disabled", theme::muted())
    };

    let hint = if app.settings_panel_focused {
        "Space/←/→  Toggle    Esc  Back"
    } else {
        "Press  Enter  to open"
    };

    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Lock on workstation lock ", theme::muted()),
                Span::styled(format!(" ‹  {}  › ", value), value_style),
            ])
            .alignment(Alignment::Center),
            Line::from(""),
            Line::from(Span::styled(hint, theme::muted())).alignment(Alignment::Center),
        ]),
        area,
    );
}

// ── Recovery shares (Shamir's Secret Sharing) ─────────────────────────────────

fn draw_recovery_panel(f: &mut Frame, app: &App, area: Rect) {
    let n_focused = app.settings_panel_focused && app.settings_active_field == 0;
    let t_focused = app.settings_panel_focused && app.settings_active_field == 1;

    let n_style = if n_focused {
        theme::accent()
    } else {
        theme::text()
    };
    let t_style = if t_focused {
        theme::accent()
    } else {
        theme::text()
    };

    let hint = if app.settings_panel_focused {
        "Tab  Switch N/T    ←/→  Adjust    Ctrl+S  Generate    Ctrl+E  Export    Esc  Back"
    } else {
        "Press  Enter  to open"
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Split your master password into N shares — any T can recover it.",
            theme::muted(),
        ))
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Total shares  (N) ", theme::muted()),
            Span::styled(format!(" ‹  {}  › ", app.settings_num_shares), n_style),
        ])
        .alignment(Alignment::Center),
        Line::from(vec![
            Span::styled("  Threshold     (T) ", theme::muted()),
            Span::styled(format!(" ‹  {}  › ", app.settings_threshold), t_style),
        ])
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled(hint, theme::muted())).alignment(Alignment::Center),
    ];

    if !app.generated_shares.is_empty() {
        lines.push(Line::from(""));
        lines.push(
            Line::from(Span::styled(
                "  Generated shares — store each one separately!",
                theme::success(),
            ))
            .alignment(Alignment::Center),
        );
        lines.push(Line::from(""));
        for (idx, share) in app.generated_shares.iter().enumerate() {
            lines.push(Line::from(vec![
                Span::styled(format!("  Share {:>2}  ", idx + 1), theme::label()),
                Span::styled(share.clone(), theme::text()),
            ]));
        }
    }

    let total_lines = lines.len();
    let scrollbar_needed = total_lines > area.height as usize;

    f.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((app.detail_scroll, 0)),
        area,
    );

    if scrollbar_needed {
        let visible = area.height as usize;
        let mut sb_state = ScrollbarState::default()
            .content_length(total_lines.saturating_sub(visible))
            .position(app.detail_scroll as usize);
        f.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼")),
            area.inner(Margin {
                horizontal: 0,
                vertical: 0,
            }),
            &mut sb_state,
        );
    }
}
