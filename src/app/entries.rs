//! App state controller implementations for journal entry actions (saving and deleting).

use super::{App, AppMode};
use crate::model::JournalEntry;
use chrono::Utc;
use uuid::Uuid;

impl App {
    /// Saves the entry currently in the text area (creating or updating), persisting it to disk.
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
                if let Some(entry) = self
                    .selected_entry_idx()
                    .and_then(|idx| self.journal.entries.get_mut(idx))
                {
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

    /// Deletes the currently selected journal entry, updates selection indices, and persists changes to disk.
    pub fn delete_selected_entry(&mut self) {
        let real_idx = match self.selected_entry_idx() {
            Some(idx) => idx,
            None => {
                self.mode = AppMode::List;
                return;
            }
        };

        self.journal.entries.remove(real_idx);

        if let Err(e) = self.save_journal() {
            self.error_msg = Some(format!("Delete write failed: {}", e));
        } else {
            self.status_msg = Some("Entry deleted".to_string());
            self.error_msg = None;
        }

        let len = self.filtered_entries().len();
        if len == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= len {
            self.selected_index = len - 1;
        }

        self.mode = AppMode::List;
        self.detail_scroll = 0;
    }
}
