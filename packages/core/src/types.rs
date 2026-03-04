//! Types that travel between modules without picking up dependencies.
//!
//! These are the data structures that every layer of voidpost agrees on.
//! Keep them dumb. Keep them serializable. Keep them out of arguments
//! about architecture.

use serde::{Deserialize, Serialize};

/// The manifest — encrypted and stuffed into DHT subkey 0.
///
/// This is the table of contents for a voidpost document. Without it
/// you're staring at a pile of numbered ciphertext blobs with no idea
/// how many there are or what order they go in. The manifest remembers
/// so you don't have to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Manifest {
    /// Schema version. Currently 1. Will be 1 for a while.
    pub version: u32,
    /// The name this file had before it entered the void.
    pub filename: String,
    /// Original size in bytes — the number that has to match on the other side.
    pub size: u64,
    /// How many encrypted chunks are scattered across subkeys 1..=N.
    pub chunks: u32,
}

/// What comes back when you pull a document out of the ether.
/// The filename it had in its previous life, and every byte of its body.
pub struct Document {
    pub filename: String,
    pub data: Vec<u8>,
}
