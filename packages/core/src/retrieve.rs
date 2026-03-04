//! Retrieve — pull a document back out of the void.
//!
//! The hit-and-run:
//! 1. Open the DHT record. Read-only. No writer key, no identity.
//! 2. Grab subkey 0, decrypt the manifest. Now you know what you're
//!    looking at and how many pieces it's in.
//! 3. Fetch subkeys 1..N, decrypt each chunk, stitch them together.
//! 4. Verify the reassembled size matches what the manifest promised.
//! 5. Hand back the [`Document`]. You were never here.
//!
//! The reader is hidden behind safety routing. The DHT nodes that
//! served the data have no idea who asked for it.

use anyhow::{Context, Result, bail};
use tracing::info;

use crate::crypto;
use crate::dht_retry;
use crate::link::SharePayload;
use crate::node::VoidpostNode;
use crate::types::{Document, Manifest};

/// Reach into the DHT, grab every chunk, decrypt, reassemble, and
/// verify. If a single byte is wrong, the whole thing fails loudly
/// rather than handing you corrupted garbage.
pub async fn retrieve_file(node: &VoidpostNode, payload: &SharePayload) -> Result<Document> {
    let rc = node.routing_context()?;

    // Open the record. No writer key — we're just here to read and leave.
    let _ = dht_retry::open_dht_record_retry(&rc, payload.record_key.clone(), None).await?;

    // Grab the manifest — our table of contents -------------------------
    let manifest_value = dht_retry::get_dht_value_retry(&rc, payload.record_key.clone(), 0)
        .await?
        .context("manifest subkey is empty")?;

    let manifest_json = crypto::decrypt_chunk(&payload.encryption_key, manifest_value.data())?;
    let manifest: Manifest =
        serde_json::from_slice(&manifest_json).context("failed to parse manifest")?;

    if manifest.version != 1 {
        bail!("unsupported manifest version: {}", manifest.version);
    }

    info!(
        "retrieving \"{}\" ({} bytes, {} chunk{})",
        manifest.filename,
        manifest.size,
        manifest.chunks,
        if manifest.chunks == 1 { "" } else { "s" }
    );

    // Now the body — chunk by chunk, decrypt, stitch --------------------
    let mut plaintext = Vec::with_capacity(manifest.size as usize);

    for i in 0..manifest.chunks {
        let subkey = i + 1;
        let chunk_value = dht_retry::get_dht_value_retry(&rc, payload.record_key.clone(), subkey)
            .await?
            .with_context(|| format!("chunk {subkey} is empty"))?;

        let decrypted = crypto::decrypt_chunk(&payload.encryption_key, chunk_value.data())?;
        plaintext.extend(decrypted);

        if (i + 1) % 10 == 0 || i + 1 == manifest.chunks {
            info!("  read chunk {}/{}", i + 1, manifest.chunks);
        }
    }

    // Release the record. Leave no trace. --------------------------------
    rc.close_dht_record(payload.record_key.clone())
        .await
        .map_err(|e| anyhow::anyhow!("failed to close DHT record: {e}"))?;

    // Sanity check — if the bytes don't add up, someone lied. -----------
    if plaintext.len() != manifest.size as usize {
        bail!(
            "size mismatch: manifest says {} bytes, got {}",
            manifest.size,
            plaintext.len()
        );
    }

    info!("retrieve complete");

    Ok(Document {
        filename: manifest.filename,
        data: plaintext,
    })
}
