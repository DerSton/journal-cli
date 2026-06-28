use crate::app::{App, AppMode, ContactField, Tab};
use crate::model::Journal;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Routes a single key press to the right handler based on the app's current mode.
pub fn handle_key(app: &mut App, key: KeyEvent) {
    match app.mode {
        AppMode::List => handle_list(app, key),
        AppMode::Writing { is_edit } => handle_writing(app, key, is_edit),
        AppMode::ContactPicker {
            is_edit,
            selected_contact_index,
        } => handle_contact_picker(app, key, is_edit, selected_contact_index),
        AppMode::DatePicker {
            is_edit,
            field_index,
            current_date,
        } => handle_date_picker(app, key, is_edit, field_index, current_date),
        AppMode::DeleteConfirm => handle_delete_confirm(app, key),
        AppMode::Login => handle_login(app, key),
        AppMode::Recovery => handle_recovery(app, key),
        AppMode::RecoveryReset => handle_recovery_reset(app, key),
        AppMode::Search => handle_search(app, key),
    }
}

fn handle_list(app: &mut App, key: KeyEvent) {
    if app.active_tab == Tab::Settings && app.settings_panel_focused {
        handle_settings_panel(app, key);
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Tab => {
            let next = match app.active_tab {
                Tab::Journal => Tab::Contacts,
                Tab::Contacts => Tab::Stats,
                Tab::Stats => Tab::Settings,
                Tab::Settings => Tab::Journal,
            };
            app.switch_tab(next);
        }
        KeyCode::Char('1') => app.switch_tab(Tab::Journal),
        KeyCode::Char('2') => app.switch_tab(Tab::Contacts),
        KeyCode::Char('3') => app.switch_tab(Tab::Stats),
        KeyCode::Char('4') => app.switch_tab(Tab::Settings),
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
                app.detail_scroll = 0;
                app.status_msg = None;
                app.error_msg = None;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let len = app.list_len();
            if len > 0 && app.selected_index < len - 1 {
                app.selected_index += 1;
                app.detail_scroll = 0;
                app.status_msg = None;
                app.error_msg = None;
            }
        }
        _ => match app.active_tab {
            Tab::Journal => handle_journal_list(app, key),
            Tab::Contacts => handle_contacts_list(app, key),
            Tab::Settings => handle_settings_list(app, key),
            Tab::Stats => handle_stats_list(app, key),
        },
    }
}

fn handle_journal_list(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('/') => {
            app.mode = AppMode::Search;
            app.selected_index = 0;
        }
        KeyCode::PageUp => {
            app.detail_scroll = app.detail_scroll.saturating_sub(1);
        }
        KeyCode::PageDown => {
            app.detail_scroll = app.detail_scroll.saturating_add(1);
        }
        KeyCode::Char('n') => {
            app.status_msg = None;
            app.error_msg = None;
            app.entry_date_for = None;
            app.textarea = ratatui_textarea::TextArea::default();
            app.mode = AppMode::Writing { is_edit: false };
        }
        KeyCode::Char('e') => {
            app.status_msg = None;
            app.error_msg = None;
            if let Some(real_idx) = app.selected_entry_idx() {
                let entry = &app.journal.entries[real_idx];
                app.entry_date_for = entry.date_for;
                let content = entry.content.clone();
                app.textarea =
                    ratatui_textarea::TextArea::new(content.lines().map(String::from).collect());
                app.mode = AppMode::Writing { is_edit: true };
            }
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            if !app.journal.entries.is_empty() {
                app.mode = AppMode::DeleteConfirm;
                app.status_msg = None;
                app.error_msg = None;
            }
        }
        KeyCode::Esc => {
            app.should_quit = true;
        }
        _ => {}
    }
}

fn handle_contacts_list(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('/') => {
            app.mode = AppMode::Search;
            app.selected_index = 0;
        }
        KeyCode::Char('n') => {
            app.status_msg = None;
            app.error_msg = None;
            app.init_contact_form(false);
        }
        KeyCode::Char('e') => {
            app.status_msg = None;
            app.error_msg = None;
            if !app.journal.contacts.is_empty() {
                app.init_contact_form(true);
            }
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            if !app.journal.contacts.is_empty() {
                app.mode = AppMode::DeleteConfirm;
                app.status_msg = None;
                app.error_msg = None;
            }
        }
        KeyCode::Esc => {
            app.should_quit = true;
        }
        _ => {}
    }
}

fn handle_settings_list(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('e') | KeyCode::Enter => {
            app.status_msg = None;
            app.error_msg = None;
            app.settings_panel_focused = true;
            app.settings_active_field = 0;
        }
        KeyCode::Esc => {
            app.should_quit = true;
        }
        _ => {}
    }
}

fn handle_stats_list(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc {
        app.should_quit = true;
    }
}

fn handle_settings_panel(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc {
        app.settings_panel_focused = false;
        return;
    }

    match app.selected_index {
        0 => handle_password_fields(
            app,
            key,
            |app| app.change_password(),
            |app| {
                app.settings_panel_focused = false;
            },
        ),
        1 => match key.code {
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Down | KeyCode::Char('j') => {
                app.adjust_autolock_timeout(-1)
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Up | KeyCode::Char('k') => {
                app.adjust_autolock_timeout(1)
            }
            _ => {}
        },
        2 => match key.code {
            KeyCode::Left
            | KeyCode::Right
            | KeyCode::Char('h')
            | KeyCode::Char('l')
            | KeyCode::Char(' ')
            | KeyCode::Up
            | KeyCode::Down
            | KeyCode::Char('j')
            | KeyCode::Char('k') => app.toggle_lock_on_suspend(),
            _ => {}
        },
        3 => match key.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Tab | KeyCode::BackTab => {
                app.settings_active_field = if app.settings_active_field == 0 { 1 } else { 0 };
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if app.settings_active_field == 0 {
                    if app.settings_num_shares > 1 {
                        app.settings_num_shares -= 1;
                        app.settings_threshold =
                            app.settings_threshold.min(app.settings_num_shares);
                    }
                } else if app.settings_threshold > 1 {
                    app.settings_threshold -= 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if app.settings_active_field == 0 {
                    if app.settings_num_shares < 255 {
                        app.settings_num_shares += 1;
                    }
                } else if app.settings_threshold < app.settings_num_shares {
                    app.settings_threshold += 1;
                }
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Err(e) = app.generate_recovery_shares() {
                    app.error_msg = Some(format!("Failed to generate shares: {}", e));
                }
            }
            _ => {}
        },
        _ => {}
    }
}

/// Shared handler for the two password-entry panels (change password, recovery reset).
/// `on_submit` runs on Ctrl+S; `on_success` lets each caller react differently afterwards
/// (settings returns focus to the list, recovery-reset unlocks straight into the journal).
fn handle_password_fields(
    app: &mut App,
    key: KeyEvent,
    on_submit: impl FnOnce(&mut App) -> Result<(), String>,
    on_success: impl FnOnce(&mut App),
) {
    if key.code == KeyCode::Tab || key.code == KeyCode::Down {
        app.settings_active_field = (app.settings_active_field + 1) % 2;
    } else if key.code == KeyCode::BackTab || key.code == KeyCode::Up {
        app.settings_active_field = if app.settings_active_field == 0 { 1 } else { 0 };
    } else if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
        match on_submit(app) {
            Ok(_) => {
                app.status_msg = Some("Password changed and journal re-encrypted".to_string());
                app.error_msg = None;
                on_success(app);
            }
            Err(e) => app.error_msg = Some(e),
        }
    } else {
        match app.settings_active_field {
            0 => {
                app.settings_password_new.input(key);
            }
            1 => {
                app.settings_password_confirm.input(key);
            }
            _ => {}
        };
    }
}

fn handle_writing(app: &mut App, key: KeyEvent, is_edit: bool) {
    match app.active_tab {
        Tab::Journal => {
            if key.code == KeyCode::Char('p') && key.modifiers.contains(KeyModifiers::ALT) {
                if !app.journal.contacts.is_empty() {
                    app.mode = AppMode::ContactPicker {
                        is_edit,
                        selected_contact_index: 0,
                    };
                }
            } else if key.code == KeyCode::Char('d') && key.modifiers.contains(KeyModifiers::ALT) {
                app.mode = AppMode::DatePicker {
                    is_edit,
                    field_index: 0,
                    current_date: app
                        .entry_date_for
                        .unwrap_or_else(|| chrono::Local::now().date_naive()),
                };
            } else if key.code == KeyCode::Char('s')
                && key.modifiers.contains(KeyModifiers::CONTROL)
            {
                app.save_entry();
            } else if key.code == KeyCode::Esc {
                app.mode = AppMode::List;
            } else {
                app.textarea.input(key);
            }
        }
        Tab::Contacts => handle_contact_form(app, key, is_edit),
        Tab::Settings | Tab::Stats => {}
    }
}

fn handle_contact_form(app: &mut App, key: KeyEvent, is_edit: bool) {
    if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.save_contact();
        return;
    }
    if key.code == KeyCode::Esc {
        app.mode = AppMode::List;
        return;
    }

    let active_field = app.contact_form.field_at(app.contact_form.active_field);
    if key.code == KeyCode::Enter
        && matches!(
            active_field,
            ContactField::Birthdate | ContactField::DateOfDeath
        )
    {
        let field_index = if active_field == ContactField::Birthdate {
            0
        } else {
            1
        };
        let raw = if field_index == 0 {
            app.contact_form.birthdate.lines().join("")
        } else {
            app.contact_form.date_of_death.lines().join("")
        };
        let current_date = crate::app::parse_localized_date(raw.trim())
            .unwrap_or_else(|| chrono::Local::now().date_naive());
        app.mode = AppMode::DatePicker {
            is_edit,
            field_index,
            current_date,
        };
        return;
    }

    if key.code == KeyCode::Tab || key.code == KeyCode::Down {
        app.contact_form.focus_next();
    } else if key.code == KeyCode::BackTab || key.code == KeyCode::Up {
        app.contact_form.focus_prev();
    } else {
        app.contact_form.handle_key(key);
    }
}

fn handle_contact_picker(
    app: &mut App,
    key: KeyEvent,
    is_edit: bool,
    selected_contact_index: usize,
) {
    match key.code {
        KeyCode::Esc => app.mode = AppMode::Writing { is_edit },
        KeyCode::Up | KeyCode::Char('k') => {
            let len = app.journal.contacts.len();
            if len > 0 {
                let next = if selected_contact_index > 0 {
                    selected_contact_index - 1
                } else {
                    len - 1
                };
                app.mode = AppMode::ContactPicker {
                    is_edit,
                    selected_contact_index: next,
                };
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let len = app.journal.contacts.len();
            if len > 0 {
                let next = (selected_contact_index + 1) % len;
                app.mode = AppMode::ContactPicker {
                    is_edit,
                    selected_contact_index: next,
                };
            }
        }
        KeyCode::Enter => {
            if let Some(contact) = app.journal.contacts.get(selected_contact_index) {
                let tag = contact.mention_tag();
                app.textarea.insert_str(tag);
            }
            app.mode = AppMode::Writing { is_edit };
        }
        _ => {}
    }
}

fn handle_date_picker(
    app: &mut App,
    key: KeyEvent,
    is_edit: bool,
    field_index: usize,
    current_date: chrono::NaiveDate,
) {
    let mut next_date = current_date;
    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::Writing { is_edit };
            return;
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if app.active_tab == Tab::Journal {
                app.entry_date_for = None;
            } else if field_index == 0 {
                app.contact_form.birthdate = ratatui_textarea::TextArea::default();
            } else {
                app.contact_form.date_of_death = ratatui_textarea::TextArea::default();
            }
            app.mode = AppMode::Writing { is_edit };
            return;
        }
        KeyCode::Enter => {
            if app.active_tab == Tab::Journal {
                app.entry_date_for = Some(current_date);
            } else {
                let formatted = crate::app::format_localized_date(current_date);
                if field_index == 0 {
                    app.contact_form.birthdate = ratatui_textarea::TextArea::new(vec![formatted]);
                } else {
                    app.contact_form.date_of_death =
                        ratatui_textarea::TextArea::new(vec![formatted]);
                }
            }
            app.mode = AppMode::Writing { is_edit };
            return;
        }
        KeyCode::Left | KeyCode::Char('h') => next_date -= chrono::Duration::days(1),
        KeyCode::Right | KeyCode::Char('l') => next_date += chrono::Duration::days(1),
        KeyCode::Up | KeyCode::Char('k') => next_date -= chrono::Duration::days(7),
        KeyCode::Down | KeyCode::Char('j') => next_date += chrono::Duration::days(7),
        KeyCode::PageUp | KeyCode::Char('[') => {
            next_date = current_date
                .checked_sub_months(chrono::Months::new(1))
                .unwrap_or(current_date);
        }
        KeyCode::PageDown | KeyCode::Char(']') => {
            next_date = current_date
                .checked_add_months(chrono::Months::new(1))
                .unwrap_or(current_date);
        }
        KeyCode::Home | KeyCode::Char('{') => {
            next_date = current_date
                .checked_sub_months(chrono::Months::new(12))
                .unwrap_or(current_date);
        }
        KeyCode::End | KeyCode::Char('}') => {
            next_date = current_date
                .checked_add_months(chrono::Months::new(12))
                .unwrap_or(current_date);
        }
        _ => return,
    }
    app.mode = AppMode::DatePicker {
        is_edit,
        field_index,
        current_date: next_date,
    };
}

fn handle_delete_confirm(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => match app.active_tab {
            Tab::Journal => app.delete_selected_entry(),
            Tab::Contacts => app.delete_selected_contact(),
            Tab::Settings | Tab::Stats => {}
        },
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.mode = AppMode::List,
        _ => {}
    }
}

fn handle_login(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            app.error_msg = None;
            match Journal::load(&app.file_path, &app.login_password) {
                Ok((journal, salt)) => {
                    app.journal = journal;
                    app.salt = salt;
                    app.password = app.login_password.clone();
                    app.mode = AppMode::List;
                    app.sort_entries();
                    app.sort_contacts();
                    app.status_msg = Some("Journal unlocked".to_string());
                }
                Err(e) => {
                    app.error_msg = Some(format!("Decryption failed: {}", e));
                    app.login_password.clear();
                }
            }
        }
        KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.mode = AppMode::Recovery;
            app.recovery_shares.clear();
            app.recovery_textarea = ratatui_textarea::TextArea::default();
            app.recovery_status_msg = None;
            app.error_msg = None;
        }
        KeyCode::Char(c) => app.login_password.push(c),
        KeyCode::Backspace => {
            app.login_password.pop();
        }
        _ => {}
    }
}

fn handle_recovery(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => submit_recovery_share(app),
        KeyCode::Esc => {
            app.mode = AppMode::Login;
            app.error_msg = None;
            app.recovery_status_msg = None;
            app.login_password.clear();
        }
        _ => {
            app.recovery_textarea.input(key);
        }
    }
}

fn submit_recovery_share(app: &mut App) {
    app.error_msg = None;
    app.recovery_status_msg = None;
    let share_str = app.recovery_textarea.lines().join("").trim().to_string();
    if share_str.is_empty() {
        app.error_msg = Some("Please enter a recovery share".to_string());
        return;
    }

    let parsed = match crate::crypto::parse_share(&share_str) {
        Ok(parsed) => parsed,
        Err(e) => {
            app.error_msg = Some(format!("Invalid recovery share format: {}", e));
            return;
        }
    };

    let already_entered = app.recovery_shares.iter().any(|s| {
        crate::crypto::parse_share(s)
            .map(|p| p.index == parsed.index)
            .unwrap_or(false)
    });
    if already_entered {
        app.error_msg = Some(format!(
            "Share with index {} was already entered",
            parsed.index
        ));
        return;
    }

    app.recovery_shares.push(share_str);
    app.recovery_textarea = ratatui_textarea::TextArea::default();
    app.recovery_status_msg = Some(format!("Added share {}.", parsed.index));

    if app.recovery_shares.len() < parsed.threshold {
        return;
    }

    app.recovery_status_msg = Some("Threshold met. Reconstructing password...".to_string());
    let reconstructed = match crate::crypto::reconstruct_password(&app.recovery_shares) {
        Ok(pwd) => pwd,
        Err(e) => {
            app.error_msg = Some(format!("Reconstruction failed: {}", e));
            app.recovery_shares.clear();
            return;
        }
    };

    match Journal::load(&app.file_path, &reconstructed) {
        Ok((journal, salt)) => {
            app.journal = journal;
            app.salt = salt;
            app.password = reconstructed;
            app.mode = AppMode::RecoveryReset;
            app.settings_password_new = ratatui_textarea::TextArea::default();
            app.settings_password_confirm = ratatui_textarea::TextArea::default();
            app.settings_active_field = 0;
            app.error_msg = None;
        }
        Err(e) => {
            app.error_msg = Some(format!(
                "Decryption with reconstructed password failed: {}",
                e
            ));
            app.recovery_shares.clear();
        }
    }
}

fn handle_recovery_reset(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc {
        app.should_quit = true;
        return;
    }
    handle_password_fields(
        app,
        key,
        |app| app.change_password(),
        |app| {
            app.mode = AppMode::List;
            app.sort_entries();
            app.sort_contacts();
        },
    );
}

fn handle_search(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.search_query.clear();
            app.selected_index = 0;
            app.mode = AppMode::List;
        }
        KeyCode::Enter => {
            app.selected_index = 0;
            app.mode = AppMode::List;
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.selected_index = 0;
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            app.selected_index = 0;
        }
        _ => {}
    }
}
