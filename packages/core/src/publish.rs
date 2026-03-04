//! Publish — the act of hurling a document into the void.
//!
//! The pipeline:
//! 1. Generate a random 256-bit encryption key. Fresh entropy, no reuse.
//! 2. Butcher the file into ≤ 32 000 B plaintext chunks.
//! 3. Seal every chunk (and the manifest) with XChaCha20-Poly1305.
//! 4. Claim a DHT record with enough subkeys to hold the whole body.
//! 5. Write the manifest to subkey 0, chunks to subkeys 1..N.
//! 6. Hand back a [`SharePayload`] — the only proof this ever happened.
//!
//! After this function returns, the file exists simultaneously on dozens
//! of machines owned by people who will never know they're carrying it.
//! Everywhere and nowhere.

use anyhow::Result;
use tracing::info;
use veilid_core::*;

use crate::crypto;
use crate::dht_retry;
use crate::link::SharePayload;
use crate::node::VoidpostNode;
use crate::types::Manifest;

/// Take a file, encrypt it, scatter it across the DHT, and return
/// the share link. The publisher is hidden behind private routing.
/// The data is ciphertext. You were never here.
pub async fn publish_file(
    node: &VoidpostNode,
    file_data: &[u8],
    filename: &str,
) -> Result<SharePayload> {
    let encryption_key = crypto::generate_key();

    // Tear it apart -------------------------------------------------------
    let chunks = crypto::chunk_data(file_data);
    let num_chunks = chunks.len() as u32;

    info!(
        "publishing \"{}\" ({} bytes, {} chunk{})",
        filename,
        file_data.len(),
        num_chunks,
        if num_chunks == 1 { "" } else { "s" }
    );

    let mut encrypted_chunks = Vec::with_capacity(chunks.len());
    for chunk in &chunks {
        encrypted_chunks.push(crypto::encrypt_chunk(&encryption_key, chunk)?);
    }

    // The manifest — table of contents, encrypted like everything else --
    let manifest = Manifest {
        version: 1,
        filename: filename.to_string(),
        size: file_data.len() as u64,
        chunks: num_chunks,
    };
    let manifest_json = serde_json::to_vec(&manifest)?;
    let encrypted_manifest = crypto::encrypt_chunk(&encryption_key, &manifest_json)?;

    // Claim our slot on the DHT -------------------------------------------
    let total_subkeys = num_chunks + 1; // manifest + chunks
    let rc = node.routing_context()?;

    let schema = DHTSchema::dflt(total_subkeys as u16)
        .map_err(|e| anyhow::anyhow!("failed to create DHT schema: {e}"))?;

    let record = dht_retry::create_dht_record_retry(&rc, CRYPTO_KIND_VLD0, schema).await?;

    let record_key = record.key().clone();
    info!("created DHT record: {record_key}");

    // Manifest goes into subkey 0 — the table of contents
    dht_retry::set_dht_value_retry(&rc, record_key.clone(), 0, encrypted_manifest).await?;

    // Scatter the body across subkeys 1..N
    for (i, enc_chunk) in encrypted_chunks.into_iter().enumerate() {
        let subkey = (i + 1) as u32;
        dht_retry::set_dht_value_retry(&rc, record_key.clone(), subkey, enc_chunk).await?;

        if (i + 1) % 10 == 0 || i + 1 == num_chunks as usize {
            info!("  wrote chunk {}/{}", i + 1, num_chunks);
        }
    }

    // Walk away clean ------------------------------------------------------
    rc.close_dht_record(record_key.clone())
        .await
        .map_err(|e| anyhow::anyhow!("failed to close DHT record: {e}"))?;

    info!("publish complete");

    Ok(SharePayload {
        record_key,
        encryption_key,
    })
}
