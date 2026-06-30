//! The main entry point for the encrypted journal CLI.
//!
//! Orchestrates the terminal alternate screen setup, user password prompt loop,
//! auto-lock timeouts, workstation suspend locking (Windows), and the central TUI event loop.

mod app;
mod crypto;
mod input;
mod model;
mod ui;

use app::{App, AppMode};
use clap::{Parser, ValueHint};
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use model::Journal;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::path::{Path, PathBuf};

const DEFAULT_JOURNAL_PATH: &str = "journal.jrnl";

/// Encrypted personal journal, right in your terminal.
#[derive(Parser)]
#[command(name = "journal-cli", version, about, long_about = None)]
struct Cli {
    /// Path to the journal file. A new encrypted journal is created here if it doesn't exist yet.
    #[arg(default_value = DEFAULT_JOURNAL_PATH, value_hint = ValueHint::FilePath)]
    journal_path: PathBuf,
}

type JournalState = (Journal, [u8; crypto::SALT_SIZE], String, bool);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let (journal, salt, password, start_in_login) = match load_or_create_journal(&cli.journal_path)
    {
        Ok(state) => state,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    let journal_path_str = cli.journal_path.to_string_lossy().into_owned();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(journal, journal_path_str, password, salt);
    if start_in_login {
        app.mode = AppMode::Login;
    }
    let run_result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = run_result {
        eprintln!("Error: {e}");
    }

    Ok(())
}

/// Loads an existing journal's login state, or interactively creates a new one at `path`.
fn load_or_create_journal(path: &Path) -> Result<JournalState, String> {
    if path.exists() {
        return Ok((
            Journal::default(),
            [0u8; crypto::SALT_SIZE],
            String::new(),
            true,
        ));
    }

    println!("No journal found at '{}'.", path.display());
    println!("Creating a new encrypted journal there.");
    let password = prompt_new_password().map_err(|e| e.to_string())?;
    let (journal, salt) = Journal::create_new(path, &password)?;
    Ok((journal, salt, password, false))
}

/// Prompts the user to set and confirm a new master password via stdin/stderr.
fn prompt_new_password() -> Result<String, Box<dyn std::error::Error>> {
    loop {
        let p1 = rpassword::prompt_password("Set master password: ")?;
        let p2 = rpassword::prompt_password("Confirm master password: ")?;
        if p1 != p2 {
            println!("Passwords do not match, try again.");
            continue;
        }
        if p1.trim().is_empty() {
            println!("Password cannot be empty, try again.");
            continue;
        }
        return Ok(p1);
    }
}

/// Executes the core event loop, processing keys and drawing the interface.
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

        if app.redraw_requested {
            terminal.clear()?;
            app.redraw_requested = false;
        }

        terminal.draw(|f| ui::draw(f, app))?;

        #[cfg(target_os = "windows")]
        if app.journal.settings.lock_on_suspend && is_workstation_locked() {
            app.should_quit = true;
            break;
        }

        if app.journal.settings.autolock_timeout_mins > 0 {
            let timeout = std::time::Duration::from_secs(
                app.journal.settings.autolock_timeout_mins as u64 * 60,
            );
            if last_activity.elapsed() >= timeout {
                app.should_quit = true;
                break;
            }
        }

        if event::poll(std::time::Duration::from_millis(500))?
            && let Event::Key(key) = event::read()?
        {
            last_activity = std::time::Instant::now();
            // crossterm on Windows also sends release events; only react to presses.
            if key.kind == event::KeyEventKind::Press {
                input::handle_key(app, key);
            }
        }
    }
    Ok(())
}

/// Detects if the current Windows user session is locked.
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
