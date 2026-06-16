mod app;
mod crypto;
mod input;
mod model;
mod ui;

use app::{App, AppMode};
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use model::Journal;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::env;
use std::io;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: journal-cli <path_to_journal_file>");
        std::process::exit(1);
    }
    let journal_path_str = &args[1];
    let journal_path = Path::new(journal_path_str);

    let (journal, salt, password, start_in_login) = if journal_path.exists() {
        (
            Journal::default(),
            [0u8; crypto::SALT_SIZE],
            String::new(),
            true,
        )
    } else {
        println!(
            "No journal file found at '{}'. Setting up a new one.",
            journal_path_str
        );
        let password = prompt_new_password()?;
        match Journal::create_new(journal_path, &password) {
            Ok((j, s)) => (j, s, password, false),
            Err(e) => {
                eprintln!("Failed to create new journal: {}", e);
                std::process::exit(1);
            }
        }
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(journal, journal_path_str.clone(), password, salt);
    if start_in_login {
        app.mode = AppMode::Login;
    }
    let run_result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = run_result {
        eprintln!("Application error: {}", e);
    }

    Ok(())
}

fn prompt_new_password() -> Result<String, Box<dyn std::error::Error>> {
    loop {
        let p1 = rpassword::prompt_password("Set Master Password: ")?;
        let p2 = rpassword::prompt_password("Confirm Master Password: ")?;
        if p1 != p2 {
            println!("Passwords do not match. Try again.");
            continue;
        }
        if p1.trim().is_empty() {
            println!("Password cannot be empty. Try again.");
            continue;
        }
        return Ok(p1);
    }
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
