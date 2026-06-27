//! App state management and controllers.
//!
//! Provides the primary [`App`] struct which tracks terminal state, navigation tabs,
//! input modes, and form states.

mod contact_form;
mod date_utils;
mod entries;
mod settings_actions;

pub use contact_form::{ContactField, ContactForm};
pub use date_utils::{format_localized_date, get_date_format_info, parse_localized_date};

use crate::crypto::SALT_SIZE;
use crate::model::Journal;
use ratatui_textarea::TextArea;

/// Tabs available in the primary application navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// Dashboard view with simple stats and Ollama summary.
    Dashboard,
    /// Journal entry view.
    Journal,
    /// Contacts directory view.
    Contacts,
    /// Journal and word statistics view.
    Stats,
    /// Settings management view.
    Settings,
}

/// Settings tab list rows, in display order.
pub const SETTINGS_GROUPS: &[&str] = &[
    "Change Password",
    "Inactivity Timeout",
    "Lock on Suspend",
    "Recovery Shares",
    "Ollama Summary",
];

/// The application's current input and focus state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    // --- Ollama Integration State ---
    /// Cached Ollama summary text.
    pub ollama_summary: Option<String>,
    /// Error from the last Ollama fetch attempt.
    pub ollama_error: Option<String>,
    /// Whether an Ollama API call is currently running.
    pub ollama_in_progress: bool,
    /// Receiver for background thread summary updates.
    pub ollama_rx: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    /// Available Ollama models fetched from the local Ollama instance.
    pub ollama_available_models: Vec<String>,
    /// Selected index in available models list (for settings cycling).
    pub ollama_model_index: usize,
    /// Receiver for background thread model list updates.
    pub ollama_models_rx: Option<std::sync::mpsc::Receiver<Vec<String>>>,
    /// Optional override date for the entry currently being written or edited.
    pub entry_date_for: Option<chrono::NaiveDate>,
}

impl App {
    /// Creates a new [`App`] instance with the specified journal database and session credentials.
    pub fn new(
        journal: Journal,
        file_path: String,
        password: String,
        salt: [u8; SALT_SIZE],
    ) -> Self {
        let (models_tx, models_rx) = std::sync::mpsc::channel();
        let user_model = journal.settings.ollama_model.clone();
        std::thread::spawn(move || {
            let mut models = fetch_local_models();
            // Ensure the user's configured model is in the list
            if !models.contains(&user_model) {
                models.insert(0, user_model);
            }
            let _ = models_tx.send(models);
        });

        let default_models = vec![journal.settings.ollama_model.clone()];

        let mut app = Self {
            journal,
            file_path,
            password,
            salt,
            active_tab: Tab::Dashboard,
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

            // Ollama State
            ollama_summary: None,
            ollama_error: None,
            ollama_in_progress: false,
            ollama_rx: None,
            ollama_available_models: default_models,
            ollama_model_index: 0,
            ollama_models_rx: Some(models_rx),
            entry_date_for: None,
        };

        app.sort_entries();
        app.sort_contacts();

        // Dedup available models and find model index
        app.ollama_available_models.dedup();
        if let Some(pos) = app
            .ollama_available_models
            .iter()
            .position(|m| m == &app.journal.settings.ollama_model)
        {
            app.ollama_model_index = pos;
        }

        // Trigger summary generation on start if enabled
        if app.journal.settings.ollama_enabled {
            app.trigger_ollama_summary();
        }

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

        // If switching to Dashboard, trigger Ollama summary if enabled and not already cached
        if new_tab == Tab::Dashboard
            && self.journal.settings.ollama_enabled
            && self.ollama_summary.is_none()
        {
            self.trigger_ollama_summary();
        }
    }

    /// Returns the length of the list in the currently active tab.
    pub fn list_len(&self) -> usize {
        match self.active_tab {
            Tab::Dashboard => 0,
            Tab::Journal => self.filtered_entries().len(),
            Tab::Contacts => self.filtered_contacts().len(),
            Tab::Settings => SETTINGS_GROUPS.len(),
            Tab::Stats => 0,
        }
    }

    /// Checks if background tasks (Ollama updates, models) have finished and updates state.
    pub fn check_ollama_update(&mut self) {
        // Check for model list updates
        if let Some(models) = self
            .ollama_models_rx
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
        {
            self.ollama_available_models = models;
            self.ollama_available_models.dedup();
            if let Some(pos) = self
                .ollama_available_models
                .iter()
                .position(|m| m == &self.journal.settings.ollama_model)
            {
                self.ollama_model_index = pos;
            } else {
                self.ollama_model_index = 0;
            }
            self.ollama_models_rx = None; // Only receive once
        }

        // Check for summary task updates
        if let Some(res) = self.ollama_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.ollama_in_progress = false;
            match res {
                Ok(summary) => {
                    self.ollama_summary = Some(summary);
                    self.ollama_error = None;
                }
                Err(e) => {
                    self.ollama_error = Some(e);
                    self.ollama_summary = None;
                }
            }
            self.ollama_rx = None;
        }
    }

    /// Triggers summary generation using local Ollama model in a background thread.
    pub fn trigger_ollama_summary(&mut self) {
        if self.ollama_in_progress {
            return;
        }

        // Gather relevant entries (strictly last 7 days only)
        let now = chrono::Utc::now();
        let seven_days_ago = now - chrono::Duration::days(7);
        let entries_to_summarize: Vec<&crate::model::JournalEntry> = self
            .journal
            .entries
            .iter()
            .filter(|e| e.timestamp >= seven_days_ago)
            .collect();

        if entries_to_summarize.is_empty() {
            self.ollama_summary =
                Some("No entries written in the last 7 days to summarize.".to_string());
            self.ollama_error = None;
            return;
        }

        self.ollama_in_progress = true;
        self.ollama_error = None;

        let model = self.journal.settings.ollama_model.clone();

        // 1. Current UTC time context
        let current_time_utc = now.format("%Y-%m-%d %H:%M:%S UTC").to_string();

        // 2. Compile prompt with replaced contact placeholders
        let mut prompt = String::new();
        prompt.push_str("You are a personal journal summarizer.\n");
        prompt.push_str(&format!("Current date and time: {}\n\n", current_time_utc));

        prompt.push_str("Below are the journal entries written by the user over the last 7 days, ordered chronologically from newest to oldest (newest first):\n\n");
        for entry in &entries_to_summarize {
            let date_str = entry
                .timestamp
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();

            // Replace contact placeholders with actual names in the content
            let mut processed_content = entry.content.clone();
            let mut search_str = entry.content.clone();
            while let Some(start_idx) = search_str.find("{{person|") {
                let after_tag = &search_str[start_idx + 9..];
                if let Some(end_idx) = after_tag.find("}}") {
                    let id = &after_tag[..end_idx];
                    let placeholder = format!("{{{{person|{}}}}}", id);
                    let replacement =
                        if let Some(contact) = self.journal.contacts.iter().find(|c| c.id == id) {
                            contact.full_name()
                        } else {
                            "Unknown Person".to_string()
                        };
                    processed_content = processed_content.replace(&placeholder, &replacement);
                    search_str = after_tag[end_idx + 2..].to_string();
                } else {
                    break;
                }
            }

            prompt.push_str(&format!(
                "Date: {}\nContent:\n{}\n---\n",
                date_str, processed_content
            ));
        }

        prompt.push_str("\nWrite a brief, continuous summary (Fließtext, do NOT use bullet points, do NOT use lists) in the same language as the entries (e.g. German if entries are in German).\n");
        prompt.push_str("Write the summary in the third-person perspective (e.g. 'Der Nutzer hat...', 'Gestern hat er...'), NEVER use the first-person ('Ich-Perspektive').\n");
        prompt.push_str("The summary should go chronologically from newest to oldest, discussing the most recent events first.\n");
        prompt.push_str("Summary:\n");

        let (tx, rx) = std::sync::mpsc::channel();
        self.ollama_rx = Some(rx);

        std::thread::spawn(move || {
            let res = generate_summary(&model, &prompt);
            let _ = tx.send(res);
        });
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

fn fetch_local_models() -> Vec<String> {
    let get_models = || -> Option<Vec<String>> {
        let resp = ureq::get("http://localhost:11434/api/tags").call().ok()?;
        let val = resp.into_json::<serde_json::Value>().ok()?;
        let arr = val["models"].as_array()?;
        let models: Vec<String> = arr
            .iter()
            .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
            .collect();
        Some(models)
    };
    get_models().unwrap_or_default()
}

fn generate_summary(model: &str, prompt: &str) -> Result<String, String> {
    let resp_val: serde_json::Value = ureq::post("http://localhost:11434/api/generate")
        .send_json(serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false
        }))
        .map_err(|e| format!("Failed to connect to Ollama: {}", e))?
        .into_json::<serde_json::Value>()
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if let Some(error) = resp_val["error"].as_str() {
        return Err(error.to_string());
    }

    let summary = resp_val["response"]
        .as_str()
        .ok_or_else(|| "No response text found in Ollama output".to_string())?
        .trim()
        .to_string();

    if summary.is_empty() {
        return Err("Ollama returned an empty summary".to_string());
    }

    Ok(summary)
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
            },
            JournalEntry {
                id: "2".to_string(),
                timestamp: Utc::now(),
                content: "Second entry with python".to_string(),
                date_for: None,
            },
            JournalEntry {
                id: "3".to_string(),
                timestamp: Utc::now(),
                content: "Third entry with rust programming".to_string(),
                date_for: None,
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
        };

        // Entry 2: Created today, but back-dated to 5 days ago
        let e2 = JournalEntry {
            id: "2".to_string(),
            timestamp: base_time + chrono::Duration::seconds(10),
            content: "Back-dated 5 days ago".to_string(),
            date_for: Some((base_time - chrono::Duration::days(5)).date_naive()),
        };

        // Entry 3: Created today, but back-dated to yesterday
        let e3 = JournalEntry {
            id: "3".to_string(),
            timestamp: base_time + chrono::Duration::seconds(20),
            content: "Back-dated yesterday".to_string(),
            date_for: Some((base_time - chrono::Duration::days(1)).date_naive()),
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
}
