//! One place for every style decision: a single cyan accent for focus, dark gray for
//! everything inactive, and the two semantic colors (red for danger, green for success).
//! No emoji, no per-screen color choices.

use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders},
};

pub const ACCENT: Color = Color::Cyan;
pub const MUTED: Color = Color::DarkGray;
pub const TEXT: Color = Color::White;
pub const DANGER: Color = Color::Red;
pub const SUCCESS: Color = Color::Green;
/// Background used to highlight the selected row in a list.
pub const SELECTED_BG: Color = Color::Indexed(236);

pub fn border_style(focused: bool) -> Style {
    Style::default().fg(if focused { ACCENT } else { MUTED })
}

pub fn title_style() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn muted_style() -> Style {
    Style::default().fg(MUTED)
}

pub fn text_style() -> Style {
    Style::default().fg(TEXT)
}

pub fn danger_style() -> Style {
    Style::default().fg(DANGER).add_modifier(Modifier::BOLD)
}

pub fn success_style() -> Style {
    Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD)
}

pub fn list_highlight_style() -> Style {
    Style::default()
        .bg(SELECTED_BG)
        .add_modifier(Modifier::BOLD)
}

/// A rounded, bordered panel with a plain (non-focusable) title, used for lists and
/// read-only preview panes.
pub fn panel_block(title: impl Into<String>) -> Block<'static> {
    let t = title.into();
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MUTED));
    if !t.trim().is_empty() {
        block = block.title(format!(" {} ", t));
    }
    block
}

/// A rounded, bordered field whose border lights up with the accent color when focused.
/// Used everywhere a single input/control is rendered (contact form fields, settings
/// controls, password inputs).
pub fn field_block(title: impl Into<String>, focused: bool) -> Block<'static> {
    let t = title.into();
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style(focused));
    if !t.trim().is_empty() {
        block = block.title(format!(" {} ", t));
    }
    block
}
