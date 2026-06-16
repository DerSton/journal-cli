use argon2::{Argon2, Algorithm, Version, Params};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce, Key
};

pub const KEY_SIZE: usize = 32; // 256 bits
pub const SALT_SIZE: usize = 16;
pub const NONCE_SIZE: usize = 12;

/// Derives a 32-byte key from a password and salt using Argon2id.
///
/// Follows OWASP recommendations for parameter choices (19 MiB memory, 2 passes, 1 thread).
pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_SIZE], String> {
    if salt.len() != SALT_SIZE {
        return Err(format!("Salt must be exactly {} bytes", SALT_SIZE));
    }
    
    let params = Params::new(19456, 2, 1, Some(KEY_SIZE))
        .map_err(|e| format!("Failed to initialize Argon2 parameters: {}", e))?;
        
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    
    let mut derived_key = [0u8; KEY_SIZE];
    argon2.hash_password_into(password.as_bytes(), salt, &mut derived_key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
        
    Ok(derived_key)
}

/// Encrypts plaintext bytes using ChaCha20Poly1305 with the derived key and a nonce.
pub fn encrypt(key: &[u8; KEY_SIZE], nonce: &[u8; NONCE_SIZE], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let key_ref = Key::from_slice(key);
    let cipher = ChaCha20Poly1305::new(key_ref);
    let nonce_ref = Nonce::from_slice(nonce);
    
    cipher.encrypt(nonce_ref, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))
}

/// Decrypts ciphertext bytes using ChaCha20Poly1305 with the derived key and a nonce.
pub fn decrypt(key: &[u8; KEY_SIZE], nonce: &[u8; NONCE_SIZE], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    let key_ref = Key::from_slice(key);
    let cipher = ChaCha20Poly1305::new(key_ref);
    let nonce_ref = Nonce::from_slice(nonce);
    
    cipher.decrypt(nonce_ref, ciphertext)
        .map_err(|e| format!("Decryption failed (invalid password or corrupted file): {}", e))
}

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
