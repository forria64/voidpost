use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

// ---------------------------------------------------------------------------
// CLI — the command-line interface for people who prefer their privacy
// without a GUI between them and the void.
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(
    name = "voidpost",
    version,
    about = "Voidpost. Encrypted at birth. Anonymous by design. Gone when you're done."
)]
struct Cli {
    /// Where Veilid keeps its state between runs. Leave it alone unless
    /// you know what you're doing.
    #[arg(long, default_value_t = default_data_dir())]
    data_dir: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Hurl a file into the void. Returns a share link.
    Publish {
        /// The file to publish.
        file: PathBuf,
    },
    /// Pull a file back out of the void using a share link.
    Read {
        /// The share link — one string, everything you need.
        link: String,
        /// Where to save it. Defaults to the original filename.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Figure out where to stash Veilid's state. Respects XDG because we're
/// not barbarians, falls back to ~/.local/share, then gives up and uses
/// the current directory like a cornered animal.
fn default_data_dir() -> String {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        format!("{xdg}/voidpost")
    } else if let Ok(home) = std::env::var("HOME") {
        format!("{home}/.local/share/voidpost")
    } else {
        ".voidpost".to_string()
    }
}

// ---------------------------------------------------------------------------
// Main — where the magic happens and the evidence disappears.
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("voidpost=info,veilid_core=warn")),
        )
        .init();

    let cli = Cli::parse();
    let data_dir = PathBuf::from(&cli.data_dir);

    match cli.command {
        Commands::Publish { file } => {
            let file_data = std::fs::read(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;
            let filename = file
                .file_name()
                .context("file has no name")?
                .to_string_lossy();

            let node = voidpost_core::VoidpostNode::start(&data_dir).await?;
            let payload =
                voidpost_core::publish::publish_file(&node, &file_data, &filename).await?;
            node.shutdown().await;

            let link = payload.encode();
            println!("\n{link}");
        }

        Commands::Read { link, output } => {
            let payload = voidpost_core::link::SharePayload::decode(&link)?;

            let node = voidpost_core::VoidpostNode::start(&data_dir).await?;
            let doc = voidpost_core::retrieve::retrieve_file(&node, &payload).await?;
            node.shutdown().await;

            let out_path = output.unwrap_or_else(|| PathBuf::from(&doc.filename));
            std::fs::write(&out_path, &doc.data)
                .with_context(|| format!("failed to write {}", out_path.display()))?;

            println!("saved to {}", out_path.display());
        }
    }

    Ok(())
}
