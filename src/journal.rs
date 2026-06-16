use crate::crypto::{self, NONCE_SIZE, SALT_SIZE};
use chrono::{DateTime, Utc};
use rand::random;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const MAGIC_BYTES: &[u8; 4] = b"JRNL";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JournalEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Contact {
    pub id: String,
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
    #[serde(default)]
    pub handle: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub birthdate: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub date_of_death: Option<chrono::NaiveDate>,
}

impl Contact {
    /// Helper to format a NaiveDate.
    pub fn format_date(date: chrono::NaiveDate) -> String {
        date.format("%Y-%m-%d").to_string()
    }

    /// Calculates current age, or age at death if date_of_death is set.
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
    /// Returns the full name with a single space separating the non-empty fields.
    pub fn full_name(&self) -> String {
        let mut parts = Vec::new();
        if !self.first_name.is_empty() {
            parts.push(self.first_name.as_str());
        }
        if !self.middle_name.is_empty() {
            parts.push(self.middle_name.as_str());
        }
        if !self.last_name.is_empty() {
            parts.push(self.last_name.as_str());
        }
        parts.join(" ")
    }

    /// Returns the name formatted for list displays (e.g. "Doe, John Middle").
    /// Avoids leading commas and double spaces when fields are missing.
    pub fn display_name(&self) -> String {
        let mut parts = Vec::new();
        if !self.last_name.is_empty() {
            let mut first_mid = Vec::new();
            if !self.first_name.is_empty() {
                first_mid.push(self.first_name.as_str());
            }
            if !self.middle_name.is_empty() {
                first_mid.push(self.middle_name.as_str());
            }
            if first_mid.is_empty() {
                parts.push(self.last_name.clone());
            } else {
                parts.push(format!("{}, {}", self.last_name, first_mid.join(" ")));
            }
        } else {
            if !self.first_name.is_empty() {
                parts.push(self.first_name.clone());
            }
            if !self.middle_name.is_empty() {
                parts.push(self.middle_name.clone());
            }
        }
        parts.join(" ")
    }

    /// Returns the initials of the contact, up to 2 characters, without using '?' unless completely empty.
    pub fn initials(&self) -> String {
        let mut initials = String::new();
        if let Some(c) = self.first_name.chars().next() {
            initials.push(c.to_uppercase().next().unwrap());
        }
        if let Some(c) = self.middle_name.chars().next() {
            initials.push(c.to_uppercase().next().unwrap());
        }
        if let Some(c) = self.last_name.chars().next() {
            initials.push(c.to_uppercase().next().unwrap());
        }

        if initials.is_empty() {
            "??".to_string()
        } else if initials.len() > 2 {
            let first_char = initials.chars().next().unwrap();
            let last_char = initials.chars().nth(2).unwrap();
            format!("{}{}", first_char, last_char)
        } else {
            initials
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Settings {}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Journal {
    pub entries: Vec<JournalEntry>,
    #[serde(default)]
    pub contacts: Vec<Contact>,
    #[serde(default)]
    pub settings: Settings,
}

impl Journal {
    /// Formats a UTC timestamp.
    pub fn format_timestamp(&self, timestamp: &chrono::DateTime<chrono::Utc>) -> String {
        timestamp.format("%A, %B %d, %Y - %H:%M:%S").to_string()
    }

    /// Formats a UTC timestamp in a short date-time format.
    pub fn format_timestamp_short(&self, timestamp: &chrono::DateTime<chrono::Utc>) -> String {
        timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Formats a UTC timestamp in a short date format (e.g. YYYY-MM-DD).
    pub fn format_date_short(&self, timestamp: &chrono::DateTime<chrono::Utc>) -> String {
        timestamp.format("%Y-%m-%d").to_string()
    }

    /// Load and decrypt a journal file using the provided password.
    ///
    /// Returns the decrypted Journal struct and the file's salt.
    pub fn load<P: AsRef<Path>>(
        path: P,
        password: &str,
    ) -> Result<(Self, [u8; SALT_SIZE]), String> {
        let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        if buffer.len() < 4 + SALT_SIZE + NONCE_SIZE {
            return Err("File is too short or corrupted".to_string());
        }

        if &buffer[0..4] != MAGIC_BYTES {
            return Err("Invalid file format: Magic bytes mismatch".to_string());
        }

        let mut salt = [0u8; SALT_SIZE];
        salt.copy_from_slice(&buffer[4..20]);

        let mut nonce = [0u8; NONCE_SIZE];
        nonce.copy_from_slice(&buffer[20..32]);

        let ciphertext = &buffer[32..];

        let key = crypto::derive_key(password, &salt)?;
        let plaintext = crypto::decrypt(&key, &nonce, ciphertext)?;

        let journal: Journal = serde_json::from_slice(&plaintext)
            .map_err(|e| format!("Failed to deserialize journal JSON: {}", e))?;

        Ok((journal, salt))
    }

    /// Encrypt and save the journal file using the provided password and salt.
    ///
    /// Always generates a fresh nonce for encryption.
    pub fn save<P: AsRef<Path>>(
        &self,
        path: P,
        password: &str,
        salt: &[u8; SALT_SIZE],
    ) -> Result<(), String> {
        let key = crypto::derive_key(password, salt)?;

        let plaintext =
            serde_json::to_vec(self).map_err(|e| format!("Failed to serialize journal: {}", e))?;

        let nonce: [u8; NONCE_SIZE] = random();

        let ciphertext = crypto::encrypt(&key, &nonce, &plaintext)?;

        let mut file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
        file.write_all(MAGIC_BYTES)
            .map_err(|e| format!("Failed to write magic bytes: {}", e))?;
        file.write_all(salt)
            .map_err(|e| format!("Failed to write salt: {}", e))?;
        file.write_all(&nonce)
            .map_err(|e| format!("Failed to write nonce: {}", e))?;
        file.write_all(&ciphertext)
            .map_err(|e| format!("Failed to write ciphertext: {}", e))?;

        Ok(())
    }

    /// Create, encrypt, and save a new empty journal file.
    ///
    /// Generates a fresh random salt.
    pub fn create_new<P: AsRef<Path>>(
        path: P,
        password: &str,
    ) -> Result<(Self, [u8; SALT_SIZE]), String> {
        let salt: [u8; SALT_SIZE] = random();

        let journal = Journal::default();
        journal.save(path, password, &salt)?;

        Ok((journal, salt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_settings_serialization_defaults() {
        let json_str = r#"{"entries": []}"#;
        let journal: Journal = serde_json::from_str(json_str).unwrap();
        // Just verify it deserializes correctly
        assert!(journal.entries.is_empty());
    }

    #[test]
    fn test_format_timestamp() {
        let journal = Journal::default();
        let dt = chrono::Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap();

        let formatted = journal.format_timestamp(&dt);
        assert!(formatted.contains("Tuesday"));
        assert!(formatted.contains("June"));
        assert!(formatted.contains("12:00:00"));
    }

    #[test]
    fn test_contact_age_and_serialization() {
        use chrono::NaiveDate;

        let mut contact = Contact {
            id: "123".to_string(),
            first_name: "John".to_string(),
            middle_name: "".to_string(),
            last_name: "Doe".to_string(),
            handle: "johndoe".to_string(),
            notes: "".to_string(),
            birthdate: Some(NaiveDate::from_ymd_opt(1990, 5, 15).unwrap()),
            date_of_death: None,
        };

        // If alive, age is calculated relative to now (non-deterministic, but we know it's at least 36 years if we are in 2026).
        assert!(contact.calculate_age().unwrap_or(0) >= 36);

        // If deceased, age is exact based on date of death
        contact.date_of_death = Some(NaiveDate::from_ymd_opt(2026, 6, 16).unwrap());
        // 1990-05-15 to 2026-06-16 is 36 years, 1 month
        assert_eq!(contact.calculate_age(), Some(36));

        // Birthday not yet reached (e.g. death day is May 14th)
        contact.date_of_death = Some(NaiveDate::from_ymd_opt(2026, 5, 14).unwrap());
        assert_eq!(contact.calculate_age(), Some(35));

        // Serialization check
        let serialized = serde_json::to_string(&contact).unwrap();
        let deserialized: Contact = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            deserialized.birthdate,
            Some(NaiveDate::from_ymd_opt(1990, 5, 15).unwrap())
        );
        assert_eq!(
            deserialized.date_of_death,
            Some(NaiveDate::from_ymd_opt(2026, 5, 14).unwrap())
        );
    }
}
