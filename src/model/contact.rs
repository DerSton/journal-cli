use serde::{Deserialize, Serialize};

pub const BLOOD_TYPE_OPTIONS: &[&str] = &["N/A", "A+", "A-", "B+", "B-", "AB+", "AB-", "O+", "O-"];
pub const MARITAL_STATUS_OPTIONS: &[&str] = &[
    "N/A",
    "Single",
    "Married",
    "Divorced",
    "Widowed",
    "Registered Partnership",
];

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Contact {
    pub id: String,
    pub title: String,
    pub first_names: Vec<String>,
    pub last_name: String,
    pub nickname: String,
    pub preferred_name: String,
    pub maiden_name: String,
    pub suffix: String,
    pub gender: String,
    pub pronouns: String,
    pub nationalities: Vec<String>,
    pub languages: Vec<String>,
    pub religion: String,
    pub marital_status: String,
    pub blood_type: String,
    pub eye_color: String,
    pub hair_color: String,
    pub height: Option<u32>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub birthdate: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub date_of_death: Option<chrono::NaiveDate>,
}

impl Contact {
    /// Formats a date as YYYY-MM-DD.
    pub fn format_date(date: chrono::NaiveDate) -> String {
        date.format("%Y-%m-%d").to_string()
    }

    /// The `{{person|<id>}}` tag used to mention this contact inside journal entries.
    pub fn mention_tag(&self) -> String {
        format!("{{{{person|{}}}}}", self.id)
    }

    /// Current age, or age at death if `date_of_death` is set.
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

    /// Name formatted for list rows, e.g. "Doe, John Edward".
    pub fn display_name(&self) -> String {
        let given: Vec<&str> = self
            .first_names
            .iter()
            .filter(|n| !n.is_empty())
            .map(|n| n.as_str())
            .collect();

        if !self.last_name.is_empty() {
            if given.is_empty() {
                self.last_name.clone()
            } else {
                format!("{}, {}", self.last_name, given.join(" "))
            }
        } else {
            given.join(" ")
        }
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
            "Dr. John Edward \"Johnny\" Doe Jr. (nee Smith)"
        );
    }

    #[test]
    fn display_name_is_last_comma_first() {
        let contact = sample_contact();
        assert_eq!(contact.display_name(), "Doe, John Edward");
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
}
