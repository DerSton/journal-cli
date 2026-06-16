use crate::crypto::SALT_SIZE;
use crate::journal::{Contact, Journal, JournalEntry};
use chrono::Utc;
use ratatui_textarea::TextArea;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Journal,
    Contacts,
    Settings,
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
    DatePicker {
        is_edit: bool,
        field_index: usize,
        current_date: chrono::NaiveDate,
    },
    DeleteConfirm,
    Recovery,
    Login,
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
    /// Index of the currently highlighted item (entry, contact, or settings group) in the list.
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
    pub contact_birthdate: Option<chrono::NaiveDate>,
    pub contact_deathdate: Option<chrono::NaiveDate>,
    pub contact_notes: TextArea<'static>,
    /// Active field index in the contact editor (0: First, 1: Middle, 2: Last, 3: Handle, 4: Birthdate, 5: Deathdate, 6: Notes).
    pub active_field_index: usize,

    // Settings Fields
    pub settings_password_new: TextArea<'static>,
    pub settings_password_confirm: TextArea<'static>,
    /// Active field index in settings password changer (0: New, 1: Confirm).
    pub settings_active_field: usize,

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
    // Temporary Settings Fields for editing
    pub temp_timeout_mins: u32,
    pub temp_lock_on_suspend: bool,
    // Recovery Mode Fields
    pub recovery_shares: Vec<String>,
    pub recovery_status_msg: Option<String>,
    pub settings_num_shares: usize,
    pub settings_threshold: usize,
    pub generated_shares: Vec<String>,
    pub recovery_textarea: TextArea<'static>,
    pub login_password: String,
}

impl App {
    /// Create a new application instance, sorting entries and contacts.
    pub fn new(
        journal: Journal,
        file_path: String,
        password: String,
        salt: [u8; SALT_SIZE],
    ) -> Self {
        let temp_timeout_mins = journal.settings.autolock_timeout_mins;
        let temp_lock_on_suspend = journal.settings.lock_on_suspend;
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
            contact_birthdate: None,
            contact_deathdate: None,
            contact_notes: TextArea::default(),
            active_field_index: 0,
            settings_password_new: TextArea::default(),
            settings_password_confirm: TextArea::default(),
            settings_active_field: 0,

            error_msg: None,
            status_msg: Some("Welcome to your secure journal CLI!".to_string()),
            should_quit: false,
            detail_scroll: 0,
            handle_edited: false,
            temp_timeout_mins,
            temp_lock_on_suspend,
            recovery_shares: Vec::new(),
            recovery_status_msg: None,
            settings_num_shares: 5,
            settings_threshold: 3,
            generated_shares: Vec::new(),
            recovery_textarea: TextArea::default(),
            login_password: String::new(),
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
                    birthdate: self.contact_birthdate,
                    date_of_death: self.contact_deathdate,
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
                    contact.birthdate = self.contact_birthdate;
                    contact.date_of_death = self.contact_deathdate;
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

    /// Transactionally updates the master password, re-encrypts the journal file, and updates memory credentials.
    pub fn handle_change_password(&mut self) -> Result<(), String> {
        let new_pw = self.settings_password_new.lines().join("");
        let confirm_pw = self.settings_password_confirm.lines().join("");

        if new_pw.is_empty() {
            return Err("New password cannot be empty".to_string());
        }
        if new_pw != confirm_pw {
            return Err("Passwords do not match".to_string());
        }

        // Generate new salt
        use rand::random;
        let new_salt: [u8; crate::crypto::SALT_SIZE] = random();

        // Transactional save to tmp first
        let tmp_path = format!("{}.tmp", self.file_path);
        if let Err(e) = self.journal.save(&tmp_path, &new_pw, &new_salt) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("Failed to write encrypted file: {}", e));
        }

        // Rename tmp to actual path
        if let Err(e) = std::fs::rename(&tmp_path, &self.file_path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("Failed to finalize new password file: {}", e));
        }

        // Update memory state
        self.password = new_pw;
        self.salt = new_salt;
        self.settings_password_new = TextArea::default();
        self.settings_password_confirm = TextArea::default();
        self.settings_active_field = 0;

        Ok(())
    }
    /// Immediately saves the current settings to disk.
    pub fn save_settings(&mut self) -> Result<(), String> {
        self.journal
            .save(&self.file_path, &self.password, &self.salt)
    }
    /// Translation helper lookup.
    pub fn tr(&self, key: TrKey) -> &'static str {
        tr(key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TrKey {
    // Navigation
    NavTitle,
    NavSwitchHint,

    // Tab Names
    TabJournal,
    TabContacts,
    TabSettings,

    // List Placeholders
    NoEntries,
    PressNewEntry,
    NoContacts,
    PressNewContact,
    NoMentions,
    MentionHistory,
    JournalEntriesTitle,
    ContactsListTitle,

    // Welcome & Status Messages
    WelcomeMsg,
    NewEntrySaved,
    EntryUpdated,
    EntryDeleted,
    NewContactSaved,
    ContactUpdated,
    ContactDeleted,
    PasswordChanged,
    SaveFailed,
    LocaleUpdated,
    TimezoneUpdated,

    // Contact Profile Preview
    ProfileTitle,
    ProfileFirstName,
    ProfileMiddleName,
    ProfileLastName,
    ProfileHandle,
    ProfileBorn,
    ProfileDeceased,
    ProfileNotes,
    ProfileAge,
    ProfileAged,

    // Contact Form Editor
    FormFirstNameTitle,
    FormMiddleNameTitle,
    FormLastNameTitle,
    FormHandleTitle,
    FormBirthdateTitle,
    FormDeathdateTitle,
    FormNotesTitle,
    FormPressEnterSelect,
    FormHintNext,
    FormHintPrev,
    FormHintOpenCalendar,
    FormHintClearDate,
    FormHintSave,
    FormHintCancel,
    FormControlsTitle,
    FormTitleNew,
    FormTitleEdit,

    // Settings Options
    SettingsHeader,
    SettingsPasswordLabel,
    SettingsPasswordDesc,
    SettingsLocaleLabel,
    SettingsLocaleDesc,
    SettingsTimezoneLabel,
    SettingsTimezoneDesc,
    SettingsChangePasswordTitle,
    SettingsNewPasswordInput,
    SettingsConfirmPasswordInput,
    SettingsSubmitHint,
    SettingsSelected,

    // Help & Buttons Bar
    HelpQuit,
    HelpNewEntry,
    HelpNewContact,
    HelpEdit,
    HelpDelete,
    HelpScrollPreview,
    HelpSelectOption,
    HelpSelectSave,
    HelpConfirmDelete,
    HelpYesDelete,
    HelpCancel,
    HelpNavigate,
    HelpMonth,
    HelpYear,
    HelpPick,
    HelpClear,

    // Modals
    ModalWarningTitle,
    ModalDeleteConfirmQuestion,
    ModalDeletePermanentWarning,
    ModalDeleteYesBtn,
    ModalDeleteCancelBtn,
    ModalContactPickerTitle,
    ModalLocaleTitle,
    ModalTimezoneTitle,
    ModalSearchPrompt,

    // Additional Preview
    ViewEntryTitle,
    ViewingEntryTitle,
    Of,
    LabelDate,
    EditorTitleEditEntry,
    EditorTitleNewEntry,
}

pub fn tr(key: TrKey) -> &'static str {
    match key {
        TrKey::NavTitle => " NAVIGATION: ",
        TrKey::NavSwitchHint => "  (Press Tab or 1-3 to switch)",
        TrKey::TabJournal => " ● Journal (1) ",
        TrKey::TabContacts => " ● Contacts (2) ",
        TrKey::TabSettings => " ● Settings (3) ",
        TrKey::NoEntries => "No entries found in database.",
        TrKey::PressNewEntry => "Press 'n' to write your first entry!",
        TrKey::NoContacts => "No contacts found in database.",
        TrKey::PressNewContact => "Press 'n' to add a new contact!",
        TrKey::NoMentions => "No mentions found in journal entries.",
        TrKey::MentionHistory => " Mentions in Journal ",
        TrKey::JournalEntriesTitle => " Journal Entries ",
        TrKey::ContactsListTitle => " Contacts ",
        TrKey::WelcomeMsg => "Welcome to your secure journal CLI!",
        TrKey::NewEntrySaved => "New entry saved",
        TrKey::EntryUpdated => "Entry updated",
        TrKey::EntryDeleted => "Entry deleted",
        TrKey::NewContactSaved => "New contact saved",
        TrKey::ContactUpdated => "Contact updated",
        TrKey::ContactDeleted => "Contact deleted",
        TrKey::PasswordChanged => "Password changed and database re-encrypted",
        TrKey::SaveFailed => "Save failed",
        TrKey::LocaleUpdated => "Locale updated to",
        TrKey::TimezoneUpdated => "Timezone updated to",
        TrKey::ProfileTitle => " Contact Profile ",
        TrKey::ProfileFirstName => "  First Name:  ",
        TrKey::ProfileMiddleName => "  Middle Name: ",
        TrKey::ProfileLastName => "  Last Name:   ",
        TrKey::ProfileHandle => "  Handle:      ",
        TrKey::ProfileBorn => "  Born:        ",
        TrKey::ProfileDeceased => "  Deceased:    ",
        TrKey::ProfileNotes => "  Notes:",
        TrKey::ProfileAge => "(Age: {})",
        TrKey::ProfileAged => "(Aged: {})",
        TrKey::FormFirstNameTitle => " First Name ",
        TrKey::FormMiddleNameTitle => " Middle Name ",
        TrKey::FormLastNameTitle => " Last Name ",
        TrKey::FormHandleTitle => " Handle (for @mentions) ",
        TrKey::FormBirthdateTitle => " Birthdate ",
        TrKey::FormDeathdateTitle => " Date of Death ",
        TrKey::FormNotesTitle => " Notes ",
        TrKey::FormPressEnterSelect => " [ Press Enter to select ]",
        TrKey::FormHintNext => "Next Field",
        TrKey::FormHintPrev => "Prev Field",
        TrKey::FormHintOpenCalendar => "Open Calendar (on Date fields)",
        TrKey::FormHintClearDate => "Clear Date",
        TrKey::FormHintSave => "Save Contact",
        TrKey::FormHintCancel => "Cancel",
        TrKey::FormControlsTitle => "Form Controls:",
        TrKey::FormTitleNew => " ➕  New Contact ",
        TrKey::FormTitleEdit => " ✏️  Edit Contact ",
        TrKey::SettingsHeader => " Settings Menu ",
        TrKey::SettingsPasswordLabel => "🔑  Change Password",
        TrKey::SettingsPasswordDesc => {
            "Change master password used to decrypt the journal database."
        }
        TrKey::SettingsLocaleLabel => "🌐  Language & Locale",
        TrKey::SettingsLocaleDesc => "Set application formatting locale for dates and times.",
        TrKey::SettingsTimezoneLabel => "🕒  Timezone",
        TrKey::SettingsTimezoneDesc => "Configure target timezone relative to UTC.",
        TrKey::SettingsChangePasswordTitle => " Change Password ",
        TrKey::SettingsNewPasswordInput => " New Master Password ",
        TrKey::SettingsConfirmPasswordInput => " Confirm New Password ",
        TrKey::SettingsSubmitHint => " Submit changes by pressing Ctrl + S ",
        TrKey::SettingsSelected => "Active",
        TrKey::HelpQuit => "Quit ",
        TrKey::HelpNewEntry => "New Entry ",
        TrKey::HelpNewContact => "New Contact ",
        TrKey::HelpEdit => "Edit ",
        TrKey::HelpDelete => "Delete ",
        TrKey::HelpScrollPreview => "Scroll Preview ",
        TrKey::HelpSelectOption => "Select Option ",
        TrKey::HelpSelectSave => "Select & Save ",
        TrKey::HelpConfirmDelete => "Confirm Delete? ",
        TrKey::HelpYesDelete => "Yes, Delete ",
        TrKey::HelpCancel => "Cancel ",
        TrKey::HelpNavigate => "Nav ",
        TrKey::HelpMonth => "Month ",
        TrKey::HelpYear => "Year ",
        TrKey::HelpPick => "Pick ",
        TrKey::HelpClear => "Clear ",
        TrKey::ModalWarningTitle => " WARNING ",
        TrKey::ModalDeleteConfirmQuestion => "Are you sure you want to delete this?",
        TrKey::ModalDeletePermanentWarning => "This action is permanent and cannot be undone.",
        TrKey::ModalDeleteYesBtn => " [y] Yes, Delete ",
        TrKey::ModalDeleteCancelBtn => " [n/Esc] Cancel ",
        TrKey::ModalContactPickerTitle => " Select Contact to Mention [Enter: Pick, Esc: Cancel] ",
        TrKey::ModalLocaleTitle => " Select Locale ",
        TrKey::ModalTimezoneTitle => " Select Timezone ",
        TrKey::ModalSearchPrompt => " Search: ",
        TrKey::ViewEntryTitle => " View Entry ",
        TrKey::ViewingEntryTitle => "Viewing Entry",
        TrKey::Of => "of",
        TrKey::LabelDate => "Date: ",
        TrKey::EditorTitleEditEntry => " ✏️  Edit Entry [Ctrl+S: Save, Esc: Cancel] ",
        TrKey::EditorTitleNewEntry => " ➕  New Entry [Ctrl+S: Save, Esc: Cancel] ",
    }
}
