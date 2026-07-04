//! Authentication screens — login, password recovery, and post-recovery password reset.
//!
//! These full-screen modes bypass the normal tab shell and are rendered directly
//! over the terminal's alternate screen.

use super::{centered_rect, theme};
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

// ── Login ─────────────────────────────────────────────────────────────────────

pub fn draw_login(f: &mut Frame, app: &App) {
    let area = centered_rect(54, 44, f.area());
    let block = theme::modal("  Vault Locked");

    let masked = "●".repeat(app.login_password.len());

    let [_, input_area, _, info_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(block.inner(area));

    // Render the outer modal frame first.
    f.render_widget(block, area);

    // Password input row.
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" {}▌", masked),
            theme::text(),
        )))
        .block(theme::field("Master password", true)),
        input_area,
    );

    // Info / hint block below.
    let mut info_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter", theme::accent()),
            Span::styled("  Unlock", theme::muted()),
            Span::styled("     Ctrl+R", theme::accent()),
            Span::styled("  Recovery mode", theme::muted()),
            Span::styled("     Esc", theme::accent()),
            Span::styled("  Quit", theme::muted()),
        ])
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled(env!("APP_VERSION"), theme::dim())).alignment(Alignment::Center),
    ];

    if let Some(ref err) = app.error_msg {
        info_lines.insert(0, Line::from(""));
        info_lines.insert(
            0,
            Line::from(Span::styled(format!("  ✕  {}", err), theme::danger()))
                .alignment(Alignment::Center),
        );
    }

    f.render_widget(
        Paragraph::new(info_lines).alignment(Alignment::Center),
        info_area,
    );
}

// ── Recovery ──────────────────────────────────────────────────────────────────

pub fn draw_recovery(f: &mut Frame, app: &mut App) {
    let area = centered_rect(80, 78, f.area());
    let block = theme::modal("  Recovery Mode");

    let [header_area, input_area, _, shares_area, footer_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .areas(block.inner(area));

    f.render_widget(block, area);

    // Header instructions.
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                "Enter your recovery shares one at a time.",
                theme::muted(),
            ))
            .alignment(Alignment::Center),
            Line::from(Span::styled(
                "The journal unlocks automatically once enough valid shares are provided.",
                theme::dim(),
            ))
            .alignment(Alignment::Center),
        ]),
        header_area,
    );

    // Share input.
    app.recovery_textarea
        .set_block(theme::field("Recovery share", true));
    f.render_widget(&app.recovery_textarea, input_area);

    // Entered shares list.
    let mut share_lines = vec![
        Line::from(Span::styled("  Entered shares", theme::label())),
        Line::from(""),
    ];
    if app.recovery_shares.is_empty() {
        share_lines.push(Line::from(Span::styled("  (none yet)", theme::dim())));
    } else {
        for (i, share) in app.recovery_shares.iter().enumerate() {
            share_lines.push(Line::from(vec![
                Span::styled(format!("  Share {:>2}  ", i + 1), theme::success()),
                Span::raw(share.as_str()),
            ]));
        }
    }

    let total_lines = share_lines.len();
    let scrollbar_needed = total_lines > shares_area.height as usize;

    f.render_widget(
        Paragraph::new(share_lines).scroll((app.detail_scroll, 0)),
        shares_area,
    );

    if scrollbar_needed {
        let visible = shares_area.height as usize;
        let mut sb_state = ScrollbarState::default()
            .content_length(total_lines.saturating_sub(visible))
            .position(app.detail_scroll as usize);
        f.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼")),
            shares_area,
            &mut sb_state,
        );
    }

    // Footer: status/error + keybinds.
    let status_line = if let Some(ref msg) = app.recovery_status_msg {
        Line::from(Span::styled(format!("  ✓  {}", msg), theme::success()))
            .alignment(Alignment::Center)
    } else if let Some(ref err) = app.error_msg {
        Line::from(Span::styled(format!("  ✕  {}", err), theme::danger()))
            .alignment(Alignment::Center)
    } else {
        Line::from("")
    };

    f.render_widget(
        Paragraph::new(vec![
            status_line,
            Line::from(vec![
                Span::styled("  Enter", theme::accent()),
                Span::styled("  Submit share", theme::muted()),
                Span::styled("     Esc", theme::accent()),
                Span::styled("  Back to login", theme::muted()),
            ])
            .alignment(Alignment::Center),
        ]),
        footer_area,
    );
}

// ── Post-recovery password reset ──────────────────────────────────────────────

pub fn draw_recovery_reset(f: &mut Frame, app: &mut App) {
    let area = centered_rect(64, 58, f.area());
    let block = theme::modal("  Set New Master Password");

    let [header_area, field0_area, field1_area, footer_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .areas(block.inner(area));

    f.render_widget(block, area);

    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  ✓  Recovery shares verified!",
                theme::success(),
            ))
            .alignment(Alignment::Center),
            Line::from(Span::styled(
                "  Choose a new master password below.",
                theme::muted(),
            ))
            .alignment(Alignment::Center),
        ]),
        header_area,
    );

    app.settings_password_new.set_block(theme::field(
        "New master password",
        app.settings_active_field == 0,
    ));
    f.render_widget(&app.settings_password_new, field0_area);

    app.settings_password_confirm.set_block(theme::field(
        "Confirm new password",
        app.settings_active_field == 1,
    ));
    f.render_widget(&app.settings_password_confirm, field1_area);

    let mut footer_lines = vec![Line::from("")];
    if let Some(ref err) = app.error_msg {
        footer_lines.push(
            Line::from(Span::styled(format!("  ✕  {}", err), theme::danger()))
                .alignment(Alignment::Center),
        );
        footer_lines.push(Line::from(""));
    }
    footer_lines.push(
        Line::from(vec![
            Span::styled("  Tab", theme::accent()),
            Span::styled("  Next field", theme::muted()),
            Span::styled("     Ctrl+S", theme::accent()),
            Span::styled("  Save & open journal", theme::muted()),
            Span::styled("     Esc", theme::accent()),
            Span::styled("  Exit", theme::muted()),
        ])
        .alignment(Alignment::Center),
    );

    f.render_widget(Paragraph::new(footer_lines), footer_area);
}
