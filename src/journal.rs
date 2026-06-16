use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use chrono::{DateTime, Utc};
use rand::random;
use serde::{Deserialize, Serialize};
use crate::crypto::{self, NONCE_SIZE, SALT_SIZE};

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
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Journal {
    pub entries: Vec<JournalEntry>,
    #[serde(default)]
    pub contacts: Vec<Contact>,
}

impl Journal {
    /// Load and decrypt a journal file using the provided password.
    ///
    /// Returns the decrypted Journal struct and the file's salt.
    pub fn load<P: AsRef<Path>>(path: P, password: &str) -> Result<(Self, [u8; SALT_SIZE]), String> {
        let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| format!("Failed to read file: {}", e))?;

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
    pub fn save<P: AsRef<Path>>(&self, path: P, password: &str, salt: &[u8; SALT_SIZE]) -> Result<(), String> {
        let key = crypto::derive_key(password, salt)?;

        let plaintext = serde_json::to_vec(self)
            .map_err(|e| format!("Failed to serialize journal: {}", e))?;

        let nonce: [u8; NONCE_SIZE] = random();

        let ciphertext = crypto::encrypt(&key, &nonce, &plaintext)?;

        let mut file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
        file.write_all(MAGIC_BYTES).map_err(|e| format!("Failed to write magic bytes: {}", e))?;
        file.write_all(salt).map_err(|e| format!("Failed to write salt: {}", e))?;
        file.write_all(&nonce).map_err(|e| format!("Failed to write nonce: {}", e))?;
        file.write_all(&ciphertext).map_err(|e| format!("Failed to write ciphertext: {}", e))?;

        Ok(())
    }

    /// Create, encrypt, and save a new empty journal file.
    ///
    /// Generates a fresh random salt.
    pub fn create_new<P: AsRef<Path>>(path: P, password: &str) -> Result<(Self, [u8; SALT_SIZE]), String> {
        let salt: [u8; SALT_SIZE] = random();

        let journal = Journal::default();
        journal.save(path, password, &salt)?;

        Ok((journal, salt))
    }
}
