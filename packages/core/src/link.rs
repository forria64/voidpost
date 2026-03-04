//! The share link — one string to rule them all.
//!
//! Everything the reader needs to pull a document out of the void lives
//! in this single token: the DHT record key (where to look) and the
//! encryption key (how to read it). No accounts. No sessions. No
//! server-side state. Just a string that is simultaneously an address
//! and a skeleton key.
//!
//! Format: `<RecordKey>#<base64url(encryption_key)>`
//!
//! Example: `VLD0:cUsJJKKC7OaKO_jkFE2Qw7d3kvFc_UOd4fWl0Wkbxlk#SGVsbG9Xb3JsZEtleQ`

use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use std::str::FromStr;
use veilid_core::RecordKey;

/// The payload. Coordinates and the key to the vault, packed into one
/// struct. Lose this and the document is gone — not "deleted," not
/// "archived" — gone, in the thermodynamic sense.
#[derive(Debug, Clone)]
pub struct SharePayload {
    /// Where on the DHT this document lives.
    pub record_key: RecordKey,
    /// The 256-bit secret that turns ciphertext back into meaning.
    pub encryption_key: [u8; 32],
}

impl SharePayload {
    /// Pack into a single shareable string. This is the thing you paste
    /// into a chat, scrawl on a napkin, or whisper into a dead drop.
    pub fn encode(&self) -> String {
        let key_b64 = URL_SAFE_NO_PAD.encode(self.encryption_key);
        format!("{}#{}", self.record_key, key_b64)
    }

    /// Unpack a share link back into its components. If the format is
    /// wrong, you get an error that explains exactly how it's wrong,
    /// because vague error messages are a crime against usability.
    pub fn decode(s: &str) -> Result<Self> {
        let (rk_str, key_b64) = s
            .rsplit_once('#')
            .context("invalid share link: missing '#' separator")?;

        let record_key =
            RecordKey::from_str(rk_str).map_err(|e| anyhow::anyhow!("invalid record key: {e}"))?;

        let key_bytes = URL_SAFE_NO_PAD
            .decode(key_b64)
            .context("invalid share link: bad base64 encryption key")?;

        if key_bytes.len() != 32 {
            bail!(
                "invalid share link: encryption key is {} bytes, expected 32",
                key_bytes.len()
            );
        }

        let mut encryption_key = [0u8; 32];
        encryption_key.copy_from_slice(&key_bytes);

        Ok(Self {
            record_key,
            encryption_key,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_round_trip() {
        let rk = RecordKey::from_str("VLD0:cUsJJKKC7OaKO_jkFE2Qw7d3kvFc_UOd4fWl0Wkbxlk")
            .expect("valid record key");

        let key = [42u8; 32];

        let payload = SharePayload {
            record_key: rk,
            encryption_key: key,
        };

        let encoded = payload.encode();
        let decoded = SharePayload::decode(&encoded).expect("decode should succeed");

        assert_eq!(
            decoded.record_key.to_string(),
            payload.record_key.to_string()
        );
        assert_eq!(decoded.encryption_key, key);
    }

    #[test]
    fn decode_missing_separator_fails() {
        let result = SharePayload::decode("no-separator-here");
        assert!(result.is_err());
    }

    #[test]
    fn decode_wrong_key_length_fails() {
        let result =
            SharePayload::decode("VLD0:cUsJJKKC7OaKO_jkFE2Qw7d3kvFc_UOd4fWl0Wkbxlk#dG9vc2hvcnQ");
        assert!(result.is_err());
    }
}
