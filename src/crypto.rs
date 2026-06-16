use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
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
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut derived_key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    Ok(derived_key)
}

/// Encrypts plaintext bytes using ChaCha20Poly1305 with the derived key and a nonce.
pub fn encrypt(
    key: &[u8; KEY_SIZE],
    nonce: &[u8; NONCE_SIZE],
    plaintext: &[u8],
) -> Result<Vec<u8>, String> {
    let key_ref = Key::from_slice(key);
    let cipher = ChaCha20Poly1305::new(key_ref);
    let nonce_ref = Nonce::from_slice(nonce);

    cipher
        .encrypt(nonce_ref, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))
}

/// Decrypts ciphertext bytes using ChaCha20Poly1305 with the derived key and a nonce.
pub fn decrypt(
    key: &[u8; KEY_SIZE],
    nonce: &[u8; NONCE_SIZE],
    ciphertext: &[u8],
) -> Result<Vec<u8>, String> {
    let key_ref = Key::from_slice(key);
    let cipher = ChaCha20Poly1305::new(key_ref);
    let nonce_ref = Nonce::from_slice(nonce);

    cipher.decrypt(nonce_ref, ciphertext).map_err(|e| {
        format!(
            "Decryption failed (invalid password or corrupted file): {}",
            e
        )
    })
}

use std::sync::OnceLock;

static TABLES: OnceLock<([u8; 256], [u8; 256])> = OnceLock::new();

fn get_tables() -> &'static ([u8; 256], [u8; 256]) {
    TABLES.get_or_init(|| {
        let mut exp = [0u8; 256];
        let mut log = [0u8; 256];
        let mut val = 1u8;
        for (i, item) in exp.iter_mut().enumerate().take(255) {
            *item = val;
            log[val as usize] = i as u8;

            let carry = val & 0x80;
            val <<= 1;
            if carry != 0 {
                val ^= 0x1d;
            }
        }
        exp[255] = exp[0];
        (exp, log)
    })
}

fn gf256_add(a: u8, b: u8) -> u8 {
    a ^ b
}

fn gf256_mul(a: u8, b: u8) -> u8 {
    if a == 0 || b == 0 {
        return 0;
    }
    let (exp, log) = get_tables();
    let log_a = log[a as usize];
    let log_b = log[b as usize];
    let sum = (log_a as usize + log_b as usize) % 255;
    exp[sum]
}

fn gf256_div(a: u8, b: u8) -> u8 {
    if b == 0 {
        panic!("Division by zero in GF(256)");
    }
    if a == 0 {
        return 0;
    }
    let (exp, log) = get_tables();
    let log_a = log[a as usize];
    let log_b = log[b as usize];
    let diff = (log_a as usize + 255 - log_b as usize) % 255;
    exp[diff]
}

fn eval_poly(coefs: &[u8], x: u8) -> u8 {
    let mut result = 0;
    for &c in coefs.iter().rev() {
        result = gf256_add(gf256_mul(result, x), c);
    }
    result
}

fn interpolate_at_zero(shares: &[(u8, u8)]) -> u8 {
    let mut secret = 0;
    for (j, &(x_j, y_j)) in shares.iter().enumerate() {
        let mut numerator = 1;
        let mut denominator = 1;
        for (k, &(x_k, _)) in shares.iter().enumerate() {
            if k != j {
                numerator = gf256_mul(numerator, x_k);
                denominator = gf256_mul(denominator, gf256_add(x_j, x_k));
            }
        }
        let lagrange_coef = gf256_div(numerator, denominator);
        let term = gf256_mul(y_j, lagrange_coef);
        secret = gf256_add(secret, term);
    }
    secret
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err("Hex string must have an even length".to_string());
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        let byte = u8::from_str_radix(&s[i..i + 2], 16)
            .map_err(|e| format!("Invalid hex character: {}", e))?;
        bytes.push(byte);
    }
    Ok(bytes)
}

pub struct ParsedShare {
    pub index: u8,
    pub threshold: usize,
    pub payload: Vec<u8>,
}

pub fn parse_share(share_str: &str) -> Result<ParsedShare, String> {
    let trimmed = share_str.trim().to_uppercase();
    let parts: Vec<&str> = trimmed.split('-').collect();
    if parts.len() != 5 || parts[0] != "JRNL" || parts[1] != "REC" {
        return Err("Invalid share format. Must start with JRNL-REC-".to_string());
    }
    let index = parts[2]
        .parse::<u8>()
        .map_err(|_| "Invalid share index".to_string())?;
    let threshold = parts[3]
        .parse::<usize>()
        .map_err(|_| "Invalid share threshold".to_string())?;
    let payload = hex_decode(parts[4])?;

    if index == 0 {
        return Err("Share index cannot be 0".to_string());
    }
    if threshold == 0 {
        return Err("Share threshold cannot be 0".to_string());
    }
    if payload.is_empty() {
        return Err("Share payload cannot be empty".to_string());
    }

    Ok(ParsedShare {
        index,
        threshold,
        payload,
    })
}

pub fn split_password(
    password: &str,
    threshold: usize,
    num_shares: usize,
) -> Result<Vec<String>, String> {
    if threshold == 0 || num_shares == 0 {
        return Err("Threshold and total shares must be greater than 0".to_string());
    }
    if threshold > num_shares {
        return Err("Threshold cannot be greater than the total number of shares".to_string());
    }
    if num_shares > 255 {
        return Err("Total number of shares cannot exceed 255".to_string());
    }
    if password.is_empty() {
        return Err("Password cannot be empty".to_string());
    }

    let password_bytes = password.as_bytes();
    let p_len = password_bytes.len();
    if p_len > 65535 {
        return Err(
            "Password is too long for recovery sharing (maximum 65535 characters)".to_string(),
        );
    }

    // Determine padded length: round up (p_len + 2) to the next multiple of 32
    let req_len = p_len + 2;
    let block_size = 32;
    let padded_len = req_len.div_ceil(block_size) * block_size;

    let mut padded = vec![0u8; padded_len];
    let len_be = (p_len as u16).to_be_bytes();
    padded[0] = len_be[0];
    padded[1] = len_be[1];
    padded[2..2 + p_len].copy_from_slice(password_bytes);

    for item in padded.iter_mut().skip(2 + p_len) {
        *item = rand::random::<u8>();
    }

    let mut share_payloads = vec![vec![0u8; padded_len]; num_shares];

    for (byte_idx, &secret_byte) in padded.iter().enumerate() {
        let mut coefs = vec![0u8; threshold];
        coefs[0] = secret_byte;
        for c in coefs.iter_mut().skip(1) {
            *c = rand::random::<u8>();
        }

        for x in 1..=num_shares {
            let y = eval_poly(&coefs, x as u8);
            share_payloads[x - 1][byte_idx] = y;
        }
    }

    let mut shares = Vec::with_capacity(num_shares);
    for x in 1..=num_shares {
        let hex_payload: String = share_payloads[x - 1]
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        shares.push(format!("JRNL-REC-{}-{}-{}", x, threshold, hex_payload));
    }

    Ok(shares)
}

pub fn reconstruct_password(share_strs: &[String]) -> Result<String, String> {
    if share_strs.is_empty() {
        return Err("No shares provided".to_string());
    }

    let mut parsed_shares = Vec::with_capacity(share_strs.len());
    for s in share_strs {
        let parsed = parse_share(s)?;
        parsed_shares.push(parsed);
    }

    let threshold = parsed_shares[0].threshold;
    let payload_len = parsed_shares[0].payload.len();

    if payload_len < 32 || payload_len % 32 != 0 {
        return Err(
            "Invalid share payload length. Must be a multiple of 32 bytes and at least 32 bytes."
                .to_string(),
        );
    }

    let mut indices = Vec::new();
    for s in &parsed_shares {
        if s.threshold != threshold {
            return Err("All shares must have the same threshold".to_string());
        }
        if s.payload.len() != payload_len {
            return Err("All shares must have the same payload length".to_string());
        }
        if indices.contains(&s.index) {
            return Err(format!("Duplicate share for index {} detected", s.index));
        }
        indices.push(s.index);
    }

    if parsed_shares.len() < threshold {
        return Err(format!(
            "Not enough shares. Need at least {}, but only got {}.",
            threshold,
            parsed_shares.len()
        ));
    }

    let reconstruction_shares = &parsed_shares[0..threshold];

    let mut reconstructed_padded = Vec::with_capacity(payload_len);

    for byte_idx in 0..payload_len {
        let mut byte_shares = Vec::with_capacity(threshold);
        for s in reconstruction_shares {
            byte_shares.push((s.index, s.payload[byte_idx]));
        }
        let reconstructed_byte = interpolate_at_zero(&byte_shares);
        reconstructed_padded.push(reconstructed_byte);
    }

    if reconstructed_padded.len() < 2 {
        return Err("Reconstructed data too short to read length prefix".to_string());
    }

    let len = u16::from_be_bytes([reconstructed_padded[0], reconstructed_padded[1]]) as usize;
    if len + 2 > reconstructed_padded.len() {
        return Err("Invalid reconstructed password length prefix".to_string());
    }

    let password_bytes = &reconstructed_padded[2..2 + len];
    String::from_utf8(password_bytes.to_vec())
        .map_err(|e| format!("Reconstructed password is not valid UTF-8: {}", e))
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

    #[test]
    fn test_shamir_split_reconstruct() {
        let password = "recovery_password_123";
        let shares = split_password(password, 3, 5).unwrap();
        assert_eq!(shares.len(), 5);

        // Test reconstruction with 3 shares (1, 2, 3)
        let subset_3 = vec![shares[0].clone(), shares[1].clone(), shares[2].clone()];
        let reconstructed = reconstruct_password(&subset_3).unwrap();
        assert_eq!(reconstructed, password);

        // Test reconstruction with a different subset of 3 shares (2, 4, 5)
        let subset_diff = vec![shares[1].clone(), shares[3].clone(), shares[4].clone()];
        let reconstructed_diff = reconstruct_password(&subset_diff).unwrap();
        assert_eq!(reconstructed_diff, password);

        // Test failure with only 2 shares
        let subset_2 = vec![shares[0].clone(), shares[4].clone()];
        let reconstruct_fail = reconstruct_password(&subset_2);
        assert!(reconstruct_fail.is_err());
    }

    #[test]
    fn test_shamir_edge_cases() {
        // 1-out-of-1
        let password = "simple_pwd";
        let shares = split_password(password, 1, 1).unwrap();
        let reconstructed = reconstruct_password(&shares).unwrap();
        assert_eq!(reconstructed, password);

        // invalid inputs
        assert!(split_password("", 2, 3).is_err());
        assert!(split_password("pwd", 0, 3).is_err());
        assert!(split_password("pwd", 3, 2).is_err());
        assert!(split_password("pwd", 2, 300).is_err());
    }
}
