//! App state management and controllers.
//!
//! Provides the primary [`App`] struct which tracks terminal state, navigation tabs,
//! input modes, and form states.

mod contact_form;
mod date_utils;
mod entries;
mod group_form;
mod settings_actions;

pub use contact_form::{ContactField, ContactForm};
pub use date_utils::{format_localized_date, get_date_format_info, parse_localized_date};
pub use group_form::{GroupField, GroupForm};

use crate::crypto::SALT_SIZE;
use crate::model::Journal;
use ratatui_textarea::TextArea;

/// Tabs available in the primary application navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// Journal entry view.
    Journal,
    /// Contacts directory view.
    Contacts,
    /// Contact groups view.
    Groups,
    /// Journal and word statistics view.
    Stats,
    /// Settings management view.
    Settings,
}

/// An item displayed inside the mention tag picker.
#[derive(Debug, Clone)]
pub struct PickerItem {
    pub name: String,
    pub tag: String,
}

/// Settings tab list rows, in display order.
pub const SETTINGS_GROUPS: &[&str] = &[
    "Change Password",
    "Inactivity Timeout",
    "Lock on Suspend",
    "Recovery Shares",
];

/// The application's current input and focus state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    /// Browsing a list (journal entries, contacts, or settings groups).
    List,
    /// Editing a journal entry or the contact form.
    Writing {
        /// Whether the editor is editing an existing item (true) or creating a new one (false).
        is_edit: bool,
    },
    /// Picking a contact to insert as a `{{person|id}}` mention.
    ContactPicker {
        /// Whether the parent context is an edit mode.
        is_edit: bool,
        /// The currently highlighted index in the contact picker modal.
        selected_contact_index: usize,
        /// The current search query inside the contact picker.
        search_query: String,
    },
    /// Calendar overlay for a date field.
    DatePicker {
        /// Whether the parent context is an edit mode.
        is_edit: bool,
        /// Field index: 0 = birthdate, 1 = date of death.
        field_index: usize,
        /// Currently highlighted date in the calendar widget.
        current_date: chrono::NaiveDate,
    },
    /// Picker for selecting group members.
    GroupMemberPicker {
        /// Whether the parent context is an edit mode.
        is_edit: bool,
        /// The currently highlighted index in the member picker.
        selected_contact_index: usize,
        /// The current search query inside the member picker.
        search_query: String,
    },
    /// Confirmation dialog before deleting an entry or contact.
    DeleteConfirm,
    /// Prompt for entering the master password.
    Login,
    /// Password recovery screen (entering Shamir shares).
    Recovery,
    /// Password recovery reset confirmation.
    RecoveryReset,
    /// Filtering entries or contacts using a search string.
    Search,
    /// Picking or managing an attachment of the currently selected journal entry.
    AttachmentPicker {
        /// The currently highlighted index in the attachment list.
        selected_attachment_index: usize,
    },
    /// Confirmation dialog before discarding unsaved changes.
    DiscardConfirm {
        /// Whether we were editing an existing item (true) or creating a new one (false).
        is_edit: bool,
    },
}

/// The global application state container.
pub struct App {
    /// The loaded decrypted journal database.
    pub journal: Journal,
    /// File path to the journal file.
    pub file_path: String,
    /// Master password used for re-encrypting the database on save.
    pub password: String,
    /// Key derivation salt resolved on load or creation.
    pub salt: [u8; SALT_SIZE],

    /// Currently active tab in the main list views.
    pub active_tab: Tab,
    /// Highlighted row in the active tab's list (entry, contact, or settings group).
    pub selected_index: usize,
    /// Current input and focus mode.
    pub mode: AppMode,

    /// Text area editor for journal entry creation and updates.
    pub textarea: TextArea<'static>,
    /// Scroll offset for the journal entry preview pane.
    pub detail_scroll: u16,

    /// Contact create/edit form state.
    pub contact_form: ContactForm,
    /// Group create/edit form state.
    pub group_form: GroupForm,

    /// Whether keys go to the settings right-hand panel (true) or the group list (false).
    pub settings_panel_focused: bool,
    /// Active field within the focused settings panel.
    pub settings_active_field: usize,
    /// Text editor for entering a new settings password.
    pub settings_password_new: TextArea<'static>,
    /// Text editor for confirming the new settings password.
    pub settings_password_confirm: TextArea<'static>,
    /// Number of recovery shares to generate.
    pub settings_num_shares: usize,
    /// Number of threshold shares required to recover the password.
    pub settings_threshold: usize,
    /// Generated Shamir share strings list.
    pub generated_shares: Vec<String>,

    /// Entered Shamir recovery share strings list.
    pub recovery_shares: Vec<String>,
    /// Status message displayed on the recovery screen.
    pub recovery_status_msg: Option<String>,
    /// Text editor for entering a Shamir share string.
    pub recovery_textarea: TextArea<'static>,
    /// Password entered during login.
    pub login_password: String,

    /// Temporary error message shown on the status bar.
    pub error_msg: Option<String>,
    /// Temporary success or info message shown on the status bar.
    pub status_msg: Option<String>,
    /// Exit condition flag.
    pub should_quit: bool,
    /// Current search query string.
    pub search_query: String,
    /// Optional override date for the entry currently being written or edited.
    pub entry_date_for: Option<chrono::NaiveDate>,
    /// Request a full terminal clear and cache reset on the next loop iteration.
    pub redraw_requested: bool,
}

impl App {
    /// Creates a new [`App`] instance with the specified journal database and session credentials.
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
            group_form: GroupForm::empty(),
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
            status_msg: Some("Welcome to your secure journal".to_string()),
            should_quit: false,
            search_query: String::new(),
            entry_date_for: None,
            redraw_requested: false,
        };

        app.sort_entries();
        app.sort_contacts();
        app.sort_groups();

        app
    }

    /// Sorts journal entries by their sort timestamp in descending order (newest first).
    pub fn sort_entries(&mut self) {
        self.journal
            .entries
            .sort_by_key(|e| std::cmp::Reverse(e.sort_timestamp()));
    }

    /// Sorts contacts alphabetically by last name, then by first names.
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

    /// Switches the active navigation tab, resetting selection index, scroll offset, and mode.
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
            Tab::Groups => self.filtered_groups().len(),
            Tab::Settings => SETTINGS_GROUPS.len(),
            Tab::Stats => 0,
        }
    }

    /// Returns a list of references to journal entries, filtered by the current search query.
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

    /// Returns a list of references to contacts, filtered by the current search query.
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

    /// Returns picker items combining groups and contacts filtered by search query.
    pub fn get_picker_items(&self, query: &str) -> Vec<PickerItem> {
        let mut items = Vec::new();
        let q = query.to_lowercase();

        // Add groups first
        for g in &self.journal.groups {
            if q.is_empty()
                || g.name.to_lowercase().contains(&q)
                || g.description.to_lowercase().contains(&q)
            {
                items.push(PickerItem {
                    name: g.name.clone(),
                    tag: g.mention_tag(),
                });
            }
        }

        // Add contacts
        for c in &self.journal.contacts {
            if q.is_empty()
                || c.full_name().to_lowercase().contains(&q)
                || c.nickname.to_lowercase().contains(&q)
            {
                items.push(PickerItem {
                    name: c.full_name(),
                    tag: c.mention_tag(),
                });
            }
        }

        items
    }

    /// Maps the currently selected filtered entry index back to its index in the master `journal.entries` list.
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

    /// Maps the currently selected filtered contact index back to its index in the master `journal.contacts` list.
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

    /// Checks if the editor in the current active tab has any unsaved modifications.
    pub fn is_dirty(&self) -> bool {
        match self.mode {
            AppMode::Writing { is_edit } => match self.active_tab {
                Tab::Journal => {
                    if is_edit {
                        if let Some(real_idx) = self.selected_entry_idx() {
                            let entry = &self.journal.entries[real_idx];
                            entry.content != self.textarea.lines().join("\n")
                                || entry.date_for != self.entry_date_for
                        } else {
                            true
                        }
                    } else {
                        !self.textarea.lines().join("\n").trim().is_empty()
                            || self.entry_date_for.is_some()
                    }
                }
                Tab::Contacts => self.is_contact_form_dirty(is_edit),
                Tab::Groups => self.is_group_form_dirty(is_edit),
                _ => false,
            },
            _ => false,
        }
    }

    /// Persists the decrypted in-memory journal database to disk under the current password and salt.
    ///
    /// # Errors
    ///
    /// Returns an error if key derivation, encryption, or writing fails.
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
                date_for: None,
                attachments: Vec::new(),
            },
            JournalEntry {
                id: "2".to_string(),
                timestamp: Utc::now(),
                content: "Second entry with python".to_string(),
                date_for: None,
                attachments: Vec::new(),
            },
            JournalEntry {
                id: "3".to_string(),
                timestamp: Utc::now(),
                content: "Third entry with rust programming".to_string(),
                date_for: None,
                attachments: Vec::new(),
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

    #[test]
    fn test_entry_date_override_sorting() {
        let mut app = test_app();
        let base_time = Utc::now();

        // Entry 1: Created today
        let e1 = JournalEntry {
            id: "1".to_string(),
            timestamp: base_time,
            content: "Today".to_string(),
            date_for: None,
            attachments: Vec::new(),
        };

        // Entry 2: Created today, but back-dated to 5 days ago
        let e2 = JournalEntry {
            id: "2".to_string(),
            timestamp: base_time + chrono::Duration::seconds(10),
            content: "Back-dated 5 days ago".to_string(),
            date_for: Some((base_time - chrono::Duration::days(5)).date_naive()),
            attachments: Vec::new(),
        };

        // Entry 3: Created today, but back-dated to yesterday
        let e3 = JournalEntry {
            id: "3".to_string(),
            timestamp: base_time + chrono::Duration::seconds(20),
            content: "Back-dated yesterday".to_string(),
            date_for: Some((base_time - chrono::Duration::days(1)).date_naive()),
            attachments: Vec::new(),
        };

        app.journal.entries = vec![e2.clone(), e1.clone(), e3.clone()];
        app.sort_entries();

        // After sorting (newest first), order should be:
        // 1. Entry 1 (Today)
        // 2. Entry 3 (Yesterday)
        // 3. Entry 2 (5 days ago)
        assert_eq!(app.journal.entries[0].id, "1");
        assert_eq!(app.journal.entries[1].id, "3");
        assert_eq!(app.journal.entries[2].id, "2");
    }

    #[test]
    fn test_group_filtering_sorting_and_picker() {
        let mut app = test_app();
        app.journal.groups = vec![
            crate::model::Group {
                id: "group-2".to_string(),
                name: "Ski trip".to_string(),
                description: "Snowboarding".to_string(),
                member_ids: vec!["1".to_string()],
                start_date: None,
                end_date: None,
            },
            crate::model::Group {
                id: "group-1".to_string(),
                name: "Family".to_string(),
                description: "Relative".to_string(),
                member_ids: vec![],
                start_date: None,
                end_date: None,
            },
        ];

        app.journal.contacts = vec![Contact {
            id: "1".to_string(),
            first_names: vec!["Alice".to_string()],
            last_name: "Smith".to_string(),
            ..Default::default()
        }];

        // Groups sorting in App: Family (group-1) first, Ski trip (group-2) second.
        app.sort_groups();
        assert_eq!(app.journal.groups[0].id, "group-1");
        assert_eq!(app.journal.groups[1].id, "group-2");

        // Filtering groups
        app.search_query = "snow".to_string();
        let filtered = app.filtered_groups();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "group-2");

        // Picker items combining contacts and groups
        let picker_items = app.get_picker_items("family");
        assert_eq!(picker_items.len(), 1);
        assert_eq!(picker_items[0].tag, "{{group|group-1}}");
    }
}
