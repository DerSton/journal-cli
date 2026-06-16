mod app;
mod crypto;
mod journal;
mod ui;

use app::{App, AppMode, Tab};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use journal::Journal;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::env;
use std::io;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. CLI Argument Parsing
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: journal-cli <path_to_journal_file>");
        std::process::exit(1);
    }
    let journal_path_str = &args[1];
    let journal_path = Path::new(journal_path_str);

    // 2. Load or Create Journal Securely
    // 2. Setup or load config details
    let (journal, salt, password, start_in_login) = if journal_path.exists() {
        (
            Journal::default(),
            [0u8; crate::crypto::SALT_SIZE],
            String::new(),
            true,
        )
    } else {
        println!(
            "No journal file found at '{}'. Initializing a new secure journal.",
            journal_path_str
        );

        let password = loop {
            let p1 = rpassword::prompt_password("Set Master Password: ")?;
            let p2 = rpassword::prompt_password("Confirm Master Password: ")?;
            if p1 == p2 {
                if p1.trim().is_empty() {
                    println!("Password cannot be empty. Please try again.");
                    continue;
                }
                break p1;
            } else {
                println!("Passwords do not match. Please try again.");
            }
        };

        match Journal::create_new(journal_path, &password) {
            Ok((j, s)) => (j, s, password, false),
            Err(e) => {
                eprintln!("Failed to create new journal: {}", e);
                std::process::exit(1);
            }
        }
    };

    // 3. Initialize Terminal for TUI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 4. Run TUI Event Loop
    let mut app = App::new(journal, journal_path_str.clone(), password, salt);
    if start_in_login {
        app.mode = AppMode::Login;
    }
    let run_result = run_app(&mut terminal, &mut app);

    // 5. Restore Terminal State
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Check for errors during run
    if let Err(e) = run_result {
        eprintln!("Application error: {}", e);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>>
where
    Box<dyn std::error::Error>: From<B::Error>,
{
    let mut last_activity = std::time::Instant::now();
    loop {
        if app.should_quit {
            break;
        }

        // Render current state
        terminal.draw(|f| ui::draw(f, app))?;

        // 1. Check PC lock state if enabled
        #[cfg(target_os = "windows")]
        {
            if app.journal.settings.lock_on_suspend && is_workstation_locked() {
                app.should_quit = true;
                break;
            }
        }

        // 2. Check inactivity timeout if enabled
        if app.journal.settings.autolock_timeout_mins > 0 {
            let timeout_duration = std::time::Duration::from_secs(
                app.journal.settings.autolock_timeout_mins as u64 * 60,
            );
            if last_activity.elapsed() >= timeout_duration {
                app.should_quit = true;
                break;
            }
        }

        // Poll for inputs
        if event::poll(std::time::Duration::from_millis(500))?
            && let Event::Key(key) = event::read()?
        {
            // Reset inactivity timer
            last_activity = std::time::Instant::now();

            // crossterm on Windows sends release events as well; only process press events.
            if key.kind == event::KeyEventKind::Press {
                match app.mode {
                    AppMode::List => match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                        }
                        // Tab Selection
                        KeyCode::Tab => {
                            let next_tab = match app.active_tab {
                                Tab::Journal => Tab::Contacts,
                                Tab::Contacts => Tab::Settings,
                                Tab::Settings => Tab::Journal,
                            };
                            app.switch_tab(next_tab);
                        }
                        KeyCode::Char('1') => {
                            app.switch_tab(Tab::Journal);
                        }
                        KeyCode::Char('2') => {
                            app.switch_tab(Tab::Contacts);
                        }
                        KeyCode::Char('3') => {
                            app.switch_tab(Tab::Settings);
                        }
                        // Selection Navigation
                        KeyCode::Up | KeyCode::Char('k') => {
                            if app.selected_index > 0 {
                                app.selected_index -= 1;
                                app.detail_scroll = 0;
                                app.status_msg = None;
                                app.error_msg = None;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let len = match app.active_tab {
                                Tab::Journal => app.journal.entries.len(),
                                Tab::Contacts => app.journal.contacts.len(),
                                Tab::Settings => 4,
                            };
                            if len > 0 && app.selected_index < len - 1 {
                                app.selected_index += 1;
                                app.detail_scroll = 0;
                                app.status_msg = None;
                                app.error_msg = None;
                            }
                        }
                        KeyCode::PageUp => {
                            if app.active_tab == Tab::Journal {
                                app.detail_scroll = app.detail_scroll.saturating_sub(1);
                            }
                        }
                        KeyCode::PageDown => {
                            if app.active_tab == Tab::Journal {
                                app.detail_scroll = app.detail_scroll.saturating_add(1);
                            }
                        }
                        // Item Operations
                        KeyCode::Char('n') => {
                            app.status_msg = None;
                            app.error_msg = None;
                            match app.active_tab {
                                Tab::Journal => {
                                    app.textarea = ratatui_textarea::TextArea::default();
                                    app.mode = AppMode::Writing { is_edit: false };
                                }
                                Tab::Contacts => {
                                    app.init_contact_form(false);
                                }
                                Tab::Settings => {}
                            }
                        }
                        KeyCode::Char('e') | KeyCode::Enter => {
                            app.status_msg = None;
                            app.error_msg = None;
                            match app.active_tab {
                                Tab::Journal => {
                                    if key.code == KeyCode::Char('e')
                                        && !app.journal.entries.is_empty()
                                    {
                                        let content =
                                            &app.journal.entries[app.selected_index].content;
                                        app.textarea = ratatui_textarea::TextArea::new(
                                            content.lines().map(String::from).collect(),
                                        );
                                        app.mode = AppMode::Writing { is_edit: true };
                                    }
                                }
                                Tab::Contacts => {
                                    if key.code == KeyCode::Char('e')
                                        && !app.journal.contacts.is_empty()
                                    {
                                        app.init_contact_form(true);
                                    }
                                }
                                Tab::Settings => match app.selected_index {
                                    0 => {
                                        app.settings_password_new =
                                            ratatui_textarea::TextArea::default();
                                        app.settings_password_confirm =
                                            ratatui_textarea::TextArea::default();
                                        app.settings_active_field = 0;
                                        app.mode = AppMode::Writing { is_edit: false };
                                    }
                                    1 => {
                                        app.temp_timeout_mins =
                                            app.journal.settings.autolock_timeout_mins;
                                        app.mode = AppMode::Writing { is_edit: false };
                                    }
                                    2 => {
                                        app.temp_lock_on_suspend =
                                            app.journal.settings.lock_on_suspend;
                                        app.mode = AppMode::Writing { is_edit: false };
                                    }
                                    3 => {
                                        app.settings_active_field = 0;
                                        app.mode = AppMode::Writing { is_edit: false };
                                    }
                                    _ => {}
                                },
                            }
                        }
                        KeyCode::Char('d') | KeyCode::Delete | KeyCode::Esc => {
                            let is_empty = match app.active_tab {
                                Tab::Journal => app.journal.entries.is_empty(),
                                Tab::Contacts => app.journal.contacts.is_empty(),
                                Tab::Settings => true,
                            };
                            if key.code == KeyCode::Esc {
                                app.should_quit = true;
                            } else if !is_empty {
                                app.mode = AppMode::DeleteConfirm;
                                app.status_msg = None;
                                app.error_msg = None;
                            }
                        }
                        _ => {}
                    },
                    AppMode::Writing { is_edit } => match app.active_tab {
                        Tab::Journal => {
                            if key.code == KeyCode::Char('p')
                                && key.modifiers.contains(KeyModifiers::ALT)
                            {
                                if !app.journal.contacts.is_empty() {
                                    app.mode = AppMode::ContactPicker {
                                        is_edit,
                                        selected_contact_index: 0,
                                    };
                                }
                            } else if key.code == KeyCode::Char('s')
                                && key.modifiers.contains(KeyModifiers::CONTROL)
                            {
                                app.handle_save_entry();
                            } else if key.code == KeyCode::Esc {
                                app.mode = AppMode::List;
                            } else {
                                app.textarea.input(key);
                            }
                        }
                        Tab::Contacts => {
                            if key.code == KeyCode::Char('s')
                                && key.modifiers.contains(KeyModifiers::CONTROL)
                            {
                                app.handle_save_contact();
                            } else if key.code == KeyCode::Esc {
                                app.mode = AppMode::List;
                            } else if key.code == KeyCode::Left
                                && key.modifiers.contains(KeyModifiers::CONTROL)
                            {
                                app.contact_form_tab = if app.contact_form_tab == 0 {
                                    2
                                } else {
                                    app.contact_form_tab - 1
                                };
                                app.active_field_index = 0;
                            } else if key.code == KeyCode::Right
                                && key.modifiers.contains(KeyModifiers::CONTROL)
                            {
                                app.contact_form_tab = (app.contact_form_tab + 1) % 3;
                                app.active_field_index = 0;
                            } else if key.modifiers.contains(KeyModifiers::ALT)
                                && (key.code == KeyCode::Char('1')
                                    || key.code == KeyCode::Char('2')
                                    || key.code == KeyCode::Char('3'))
                            {
                                match key.code {
                                    KeyCode::Char('1') => {
                                        app.contact_form_tab = 0;
                                        app.active_field_index = 0;
                                    }
                                    KeyCode::Char('2') => {
                                        app.contact_form_tab = 1;
                                        app.active_field_index = 0;
                                    }
                                    KeyCode::Char('3') => {
                                        app.contact_form_tab = 2;
                                        app.active_field_index = 0;
                                    }
                                    _ => {}
                                }
                            } else if key.code == KeyCode::Tab || key.code == KeyCode::Down {
                                let num_fields = app.contact_form_num_fields();
                                if num_fields > 0 {
                                    app.active_field_index =
                                        (app.active_field_index + 1) % num_fields;
                                }
                            } else if key.code == KeyCode::BackTab || key.code == KeyCode::Up {
                                let num_fields = app.contact_form_num_fields();
                                if num_fields > 0 {
                                    app.active_field_index = if app.active_field_index == 0 {
                                        num_fields - 1
                                    } else {
                                        app.active_field_index - 1
                                    };
                                }
                            } else {
                                match app.contact_form_tab {
                                    0 => {
                                        let f_names_len = app.contact_form_first_names.len();
                                        if app.active_field_index == 0 {
                                            app.contact_form_title.input(key);
                                        } else if app.active_field_index >= 1
                                            && app.active_field_index < 1 + f_names_len
                                        {
                                            let idx = app.active_field_index - 1;
                                            app.contact_form_first_names[idx].input(key);
                                            if idx == f_names_len - 1 {
                                                let content = app.contact_form_first_names[idx]
                                                    .lines()
                                                    .join("");
                                                if !content.trim().is_empty() {
                                                    app.contact_form_first_names.push(ratatui_textarea::TextArea::default());
                                                }
                                            }
                                        } else if app.active_field_index == 1 + f_names_len {
                                            app.contact_form_last_name.input(key);
                                        } else if app.active_field_index == 2 + f_names_len {
                                            app.contact_form_preferred_name.input(key);
                                        } else if app.active_field_index == 3 + f_names_len {
                                            app.contact_form_maiden_name.input(key);
                                        } else if app.active_field_index == 4 + f_names_len {
                                            app.contact_form_suffix.input(key);
                                        }
                                    }
                                    1 => {
                                        let nats_len = app.contact_form_nationalities.len();
                                        let langs_len = app.contact_form_languages.len();
                                        let start_lang = 4 + nats_len;
                                        let idx_marital = start_lang + langs_len;

                                        if app.active_field_index == 0 {
                                            if key.code == KeyCode::Enter {
                                                let val_str =
                                                    app.contact_form_birthdate.lines().join("");
                                                let parsed_date =
                                                    chrono::NaiveDate::parse_from_str(
                                                        val_str.trim(),
                                                        "%Y-%m-%d",
                                                    )
                                                    .unwrap_or_else(|_| {
                                                        chrono::Local::now().date_naive()
                                                    });
                                                app.mode = AppMode::DatePicker {
                                                    is_edit,
                                                    field_index: 0,
                                                    current_date: parsed_date,
                                                };
                                            } else {
                                                app.contact_form_birthdate.input(key);
                                            }
                                        } else if app.active_field_index == 1 {
                                            if key.code == KeyCode::Enter {
                                                let val_str =
                                                    app.contact_form_deathdate.lines().join("");
                                                let parsed_date =
                                                    chrono::NaiveDate::parse_from_str(
                                                        val_str.trim(),
                                                        "%Y-%m-%d",
                                                    )
                                                    .unwrap_or_else(|_| {
                                                        chrono::Local::now().date_naive()
                                                    });
                                                app.mode = AppMode::DatePicker {
                                                    is_edit,
                                                    field_index: 1,
                                                    current_date: parsed_date,
                                                };
                                            } else {
                                                app.contact_form_deathdate.input(key);
                                            }
                                        } else if app.active_field_index == 2 {
                                            app.contact_form_gender.input(key);
                                        } else if app.active_field_index == 3 {
                                            app.contact_form_pronouns.input(key);
                                        } else if app.active_field_index >= 4
                                            && app.active_field_index < 4 + nats_len
                                        {
                                            let idx = app.active_field_index - 4;
                                            app.contact_form_nationalities[idx].input(key);
                                            if idx == nats_len - 1 {
                                                let content = app.contact_form_nationalities[idx]
                                                    .lines()
                                                    .join("");
                                                if !content.trim().is_empty() {
                                                    app.contact_form_nationalities.push(ratatui_textarea::TextArea::default());
                                                }
                                            }
                                        } else if app.active_field_index >= start_lang
                                            && app.active_field_index < start_lang + langs_len
                                        {
                                            let idx = app.active_field_index - start_lang;
                                            app.contact_form_languages[idx].input(key);
                                            if idx == langs_len - 1 {
                                                let content = app.contact_form_languages[idx]
                                                    .lines()
                                                    .join("");
                                                if !content.trim().is_empty() {
                                                    app.contact_form_languages.push(ratatui_textarea::TextArea::default());
                                                }
                                            }
                                        } else if app.active_field_index == idx_marital {
                                            if key.code == KeyCode::Left {
                                                app.contact_form_marital_status_idx =
                                                    if app.contact_form_marital_status_idx == 0 {
                                                        crate::app::MARITAL_STATUS_OPTIONS.len() - 1
                                                    } else {
                                                        app.contact_form_marital_status_idx - 1
                                                    };
                                            } else if key.code == KeyCode::Right {
                                                app.contact_form_marital_status_idx =
                                                    (app.contact_form_marital_status_idx + 1)
                                                        % crate::app::MARITAL_STATUS_OPTIONS.len();
                                            }
                                        } else if app.active_field_index == idx_marital + 1 {
                                            app.contact_form_religion.input(key);
                                        }
                                    }
                                    2 => {
                                        if app.active_field_index == 0 {
                                            if key.code == KeyCode::Left {
                                                app.contact_form_blood_type_idx =
                                                    if app.contact_form_blood_type_idx == 0 {
                                                        crate::app::BLOOD_TYPE_OPTIONS.len() - 1
                                                    } else {
                                                        app.contact_form_blood_type_idx - 1
                                                    };
                                            } else if key.code == KeyCode::Right {
                                                app.contact_form_blood_type_idx =
                                                    (app.contact_form_blood_type_idx + 1)
                                                        % crate::app::BLOOD_TYPE_OPTIONS.len();
                                            }
                                        } else if app.active_field_index == 1 {
                                            app.contact_form_eye_color.input(key);
                                        } else if app.active_field_index == 2 {
                                            app.contact_form_hair_color.input(key);
                                        } else if app.active_field_index == 3 {
                                            app.contact_form_height.input(key);
                                        } else if app.active_field_index == 4 {
                                            app.contact_form_notes.input(key);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Tab::Settings => match app.selected_index {
                            0 => {
                                if key.code == KeyCode::Esc {
                                    app.mode = AppMode::List;
                                } else if key.code == KeyCode::Char('s')
                                    && key.modifiers.contains(KeyModifiers::CONTROL)
                                {
                                    match app.handle_change_password() {
                                        Ok(_) => {
                                            app.status_msg = Some(
                                                "Password changed and database re-encrypted"
                                                    .to_string(),
                                            );
                                            app.error_msg = None;
                                            app.mode = AppMode::List;
                                        }
                                        Err(e) => {
                                            app.error_msg = Some(e);
                                        }
                                    }
                                } else if key.code == KeyCode::Tab || key.code == KeyCode::Down {
                                    app.settings_active_field = (app.settings_active_field + 1) % 2;
                                } else if key.code == KeyCode::BackTab || key.code == KeyCode::Up {
                                    app.settings_active_field =
                                        if app.settings_active_field == 0 { 1 } else { 0 };
                                } else {
                                    match app.settings_active_field {
                                        0 => {
                                            app.settings_password_new.input(key);
                                        }
                                        1 => {
                                            app.settings_password_confirm.input(key);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            1 => match key.code {
                                KeyCode::Left
                                | KeyCode::Char('h')
                                | KeyCode::Down
                                | KeyCode::Char('j') => {
                                    app.temp_timeout_mins = app.temp_timeout_mins.saturating_sub(1);
                                }
                                KeyCode::Right
                                | KeyCode::Char('l')
                                | KeyCode::Up
                                | KeyCode::Char('k') => {
                                    app.temp_timeout_mins = app.temp_timeout_mins.saturating_add(1);
                                }
                                KeyCode::Esc => {
                                    app.mode = AppMode::List;
                                }
                                KeyCode::Char('s')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    app.journal.settings.autolock_timeout_mins =
                                        app.temp_timeout_mins;
                                    if let Err(e) = app.save_settings() {
                                        app.error_msg = Some(format!("Save failed: {}", e));
                                    } else {
                                        app.status_msg =
                                            Some("Inactivity timeout updated".to_string());
                                        app.mode = AppMode::List;
                                    }
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
                                | KeyCode::Char('k') => {
                                    app.temp_lock_on_suspend = !app.temp_lock_on_suspend;
                                }
                                KeyCode::Esc => {
                                    app.mode = AppMode::List;
                                }
                                KeyCode::Char('s')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    app.journal.settings.lock_on_suspend = app.temp_lock_on_suspend;
                                    if let Err(e) = app.save_settings() {
                                        app.error_msg = Some(format!("Save failed: {}", e));
                                    } else {
                                        app.status_msg =
                                            Some("PC lock settings updated".to_string());
                                        app.mode = AppMode::List;
                                    }
                                }
                                _ => {}
                            },
                            3 => match key.code {
                                KeyCode::Up | KeyCode::Down | KeyCode::Tab | KeyCode::BackTab => {
                                    app.settings_active_field =
                                        if app.settings_active_field == 0 { 1 } else { 0 };
                                }
                                KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('j') => {
                                    if app.settings_active_field == 0 {
                                        if app.settings_num_shares > 1 {
                                            app.settings_num_shares -= 1;
                                            if app.settings_threshold > app.settings_num_shares {
                                                app.settings_threshold = app.settings_num_shares;
                                            }
                                        }
                                    } else {
                                        if app.settings_threshold > 1 {
                                            app.settings_threshold -= 1;
                                        }
                                    }
                                }
                                KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('k') => {
                                    if app.settings_active_field == 0 {
                                        if app.settings_num_shares < 255 {
                                            app.settings_num_shares += 1;
                                        }
                                    } else {
                                        if app.settings_threshold < app.settings_num_shares {
                                            app.settings_threshold += 1;
                                        }
                                    }
                                }
                                KeyCode::Esc => {
                                    app.mode = AppMode::List;
                                }
                                KeyCode::Char('s')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    match crate::crypto::split_password(
                                        &app.password,
                                        app.settings_threshold,
                                        app.settings_num_shares,
                                    ) {
                                        Ok(shares) => {
                                            app.generated_shares = shares;
                                            app.status_msg = Some(
                                                "Recovery shares generated successfully!"
                                                    .to_string(),
                                            );
                                            app.mode = AppMode::List;
                                        }
                                        Err(e) => {
                                            app.error_msg =
                                                Some(format!("Failed to generate shares: {}", e));
                                        }
                                    }
                                }
                                _ => {}
                            },
                            _ => {}
                        },
                    },

                    AppMode::ContactPicker {
                        is_edit,
                        selected_contact_index,
                    } => match key.code {
                        KeyCode::Esc => {
                            app.mode = AppMode::Writing { is_edit };
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let len = app.journal.contacts.len();
                            if len > 0 {
                                let next_idx = if selected_contact_index > 0 {
                                    selected_contact_index - 1
                                } else {
                                    len - 1
                                };
                                app.mode = AppMode::ContactPicker {
                                    is_edit,
                                    selected_contact_index: next_idx,
                                };
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let len = app.journal.contacts.len();
                            if len > 0 {
                                let next_idx = (selected_contact_index + 1) % len;
                                app.mode = AppMode::ContactPicker {
                                    is_edit,
                                    selected_contact_index: next_idx,
                                };
                            }
                        }
                        KeyCode::Enter => {
                            let len = app.journal.contacts.len();
                            if len > 0 && selected_contact_index < len {
                                let id = &app.journal.contacts[selected_contact_index].id;
                                app.textarea.insert_str(format!("{{{{person|{}}}}}", id));
                            }
                            app.mode = AppMode::Writing { is_edit };
                        }
                        _ => {}
                    },
                    AppMode::DatePicker {
                        is_edit,
                        field_index,
                        current_date,
                    } => match key.code {
                        KeyCode::Esc => {
                            app.mode = AppMode::Writing { is_edit };
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            if field_index == 0 {
                                app.contact_form_birthdate = ratatui_textarea::TextArea::default();
                            } else {
                                app.contact_form_deathdate = ratatui_textarea::TextArea::default();
                            }
                            app.mode = AppMode::Writing { is_edit };
                        }
                        KeyCode::Enter => {
                            let formatted = current_date.format("%Y-%m-%d").to_string();
                            if field_index == 0 {
                                app.contact_form_birthdate =
                                    ratatui_textarea::TextArea::new(vec![formatted]);
                            } else {
                                app.contact_form_deathdate =
                                    ratatui_textarea::TextArea::new(vec![formatted]);
                            }
                            app.mode = AppMode::Writing { is_edit };
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            let next_date = current_date - chrono::Duration::days(1);
                            app.mode = AppMode::DatePicker {
                                is_edit,
                                field_index,
                                current_date: next_date,
                            };
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            let next_date = current_date + chrono::Duration::days(1);
                            app.mode = AppMode::DatePicker {
                                is_edit,
                                field_index,
                                current_date: next_date,
                            };
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let next_date = current_date - chrono::Duration::days(7);
                            app.mode = AppMode::DatePicker {
                                is_edit,
                                field_index,
                                current_date: next_date,
                            };
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let next_date = current_date + chrono::Duration::days(7);
                            app.mode = AppMode::DatePicker {
                                is_edit,
                                field_index,
                                current_date: next_date,
                            };
                        }
                        KeyCode::PageUp | KeyCode::Char('[') => {
                            let next_date = current_date
                                .checked_sub_months(chrono::Months::new(1))
                                .unwrap_or(current_date);
                            app.mode = AppMode::DatePicker {
                                is_edit,
                                field_index,
                                current_date: next_date,
                            };
                        }
                        KeyCode::PageDown | KeyCode::Char(']') => {
                            let next_date = current_date
                                .checked_add_months(chrono::Months::new(1))
                                .unwrap_or(current_date);
                            app.mode = AppMode::DatePicker {
                                is_edit,
                                field_index,
                                current_date: next_date,
                            };
                        }
                        KeyCode::Home | KeyCode::Char('{') => {
                            let next_date = current_date
                                .checked_sub_months(chrono::Months::new(12))
                                .unwrap_or(current_date);
                            app.mode = AppMode::DatePicker {
                                is_edit,
                                field_index,
                                current_date: next_date,
                            };
                        }
                        KeyCode::End | KeyCode::Char('}') => {
                            let next_date = current_date
                                .checked_add_months(chrono::Months::new(12))
                                .unwrap_or(current_date);
                            app.mode = AppMode::DatePicker {
                                is_edit,
                                field_index,
                                current_date: next_date,
                            };
                        }
                        _ => {}
                    },
                    AppMode::DeleteConfirm => match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            match app.active_tab {
                                Tab::Journal => app.delete_selected_entry(),
                                Tab::Contacts => app.delete_selected_contact(),
                                Tab::Settings => {}
                            };
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            app.mode = AppMode::List;
                        }
                        _ => {}
                    },
                    AppMode::Login => match key.code {
                        KeyCode::Enter => {
                            app.error_msg = None;
                            let path = std::path::Path::new(&app.file_path);
                            match crate::journal::Journal::load(path, &app.login_password) {
                                Ok((j, s)) => {
                                    app.journal = j;
                                    app.salt = s;
                                    app.password = app.login_password.clone();
                                    app.mode = AppMode::List;
                                    app.sort_entries();
                                    app.sort_contacts();
                                    app.status_msg =
                                        Some("Journal successfully decrypted!".to_string());
                                }
                                Err(e) => {
                                    app.error_msg = Some(format!("Decryption failed: {}", e));
                                    app.login_password.clear();
                                }
                            }
                        }
                        KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.mode = AppMode::Recovery;
                            app.recovery_shares.clear();
                            app.recovery_textarea = ratatui_textarea::TextArea::default();
                            app.recovery_status_msg = None;
                            app.error_msg = None;
                        }
                        KeyCode::Char(c) => {
                            app.login_password.push(c);
                        }
                        KeyCode::Backspace => {
                            app.login_password.pop();
                        }
                        _ => {}
                    },
                    AppMode::Recovery => match key.code {
                        KeyCode::Enter => {
                            app.error_msg = None;
                            app.recovery_status_msg = None;
                            let share_str =
                                app.recovery_textarea.lines().join("").trim().to_string();
                            if share_str.is_empty() {
                                app.error_msg = Some("Please enter a recovery share".to_string());
                            } else {
                                match crate::crypto::parse_share(&share_str) {
                                    Ok(parsed) => {
                                        let mut already_entered = false;
                                        for s in &app.recovery_shares {
                                            if let Ok(p) = crate::crypto::parse_share(s)
                                                && p.index == parsed.index
                                            {
                                                already_entered = true;
                                                break;
                                            }
                                        }
                                        if already_entered {
                                            app.error_msg = Some(format!(
                                                "Share with index {} was already entered",
                                                parsed.index
                                            ));
                                        } else {
                                            app.recovery_shares.push(share_str);
                                            app.recovery_textarea =
                                                ratatui_textarea::TextArea::default();
                                            app.recovery_status_msg = Some(format!(
                                                "Successfully added Share {}.",
                                                parsed.index
                                            ));

                                            if app.recovery_shares.len() >= parsed.threshold {
                                                app.recovery_status_msg = Some(
                                                    "Threshold met! Reconstructing password..."
                                                        .to_string(),
                                                );
                                                match crate::crypto::reconstruct_password(
                                                    &app.recovery_shares,
                                                ) {
                                                    Ok(reconstructed_pwd) => {
                                                        let path =
                                                            std::path::Path::new(&app.file_path);
                                                        match crate::journal::Journal::load(
                                                            path,
                                                            &reconstructed_pwd,
                                                        ) {
                                                            Ok((j, s)) => {
                                                                app.journal = j;
                                                                app.salt = s;
                                                                app.password = reconstructed_pwd;
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
                                                    Err(e) => {
                                                        app.error_msg = Some(format!(
                                                            "Reconstruction failed: {}",
                                                            e
                                                        ));
                                                        app.recovery_shares.clear();
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        app.error_msg =
                                            Some(format!("Invalid recovery share format: {}", e));
                                    }
                                }
                            }
                        }
                        KeyCode::Esc => {
                            app.mode = AppMode::Login;
                            app.error_msg = None;
                            app.recovery_status_msg = None;
                            app.login_password.clear();
                        }
                        _ => {
                            app.recovery_textarea.input(key);
                        }
                    },
                    AppMode::RecoveryReset => match key.code {
                        KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            match app.handle_change_password() {
                                Ok(_) => {
                                    app.status_msg = Some(
                                        "Password successfully set! Database re-encrypted."
                                            .to_string(),
                                    );
                                    app.error_msg = None;
                                    app.mode = AppMode::List;
                                    app.sort_entries();
                                    app.sort_contacts();
                                }
                                Err(e) => {
                                    app.error_msg = Some(e);
                                }
                            }
                        }
                        KeyCode::Tab | KeyCode::Down => {
                            app.settings_active_field = (app.settings_active_field + 1) % 2;
                        }
                        KeyCode::BackTab | KeyCode::Up => {
                            app.settings_active_field =
                                if app.settings_active_field == 0 { 1 } else { 0 };
                        }
                        _ => match app.settings_active_field {
                            0 => {
                                app.settings_password_new.input(key);
                            }
                            1 => {
                                app.settings_password_confirm.input(key);
                            }
                            _ => {}
                        },
                    },
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn is_workstation_locked() -> bool {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn OpenInputDesktop(dwFlags: u32, fInherit: i32, dwDesiredAccess: u32) -> isize;
        fn CloseDesktop(hDesktop: isize) -> i32;
    }

    let h = unsafe { OpenInputDesktop(0, 0, 0) };
    if h == 0 {
        true
    } else {
        unsafe {
            CloseDesktop(h);
        }
        false
    }
}

#[cfg(not(target_os = "windows"))]
fn is_workstation_locked() -> bool {
    false
}
