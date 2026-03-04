//! Retry logic for DHT operations that fail with transient errors.
//!
//! Veilid's DHT is a distributed system running on volunteer nodes
//! across the planet. Sometimes the routing table isn't warmed up yet.
//! Sometimes a node is having a bad day. The correct response is not
//! to panic — it's to wait a beat and try again, like a patient
//! predator that knows the prey will eventually move.
//!
//! All four public functions are thin wrappers around `with_retry`,
//! which handles the exponential backoff dance so nobody has to
//! copy-paste the same loop four times like an animal.

use std::future::Future;
use std::time::Duration;

use anyhow::Result;
use tracing::warn;
use veilid_core::*;

/// Maximum number of retry attempts before we accept defeat.
const MAX_RETRIES: u32 = 5;

/// Initial backoff duration. Each retry doubles this.
const INITIAL_BACKOFF: Duration = Duration::from_secs(2);

/// Generic retry wrapper with exponential backoff for VeilidAPIError::TryAgain.
///
/// The "TryAgain" error from Veilid means the DHT routing table
/// isn't ready yet — the node knows it's attached but hasn't found
/// enough peers to actually serve requests. Hammering it won't help.
/// Patience will.
async fn with_retry<F, Fut, T>(op_name: &str, mut f: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = std::result::Result<T, VeilidAPIError>>,
{
    let mut backoff = INITIAL_BACKOFF;

    for attempt in 1..=MAX_RETRIES {
        match f().await {
            Ok(val) => return Ok(val),
            Err(VeilidAPIError::TryAgain { message }) => {
                if attempt == MAX_RETRIES {
                    anyhow::bail!("DHT {op_name} failed after {MAX_RETRIES} attempts: {message}");
                }
                warn!(
                    "DHT not ready (attempt {attempt}/{MAX_RETRIES}): {message} — \
                     retrying in {backoff:?}"
                );
                tokio::time::sleep(backoff).await;
                backoff *= 2;
            }
            Err(e) => anyhow::bail!("DHT {op_name} failed: {e}"),
        }
    }

    unreachable!()
}

/// Retry a DHT get_value operation with exponential backoff.
pub async fn get_dht_value_retry(
    rc: &RoutingContext,
    key: RecordKey,
    subkey: ValueSubkey,
) -> Result<Option<ValueData>> {
    with_retry("read", || rc.get_dht_value(key.clone(), subkey, true)).await
}

/// Retry a DHT set_value operation with exponential backoff.
pub async fn set_dht_value_retry(
    rc: &RoutingContext,
    key: RecordKey,
    subkey: ValueSubkey,
    data: Vec<u8>,
) -> Result<Option<ValueData>> {
    with_retry("write", || {
        rc.set_dht_value(key.clone(), subkey, data.clone(), None)
    })
    .await
}

/// Retry opening a DHT record with exponential backoff.
pub async fn open_dht_record_retry(
    rc: &RoutingContext,
    key: RecordKey,
    writer: Option<KeyPair>,
) -> Result<DHTRecordDescriptor> {
    with_retry("open", || rc.open_dht_record(key.clone(), writer.clone())).await
}

/// Retry creating a DHT record with exponential backoff.
pub async fn create_dht_record_retry(
    rc: &RoutingContext,
    kind: CryptoKind,
    schema: DHTSchema,
) -> Result<DHTRecordDescriptor> {
    with_retry("create", || {
        rc.create_dht_record(kind, schema.clone(), None)
    })
    .await
}
