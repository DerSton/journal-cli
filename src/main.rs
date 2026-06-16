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
use app::{App, AppMode};

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
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.should_quit = true;
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if !app.journal.entries.is_empty() && app.selected_index > 0 {
                                    app.selected_index -= 1;
                                    app.detail_scroll = 0;
                                    app.status_msg = None;
                                    app.error_msg = None;
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if !app.journal.entries.is_empty() && app.selected_index < app.journal.entries.len() - 1 {
                                    app.selected_index += 1;
                                    app.detail_scroll = 0;
                                    app.status_msg = None;
                                    app.error_msg = None;
                                }
                            }
                            KeyCode::PageUp => {
                                app.detail_scroll = app.detail_scroll.saturating_sub(1);
                            }
                            KeyCode::PageDown => {
                                app.detail_scroll = app.detail_scroll.saturating_add(1);
                            }
                            KeyCode::Char('n') => {
                                app.textarea = ratatui_textarea::TextArea::default();
                                app.mode = AppMode::Writing { is_edit: false };
                                app.status_msg = None;
                                app.error_msg = None;
                            }
                            KeyCode::Char('e') => {
                                if !app.journal.entries.is_empty() {
                                    let content = &app.journal.entries[app.selected_index].content;
                                    app.textarea = ratatui_textarea::TextArea::new(
                                        content.lines().map(String::from).collect()
                                    );
                                    app.mode = AppMode::Writing { is_edit: true };
                                    app.status_msg = None;
                                    app.error_msg = None;
                                }
                            }
                            KeyCode::Char('d') | KeyCode::Delete => {
                                if !app.journal.entries.is_empty() {
                                    app.mode = AppMode::DeleteConfirm;
                                    app.status_msg = None;
                                    app.error_msg = None;
                                }
                            }
                            _ => {}
                        },
                        AppMode::Writing { .. } => {
                            if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
                                app.handle_save_entry();
                            } else if key.code == KeyCode::Esc {
                                app.mode = AppMode::List;
                            } else {
                                app.textarea.input(key);
                            }
                        }
                        AppMode::DeleteConfirm => match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                app.delete_selected_entry();
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
