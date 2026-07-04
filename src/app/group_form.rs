use super::{App, format_localized_date, get_date_format_info, parse_localized_date};
use crate::model::Group;
use chrono::NaiveDate;
use crossterm::event::KeyEvent;
use ratatui_textarea::TextArea;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupField {
    Name,
    Description,
    StartDate,
    EndDate,
    Members,
}

pub struct GroupForm {
    pub name: TextArea<'static>,
    pub description: TextArea<'static>,
    pub start_date: TextArea<'static>,
    pub end_date: TextArea<'static>,
    pub selected_member_ids: HashSet<String>,
    pub active_field: usize,
    pub scroll: u16,
}

fn single(value: &str) -> TextArea<'static> {
    TextArea::new(vec![value.to_string()])
}

impl GroupForm {
    pub fn empty() -> Self {
        Self {
            name: TextArea::default(),
            description: TextArea::default(),
            start_date: TextArea::default(),
            end_date: TextArea::default(),
            selected_member_ids: HashSet::new(),
            active_field: 0,
            scroll: 0,
        }
    }

    pub fn from_group(group: &Group) -> Self {
        Self {
            name: single(&group.name),
            description: if group.description.is_empty() {
                TextArea::default()
            } else {
                TextArea::new(group.description.lines().map(String::from).collect())
            },
            start_date: single(
                &group
                    .start_date
                    .map(format_localized_date)
                    .unwrap_or_default(),
            ),
            end_date: single(
                &group
                    .end_date
                    .map(format_localized_date)
                    .unwrap_or_default(),
            ),
            selected_member_ids: group.member_ids.iter().cloned().collect(),
            active_field: 0,
            scroll: 0,
        }
    }

    pub fn to_group(&self, id: String) -> Result<Group, String> {
        let text = |ta: &TextArea<'static>| ta.lines().join("").trim().to_string();

        let name = text(&self.name);
        if name.is_empty() {
            return Err("Group Name is required".to_string());
        }

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

        let start_date = parse_date(&text(&self.start_date), "Start Date")?;
        let end_date = parse_date(&text(&self.end_date), "End Date")?;

        Ok(Group {
            id,
            name,
            description: self.description.lines().join("\n").trim().to_string(),
            member_ids: self.selected_member_ids.iter().cloned().collect(),
            start_date,
            end_date,
        })
    }

    pub fn field_order(&self) -> Vec<GroupField> {
        vec![
            GroupField::Name,
            GroupField::Description,
            GroupField::StartDate,
            GroupField::EndDate,
            GroupField::Members,
        ]
    }

    pub fn num_fields(&self) -> usize {
        self.field_order().len()
    }

    pub fn field_at(&self, idx: usize) -> GroupField {
        self.field_order()[idx]
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.active_field >= self.num_fields() {
            return;
        }
        match self.field_at(self.active_field) {
            GroupField::Name => {
                self.name.input(key);
            }
            GroupField::Description => {
                self.description.input(key);
            }
            GroupField::StartDate => {
                self.start_date.input(key);
            }
            GroupField::EndDate => {
                self.end_date.input(key);
            }
            GroupField::Members => {}
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
    pub fn init_group_form(&mut self, is_edit: bool) {
        self.error_msg = None;
        self.group_form = if is_edit {
            if let Some(real_idx) = self.selected_group_idx() {
                GroupForm::from_group(&self.journal.groups[real_idx])
            } else {
                GroupForm::empty()
            }
        } else {
            GroupForm::empty()
        };
        self.mode = super::AppMode::Writing { is_edit };
    }

    pub fn save_group(&mut self) {
        let real_idx = if let super::AppMode::Writing { is_edit: true } = self.mode {
            match self.selected_group_idx() {
                Some(idx) => Some(idx),
                None => return,
            }
        } else {
            None
        };

        let id = match real_idx {
            None => Uuid::new_v4().to_string(),
            Some(idx) => self.journal.groups[idx].id.clone(),
        };

        let group = match self.group_form.to_group(id) {
            Ok(group) => group,
            Err(e) => {
                self.error_msg = Some(format!("Error: {}", e));
                return;
            }
        };

        if let Some(idx) = real_idx {
            self.journal.groups[idx] = group;
            self.sort_groups();
            self.status_msg = Some("Group updated".to_string());
        } else {
            self.journal.groups.push(group);
            self.sort_groups();
            self.selected_index = 0;
            self.status_msg = Some("New group saved".to_string());
        }

        if let Err(e) = self.save_journal() {
            self.error_msg = Some(format!("Write failed: {}", e));
        } else {
            self.mode = super::AppMode::List;
            self.detail_scroll = 0;
            self.error_msg = None;
        }
    }

    pub fn delete_selected_group(&mut self) {
        let real_idx = match self.selected_group_idx() {
            Some(idx) => idx,
            None => {
                self.mode = super::AppMode::List;
                return;
            }
        };

        self.journal.groups.remove(real_idx);

        if let Err(e) = self.save_journal() {
            self.error_msg = Some(format!("Delete write failed: {}", e));
        } else {
            self.status_msg = Some("Group deleted".to_string());
            self.error_msg = None;
        }

        let len = self.filtered_groups().len();
        if len == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= len {
            self.selected_index = len - 1;
        }

        self.mode = super::AppMode::List;
        self.detail_scroll = 0;
    }

    pub fn filtered_groups(&self) -> Vec<&Group> {
        if self.search_query.trim().is_empty() {
            self.journal.groups.iter().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.journal
                .groups
                .iter()
                .filter(|g| {
                    g.name.to_lowercase().contains(&query)
                        || g.description.to_lowercase().contains(&query)
                })
                .collect()
        }
    }

    pub fn selected_group_idx(&self) -> Option<usize> {
        let filtered = self.filtered_groups();
        if filtered.is_empty() {
            None
        } else {
            let selected = filtered.get(self.selected_index)?;
            self.journal.groups.iter().position(|g| g.id == selected.id)
        }
    }

    pub fn sort_groups(&mut self) {
        self.journal.groups.sort_by_key(|g| g.name.to_lowercase());
    }

    pub fn get_mentions_for_group(&self, group_id: &str) -> Vec<&crate::model::JournalEntry> {
        if group_id.is_empty() {
            return Vec::new();
        }
        let target = format!("{{{{group|{}}}}}", group_id);
        self.journal
            .entries
            .iter()
            .filter(|entry| entry.content.contains(&target))
            .collect()
    }

    /// Checks if the group form has any unsaved modifications.
    pub fn is_group_form_dirty(&self, is_edit: bool) -> bool {
        let current = match self.group_form.to_group(String::new()) {
            Ok(g) => g,
            Err(_) => return true,
        };
        if is_edit {
            if let Some(real_idx) = self.selected_group_idx() {
                let original = &self.journal.groups[real_idx];
                let original_member_ids: HashSet<String> =
                    original.member_ids.iter().cloned().collect();
                original.name != current.name
                    || original.description != current.description
                    || original.start_date != current.start_date
                    || original.end_date != current.end_date
                    || original_member_ids != self.group_form.selected_member_ids
            } else {
                true
            }
        } else {
            !current.name.is_empty()
                || !current.description.is_empty()
                || current.start_date.is_some()
                || current.end_date.is_some()
                || !current.member_ids.is_empty()
        }
    }
}
