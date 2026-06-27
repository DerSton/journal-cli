use crate::crypto::{self, NONCE_SIZE, SALT_SIZE};
use crate::model::{Contact, JournalEntry, Settings};
use rand::random;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const MAGIC_BYTES: &[u8; 4] = b"JRNL";

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Journal {
    pub entries: Vec<JournalEntry>,
    #[serde(default)]
    pub contacts: Vec<Contact>,
    #[serde(default)]
    pub settings: Settings,
}

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
    pub fn format_timestamp(&self, timestamp: &chrono::DateTime<chrono::Utc>) -> String {
        let local_dt = timestamp.with_timezone(&chrono::Local);
        let locale = get_system_locale();
        local_dt
            .format_localized("%A, %B %d, %Y - %H:%M:%S", locale)
            .to_string()
    }

    pub fn format_timestamp_short(&self, timestamp: &chrono::DateTime<chrono::Utc>) -> String {
        let local_dt = timestamp.with_timezone(&chrono::Local);
        let locale = get_system_locale();
        local_dt
            .format_localized("%Y-%m-%d %H:%M:%S", locale)
            .to_string()
    }

    pub fn format_date_short(&self, timestamp: &chrono::DateTime<chrono::Utc>) -> String {
        let local_dt = timestamp.with_timezone(&chrono::Local);
        let locale = get_system_locale();
        local_dt.format_localized("%Y-%m-%d", locale).to_string()
    }

    /// Load and decrypt a journal file using the provided password.
    ///
    /// Returns the decrypted Journal and the file's salt.
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

    /// Create, encrypt, and save a new empty journal file with a fresh random salt.
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
