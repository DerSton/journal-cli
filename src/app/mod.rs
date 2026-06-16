mod contact_form;
mod entries;
mod settings_actions;

pub use contact_form::{ContactField, ContactForm};

use crate::crypto::SALT_SIZE;
use crate::model::Journal;
use ratatui_textarea::TextArea;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Journal,
    Contacts,
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
            Tab::Journal => self.journal.entries.len(),
            Tab::Contacts => self.journal.contacts.len(),
            Tab::Settings => SETTINGS_GROUPS.len(),
        }
    }

    /// Persist the journal under the current password and salt.
    pub fn save_journal(&mut self) -> Result<(), String> {
        self.journal
            .save(&self.file_path, &self.password, &self.salt)
    }
}
