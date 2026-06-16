use crate::crypto::SALT_SIZE;
use crate::journal::{Contact, Journal, JournalEntry};
use chrono::Utc;
use ratatui_textarea::TextArea;
use uuid::Uuid;

pub const BLOOD_TYPE_OPTIONS: &[&str] = &["N/A", "A+", "A-", "B+", "B-", "AB+", "AB-", "O+", "O-"];
pub const MARITAL_STATUS_OPTIONS: &[&str] = &[
    "N/A",
    "Single",
    "Married",
    "Divorced",
    "Widowed",
    "Registered Partnership",
];

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
    RecoveryReset,
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
    pub contact_form_tab: usize,
    pub contact_form_title: TextArea<'static>,
    pub contact_form_first_names: Vec<TextArea<'static>>,
    pub contact_form_last_name: TextArea<'static>,
    pub contact_form_nickname: TextArea<'static>,
    pub contact_form_preferred_name: TextArea<'static>,
    pub contact_form_maiden_name: TextArea<'static>,
    pub contact_form_suffix: TextArea<'static>,
    pub contact_form_birthdate: TextArea<'static>,
    pub contact_form_deathdate: TextArea<'static>,
    pub contact_form_gender: TextArea<'static>,
    pub contact_form_pronouns: TextArea<'static>,
    pub contact_form_nationalities: Vec<TextArea<'static>>,
    pub contact_form_languages: Vec<TextArea<'static>>,
    pub contact_form_marital_status_idx: usize,
    pub contact_form_blood_type_idx: usize,
    pub contact_form_religion: TextArea<'static>,
    pub contact_form_eye_color: TextArea<'static>,
    pub contact_form_hair_color: TextArea<'static>,
    pub contact_form_height: TextArea<'static>,
    pub contact_form_notes: TextArea<'static>,
    /// Active field index in the contact editor (indexes fields of the current active tab).
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
            contact_form_tab: 0,
            contact_form_title: TextArea::default(),
            contact_form_first_names: vec![TextArea::default()],
            contact_form_last_name: TextArea::default(),
            contact_form_nickname: TextArea::default(),
            contact_form_preferred_name: TextArea::default(),
            contact_form_maiden_name: TextArea::default(),
            contact_form_suffix: TextArea::default(),
            contact_form_birthdate: TextArea::default(),
            contact_form_deathdate: TextArea::default(),
            contact_form_gender: TextArea::default(),
            contact_form_pronouns: TextArea::default(),
            contact_form_nationalities: vec![TextArea::default()],
            contact_form_languages: vec![TextArea::default()],
            contact_form_marital_status_idx: 0,
            contact_form_blood_type_idx: 0,
            contact_form_religion: TextArea::default(),
            contact_form_eye_color: TextArea::default(),
            contact_form_hair_color: TextArea::default(),
            contact_form_height: TextArea::default(),
            contact_form_notes: TextArea::default(),
            active_field_index: 0,
            settings_password_new: TextArea::default(),
            settings_password_confirm: TextArea::default(),
            settings_active_field: 0,

            error_msg: None,
            status_msg: Some("Welcome to your secure journal CLI!".to_string()),
            should_quit: false,
            detail_scroll: 0,
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

    /// Initializes the contact form state.
    pub fn init_contact_form(&mut self, is_edit: bool) {
        self.contact_form_tab = 0;
        self.active_field_index = 0;
        self.error_msg = None;

        if is_edit
            && !self.journal.contacts.is_empty()
            && self.selected_index < self.journal.contacts.len()
        {
            let contact = &self.journal.contacts[self.selected_index];
            self.contact_form_title = TextArea::new(vec![contact.title.clone()]);

            // First names list: populate and make sure we have at least one empty at the end
            let mut f_names: Vec<TextArea<'static>> = contact
                .first_names
                .iter()
                .map(|name| TextArea::new(vec![name.clone()]))
                .collect();
            f_names.push(TextArea::default());
            self.contact_form_first_names = f_names;

            self.contact_form_last_name = TextArea::new(vec![contact.last_name.clone()]);
            self.contact_form_nickname = TextArea::new(vec![contact.nickname.clone()]);
            self.contact_form_preferred_name = TextArea::new(vec![contact.preferred_name.clone()]);
            self.contact_form_maiden_name = TextArea::new(vec![contact.maiden_name.clone()]);
            self.contact_form_suffix = TextArea::new(vec![contact.suffix.clone()]);

            let birth_str = contact
                .birthdate
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default();
            self.contact_form_birthdate = TextArea::new(vec![birth_str]);

            let death_str = contact
                .date_of_death
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default();
            self.contact_form_deathdate = TextArea::new(vec![death_str]);

            self.contact_form_gender = TextArea::new(vec![contact.gender.clone()]);
            self.contact_form_pronouns = TextArea::new(vec![contact.pronouns.clone()]);

            // Nationalities list
            let mut nats: Vec<TextArea<'static>> = contact
                .nationalities
                .iter()
                .map(|nat| TextArea::new(vec![nat.clone()]))
                .collect();
            nats.push(TextArea::default());
            self.contact_form_nationalities = nats;

            // Languages list
            let mut langs: Vec<TextArea<'static>> = contact
                .languages
                .iter()
                .map(|lang| TextArea::new(vec![lang.clone()]))
                .collect();
            langs.push(TextArea::default());
            self.contact_form_languages = langs;

            // Select index matching
            self.contact_form_marital_status_idx = MARITAL_STATUS_OPTIONS
                .iter()
                .position(|&opt| opt == contact.marital_status)
                .unwrap_or(0);

            self.contact_form_blood_type_idx = BLOOD_TYPE_OPTIONS
                .iter()
                .position(|&opt| opt == contact.blood_type)
                .unwrap_or(0);

            self.contact_form_religion = TextArea::new(vec![contact.religion.clone()]);
            self.contact_form_eye_color = TextArea::new(vec![contact.eye_color.clone()]);
            self.contact_form_hair_color = TextArea::new(vec![contact.hair_color.clone()]);

            let height_str = contact.height.map(|h| h.to_string()).unwrap_or_default();
            self.contact_form_height = TextArea::new(vec![height_str]);

            self.contact_form_notes =
                TextArea::new(contact.notes.lines().map(String::from).collect());
        } else {
            // New contact defaults
            self.contact_form_title = TextArea::default();
            self.contact_form_first_names = vec![TextArea::default()];
            self.contact_form_last_name = TextArea::default();
            self.contact_form_nickname = TextArea::default();
            self.contact_form_preferred_name = TextArea::default();
            self.contact_form_maiden_name = TextArea::default();
            self.contact_form_suffix = TextArea::default();
            self.contact_form_birthdate = TextArea::default();
            self.contact_form_deathdate = TextArea::default();
            self.contact_form_gender = TextArea::default();
            self.contact_form_pronouns = TextArea::default();
            self.contact_form_nationalities = vec![TextArea::default()];
            self.contact_form_languages = vec![TextArea::default()];
            self.contact_form_marital_status_idx = 0;
            self.contact_form_blood_type_idx = 0;
            self.contact_form_religion = TextArea::default();
            self.contact_form_eye_color = TextArea::default();
            self.contact_form_hair_color = TextArea::default();
            self.contact_form_height = TextArea::default();
            self.contact_form_notes = TextArea::default();
        }

        self.mode = AppMode::Writing { is_edit };
    }

    /// Save the contact form fields to the database.
    pub fn handle_save_contact(&mut self) {
        let title = self.contact_form_title.lines().join("").trim().to_string();
        let first_names: Vec<String> = self
            .contact_form_first_names
            .iter()
            .map(|ta| ta.lines().join("").trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let last_name = self
            .contact_form_last_name
            .lines()
            .join("")
            .trim()
            .to_string();
        let nickname = self
            .contact_form_nickname
            .lines()
            .join("")
            .trim()
            .to_string();
        let preferred_name = self
            .contact_form_preferred_name
            .lines()
            .join("")
            .trim()
            .to_string();
        let maiden_name = self
            .contact_form_maiden_name
            .lines()
            .join("")
            .trim()
            .to_string();
        let suffix = self.contact_form_suffix.lines().join("").trim().to_string();
        let gender = self.contact_form_gender.lines().join("").trim().to_string();
        let pronouns = self
            .contact_form_pronouns
            .lines()
            .join("")
            .trim()
            .to_string();
        let nationalities: Vec<String> = self
            .contact_form_nationalities
            .iter()
            .map(|ta| ta.lines().join("").trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let languages: Vec<String> = self
            .contact_form_languages
            .iter()
            .map(|ta| ta.lines().join("").trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let marital_status =
            MARITAL_STATUS_OPTIONS[self.contact_form_marital_status_idx].to_string();
        let blood_type = BLOOD_TYPE_OPTIONS[self.contact_form_blood_type_idx].to_string();
        let religion = self
            .contact_form_religion
            .lines()
            .join("")
            .trim()
            .to_string();
        let eye_color = self
            .contact_form_eye_color
            .lines()
            .join("")
            .trim()
            .to_string();
        let hair_color = self
            .contact_form_hair_color
            .lines()
            .join("")
            .trim()
            .to_string();
        let notes = self
            .contact_form_notes
            .lines()
            .join("\n")
            .trim()
            .to_string();

        let height_str = self.contact_form_height.lines().join("").trim().to_string();
        let height = if height_str.is_empty() {
            None
        } else {
            match height_str.parse::<u32>() {
                Ok(h) => Some(h),
                Err(_) => {
                    self.error_msg =
                        Some("Error: Height must be a valid positive number".to_string());
                    return;
                }
            }
        };

        let birth_str = self
            .contact_form_birthdate
            .lines()
            .join("")
            .trim()
            .to_string();
        let birthdate = if birth_str.is_empty() {
            None
        } else {
            match chrono::NaiveDate::parse_from_str(&birth_str, "%Y-%m-%d") {
                Ok(d) => Some(d),
                Err(_) => {
                    self.error_msg =
                        Some("Error: Birthdate must be in YYYY-MM-DD format".to_string());
                    return;
                }
            }
        };

        let death_str = self
            .contact_form_deathdate
            .lines()
            .join("")
            .trim()
            .to_string();
        let date_of_death = if death_str.is_empty() {
            None
        } else {
            match chrono::NaiveDate::parse_from_str(&death_str, "%Y-%m-%d") {
                Ok(d) => Some(d),
                Err(_) => {
                    self.error_msg =
                        Some("Error: Date of death must be in YYYY-MM-DD format".to_string());
                    return;
                }
            }
        };

        if first_names.is_empty() && last_name.is_empty() {
            self.error_msg = Some("Error: First Name or Last Name must be provided".to_string());
            return;
        }

        match self.mode {
            AppMode::Writing { is_edit: false } => {
                let new_contact = Contact {
                    id: Uuid::new_v4().to_string(),
                    first_name: String::new(),
                    middle_name: String::new(),
                    handle: String::new(),
                    first_names,
                    last_name,
                    title,
                    nickname,
                    preferred_name,
                    maiden_name,
                    suffix,
                    gender,
                    pronouns,
                    nationalities,
                    languages,
                    religion,
                    marital_status,
                    blood_type,
                    eye_color,
                    hair_color,
                    height,
                    notes,
                    birthdate,
                    date_of_death,
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
                    contact.first_names = first_names;
                    contact.last_name = last_name;
                    contact.title = title;
                    contact.nickname = nickname;
                    contact.preferred_name = preferred_name;
                    contact.maiden_name = maiden_name;
                    contact.suffix = suffix;
                    contact.gender = gender;
                    contact.pronouns = pronouns;
                    contact.nationalities = nationalities;
                    contact.languages = languages;
                    contact.marital_status = marital_status;
                    contact.blood_type = blood_type;
                    contact.religion = religion;
                    contact.eye_color = eye_color;
                    contact.hair_color = hair_color;
                    contact.height = height;
                    contact.notes = notes;
                    contact.birthdate = birthdate;
                    contact.date_of_death = date_of_death;
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

    pub fn contact_form_num_fields(&self) -> usize {
        match self.contact_form_tab {
            0 => 5 + self.contact_form_first_names.len(),
            1 => 6 + self.contact_form_nationalities.len() + self.contact_form_languages.len(),
            2 => 5,
            _ => 0,
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
