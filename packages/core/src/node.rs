//! The Veilid node — our on-ramp to the network.
//!
//! [`VoidpostNode`] boots a full Veilid peer, waits for it to find
//! friends on the DHT, and then hands you a live connection to a
//! distributed hash table smeared across thousands of machines on
//! every continent that has electricity and an opinion. When you're
//! done, it shuts down without leaving forwarding addresses.

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::watch;
use tracing::info;
use veilid_core::*;

/// A running Veilid node. The live wire between your machine and
/// a planetary-scale distributed hash table. Handle with intent.
pub struct VoidpostNode {
    api: VeilidAPI,
}

impl VoidpostNode {
    /// Fire up a Veilid node and wait until it latches onto the network.
    ///
    /// `data_dir` is where Veilid keeps its state between runs — routing
    /// tables, crypto keys, the whole nervous system. Created if missing.
    pub async fn start(data_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(data_dir).context("failed to create data directory")?;

        // Watch channel: the update callback whispers attachment state
        // changes and we listen here like paranoid sentries.
        let (tx, mut rx) = watch::channel(AttachmentState::Detached);

        let update_callback: UpdateCallback = Arc::new(move |update| {
            if let VeilidUpdate::Attachment(att) = update {
                let _ = tx.send(att.state);
            }
        });

        let config_json = build_config(data_dir)?;

        let api = api_startup_json(update_callback, config_json)
            .await
            .map_err(|e| anyhow::anyhow!("veilid startup failed: {e}"))?;

        api.attach()
            .await
            .map_err(|e| anyhow::anyhow!("veilid attach failed: {e}"))?;

        info!("waiting for network attachment...");

        tokio::time::timeout(std::time::Duration::from_secs(60), async {
            loop {
                if rx.changed().await.is_err() {
                    anyhow::bail!("veilid update channel closed unexpectedly");
                }
                let state = *rx.borrow();
                match state {
                    AttachmentState::AttachedWeak
                    | AttachmentState::AttachedGood
                    | AttachmentState::AttachedStrong
                    | AttachmentState::FullyAttached
                    | AttachmentState::OverAttached => {
                        info!("attached to network: {state:?}");
                        return Ok(());
                    }
                    _ => {}
                }
            }
        })
        .await
        .context("timed out waiting for network attachment (60 s)")??;

        Ok(Self { api })
    }

    /// Get a [`RoutingContext`] — the handle you use to talk to the DHT.
    /// Safety routes are on by default because we're not animals.
    pub fn routing_context(&self) -> Result<RoutingContext> {
        self.api
            .routing_context()
            .map_err(|e| anyhow::anyhow!("failed to create routing context: {e}"))
    }

    /// Kill the node. Clean shutdown, no traces, no lingering sockets.
    /// The network forgets you were ever here.
    pub async fn shutdown(self) {
        self.api.shutdown().await;
    }
}

// ---------------------------------------------------------------------------
// Config builder
// ---------------------------------------------------------------------------

/// Take Veilid's default config, point its storage at our directory, and
/// stamp our name on it. The defaults are sane — we only override what
/// we have to. Don't touch the DHT tuning knobs unless you enjoy
/// debugging distributed systems at night with nothing but `tracing`
/// output and regret.
fn build_config(data_dir: &Path) -> Result<String> {
    let default_json = default_veilid_config();
    let mut config: serde_json::Value =
        serde_json::from_str(&default_json).context("failed to parse default veilid config")?;

    let dir = data_dir.display().to_string();

    config["program_name"] = serde_json::json!("voidpost");
    config["namespace"] = serde_json::json!("");

    // Point each store at our data dir. Veilid will happily scatter files
    // across your filesystem if you let it — we don't let it.
    if let Some(ts) = config.get_mut("table_store") {
        ts["directory"] = serde_json::json!(format!("{dir}/table_store"));
    }
    if let Some(bs) = config.get_mut("block_store") {
        bs["directory"] = serde_json::json!(format!("{dir}/block_store"));
    }
    if let Some(ps) = config.get_mut("protected_store") {
        ps["directory"] = serde_json::json!(format!("{dir}/protected_store"));
        ps["allow_insecure_fallback"] = serde_json::json!(true);
        ps["always_use_insecure_storage"] = serde_json::json!(true);
    }

    serde_json::to_string(&config).context("failed to serialize veilid config")
}
