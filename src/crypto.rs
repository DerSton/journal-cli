//! Cryptographic module for secure key derivation and symmetric authenticated encryption.
//!
//! Provides utilities to derive strong keys using Argon2id and perform authenticated
//! encryption/decryption using ChaCha20Poly1305.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};

/// The size of the derived symmetric key in bytes (256 bits).
pub const KEY_SIZE: usize = 32;
/// The required size of the salt in bytes (128 bits).
pub const SALT_SIZE: usize = 16;
/// The required size of the initialization vector (nonce) in bytes (96 bits).
pub const NONCE_SIZE: usize = 12;

/// Derives a 32-byte key from a password and salt using Argon2id.
///
/// Follows OWASP recommendations for parameter choices (19 MiB memory, 2 passes, 1 thread).
///
/// # Examples
///
/// ```
/// use journal_cli::crypto::{derive_key, SALT_SIZE};
///
/// let password = "my_secure_password";
/// let salt = [0u8; SALT_SIZE];
/// let key = derive_key(password, &salt).unwrap();
/// assert_eq!(key.len(), 32);
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The provided salt does not match [`SALT_SIZE`].
/// - The Argon2id hashing process fails.
pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_SIZE], String> {
    if salt.len() != SALT_SIZE {
        return Err(format!("Salt must be exactly {} bytes", SALT_SIZE));
    }

    let params = Params::new(19456, 2, 1, Some(KEY_SIZE))
        .map_err(|e| format!("Failed to initialize Argon2 parameters: {}", e))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut derived_key = [0u8; KEY_SIZE];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut derived_key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    Ok(derived_key)
}

/// Encrypts plaintext bytes using ChaCha20Poly1305 with the derived key and a nonce.
///
/// # Examples
///
/// ```
/// use journal_cli::crypto::{encrypt, KEY_SIZE, NONCE_SIZE};
///
/// let key = [0u8; KEY_SIZE];
/// let nonce = [0u8; NONCE_SIZE];
/// let plaintext = b"Hello, World!";
/// let ciphertext = encrypt(&key, &nonce, plaintext).unwrap();
/// assert!(!ciphertext.is_empty());
/// ```
///
/// # Errors
///
/// Returns an error if the authenticated encryption operation fails.
pub fn encrypt(
    key: &[u8; KEY_SIZE],
    nonce: &[u8; NONCE_SIZE],
    plaintext: &[u8],
) -> Result<Vec<u8>, String> {
    let key_ref = Key::from(*key);
    let cipher = ChaCha20Poly1305::new(&key_ref);
    let nonce_ref = Nonce::from(*nonce);

    cipher
        .encrypt(&nonce_ref, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))
}

/// Decrypts ciphertext bytes using ChaCha20Poly1305 with the derived key and a nonce.
///
/// # Examples
///
/// ```
/// use journal_cli::crypto::{encrypt, decrypt, KEY_SIZE, NONCE_SIZE};
///
/// let key = [0u8; KEY_SIZE];
/// let nonce = [0u8; NONCE_SIZE];
/// let plaintext = b"Hello, World!";
/// let ciphertext = encrypt(&key, &nonce, plaintext).unwrap();
/// let decrypted = decrypt(&key, &nonce, &ciphertext).unwrap();
/// assert_eq!(decrypted, plaintext);
/// ```
///
/// # Errors
///
/// Returns an error if decryption fails, which usually indicates:
/// - An invalid password or key.
/// - Corrupted or tampered ciphertext.
/// - An invalid nonce.
pub fn decrypt(
    key: &[u8; KEY_SIZE],
    nonce: &[u8; NONCE_SIZE],
    ciphertext: &[u8],
) -> Result<Vec<u8>, String> {
    let key_ref = Key::from(*key);
    let cipher = ChaCha20Poly1305::new(&key_ref);
    let nonce_ref = Nonce::from(*nonce);

    cipher.decrypt(&nonce_ref, ciphertext).map_err(|e| {
        format!(
            "Decryption failed (invalid password or corrupted file): {}",
            e
        )
    })
}

pub mod shamir;
pub use shamir::{parse_share, reconstruct_password, split_password};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let password = "my_secure_password";
        let salt = [0u8; SALT_SIZE];
        let nonce = [1u8; NONCE_SIZE];
        let plaintext = b"Hello, secure journal world!";

        let key = derive_key(password, &salt).unwrap();
        let ciphertext = encrypt(&key, &nonce, plaintext).unwrap();
        let decrypted = decrypt(&key, &nonce, &ciphertext).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_bad_password_fails_decryption() {
        let password = "my_secure_password";
        let bad_password = "wrong_password";
        let salt = [0u8; SALT_SIZE];
        let nonce = [1u8; NONCE_SIZE];
        let plaintext = b"Hello, secure journal world!";

        let key = derive_key(password, &salt).unwrap();
        let bad_key = derive_key(bad_password, &salt).unwrap();

        let ciphertext = encrypt(&key, &nonce, plaintext).unwrap();
        let decrypt_result = decrypt(&bad_key, &nonce, &ciphertext);

        assert!(decrypt_result.is_err());
    }
}
