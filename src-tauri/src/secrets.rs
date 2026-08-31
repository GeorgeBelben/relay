//! At-rest encryption for the two RetroAchievements credentials (Web API key, Connect session
//! token) once they're stored per-profile in SQLite rather than as an app-wide plaintext setting.
//! There's no OS secret-service daemon to rely on here (this kiosk is headless -- Ubuntu Server +
//! cage, no desktop environment, so no gnome-keyring/kwallet for a `keyring`-crate approach to
//! talk to) -- conceptually the same job as the Electron MVP's secretStorage.ts (which wrapped
//! Electron's own OS-keychain-backed `safeStorage`), but done here as an app-local encrypted file
//! instead: a random key generated once on first use, stored with owner-only permissions,
//! deliberately separate from the SQLite DB itself so a DB copy/backup alone doesn't leak
//! credentials. This defends against casual disk/backup exposure, which is the actual threat model
//! for a personal single-user kiosk -- not a sophisticated adversary who'd already have full device
//! access (and therefore the app's own decrypted view) anyway.

use std::path::Path;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};

const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

fn load_or_create_key(key_path: &Path) -> std::io::Result<[u8; KEY_LEN]> {
    if let Ok(bytes) = std::fs::read(key_path) {
        if bytes.len() == KEY_LEN {
            let mut key = [0u8; KEY_LEN];
            key.copy_from_slice(&bytes);
            return Ok(key);
        }
    }

    let key = ChaCha20Poly1305::generate_key(&mut OsRng);
    if let Some(parent) = key_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(key_path, key)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(key_path, std::fs::Permissions::from_mode(0o600))?;
    }

    let mut out = [0u8; KEY_LEN];
    out.copy_from_slice(&key);
    Ok(out)
}

/// Encrypts `value`, returning a base64 string (a fresh random nonce prepended to the ciphertext)
/// suitable for a TEXT column -- base64 because the cipher returns raw bytes and this is a text
/// column, same reasoning as the Electron original.
pub fn encrypt_secret(key_path: &Path, value: &str) -> std::io::Result<String> {
    let key_bytes = load_or_create_key(key_path)?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_bytes));
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, value.as_bytes()).map_err(|e| std::io::Error::other(format!("encryption failed: {e}")))?;

    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(BASE64.encode(combined))
}

/// `None` for a missing/corrupt blob -- a repository read shouldn't fail over a credential that's
/// simply not set (the common case: a profile with no RA link at all).
pub fn decrypt_secret(key_path: &Path, encrypted: Option<&str>) -> Option<String> {
    let encrypted = encrypted?;
    let key_bytes = load_or_create_key(key_path).ok()?;
    let combined = BASE64.decode(encrypted).ok()?;
    if combined.len() < NONCE_LEN {
        return None;
    }
    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_LEN);

    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_bytes));
    let plaintext = cipher.decrypt(Nonce::from_slice(nonce_bytes), ciphertext).ok()?;
    String::from_utf8(plaintext).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_value_through_encrypt_and_decrypt() {
        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join("secret.key");

        let encrypted = encrypt_secret(&key_path, "my-ra-web-api-key").unwrap();
        assert_ne!(encrypted, "my-ra-web-api-key");
        assert_eq!(decrypt_secret(&key_path, Some(&encrypted)), Some("my-ra-web-api-key".to_string()));
    }

    #[test]
    fn decrypt_returns_none_for_a_missing_value() {
        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join("secret.key");
        assert_eq!(decrypt_secret(&key_path, None), None);
    }

    #[test]
    fn decrypt_returns_none_for_corrupt_base64_rather_than_erroring() {
        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join("secret.key");
        assert_eq!(decrypt_secret(&key_path, Some("not valid base64!!")), None);
    }

    #[test]
    fn decrypt_fails_closed_when_encrypted_with_a_different_key() {
        let dir = tempfile::tempdir().unwrap();
        let key_path_a = dir.path().join("a.key");
        let key_path_b = dir.path().join("b.key");

        let encrypted = encrypt_secret(&key_path_a, "secret").unwrap();
        assert_eq!(decrypt_secret(&key_path_b, Some(&encrypted)), None);
    }

    #[test]
    fn reuses_the_same_key_across_calls_instead_of_regenerating_it() {
        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join("secret.key");

        let first = encrypt_secret(&key_path, "value-one").unwrap();
        let second = encrypt_secret(&key_path, "value-two").unwrap();

        // Different ciphertexts (fresh nonce each time), but both decryptable by the one
        // persisted key -- proves the key wasn't silently regenerated between calls.
        assert_eq!(decrypt_secret(&key_path, Some(&first)), Some("value-one".to_string()));
        assert_eq!(decrypt_secret(&key_path, Some(&second)), Some("value-two".to_string()));
    }

    #[cfg(unix)]
    #[test]
    fn the_key_file_is_created_with_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join("secret.key");
        encrypt_secret(&key_path, "value").unwrap();

        let mode = std::fs::metadata(&key_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
