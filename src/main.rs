use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use include_dir::{include_dir, Dir};
use std::fs;
use std::path::{Path, PathBuf};

static REFERENCE_VAULT: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/reference-vault");

const OPTIONAL_FOLDERS: &[&str] = &["Projects", "Daily", "Tasks", "Inbox"];

#[derive(Parser)]
#[command(
    name = "memcrate",
    version,
    about = "Markdown-native personal context vault for AI tools.",
    long_about = "Memcrate scaffolds and maintains a portable, local-first markdown vault that any AI tool can read. Three verbs — /save, /pin, /load — operate on a defined directory shape. The CLI is the install layer; the vault is the system."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scaffold a new vault at the given path (default: ~/vault).
    Init {
        /// Path where the vault should be created. Defaults to ~/vault.
        path: Option<PathBuf>,

        /// Also scaffold optional human-only folders (Projects/, Daily/, Tasks/, Inbox/).
        #[arg(long)]
        full: bool,

        /// Overwrite an existing vault at this path.
        #[arg(long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { path, full, force } => init(path, full, force),
    }
}

fn init(path: Option<PathBuf>, full: bool, force: bool) -> Result<()> {
    let target = resolve_target(path)?;
    ensure_writable(&target, force)?;

    fs::create_dir_all(&target)
        .with_context(|| format!("Failed to create {}", target.display()))?;

    REFERENCE_VAULT
        .extract(&target)
        .with_context(|| format!("Failed to extract reference vault to {}", target.display()))?;

    if full {
        for folder in OPTIONAL_FOLDERS {
            let dir = target.join(folder);
            fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create {}", dir.display()))?;
            fs::write(dir.join(".gitkeep"), "")
                .with_context(|| format!("Failed to write .gitkeep in {}", dir.display()))?;
        }
    }

    print_success(&target, full);
    Ok(())
}

fn resolve_target(path: Option<PathBuf>) -> Result<PathBuf> {
    match path {
        Some(p) => Ok(p),
        None => {
            let home = std::env::var("HOME").context(
                "Cannot resolve default vault path: HOME environment variable is not set. \
                 Pass an explicit path: memcrate init <path>",
            )?;
            Ok(PathBuf::from(home).join("vault"))
        }
    }
}

fn ensure_writable(target: &Path, force: bool) -> Result<()> {
    if !target.exists() {
        return Ok(());
    }

    if force {
        return Ok(());
    }

    if target.join(".memcrate").exists() {
        bail!(
            "A Memcrate vault already exists at {}. Pass --force to overwrite.",
            target.display()
        );
    }

    let is_empty = fs::read_dir(target)
        .with_context(|| format!("Failed to read {}", target.display()))?
        .next()
        .is_none();

    if !is_empty {
        bail!(
            "Directory {} exists and is not empty. Pass --force to scaffold anyway, \
             or choose a different path.",
            target.display()
        );
    }

    Ok(())
}

fn print_success(target: &Path, full: bool) {
    println!();
    println!("Vault scaffolded at {}", target.display());
    println!();
    println!("Shape:");
    println!("  Core/");
    println!("    Context/   (Profile.md, Projects.md, Current State.md)");
    println!("    Sessions/  (session logs from /save)");
    if full {
        println!("  Projects/  (per-project thinking layer)");
        println!("  Daily/     (daily notes)");
        println!("  Tasks/     (short-term work queue)");
        println!("  Inbox/     (unprocessed capture)");
    }
    println!();
    println!("Next:");
    println!("  - Edit Core/Context/Profile.md to describe yourself.");
    println!("  - Edit Core/Context/Projects.md to list your projects.");
    println!("  - Install skills for your AI tool (coming in a later release):");
    println!("      memcrate install claude-code");
    println!();
    println!("Docs: https://memcrate.dev");
    println!();
}
