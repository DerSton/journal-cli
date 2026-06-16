mod crypto;
mod journal;
mod app;
mod ui;

use std::env;
use std::io;
use std::path::Path;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use journal::Journal;
use app::{App, AppMode, Tab};

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
    let (journal, salt, password) = if journal_path.exists() {
        println!("Opening existing journal: {}", journal_path_str);
        let password = rpassword::prompt_password("Enter Master Password: ")?;
        
        match Journal::load(journal_path, &password) {
            Ok((j, s)) => (j, s, password),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("No journal file found at '{}'. Initializing a new secure journal.", journal_path_str);
        
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
            Ok((j, s)) => (j, s, password),
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
    loop {
        // Render current state
        terminal.draw(|f| ui::draw(f, app))?;

        // Poll for inputs
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
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
                                    Tab::Contacts => Tab::Journal,
                                };
                                app.switch_tab(next_tab);
                            }
                            KeyCode::Char('1') => {
                                app.switch_tab(Tab::Journal);
                            }
                            KeyCode::Char('2') => {
                                app.switch_tab(Tab::Contacts);
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
                                        app.contact_first_name = ratatui_textarea::TextArea::default();
                                        app.contact_middle_name = ratatui_textarea::TextArea::default();
                                        app.contact_last_name = ratatui_textarea::TextArea::default();
                                        app.active_field_index = 0;
                                        app.mode = AppMode::Writing { is_edit: false };
                                    }
                                }
                            }
                            KeyCode::Char('e') => {
                                app.status_msg = None;
                                app.error_msg = None;
                                match app.active_tab {
                                    Tab::Journal => {
                                        if !app.journal.entries.is_empty() {
                                            let content = &app.journal.entries[app.selected_index].content;
                                            app.textarea = ratatui_textarea::TextArea::new(
                                                content.lines().map(String::from).collect()
                                            );
                                            app.mode = AppMode::Writing { is_edit: true };
                                        }
                                    }
                                    Tab::Contacts => {
                                        if !app.journal.contacts.is_empty() {
                                            let contact = &app.journal.contacts[app.selected_index];
                                            app.contact_first_name = ratatui_textarea::TextArea::new(vec![contact.first_name.clone()]);
                                            app.contact_middle_name = ratatui_textarea::TextArea::new(vec![contact.middle_name.clone()]);
                                            app.contact_last_name = ratatui_textarea::TextArea::new(vec![contact.last_name.clone()]);
                                            app.active_field_index = 0;
                                            app.mode = AppMode::Writing { is_edit: true };
                                        }
                                    }
                                }
                            }
                            KeyCode::Char('d') | KeyCode::Delete | KeyCode::Esc => {
                                let is_empty = match app.active_tab {
                                    Tab::Journal => app.journal.entries.is_empty(),
                                    Tab::Contacts => app.journal.contacts.is_empty(),
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
                        AppMode::Writing { .. } => {
                            match app.active_tab {
                                Tab::Journal => {
                                    if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
                                        app.handle_save_entry();
                                    } else if key.code == KeyCode::Esc {
                                        app.mode = AppMode::List;
                                    } else {
                                        app.textarea.input(key);
                                    }
                                }
                                Tab::Contacts => {
                                    if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
                                        app.handle_save_contact();
                                    } else if key.code == KeyCode::Esc {
                                        app.mode = AppMode::List;
                                    } else if key.code == KeyCode::Tab || key.code == KeyCode::Down {
                                        app.active_field_index = (app.active_field_index + 1) % 3;
                                    } else if key.code == KeyCode::BackTab || key.code == KeyCode::Up {
                                        app.active_field_index = if app.active_field_index == 0 { 2 } else { app.active_field_index - 1 };
                                    } else {
                                        match app.active_field_index {
                                            0 => app.contact_first_name.input(key),
                                            1 => app.contact_middle_name.input(key),
                                            2 => app.contact_last_name.input(key),
                                            _ => false,
                                        };
                                    }
                                }
                            }
                        }
                        AppMode::DeleteConfirm => match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                match app.active_tab {
                                    Tab::Journal => app.delete_selected_entry(),
                                    Tab::Contacts => app.delete_selected_contact(),
                                };
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                app.mode = AppMode::List;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
