//! App state controller implementations for settings panel actions.

use super::App;
use crate::crypto::{self, SALT_SIZE};
use rand::random;
use ratatui_textarea::TextArea;

impl App {
    /// Adjusts the inactivity autolock timeout (in minutes) and saves immediately. A value of 0 disables it.
    pub fn adjust_autolock_timeout(&mut self, delta: i32) {
        let current = self.journal.settings.autolock_timeout_mins as i32;
        self.journal.settings.autolock_timeout_mins = (current + delta).max(0) as u32;
        self.persist_settings();
    }

    /// Flips the lock-on-suspend flag and saves immediately.
    pub fn toggle_lock_on_suspend(&mut self) {
        self.journal.settings.lock_on_suspend = !self.journal.settings.lock_on_suspend;
        self.persist_settings();
    }

    /// Helper to persist current settings changes to the encrypted journal file.
    fn persist_settings(&mut self) {
        if let Err(e) = self.save_journal() {
            self.error_msg = Some(format!("Save failed: {}", e));
        } else {
            self.status_msg = Some("Settings saved".to_string());
        }
    }

    /// Transactionally re-encrypts the journal under a new master password and updates in-memory credentials.
    ///
    /// Writes to a temporary `.tmp` file first, then renames it on success to prevent data loss on write failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The new password is empty.
    /// - The confirmation password does not match.
    /// - File creation, encryption, or rename operations fail.
    pub fn change_password(&mut self) -> Result<(), String> {
        let new_pw = self.settings_password_new.lines().join("");
        let confirm_pw = self.settings_password_confirm.lines().join("");

        if new_pw.is_empty() {
            return Err("New password cannot be empty".to_string());
        }
        if new_pw != confirm_pw {
            return Err("Passwords do not match".to_string());
        }

        let new_salt: [u8; SALT_SIZE] = random();
        let tmp_path = format!("{}.tmp", self.file_path);

        if let Err(e) = self.journal.save(&tmp_path, &new_pw, &new_salt) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("Failed to write encrypted file: {}", e));
        }
        if let Err(e) = std::fs::rename(&tmp_path, &self.file_path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("Failed to finalize new password file: {}", e));
        }

        self.password = new_pw;
        self.salt = new_salt;
        self.settings_password_new = TextArea::default();
        self.settings_password_confirm = TextArea::default();
        self.settings_active_field = 0;

        Ok(())
    }

    /// Splits the master password into recovery shares using Shamir's Secret Sharing (SSS).
    ///
    /// # Errors
    ///
    /// Returns an error if SSS key generation thresholds are invalid.
    pub fn generate_recovery_shares(&mut self) -> Result<(), String> {
        let shares = crypto::split_password(
            &self.password,
            self.settings_threshold,
            self.settings_num_shares,
        )?;
        self.generated_shares = shares;
        self.status_msg = Some("Recovery shares generated".to_string());
        Ok(())
    }

    /// Exports the generated recovery shares to a text file.
    pub fn export_recovery_shares(&mut self) {
        if self.generated_shares.is_empty() {
            self.error_msg = Some("No recovery shares generated yet".to_string());
            return;
        }

        let mut content = String::new();
        content.push_str("=== journal-cli Recovery Shares ===\n");
        content.push_str(
            "Keep these shares separate! Any T of them can reconstruct your master password.\n\n",
        );
        for (idx, share) in self.generated_shares.iter().enumerate() {
            content.push_str(&format!("Share {}: {}\n", idx + 1, share));
        }

        // Suspend TUI for native dialog.
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);

        let dest = std::thread::spawn(move || {
            rfd::FileDialog::new()
                .set_title("Export recovery shares")
                .set_file_name("journal_recovery_shares.txt")
                .add_filter("Text", &["txt"])
                .save_file()
        })
        .join()
        .unwrap_or(None);

        // Restore TUI.
        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen);
        let _ = crossterm::terminal::enable_raw_mode();
        self.redraw_requested = true;

        let path = match dest {
            Some(p) => p,
            None => {
                self.status_msg = Some("Export cancelled".to_string());
                return;
            }
        };

        match std::fs::write(&path, content.as_bytes()) {
            Ok(_) => {
                self.status_msg = Some(format!("Exported shares to {}", path.display()));
                self.error_msg = None;
            }
            Err(e) => {
                self.error_msg = Some(format!("Export failed: {}", e));
            }
        }
    }
}
