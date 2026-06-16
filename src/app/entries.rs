use super::{App, AppMode};
use crate::model::JournalEntry;
use chrono::Utc;
use uuid::Uuid;

impl App {
    /// Saves the entry currently in the text area (creating or updating), persisting to disk.
    pub fn save_entry(&mut self) {
        let content = self.textarea.lines().join("\n");
        if content.trim().is_empty() {
            self.error_msg = Some("Error: Entry content cannot be empty".to_string());
            return;
        }

        match self.mode {
            AppMode::Writing { is_edit: false } => {
                self.journal.entries.push(JournalEntry {
                    id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    content,
                });
                self.sort_entries();
                self.selected_index = 0;
                self.status_msg = Some("New entry saved".to_string());
            }
            AppMode::Writing { is_edit: true } => {
                if let Some(entry) = self.journal.entries.get_mut(self.selected_index) {
                    entry.content = content;
                    self.status_msg = Some("Entry updated".to_string());
                }
            }
            _ => return,
        }

        if let Err(e) = self.save_journal() {
            self.error_msg = Some(format!("Write failed: {}", e));
        } else {
            self.mode = AppMode::List;
            self.detail_scroll = 0;
            self.error_msg = None;
        }
    }

    pub fn delete_selected_entry(&mut self) {
        if self.journal.entries.is_empty() || self.selected_index >= self.journal.entries.len() {
            self.mode = AppMode::List;
            return;
        }

        self.journal.entries.remove(self.selected_index);

        if let Err(e) = self.save_journal() {
            self.error_msg = Some(format!("Delete write failed: {}", e));
        } else {
            self.status_msg = Some("Entry deleted".to_string());
            self.error_msg = None;
        }

        if self.journal.entries.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.journal.entries.len() {
            self.selected_index = self.journal.entries.len() - 1;
        }

        self.mode = AppMode::List;
        self.detail_scroll = 0;
    }
}
