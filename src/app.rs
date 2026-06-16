use crate::journal::{Journal, JournalEntry};
use crate::crypto::SALT_SIZE;
use chrono::Utc;
use ratatui_textarea::TextArea;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    List,
    Writing { is_edit: bool },
    DeleteConfirm,
}

pub struct App {
    /// The journal database containing all entries.
    pub journal: Journal,
    /// Path to the journal file on disk.
    pub file_path: String,
    /// Password in memory for encrypting changes.
    pub password: String,
    /// Salt in memory for key derivation.
    pub salt: [u8; SALT_SIZE],

    /// Index of the currently highlighted entry in the list.
    pub selected_index: usize,
    /// Current view/interaction mode.
    pub mode: AppMode,
    /// Text editing component.
    pub textarea: TextArea<'static>,
    /// Global error toast message.
    pub error_msg: Option<String>,
    /// Global status notification message.
    pub status_msg: Option<String>,
    /// Flag indicating whether the TUI should exit.
    pub should_quit: bool,
    /// Vertical scroll offset for the entry preview pane.
    pub detail_scroll: u16,
}

impl App {
    /// Create a new application instance, sorting entries from newest to oldest.
    pub fn new(journal: Journal, file_path: String, password: String, salt: [u8; SALT_SIZE]) -> Self {
        let mut app = Self {
            journal,
            file_path,
            password,
            salt,
            selected_index: 0,
            mode: AppMode::List,
            textarea: TextArea::default(),
            error_msg: None,
            status_msg: Some("Welcome to your secure journal CLI!".to_string()),
            should_quit: false,
            detail_scroll: 0,
        };
        app.sort_entries();
        app
    }

    /// Sort entries in-place: newest (latest timestamp) to oldest (earliest timestamp).
    pub fn sort_entries(&mut self) {
        self.journal.entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    }

    /// Save the entry currently in the text area (creating or updating).
    pub fn handle_save_entry(&mut self) {
        let content = self.textarea.lines().join("\n");
        if content.trim().is_empty() {
            self.error_msg = Some("Error: Entry content cannot be empty".to_string());
            return;
        }

        match self.mode {
            AppMode::Writing { is_edit: false } => {
                let new_entry = JournalEntry {
                    id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    content,
                };
                self.journal.entries.push(new_entry);
                self.sort_entries();
                self.selected_index = 0; // The new entry is the newest, so it's at index 0.
                self.status_msg = Some("New entry saved".to_string());
            }
            AppMode::Writing { is_edit: true } => {
                if !self.journal.entries.is_empty() && self.selected_index < self.journal.entries.len() {
                    self.journal.entries[self.selected_index].content = content;
                    self.status_msg = Some("Entry updated".to_string());
                }
            }
            _ => return,
        }

        // Save to disk immediately
        if let Err(e) = self.journal.save(&self.file_path, &self.password, &self.salt) {
            self.error_msg = Some(format!("Write failed: {}", e));
        } else {
            self.mode = AppMode::List;
            self.detail_scroll = 0;
            self.error_msg = None;
        }
    }

    /// Delete the selected entry and save the changes.
    pub fn delete_selected_entry(&mut self) {
        if self.journal.entries.is_empty() || self.selected_index >= self.journal.entries.len() {
            self.mode = AppMode::List;
            return;
        }

        self.journal.entries.remove(self.selected_index);

        if let Err(e) = self.journal.save(&self.file_path, &self.password, &self.salt) {
            self.error_msg = Some(format!("Delete write failed: {}", e));
        } else {
            self.status_msg = Some("Entry deleted".to_string());
            self.error_msg = None;
        }

        // Adjust selection index bounds
        if self.journal.entries.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.journal.entries.len() {
            self.selected_index = self.journal.entries.len() - 1;
        }

        self.mode = AppMode::List;
        self.detail_scroll = 0;
    }
}
