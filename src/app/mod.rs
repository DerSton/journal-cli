mod contact_form;
mod date_utils;
mod entries;
mod settings_actions;

pub use contact_form::{ContactField, ContactForm};
pub use date_utils::{format_localized_date, get_date_format_info, parse_localized_date};

use crate::crypto::SALT_SIZE;
use crate::model::Journal;
use ratatui_textarea::TextArea;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Journal,
    Contacts,
    Stats,
    Settings,
}

/// Settings tab list rows, in display order.
pub const SETTINGS_GROUPS: &[&str] = &[
    "Change Password",
    "Inactivity Timeout",
    "Lock on Suspend",
    "Recovery Shares",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Browsing a list (journal entries, contacts, or settings groups).
    List,
    /// Editing a journal entry or the contact form.
    Writing {
        is_edit: bool,
    },
    /// Picking a contact to insert as a `{{person|id}}` mention.
    ContactPicker {
        is_edit: bool,
        selected_contact_index: usize,
    },
    /// Calendar overlay for a date field. `field_index` 0 = birthdate, 1 = date of death.
    DatePicker {
        is_edit: bool,
        field_index: usize,
        current_date: chrono::NaiveDate,
    },
    DeleteConfirm,
    Login,
    Recovery,
    RecoveryReset,
    Search,
}

pub struct App {
    pub journal: Journal,
    pub file_path: String,
    pub password: String,
    pub salt: [u8; SALT_SIZE],

    pub active_tab: Tab,
    /// Highlighted row in the active tab's list (entry, contact, or settings group).
    pub selected_index: usize,
    pub mode: AppMode,

    /// Journal entry editor.
    pub textarea: TextArea<'static>,
    /// Scroll offset for the journal entry preview pane.
    pub detail_scroll: u16,

    /// Contact create/edit form state.
    pub contact_form: ContactForm,

    /// Whether keys go to the settings right-hand panel (true) or the group list (false).
    pub settings_panel_focused: bool,
    /// Active field within the focused settings panel (0/1, meaning depends on the group).
    pub settings_active_field: usize,
    pub settings_password_new: TextArea<'static>,
    pub settings_password_confirm: TextArea<'static>,
    pub settings_num_shares: usize,
    pub settings_threshold: usize,
    pub generated_shares: Vec<String>,

    pub recovery_shares: Vec<String>,
    pub recovery_status_msg: Option<String>,
    pub recovery_textarea: TextArea<'static>,
    pub login_password: String,

    pub error_msg: Option<String>,
    pub status_msg: Option<String>,
    pub should_quit: bool,
    pub search_query: String,
}

impl App {
    pub fn new(
        journal: Journal,
        file_path: String,
        password: String,
        salt: [u8; SALT_SIZE],
    ) -> Self {
        let mut app = Self {
            journal,
            file_path,
            password,
            salt,
            active_tab: Tab::Journal,
            selected_index: 0,
            mode: AppMode::List,
            textarea: TextArea::default(),
            detail_scroll: 0,
            contact_form: ContactForm::empty(),
            settings_panel_focused: false,
            settings_active_field: 0,
            settings_password_new: TextArea::default(),
            settings_password_confirm: TextArea::default(),
            settings_num_shares: 5,
            settings_threshold: 3,
            generated_shares: Vec::new(),
            recovery_shares: Vec::new(),
            recovery_status_msg: None,
            recovery_textarea: TextArea::default(),
            login_password: String::new(),
            error_msg: None,
            status_msg: Some("Welcome to your secure journal.".to_string()),
            should_quit: false,
            search_query: String::new(),
        };
        app.sort_entries();
        app.sort_contacts();
        app
    }

    /// Newest entries first.
    pub fn sort_entries(&mut self) {
        self.journal
            .entries
            .sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    }

    /// Alphabetical by last name, then given names.
    pub fn sort_contacts(&mut self) {
        self.journal.contacts.sort_by(|a, b| {
            let last_cmp = a.last_name.to_lowercase().cmp(&b.last_name.to_lowercase());
            if last_cmp == std::cmp::Ordering::Equal {
                a.first_names
                    .join(" ")
                    .to_lowercase()
                    .cmp(&b.first_names.join(" ").to_lowercase())
            } else {
                last_cmp
            }
        });
    }

    pub fn switch_tab(&mut self, new_tab: Tab) {
        self.active_tab = new_tab;
        self.selected_index = 0;
        self.detail_scroll = 0;
        self.settings_panel_focused = false;
        self.settings_active_field = 0;
        self.status_msg = None;
        self.error_msg = None;
        self.mode = AppMode::List;
    }

    pub fn list_len(&self) -> usize {
        match self.active_tab {
            Tab::Journal => self.filtered_entries().len(),
            Tab::Contacts => self.filtered_contacts().len(),
            Tab::Settings => SETTINGS_GROUPS.len(),
            Tab::Stats => 0,
        }
    }

    pub fn filtered_entries(&self) -> Vec<&crate::model::JournalEntry> {
        if self.search_query.trim().is_empty() {
            self.journal.entries.iter().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.journal
                .entries
                .iter()
                .filter(|entry| entry.content.to_lowercase().contains(&query))
                .collect()
        }
    }

    pub fn filtered_contacts(&self) -> Vec<&crate::model::Contact> {
        if self.search_query.trim().is_empty() {
            self.journal.contacts.iter().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.journal
                .contacts
                .iter()
                .filter(|c| {
                    c.full_name().to_lowercase().contains(&query)
                        || c.nickname.to_lowercase().contains(&query)
                        || c.notes.to_lowercase().contains(&query)
                })
                .collect()
        }
    }

    pub fn selected_entry_idx(&self) -> Option<usize> {
        let filtered = self.filtered_entries();
        if filtered.is_empty() {
            None
        } else {
            let selected = filtered.get(self.selected_index)?;
            self.journal
                .entries
                .iter()
                .position(|e| e.id == selected.id)
        }
    }

    pub fn selected_contact_idx(&self) -> Option<usize> {
        let filtered = self.filtered_contacts();
        if filtered.is_empty() {
            None
        } else {
            let selected = filtered.get(self.selected_index)?;
            self.journal
                .contacts
                .iter()
                .position(|c| c.id == selected.id)
        }
    }

    /// Persist the journal under the current password and salt.
    pub fn save_journal(&mut self) -> Result<(), String> {
        self.journal
            .save(&self.file_path, &self.password, &self.salt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Contact, JournalEntry};
    use chrono::Utc;

    fn test_app() -> App {
        App::new(
            Journal::default(),
            "dummy.jrnl".to_string(),
            "password".to_string(),
            [0u8; crate::crypto::SALT_SIZE],
        )
    }

    #[test]
    fn test_filtering_and_indexing() {
        let mut app = test_app();
        app.journal.entries = vec![
            JournalEntry {
                id: "1".to_string(),
                timestamp: Utc::now(),
                content: "First entry with rust".to_string(),
            },
            JournalEntry {
                id: "2".to_string(),
                timestamp: Utc::now(),
                content: "Second entry with python".to_string(),
            },
            JournalEntry {
                id: "3".to_string(),
                timestamp: Utc::now(),
                content: "Third entry with rust programming".to_string(),
            },
        ];

        // Without query, all entries should be shown in original order
        app.search_query = String::new();
        assert_eq!(app.filtered_entries().len(), 3);

        // With query "rust", 2 entries should be shown
        app.search_query = "rust".to_string();
        let filtered = app.filtered_entries();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, "1");
        assert_eq!(filtered[1].id, "3");

        // Test selected_entry_idx resolves correct master index
        // If we select the second filtered item (index 1, which has ID "3")
        app.selected_index = 1;
        assert_eq!(app.selected_entry_idx(), Some(2)); // maps to master index 2 (ID "3")

        // If we select the first filtered item (index 0, which has ID "1")
        app.selected_index = 0;
        assert_eq!(app.selected_entry_idx(), Some(0)); // maps to master index 0 (ID "1")
    }

    #[test]
    fn test_contact_filtering_and_indexing() {
        let mut app = test_app();
        app.journal.contacts = vec![
            Contact {
                id: "1".to_string(),
                first_names: vec!["Alice".to_string()],
                last_name: "Smith".to_string(),
                nickname: "Ali".to_string(),
                ..Default::default()
            },
            Contact {
                id: "2".to_string(),
                first_names: vec!["Bob".to_string()],
                last_name: "Jones".to_string(),
                ..Default::default()
            },
        ];

        app.search_query = "ali".to_string();
        let filtered = app.filtered_contacts();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "1");

        app.selected_index = 0;
        assert_eq!(app.selected_contact_idx(), Some(0));
    }
}
