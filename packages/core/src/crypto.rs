//! The cryptographic engine room.
//!
//! XChaCha20-Poly1305 — the same cipher the Signal protocol trusts with
//! billions of messages. Each chunk gets a fresh 24-byte random nonce
//! because reusing nonces is how you end up on a conference slide titled
//! "What Not To Do."
//!
//! Wire format per chunk: `nonce (24 B) || ciphertext || auth tag (16 B)`.
//! No metadata. No headers. Just noise that only the key holder can
//! turn back into meaning.

use anyhow::{Result, bail};
use chacha20poly1305::{
    XChaCha20Poly1305,
    aead::{Aead, KeyInit},
};
use rand::RngCore;

/// Maximum plaintext bytes per DHT chunk.
///
/// Veilid DHT values cap out at 32 768 bytes. Subtract the nonce (24 B)
/// and the AEAD auth tag (16 B), round down to something that doesn't
/// make a mathematician cry, and you land here.
pub const CHUNK_SIZE: usize = 32_000;

/// XChaCha20-Poly1305 nonce: 24 bytes of chaos.
const NONCE_LEN: usize = 24;

// ---------------------------------------------------------------------------
// Key generation
// ---------------------------------------------------------------------------

/// Conjure 256 bits of entropy from the void. This is the secret that
/// makes your document unreadable to every node operator, relay, and
/// three-letter agency between here and the heat death of the universe.
pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

// ---------------------------------------------------------------------------
// Encrypt / Decrypt
// ---------------------------------------------------------------------------

/// Seal a chunk with XChaCha20-Poly1305.
///
/// Returns `nonce (24 B) || ciphertext || tag (16 B)` — a blob that
/// looks like static to anyone without the key.
pub fn encrypt_chunk(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new(key.into());

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = &nonce_bytes.into();

    let ct = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("encryption failed: {e}"))?;

    let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend(ct);
    Ok(out)
}

/// Unseal a chunk. If the key is wrong or a single bit was flipped in
/// transit, the auth tag check fails and you get an error instead of
/// garbage. That's the "poly1305" part earning its keep.
pub fn decrypt_chunk(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>> {
    if data.len() <= NONCE_LEN {
        bail!("encrypted chunk too short ({} bytes)", data.len());
    }

    let (nonce_bytes, ct) = data.split_at(NONCE_LEN);
    let nonce = nonce_bytes.into();
    let cipher = XChaCha20Poly1305::new(key.into());

    cipher
        .decrypt(nonce, ct)
        .map_err(|e| anyhow::anyhow!("decryption failed (wrong key or corrupted data): {e}"))
}

/// Butcher `data` into pieces that fit inside a DHT value.
/// No intelligence here — just a meat cleaver at regular intervals.
pub fn chunk_data(data: &[u8]) -> Vec<&[u8]> {
    data.chunks(CHUNK_SIZE).collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let key = generate_key();
        let msg = b"the void remembers nothing";
        let enc = encrypt_chunk(&key, msg).unwrap();
        let dec = decrypt_chunk(&key, &enc).unwrap();
        assert_eq!(&dec, msg);
    }

    #[test]
    fn wrong_key_fails() {
        let k1 = generate_key();
        let k2 = generate_key();
        let enc = encrypt_chunk(&k1, b"secret").unwrap();
        assert!(decrypt_chunk(&k2, &enc).is_err());
    }

    #[test]
    fn chunking() {
        let data = vec![0u8; CHUNK_SIZE * 2 + 100];
        let chunks = chunk_data(&data);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), CHUNK_SIZE);
        assert_eq!(chunks[1].len(), CHUNK_SIZE);
        assert_eq!(chunks[2].len(), 100);
    }

    #[test]
    fn empty_plaintext_round_trip() {
        let key = generate_key();
        let enc = encrypt_chunk(&key, &[]).unwrap();
        let dec = decrypt_chunk(&key, &enc).unwrap();
        assert!(dec.is_empty());
    }

    #[test]
    fn exact_chunk_size_round_trip() {
        let key = generate_key();
        let data = vec![0xAB; CHUNK_SIZE];
        let enc = encrypt_chunk(&key, &data).unwrap();
        let dec = decrypt_chunk(&key, &enc).unwrap();
        assert_eq!(dec, data);
    }

    #[test]
    fn decrypt_too_short_data_fails() {
        let key = generate_key();
        let short = vec![0u8; 10];
        assert!(decrypt_chunk(&key, &short).is_err());
    }

    #[test]
    fn generated_keys_are_unique() {
        let k1 = generate_key();
        let k2 = generate_key();
        assert_ne!(k1, k2);
    }
}
