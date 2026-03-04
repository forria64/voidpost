//! voidpost-core — the guts of the machine.
//!
//! Everything between "you have a file" and "it no longer exists in any
//! jurisdiction" lives here. Encryption, chunking, DHT operations, share
//! link encoding — the full pipeline from cleartext to plausible deniability.
//! No UI opinions, no framework dependencies, no feelings.

pub mod crypto;
pub mod dht_retry;
pub mod link;
pub mod node;
pub mod publish;
pub mod retrieve;
mod types;

pub use link::SharePayload;
pub use node::VoidpostNode;
pub use types::Document;
