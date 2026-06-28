use super::theme;
use crate::app::{App, SETTINGS_GROUPS};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, app: &mut App, list_area: Rect, content_area: Rect) {
    draw_list(f, app, list_area);

    let inner = content_area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });
    f.render_widget(
        theme::field_block("", app.settings_panel_focused),
        content_area,
    );

    match app.selected_index {
        0 => draw_password_panel(f, app, inner),
        1 => draw_timeout_panel(f, app, inner),
        2 => draw_lock_panel(f, app, inner),
        3 => draw_recovery_panel(f, app, inner),
        _ => {}
    }
}

fn draw_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = SETTINGS_GROUPS
        .iter()
        .map(|label| ListItem::new(Line::from(Span::raw(format!(" {}", label)))))
        .collect();

    let block = theme::panel_block("Settings");
    let list = List::new(items)
        .block(block)
        .style(theme::text_style())
        .highlight_style(theme::list_highlight_style());

    let mut state = ListState::default();
    state.select(Some(app.selected_index));
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_password_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let field_0_focused = app.settings_panel_focused && app.settings_active_field == 0;
    let field_1_focused = app.settings_panel_focused && app.settings_active_field == 1;

    app.settings_password_new
        .set_block(theme::field_block("New Master Password", field_0_focused));
    f.render_widget(&app.settings_password_new, chunks[0]);

    app.settings_password_confirm
        .set_block(theme::field_block("Confirm New Password", field_1_focused));
    f.render_widget(&app.settings_password_confirm, chunks[1]);

    let instructions = if app.settings_panel_focused {
        "Tab: Next Field | Ctrl+S: Save and Re-Encrypt | Esc: Back to List"
    } else {
        "Enter: Open"
    };
    f.render_widget(
        Paragraph::new(vec![Line::from(""), Line::from(instructions)])
            .alignment(Alignment::Center)
            .style(theme::muted_style()),
        chunks[2],
    );
}

fn draw_timeout_panel(f: &mut Frame, app: &App, area: Rect) {
    let mins = app.journal.settings.autolock_timeout_mins;
    let value = if mins == 0 {
        "Disabled".to_string()
    } else {
        format!("{} minutes", mins)
    };

    let instructions = if app.settings_panel_focused {
        "Left/Right or Up/Down: Adjust | Esc: Back to List"
    } else {
        "Enter: Open"
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Inactivity Timeout ", theme::muted_style()),
            Span::styled(format!("< {} >", value), theme::text_style()),
        ])
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(instructions)
            .alignment(Alignment::Center)
            .style(theme::muted_style()),
    ];
    f.render_widget(Paragraph::new(lines), area);
}

fn draw_lock_panel(f: &mut Frame, app: &App, area: Rect) {
    let value = if app.journal.settings.lock_on_suspend {
        "Enabled"
    } else {
        "Disabled"
    };

    let instructions = if app.settings_panel_focused {
        "Left/Right or Space: Toggle | Esc: Back to List"
    } else {
        "Enter: Open"
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Lock on Workstation Lock ", theme::muted_style()),
            Span::styled(format!("< {} >", value), theme::text_style()),
        ])
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(instructions)
            .alignment(Alignment::Center)
            .style(theme::muted_style()),
    ];
    f.render_widget(Paragraph::new(lines), area);
}

fn draw_recovery_panel(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![
        Line::from(""),
        Line::from("Split master password into N shares; any T can recover it")
            .alignment(Alignment::Center)
            .style(theme::muted_style()),
        Line::from(""),
    ];

    let n_focused = app.settings_panel_focused && app.settings_active_field == 0;
    let t_focused = app.settings_panel_focused && app.settings_active_field == 1;

    lines.push(
        Line::from(vec![
            Span::styled("Total Shares (N) ", theme::muted_style()),
            Span::styled(
                format!(" < {} > ", app.settings_num_shares),
                if n_focused {
                    theme::title_style()
                } else {
                    theme::text_style()
                },
            ),
        ])
        .alignment(Alignment::Center),
    );
    lines.push(
        Line::from(vec![
            Span::styled("Required Threshold (T) ", theme::muted_style()),
            Span::styled(
                format!(" < {} > ", app.settings_threshold),
                if t_focused {
                    theme::title_style()
                } else {
                    theme::text_style()
                },
            ),
        ])
        .alignment(Alignment::Center),
    );
    lines.push(Line::from(""));

    let instructions = if app.settings_panel_focused {
        "Tab: Switch N/T | Left/Right: Adjust | Ctrl+S: Generate Shares | Esc: Back to List"
    } else {
        "Enter: Open"
    };
    lines.push(
        Line::from(instructions)
            .alignment(Alignment::Center)
            .style(theme::muted_style()),
    );

    if !app.generated_shares.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from("Generated Shares").style(theme::success_style()));
        lines.push(Line::from(""));
        for (idx, share) in app.generated_shares.iter().enumerate() {
            lines.push(Line::from(vec![
                Span::styled(format!("Share {} ", idx + 1), theme::title_style()),
                Span::styled(share.clone(), theme::text_style()),
            ]));
        }
    }

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}
