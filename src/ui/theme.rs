//! Centralised theme definitions.
//!
//! A single source of truth for every colour and style decision in the UI.
//! All rendering code must consume only the constants and functions exported here —
//! never make ad-hoc colour choices in tab/modal code.
//!
//! ## Palette concept
//! Dark background (terminal default) · Indigo accent for primary focus ·
//! Amber warm-highlight for secondary labels · Muted slate for inactive chrome.

use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders},
};

// ── Palette constants ─────────────────────────────────────────────────────────

/// Primary accent — indigo blue used on focused borders, selected tab, key hints.
pub const ACCENT: Color = Color::Indexed(111); // #87afff
/// Secondary warm accent — amber/gold for section labels and field titles.
pub const LABEL: Color = Color::Indexed(179); // #d7af5f
/// Bright foreground for body text.
pub const TEXT: Color = Color::Indexed(253); // #dadada
/// Subdued chrome colour for inactive borders and decorative elements.
pub const MUTED: Color = Color::Indexed(241); // #626262
/// Subtle, slightly-lighter muted for secondary prose.
pub const DIM: Color = Color::Indexed(238); // #444444
/// Danger/error indicator — soft red with good contrast.
pub const DANGER: Color = Color::Indexed(203); // #ff5f5f
/// Success/confirm indicator — soft green.
pub const SUCCESS: Color = Color::Indexed(114); // #87d787
/// Background of the selected list row.
pub const SELECTED_BG: Color = Color::Indexed(235); // #262626
/// Foreground of the selected list row — full brightness.
pub const SELECTED_FG: Color = Color::Indexed(255); // #eeeeee
/// Calendar highlight cell — indigo bg.
pub const CAL_SELECTED_BG: Color = Color::Indexed(62); // #5f5faf
/// Streak / live counter — amber glow.
pub const STREAK: Color = Color::Indexed(214); // #ffaf00

// ── Style constructors ────────────────────────────────────────────────────────

/// Full-brightness body text.
#[inline]
pub fn text() -> Style {
    Style::default().fg(TEXT)
}

/// Subdued / inactive text.
#[inline]
pub fn muted() -> Style {
    Style::default().fg(MUTED)
}

/// Very dim decorative text.
#[inline]
pub fn dim() -> Style {
    Style::default().fg(DIM)
}

/// Primary accent — bold indigo, used for key hints and focused tab labels.
#[inline]
pub fn accent() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

/// Warm label style for field headings and section titles.
#[inline]
pub fn label() -> Style {
    Style::default().fg(LABEL)
}

/// Danger / error style — bold red.
#[inline]
pub fn danger() -> Style {
    Style::default().fg(DANGER).add_modifier(Modifier::BOLD)
}

/// Success / confirmation style — bold green.
#[inline]
pub fn success() -> Style {
    Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD)
}

/// Streak counter style — amber glow, bold.
#[inline]
pub fn streak() -> Style {
    Style::default().fg(STREAK).add_modifier(Modifier::BOLD)
}

/// Border style for a panel that is currently focused.
#[inline]
pub fn border_focused() -> Style {
    Style::default().fg(ACCENT)
}

/// Border style for an unfocused / read-only panel.
#[inline]
pub fn border_idle() -> Style {
    Style::default().fg(MUTED)
}

/// Border style contextualised by focus flag.
#[inline]
pub fn border(focused: bool) -> Style {
    if focused {
        border_focused()
    } else {
        border_idle()
    }
}

/// List-row highlight (selected item).
#[inline]
pub fn list_highlight() -> Style {
    Style::default()
        .bg(SELECTED_BG)
        .fg(SELECTED_FG)
        .add_modifier(Modifier::BOLD)
}

/// Mention tag style inside journal entry preview (underlined accent).
#[inline]
pub fn mention() -> Style {
    Style::default()
        .fg(ACCENT)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}

/// Editor cursor-line background.
#[inline]
pub fn editor_cursor_line() -> Style {
    Style::default().bg(Color::Indexed(233))
}

// ── Block constructors ────────────────────────────────────────────────────────

/// A plain, read-only rounded panel.  Pass an empty string to omit the title.
pub fn panel(title: impl Into<String>) -> Block<'static> {
    let t = title.into();
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_idle());
    if !t.trim().is_empty() {
        block = block.title(format!(" {} ", t));
    }
    block
}

/// An interactive input / control panel whose border reacts to focus.
pub fn field(title: impl Into<String>, focused: bool) -> Block<'static> {
    let t = title.into();
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border(focused));
    if !t.trim().is_empty() {
        block = block.title(format!(" {} ", t));
    }
    block
}

/// A modal overlay panel — uses a thick double border in accent colour.
pub fn modal(title: impl Into<String>) -> Block<'static> {
    let t = title.into();
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(border_focused())
        .title(format!(" {} ", t))
}

/// A danger-state modal (e.g. delete confirmation) — thick double border in danger colour.
pub fn modal_danger(title: impl Into<String>) -> Block<'static> {
    let t = title.into();
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(danger())
        .title(format!(" {} ", t))
}
