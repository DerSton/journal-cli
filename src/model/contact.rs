//! Data model representing contacts/people tracked in the journal.
//!
//! Provides the [`Contact`] struct and formatting utilities for rendering
//! names and computing age.

use serde::{Deserialize, Serialize};

/// Predefined list of standard human blood types.
pub const BLOOD_TYPE_OPTIONS: &[&str] = &["N/A", "A+", "A-", "B+", "B-", "AB+", "AB-", "O+", "O-"];
/// Predefined list of standard marital statuses.
pub const MARITAL_STATUS_OPTIONS: &[&str] = &[
    "N/A",
    "Single",
    "Married",
    "Divorced",
    "Widowed",
    "Registered Partnership",
];

/// Detailed contact information for a person.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Contact {
    /// A unique identifier for the contact, usually a UUID.
    pub id: String,
    /// Professional or social title (e.g., "Dr.", "Ms.").
    pub title: String,
    /// Given or first names.
    pub first_names: Vec<String>,
    /// Family name or surname.
    pub last_name: String,
    /// An informal, short name.
    pub nickname: String,
    /// The contact's preferred first name.
    pub preferred_name: String,
    /// Surname before marriage.
    pub maiden_name: String,
    /// Suffix appended to name (e.g., "Jr.", "III").
    pub suffix: String,
    /// Social gender identity.
    pub gender: String,
    /// Preferred pronouns (e.g., "they/them").
    pub pronouns: String,
    /// Countries of nationality.
    pub nationalities: Vec<String>,
    /// Spoken or written languages.
    pub languages: Vec<String>,
    /// Religious affiliation.
    pub religion: String,
    /// Legal marital status.
    pub marital_status: String,
    /// Blood type, selected from [`BLOOD_TYPE_OPTIONS`].
    pub blood_type: String,
    /// Described eye color.
    pub eye_color: String,
    /// Described hair color.
    pub hair_color: String,
    /// Height in centimeters.
    pub height: Option<u32>,
    /// Additional biography or notes.
    pub notes: String,
    /// Birthdate.
    #[serde(default)]
    pub birthdate: Option<chrono::NaiveDate>,
    /// Date of death, if deceased.
    #[serde(default)]
    pub date_of_death: Option<chrono::NaiveDate>,
}

impl Contact {
    /// The `{{person|<id>}}` tag used to mention this contact inside journal entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use journal_cli::model::Contact;
    ///
    /// let contact = Contact { id: "123-abc".to_string(), ..Default::default() };
    /// assert_eq!(contact.mention_tag(), "{{person|123-abc}}");
    /// ```
    pub fn mention_tag(&self) -> String {
        format!("{{{{person|{}}}}}", self.id)
    }

    /// Current age, or age at death if `date_of_death` is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use journal_cli::model::Contact;
    /// use chrono::NaiveDate;
    ///
    /// let mut contact = Contact::default();
    /// contact.birthdate = Some(NaiveDate::from_ymd_opt(2000, 5, 15).unwrap());
    /// contact.date_of_death = Some(NaiveDate::from_ymd_opt(2020, 5, 15).unwrap());
    ///
    /// assert_eq!(contact.calculate_age(), Some(20));
    /// ```
    pub fn calculate_age(&self) -> Option<u32> {
        use chrono::Datelike;
        let birth = self.birthdate?;
        let end_date = self
            .date_of_death
            .unwrap_or_else(|| chrono::Local::now().date_naive());
        if end_date < birth {
            return None;
        }
        let mut age = end_date.year() - birth.year();
        if end_date.month() < birth.month()
            || (end_date.month() == birth.month() && end_date.day() < birth.day())
        {
            age -= 1;
        }
        Some(age as u32)
    }

    /// Full display name including title, all given names, preferred/maiden names, and suffix.
    ///
    /// # Examples
    ///
    /// ```
    /// use journal_cli::model::Contact;
    ///
    /// let contact = Contact {
    ///     title: "Dr.".to_string(),
    ///     first_names: vec!["John".to_string(), "Edward".to_string()],
    ///     last_name: "Doe".to_string(),
    ///     preferred_name: "Johnny".to_string(),
    ///     nickname: "Jack".to_string(),
    ///     suffix: "Jr.".to_string(),
    ///     maiden_name: "Smith".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(contact.full_name(), "Dr. John Edward \"Johnny\" 'Jack' Doe Jr. (nee Smith)");
    /// ```
    pub fn full_name(&self) -> String {
        let mut parts = Vec::new();
        if !self.title.is_empty() {
            parts.push(self.title.clone());
        }
        for name in &self.first_names {
            if !name.is_empty() {
                parts.push(name.clone());
            }
        }
        if !self.preferred_name.is_empty() {
            parts.push(format!("\"{}\"", self.preferred_name));
        }
        if !self.nickname.is_empty() {
            parts.push(format!("'{}'", self.nickname));
        }
        if !self.last_name.is_empty() {
            parts.push(self.last_name.clone());
        }
        if !self.suffix.is_empty() {
            parts.push(self.suffix.clone());
        }
        if !self.maiden_name.is_empty() {
            parts.push(format!("(nee {})", self.maiden_name));
        }
        parts.join(" ")
    }

    /// Name formatted for list rows (using full name formatting for consistency).
    ///
    /// # Examples
    ///
    /// ```
    /// use journal_cli::model::Contact;
    ///
    /// let contact = Contact {
    ///     first_names: vec!["John".to_string(), "Edward".to_string()],
    ///     last_name: "Doe".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(contact.display_name(), "John Edward Doe");
    /// ```
    pub fn display_name(&self) -> String {
        self.full_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn sample_contact() -> Contact {
        Contact {
            id: "123".to_string(),
            first_names: vec!["John".to_string(), "Edward".to_string()],
            last_name: "Doe".to_string(),
            title: "Dr.".to_string(),
            nickname: "Jack".to_string(),
            preferred_name: "Johnny".to_string(),
            suffix: "Jr.".to_string(),
            maiden_name: "Smith".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn full_name_combines_all_name_parts() {
        let contact = sample_contact();
        assert_eq!(
            contact.full_name(),
            "Dr. John Edward \"Johnny\" 'Jack' Doe Jr. (nee Smith)"
        );
    }

    #[test]
    fn display_name_uses_full_name() {
        let contact = sample_contact();
        assert_eq!(
            contact.display_name(),
            "Dr. John Edward \"Johnny\" 'Jack' Doe Jr. (nee Smith)"
        );
    }

    #[test]
    fn mention_tag_uses_id() {
        let contact = sample_contact();
        assert_eq!(contact.mention_tag(), "{{person|123}}");
    }

    #[test]
    fn age_and_serialization_roundtrip() {
        let mut contact = sample_contact();
        contact.birthdate = Some(NaiveDate::from_ymd_opt(1990, 5, 15).unwrap());
        contact.date_of_death = None;

        // Alive: age relative to now, at least 36 by 2026.
        assert!(contact.calculate_age().unwrap_or(0) >= 36);

        // Deceased: age is exact based on date of death.
        contact.date_of_death = Some(NaiveDate::from_ymd_opt(2026, 6, 16).unwrap());
        assert_eq!(contact.calculate_age(), Some(36));

        // Birthday not yet reached in the year of death.
        contact.date_of_death = Some(NaiveDate::from_ymd_opt(2026, 5, 14).unwrap());
        assert_eq!(contact.calculate_age(), Some(35));

        let serialized = serde_json::to_string(&contact).unwrap();
        let deserialized: Contact = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.birthdate, contact.birthdate);
        assert_eq!(deserialized.date_of_death, contact.date_of_death);
    }

    #[test]
    fn test_localized_date_parsing_and_formatting() {
        let date = NaiveDate::from_ymd_opt(1990, 5, 15).unwrap();

        let parsed_iso = crate::app::parse_localized_date("1990-05-15");
        assert_eq!(parsed_iso, Some(date));

        let formatted = crate::app::format_localized_date(date);
        let parsed_formatted = crate::app::parse_localized_date(&formatted);
        assert_eq!(parsed_formatted, Some(date));
    }
}
