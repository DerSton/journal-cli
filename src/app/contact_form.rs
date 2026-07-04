use super::{App, format_localized_date, get_date_format_info, parse_localized_date};
use crate::model::{BLOOD_TYPE_OPTIONS, Contact, JournalEntry, MARITAL_STATUS_OPTIONS};
use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui_textarea::TextArea;
use uuid::Uuid;

/// A field that can hold any number of values (first names, nationalities, languages),
/// rendered as one input box per value plus a trailing empty box to add another.
pub struct MultiValueField {
    pub boxes: Vec<TextArea<'static>>,
}

impl MultiValueField {
    pub fn new() -> Self {
        Self {
            boxes: vec![TextArea::default()],
        }
    }

    pub fn from_values(values: &[String]) -> Self {
        let mut boxes: Vec<TextArea<'static>> = values
            .iter()
            .filter(|v| !v.is_empty())
            .map(|v| TextArea::new(vec![v.clone()]))
            .collect();
        boxes.push(TextArea::default());
        Self { boxes }
    }

    pub fn values(&self) -> Vec<String> {
        self.boxes
            .iter()
            .map(|b| b.lines().join("").trim().to_string())
            .filter(|v| !v.is_empty())
            .collect()
    }

    /// Routes a key press to the box at `idx`. Typing into the last box appends a new
    /// trailing empty box; Backspace on an empty, non-last box removes it instead of
    /// being a no-op. Returns `true` if a box was removed (caller should move focus back).
    pub fn input(&mut self, idx: usize, key: KeyEvent) -> bool {
        let is_empty = self.boxes[idx].lines().join("").trim().is_empty();
        let is_last = idx == self.boxes.len() - 1;

        if key.code == KeyCode::Backspace && is_empty && !is_last {
            self.boxes.remove(idx);
            return true;
        }

        self.boxes[idx].input(key);

        if is_last {
            let content = self.boxes[idx].lines().join("");
            if !content.trim().is_empty() {
                self.boxes.push(TextArea::default());
            }
        }
        false
    }
}

/// Identifies one slot in the flat, scrollable contact form. `usize` payloads index into
/// a `MultiValueField`'s boxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactField {
    Title,
    FirstName(usize),
    LastName,
    Nickname,
    PreferredName,
    MaidenName,
    Suffix,
    Birthdate,
    DateOfDeath,
    Gender,
    Pronouns,
    Nationality(usize),
    Language(usize),
    MaritalStatus,
    Religion,
    BloodType,
    EyeColor,
    HairColor,
    Height,
    Notes,
}

pub struct ContactForm {
    pub title: TextArea<'static>,
    pub first_names: MultiValueField,
    pub last_name: TextArea<'static>,
    pub nickname: TextArea<'static>,
    pub preferred_name: TextArea<'static>,
    pub maiden_name: TextArea<'static>,
    pub suffix: TextArea<'static>,
    pub birthdate: TextArea<'static>,
    pub date_of_death: TextArea<'static>,
    pub gender: TextArea<'static>,
    pub pronouns: TextArea<'static>,
    pub nationalities: MultiValueField,
    pub languages: MultiValueField,
    pub marital_status_idx: usize,
    pub religion: TextArea<'static>,
    pub blood_type_idx: usize,
    pub eye_color: TextArea<'static>,
    pub hair_color: TextArea<'static>,
    pub height: TextArea<'static>,
    pub notes: TextArea<'static>,
    /// Flat index into `field_order()`.
    pub active_field: usize,
    /// Scroll offset so the focused field stays visible.
    pub scroll: u16,
}

fn single(value: &str) -> TextArea<'static> {
    TextArea::new(vec![value.to_string()])
}

impl ContactForm {
    pub fn empty() -> Self {
        Self {
            title: TextArea::default(),
            first_names: MultiValueField::new(),
            last_name: TextArea::default(),
            nickname: TextArea::default(),
            preferred_name: TextArea::default(),
            maiden_name: TextArea::default(),
            suffix: TextArea::default(),
            birthdate: TextArea::default(),
            date_of_death: TextArea::default(),
            gender: TextArea::default(),
            pronouns: TextArea::default(),
            nationalities: MultiValueField::new(),
            languages: MultiValueField::new(),
            marital_status_idx: 0,
            religion: TextArea::default(),
            blood_type_idx: 0,
            eye_color: TextArea::default(),
            hair_color: TextArea::default(),
            height: TextArea::default(),
            notes: TextArea::default(),
            active_field: 0,
            scroll: 0,
        }
    }

    pub fn from_contact(contact: &Contact) -> Self {
        Self {
            title: single(&contact.title),
            first_names: MultiValueField::from_values(&contact.first_names),
            last_name: single(&contact.last_name),
            nickname: single(&contact.nickname),
            preferred_name: single(&contact.preferred_name),
            maiden_name: single(&contact.maiden_name),
            suffix: single(&contact.suffix),
            birthdate: single(
                &contact
                    .birthdate
                    .map(format_localized_date)
                    .unwrap_or_default(),
            ),
            date_of_death: single(
                &contact
                    .date_of_death
                    .map(format_localized_date)
                    .unwrap_or_default(),
            ),
            gender: single(&contact.gender),
            pronouns: single(&contact.pronouns),
            nationalities: MultiValueField::from_values(&contact.nationalities),
            languages: MultiValueField::from_values(&contact.languages),
            marital_status_idx: MARITAL_STATUS_OPTIONS
                .iter()
                .position(|&opt| opt == contact.marital_status)
                .unwrap_or(0),
            religion: single(&contact.religion),
            blood_type_idx: BLOOD_TYPE_OPTIONS
                .iter()
                .position(|&opt| opt == contact.blood_type)
                .unwrap_or(0),
            eye_color: single(&contact.eye_color),
            hair_color: single(&contact.hair_color),
            height: single(&contact.height.map(|h| h.to_string()).unwrap_or_default()),
            notes: TextArea::new(contact.notes.lines().map(String::from).collect()),
            active_field: 0,
            scroll: 0,
        }
    }

    /// Validates and builds a `Contact` from the current form values.
    pub fn to_contact(&self, id: String) -> Result<Contact, String> {
        let text = |ta: &TextArea<'static>| ta.lines().join("").trim().to_string();

        let first_names = self.first_names.values();
        let last_name = text(&self.last_name);
        if first_names.is_empty() && last_name.is_empty() {
            return Err("First Name or Last Name is required".to_string());
        }

        let height_str = text(&self.height);
        let height = if height_str.is_empty() {
            None
        } else {
            Some(
                height_str
                    .parse::<u32>()
                    .map_err(|_| "Height must be a positive number".to_string())?,
            )
        };

        let parse_date = |s: &str, field: &str| -> Result<Option<NaiveDate>, String> {
            if s.is_empty() {
                return Ok(None);
            }
            let (placeholder, _) = get_date_format_info();
            parse_localized_date(s).map(Some).ok_or_else(|| {
                if placeholder == "YYYY-MM-DD" {
                    format!("{} must be in YYYY-MM-DD format", field)
                } else {
                    format!("{} must be in {} or YYYY-MM-DD format", field, placeholder)
                }
            })
        };

        let birthdate = parse_date(&text(&self.birthdate), "Birthdate")?;
        let date_of_death = parse_date(&text(&self.date_of_death), "Date of Death")?;

        Ok(Contact {
            id,
            title: text(&self.title),
            first_names,
            last_name,
            nickname: text(&self.nickname),
            preferred_name: text(&self.preferred_name),
            maiden_name: text(&self.maiden_name),
            suffix: text(&self.suffix),
            gender: text(&self.gender),
            pronouns: text(&self.pronouns),
            nationalities: self.nationalities.values(),
            languages: self.languages.values(),
            religion: text(&self.religion),
            marital_status: MARITAL_STATUS_OPTIONS[self.marital_status_idx].to_string(),
            blood_type: BLOOD_TYPE_OPTIONS[self.blood_type_idx].to_string(),
            eye_color: text(&self.eye_color),
            hair_color: text(&self.hair_color),
            height,
            notes: self.notes.lines().join("\n").trim().to_string(),
            birthdate,
            date_of_death,
        })
    }

    /// The flat, ordered list of fields shown in the scrollable form.
    pub fn field_order(&self) -> Vec<ContactField> {
        let mut fields = vec![ContactField::Title];
        for i in 0..self.first_names.boxes.len() {
            fields.push(ContactField::FirstName(i));
        }
        fields.push(ContactField::LastName);
        fields.push(ContactField::Nickname);
        fields.push(ContactField::PreferredName);
        fields.push(ContactField::MaidenName);
        fields.push(ContactField::Suffix);
        fields.push(ContactField::Birthdate);
        fields.push(ContactField::DateOfDeath);
        fields.push(ContactField::Gender);
        fields.push(ContactField::Pronouns);
        for i in 0..self.nationalities.boxes.len() {
            fields.push(ContactField::Nationality(i));
        }
        for i in 0..self.languages.boxes.len() {
            fields.push(ContactField::Language(i));
        }
        fields.push(ContactField::MaritalStatus);
        fields.push(ContactField::Religion);
        fields.push(ContactField::BloodType);
        fields.push(ContactField::EyeColor);
        fields.push(ContactField::HairColor);
        fields.push(ContactField::Height);
        fields.push(ContactField::Notes);
        fields
    }

    pub fn num_fields(&self) -> usize {
        self.field_order().len()
    }

    pub fn field_at(&self, idx: usize) -> ContactField {
        self.field_order()[idx]
    }

    /// Routes a key press to whichever field is currently focused.
    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.field_at(self.active_field) {
            ContactField::Title => {
                self.title.input(key);
            }
            ContactField::FirstName(i) => {
                if self.first_names.input(i, key) {
                    self.active_field = self.active_field.saturating_sub(1);
                }
            }
            ContactField::LastName => {
                self.last_name.input(key);
            }
            ContactField::Nickname => {
                self.nickname.input(key);
            }
            ContactField::PreferredName => {
                self.preferred_name.input(key);
            }
            ContactField::MaidenName => {
                self.maiden_name.input(key);
            }
            ContactField::Suffix => {
                self.suffix.input(key);
            }
            ContactField::Birthdate => {
                self.birthdate.input(key);
            }
            ContactField::DateOfDeath => {
                self.date_of_death.input(key);
            }
            ContactField::Gender => {
                self.gender.input(key);
            }
            ContactField::Pronouns => {
                self.pronouns.input(key);
            }
            ContactField::Nationality(i) => {
                if self.nationalities.input(i, key) {
                    self.active_field = self.active_field.saturating_sub(1);
                }
            }
            ContactField::Language(i) => {
                if self.languages.input(i, key) {
                    self.active_field = self.active_field.saturating_sub(1);
                }
            }
            ContactField::MaritalStatus => match key.code {
                KeyCode::Left => {
                    self.marital_status_idx = if self.marital_status_idx == 0 {
                        MARITAL_STATUS_OPTIONS.len() - 1
                    } else {
                        self.marital_status_idx - 1
                    };
                }
                KeyCode::Right => {
                    self.marital_status_idx =
                        (self.marital_status_idx + 1) % MARITAL_STATUS_OPTIONS.len();
                }
                _ => {}
            },
            ContactField::Religion => {
                self.religion.input(key);
            }
            ContactField::BloodType => match key.code {
                KeyCode::Left => {
                    self.blood_type_idx = if self.blood_type_idx == 0 {
                        BLOOD_TYPE_OPTIONS.len() - 1
                    } else {
                        self.blood_type_idx - 1
                    };
                }
                KeyCode::Right => {
                    self.blood_type_idx = (self.blood_type_idx + 1) % BLOOD_TYPE_OPTIONS.len();
                }
                _ => {}
            },
            ContactField::EyeColor => {
                self.eye_color.input(key);
            }
            ContactField::HairColor => {
                self.hair_color.input(key);
            }
            ContactField::Height => {
                self.height.input(key);
            }
            ContactField::Notes => {
                self.notes.input(key);
            }
        }
    }

    pub fn focus_next(&mut self) {
        let len = self.num_fields();
        if len > 0 {
            self.active_field = (self.active_field + 1) % len;
        }
    }

    pub fn focus_prev(&mut self) {
        let len = self.num_fields();
        if len > 0 {
            self.active_field = if self.active_field == 0 {
                len - 1
            } else {
                self.active_field - 1
            };
        }
    }
}

impl App {
    /// Resets the contact form for creating a new contact, or loads the selected
    /// contact's data for editing.
    pub fn init_contact_form(&mut self, is_edit: bool) {
        self.error_msg = None;
        self.contact_form = if is_edit {
            if let Some(real_idx) = self.selected_contact_idx() {
                ContactForm::from_contact(&self.journal.contacts[real_idx])
            } else {
                ContactForm::empty()
            }
        } else {
            ContactForm::empty()
        };
        self.mode = super::AppMode::Writing { is_edit };
    }

    /// Validates the contact form and saves it (creating or updating), persisting to disk.
    pub fn save_contact(&mut self) {
        let real_idx = if let super::AppMode::Writing { is_edit: true } = self.mode {
            match self.selected_contact_idx() {
                Some(idx) => Some(idx),
                None => return,
            }
        } else {
            None
        };

        let id = match real_idx {
            None => Uuid::new_v4().to_string(),
            Some(idx) => self.journal.contacts[idx].id.clone(),
        };

        let contact = match self.contact_form.to_contact(id) {
            Ok(contact) => contact,
            Err(e) => {
                self.error_msg = Some(format!("Error: {}", e));
                return;
            }
        };

        if let Some(idx) = real_idx {
            self.journal.contacts[idx] = contact;
            self.sort_contacts();
            self.status_msg = Some("Contact updated".to_string());
        } else {
            self.journal.contacts.push(contact);
            self.sort_contacts();
            self.selected_index = 0;
            self.status_msg = Some("New contact saved".to_string());
        }

        if let Err(e) = self.save_journal() {
            self.error_msg = Some(format!("Write failed: {}", e));
        } else {
            self.mode = super::AppMode::List;
            self.detail_scroll = 0;
            self.error_msg = None;
        }
    }

    pub fn delete_selected_contact(&mut self) {
        let real_idx = match self.selected_contact_idx() {
            Some(idx) => idx,
            None => {
                self.mode = super::AppMode::List;
                return;
            }
        };

        self.journal.contacts.remove(real_idx);

        if let Err(e) = self.save_journal() {
            self.error_msg = Some(format!("Delete write failed: {}", e));
        } else {
            self.status_msg = Some("Contact deleted".to_string());
            self.error_msg = None;
        }

        let len = self.filtered_contacts().len();
        if len == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= len {
            self.selected_index = len - 1;
        }

        self.mode = super::AppMode::List;
        self.detail_scroll = 0;
    }

    /// Journal entries that mention the given contact via its `{{person|id}}` tag.
    pub fn get_mentions_for_contact(&self, contact_id: &str) -> Vec<&JournalEntry> {
        if contact_id.is_empty() {
            return Vec::new();
        }
        let target = format!("{{{{person|{}}}}}", contact_id);
        self.journal
            .entries
            .iter()
            .filter(|entry| entry.content.contains(&target))
            .collect()
    }

    /// Checks if the contact form has any unsaved modifications.
    pub fn is_contact_form_dirty(&self, is_edit: bool) -> bool {
        let current = match self.contact_form.to_contact(String::new()) {
            Ok(c) => c,
            Err(_) => return true,
        };
        if is_edit {
            if let Some(real_idx) = self.selected_contact_idx() {
                let original = &self.journal.contacts[real_idx];
                original.title != current.title
                    || original.first_names != current.first_names
                    || original.last_name != current.last_name
                    || original.nickname != current.nickname
                    || original.preferred_name != current.preferred_name
                    || original.maiden_name != current.maiden_name
                    || original.suffix != current.suffix
                    || original.gender != current.gender
                    || original.pronouns != current.pronouns
                    || original.nationalities != current.nationalities
                    || original.languages != current.languages
                    || original.religion != current.religion
                    || original.marital_status != current.marital_status
                    || original.blood_type != current.blood_type
                    || original.eye_color != current.eye_color
                    || original.hair_color != current.hair_color
                    || original.height != current.height
                    || original.notes != current.notes
                    || original.birthdate != current.birthdate
                    || original.date_of_death != current.date_of_death
            } else {
                true
            }
        } else {
            let default = Contact::default();
            default.title != current.title
                || default.first_names != current.first_names
                || default.last_name != current.last_name
                || default.nickname != current.nickname
                || default.preferred_name != current.preferred_name
                || default.maiden_name != current.maiden_name
                || default.suffix != current.suffix
                || default.gender != current.gender
                || default.pronouns != current.pronouns
                || default.nationalities != current.nationalities
                || default.languages != current.languages
                || default.religion != current.religion
                || current.marital_status != "N/A"
                || current.blood_type != "N/A"
                || default.eye_color != current.eye_color
                || default.hair_color != current.hair_color
                || default.height != current.height
                || default.notes != current.notes
                || default.birthdate != current.birthdate
                || default.date_of_death != current.date_of_death
        }
    }
}
