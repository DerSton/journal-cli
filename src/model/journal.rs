//! Data model representing the master encrypted journal database.
//!
//! Handles file serialization, symmetric encryption/decryption on disk,
//! and localized timestamp formatting.

use crate::crypto::{self, NONCE_SIZE, SALT_SIZE};
use crate::model::{Contact, JournalEntry, Settings};
use rand::random;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const MAGIC_BYTES: &[u8; 4] = b"JRNL";

/// The root journal database containing all entries, contacts, and settings.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Journal {
    /// List of journal entries.
    pub entries: Vec<JournalEntry>,
    /// List of contacts/people mentioned in entries.
    #[serde(default)]
    pub contacts: Vec<Contact>,
    /// User settings.
    #[serde(default)]
    pub settings: Settings,
}

/// Helper function to retrieve the user's active system locale.
///
/// Tries to parse the platform locale name into a [`chrono::format::Locale`],
/// falling back to [`chrono::format::Locale::POSIX`] on resolution failure.
pub fn get_system_locale() -> chrono::format::Locale {
    if let Some(locale_str) = sys_locale::get_locale() {
        let normalized = locale_str.replace('-', "_");
        if let Ok(locale) = normalized.parse::<chrono::format::Locale>() {
            return locale;
        }
        if let Some(locale) = normalized
            .find('_')
            .and_then(|pos| normalized[..pos].parse::<chrono::format::Locale>().ok())
        {
            return locale;
        }
    }
    chrono::format::Locale::POSIX
}

impl Journal {
    /// Formats a timestamp into a long human-readable localized string.
    pub fn format_timestamp(&self, timestamp: &chrono::DateTime<chrono::Utc>) -> String {
        let local_dt = timestamp.with_timezone(&chrono::Local);
        let locale = get_system_locale();
        local_dt
            .format_localized("%A, %B %d, %Y - %H:%M:%S", locale)
            .to_string()
    }

    /// Formats a timestamp into a short date-time string (e.g. "2026-06-16 12:00:00").
    pub fn format_timestamp_short(&self, timestamp: &chrono::DateTime<chrono::Utc>) -> String {
        let local_dt = timestamp.with_timezone(&chrono::Local);
        let locale = get_system_locale();
        local_dt
            .format_localized("%Y-%m-%d %H:%M:%S", locale)
            .to_string()
    }

    /// Formats a timestamp into a short date string (e.g. "2026-06-16").
    ///
    /// # Examples
    ///
    /// ```
    /// use journal_cli::model::Journal;
    /// use chrono::TimeZone;
    ///
    /// let journal = Journal::default();
    /// let dt = chrono::Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap();
    /// assert_eq!(journal.format_date_short(&dt), "2026-06-16");
    /// ```
    pub fn format_date_short(&self, timestamp: &chrono::DateTime<chrono::Utc>) -> String {
        let local_dt = timestamp.with_timezone(&chrono::Local);
        let locale = get_system_locale();
        local_dt.format_localized("%Y-%m-%d", locale).to_string()
    }

    /// Formats a timestamp into a human-readable localized date string without time (e.g. "Tuesday, June 16, 2026").
    pub fn format_date(&self, timestamp: &chrono::DateTime<chrono::Utc>) -> String {
        let local_dt = timestamp.with_timezone(&chrono::Local);
        let locale = get_system_locale();
        local_dt
            .format_localized("%A, %B %d, %Y", locale)
            .to_string()
    }

    /// Loads and decrypts a journal file using the provided password.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read or opened.
    /// - The file size is too short to contain magic bytes, salt, and nonce.
    /// - The magic bytes do not match `JRNL`.
    /// - Decryption fails (indicating an incorrect password or corrupted data).
    /// - JSON deserialization fails.
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

    /// Encrypts and saves the journal to disk using the provided password and salt.
    ///
    /// Always generates a fresh nonce for encryption.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization, encryption, or writing fails.
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

    /// Creates, encrypts, and saves a new empty journal file with a fresh random salt.
    ///
    /// # Errors
    ///
    /// Returns an error if encryption or file writing fails.
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
    fn settings_default_when_missing() {
        let journal: Journal = serde_json::from_str(r#"{"entries": []}"#).unwrap();
        assert!(journal.entries.is_empty());
        assert_eq!(journal.settings.autolock_timeout_mins, 5);
        assert!(journal.settings.lock_on_suspend);
        assert_eq!(journal.settings.ollama_days, 7);
    }

    #[test]
    fn format_timestamp_is_human_readable() {
        let journal = Journal::default();
        let dt = chrono::Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap();
        let formatted = journal.format_timestamp(&dt);
        let local_dt = dt.with_timezone(&chrono::Local);
        let expected_hour = local_dt.format("%H:%M:%S").to_string();
        assert!(formatted.contains("2026"));
        assert!(formatted.contains(&expected_hour));
    }
}
