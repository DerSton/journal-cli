use crate::crypto::SALT_SIZE;
use crate::journal::{Contact, Journal, JournalEntry};
use chrono::Utc;
use ratatui_textarea::TextArea;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Journal,
    Contacts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    List,
    Writing {
        is_edit: bool,
    },
    ContactPicker {
        is_edit: bool,
        selected_contact_index: usize,
    },
    DeleteConfirm,
}

pub struct App {
    /// The journal database containing all entries and contacts.
    pub journal: Journal,
    /// Path to the journal file on disk.
    pub file_path: String,
    /// Password in memory for encrypting changes.
    pub password: String,
    /// Salt in memory for key derivation.
    pub salt: [u8; SALT_SIZE],

    /// Current active tab.
    pub active_tab: Tab,
    /// Index of the currently highlighted item (entry or contact) in the list.
    pub selected_index: usize,
    /// Current view/interaction mode.
    pub mode: AppMode,
    /// Text editing component for journal entries.
    pub textarea: TextArea<'static>,

    // Contact Form Fields
    pub contact_first_name: TextArea<'static>,
    pub contact_middle_name: TextArea<'static>,
    pub contact_last_name: TextArea<'static>,
    pub contact_handle: TextArea<'static>,
    pub contact_notes: TextArea<'static>,
    /// Active field index in the contact editor (0: First, 1: Middle, 2: Last, 3: Handle, 4: Notes).
    pub active_field_index: usize,

    /// Global error toast message.
    pub error_msg: Option<String>,
    /// Global status notification message.
    pub status_msg: Option<String>,
    /// Flag indicating whether the TUI should exit.
    pub should_quit: bool,
    /// Vertical scroll offset for the details pane.
    pub detail_scroll: u16,
    /// Whether the contact handle was manually edited.
    pub handle_edited: bool,
}

impl App {
    /// Create a new application instance, sorting entries and contacts.
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
            contact_first_name: TextArea::default(),
            contact_middle_name: TextArea::default(),
            contact_last_name: TextArea::default(),
            contact_handle: TextArea::default(),
            contact_notes: TextArea::default(),
            active_field_index: 0,
            error_msg: None,
            status_msg: Some("Welcome to your secure journal CLI!".to_string()),
            should_quit: false,
            detail_scroll: 0,
            handle_edited: false,
        };
        app.sort_entries();
        app.sort_contacts();
        app
    }

    /// Sort entries in-place: newest (latest timestamp) to oldest (earliest timestamp).
    pub fn sort_entries(&mut self) {
        self.journal
            .entries
            .sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    }

    /// Sort contacts in-place alphabetically: last name first, then first name.
    pub fn sort_contacts(&mut self) {
        self.journal.contacts.sort_by(|a, b| {
            let last_cmp = a.last_name.to_lowercase().cmp(&b.last_name.to_lowercase());
            if last_cmp == std::cmp::Ordering::Equal {
                a.first_name
                    .to_lowercase()
                    .cmp(&b.first_name.to_lowercase())
            } else {
                last_cmp
            }
        });
    }

    /// Safely switch to a new tab and reset cursor focus.
    pub fn switch_tab(&mut self, new_tab: Tab) {
        self.active_tab = new_tab;
        self.selected_index = 0;
        self.detail_scroll = 0;
        self.status_msg = None;
        self.error_msg = None;
        self.mode = AppMode::List;
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
                self.selected_index = 0;
                self.status_msg = Some("New entry saved".to_string());
            }
            AppMode::Writing { is_edit: true } => {
                if !self.journal.entries.is_empty()
                    && self.selected_index < self.journal.entries.len()
                {
                    self.journal.entries[self.selected_index].content = content;
                    self.status_msg = Some("Entry updated".to_string());
                }
            }
            _ => return,
        }

        // Save to disk immediately
        if let Err(e) = self
            .journal
            .save(&self.file_path, &self.password, &self.salt)
        {
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

        if let Err(e) = self
            .journal
            .save(&self.file_path, &self.password, &self.salt)
        {
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

    /// Save the contact form fields to the database.
    pub fn handle_save_contact(&mut self) {
        let first = self.contact_first_name.lines().join("").trim().to_string();
        let middle = self.contact_middle_name.lines().join("").trim().to_string();
        let last = self.contact_last_name.lines().join("").trim().to_string();
        let mut handle = self.contact_handle.lines().join("").trim().to_string();
        let notes = self.contact_notes.lines().join("\n").trim().to_string();

        if first.is_empty() && last.is_empty() {
            self.error_msg = Some("Error: First Name or Last Name must be provided".to_string());
            return;
        }

        // Clean handle: remove starting '@' if present, make alphanumeric
        if handle.starts_with('@') {
            handle.remove(0);
        }
        let handle = handle.replace(' ', "");

        if handle.is_empty() {
            self.error_msg = Some("Error: Handle must not be empty".to_string());
            return;
        }

        // Check handle uniqueness
        let handle_lower = handle.to_lowercase();
        let is_unique = !self.journal.contacts.iter().any(|c| {
            let is_same_contact = match self.mode {
                AppMode::Writing { is_edit: true } => {
                    c.id == self.journal.contacts[self.selected_index].id
                }
                _ => false,
            };
            !is_same_contact && c.handle.to_lowercase() == handle_lower
        });

        if !is_unique {
            self.error_msg = Some(format!("Error: Handle '@{}' is already taken", handle));
            return;
        }

        match self.mode {
            AppMode::Writing { is_edit: false } => {
                let new_contact = Contact {
                    id: Uuid::new_v4().to_string(),
                    first_name: first,
                    middle_name: middle,
                    last_name: last,
                    handle,
                    notes,
                };
                self.journal.contacts.push(new_contact);
                self.sort_contacts();
                self.selected_index = 0;
                self.status_msg = Some("New contact saved".to_string());
            }
            AppMode::Writing { is_edit: true } => {
                if !self.journal.contacts.is_empty()
                    && self.selected_index < self.journal.contacts.len()
                {
                    let contact = &mut self.journal.contacts[self.selected_index];
                    contact.first_name = first;
                    contact.middle_name = middle;
                    contact.last_name = last;
                    contact.handle = handle;
                    contact.notes = notes;
                    self.sort_contacts();
                    self.status_msg = Some("Contact updated".to_string());
                }
            }
            _ => return,
        }

        // Save to disk immediately
        if let Err(e) = self
            .journal
            .save(&self.file_path, &self.password, &self.salt)
        {
            self.error_msg = Some(format!("Write failed: {}", e));
        } else {
            self.mode = AppMode::List;
            self.detail_scroll = 0;
            self.error_msg = None;
        }
    }

    /// Delete the selected contact and save the changes.
    pub fn delete_selected_contact(&mut self) {
        if self.journal.contacts.is_empty() || self.selected_index >= self.journal.contacts.len() {
            self.mode = AppMode::List;
            return;
        }

        self.journal.contacts.remove(self.selected_index);

        if let Err(e) = self
            .journal
            .save(&self.file_path, &self.password, &self.salt)
        {
            self.error_msg = Some(format!("Delete write failed: {}", e));
        } else {
            self.status_msg = Some("Contact deleted".to_string());
            self.error_msg = None;
        }

        // Adjust selection index bounds
        if self.journal.contacts.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.journal.contacts.len() {
            self.selected_index = self.journal.contacts.len() - 1;
        }

        self.mode = AppMode::List;
        self.detail_scroll = 0;
    }

    /// Find all journal entries that contain a mention of the given handle.
    pub fn get_mentions_for_contact(&self, handle: &str) -> Vec<&JournalEntry> {
        if handle.is_empty() {
            return Vec::new();
        }
        let target = format!("{{{{person|{}}}}}", handle.to_lowercase());
        self.journal
            .entries
            .iter()
            .filter(|entry| entry.content.to_lowercase().contains(&target))
            .collect()
    }
}
