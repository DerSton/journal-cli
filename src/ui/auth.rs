use super::{centered_rect, theme};
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::Paragraph,
};

pub fn draw_login(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 40, f.area());
    let block = theme::field_block("Journal Locked", true);

    let masked = "*".repeat(app.login_password.len());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(block.inner(area));

    let input =
        Paragraph::new(format!(" {}_", masked)).block(theme::field_block("Master Password", false));

    let mut lines = vec![
        Line::from("Press Enter to Unlock"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Ctrl+R", theme::title_style()),
            Span::raw(" - Recovery Mode"),
        ]),
        Line::from(""),
        Line::from(env!("APP_VERSION")).style(theme::muted_style()),
    ];
    if let Some(ref err) = app.error_msg {
        lines.insert(0, Line::from(""));
        lines.insert(0, Line::from(err.as_str()).style(theme::danger_style()));
    }

    f.render_widget(block, area);
    f.render_widget(input, chunks[1]);
    f.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        chunks[3],
    );
}

pub fn draw_recovery(f: &mut Frame, app: &mut App) {
    let area = centered_rect(80, 75, f.area());
    let block = theme::field_block("Recovery Mode", true);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(block.inner(area));

    let header = Paragraph::new(vec![
        Line::from("Enter your recovery shares one at a time"),
        Line::from("The journal unlocks automatically once enough shares are entered"),
    ])
    .alignment(Alignment::Center)
    .style(theme::muted_style());

    app.recovery_textarea
        .set_block(theme::field_block("Recovery Share", true));

    let mut share_lines = vec![
        Line::from("Entered Shares").style(theme::title_style()),
        Line::from(""),
    ];
    if app.recovery_shares.is_empty() {
        share_lines.push(Line::from("  (none yet)").style(theme::muted_style()));
    } else {
        for (idx, share) in app.recovery_shares.iter().enumerate() {
            share_lines.push(Line::from(vec![
                Span::styled(format!("  Share {}: ", idx + 1), theme::success_style()),
                Span::raw(share.as_str()),
            ]));
        }
    }

    let mut footer = vec![];
    if let Some(ref status) = app.recovery_status_msg {
        footer.push(Line::from(status.as_str()).style(theme::success_style()));
    } else if let Some(ref err) = app.error_msg {
        footer.push(Line::from(err.as_str()).style(theme::danger_style()));
    } else {
        footer.push(Line::from(""));
    }
    footer.push(
        Line::from(vec![
            Span::styled("Enter", theme::title_style()),
            Span::raw(" - Submit Share   "),
            Span::styled("Esc", theme::title_style()),
            Span::raw(" - Back to Login"),
        ])
        .alignment(Alignment::Center),
    );

    f.render_widget(block, area);
    f.render_widget(header, chunks[0]);
    f.render_widget(&app.recovery_textarea, chunks[1]);
    f.render_widget(
        Paragraph::new(share_lines).wrap(ratatui::widgets::Wrap { trim: false }),
        chunks[3],
    );
    f.render_widget(
        Paragraph::new(footer).alignment(Alignment::Center),
        chunks[4],
    );
}

pub fn draw_recovery_reset(f: &mut Frame, app: &mut App) {
    let area = centered_rect(70, 60, f.area());
    let block = theme::field_block("Reset Master Password", true);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(block.inner(area));

    let header = Paragraph::new(vec![
        Line::from("Recovery shares matched").style(theme::success_style()),
        Line::from(""),
        Line::from("Set a new master password"),
    ])
    .alignment(Alignment::Center);

    app.settings_password_new.set_block(theme::field_block(
        "New Master Password",
        app.settings_active_field == 0,
    ));
    app.settings_password_confirm.set_block(theme::field_block(
        "Confirm New Password",
        app.settings_active_field == 1,
    ));

    let mut footer = vec![Line::from("")];
    if let Some(ref err) = app.error_msg {
        footer.push(Line::from(err.as_str()).style(theme::danger_style()));
        footer.push(Line::from(""));
    }
    footer.push(
        Line::from(vec![
            Span::styled("Tab", theme::title_style()),
            Span::raw(" - Next Field   "),
            Span::styled("Ctrl+S", theme::title_style()),
            Span::raw(" - Save and Open   "),
            Span::styled("Esc", theme::title_style()),
            Span::raw(" - Exit"),
        ])
        .alignment(Alignment::Center),
    );

    f.render_widget(block, area);
    f.render_widget(header, chunks[0]);
    f.render_widget(&app.settings_password_new, chunks[1]);
    f.render_widget(&app.settings_password_confirm, chunks[2]);
    f.render_widget(Paragraph::new(footer), chunks[3]);
}
