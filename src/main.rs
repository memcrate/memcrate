use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use include_dir::{include_dir, Dir};
use std::fs;
use std::path::{Path, PathBuf};

static REFERENCE_VAULT: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/reference-vault");
static SKILLS_CLAUDE_CODE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/skills/claude-code");

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

    /// Install Memcrate skills for an AI tool.
    Install {
        /// Which tool to install skills for.
        #[arg(value_enum)]
        tool: Tool,

        /// Override the default install path for the tool's skills.
        #[arg(long)]
        target: Option<PathBuf>,

        /// Overwrite skills already installed at the target path.
        #[arg(long)]
        force: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Tool {
    /// Claude Code (Anthropic's CLI). Installs to ~/.claude/skills/.
    ClaudeCode,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { path, full, force } => init(path, full, force),
        Commands::Install { tool, target, force } => install(tool, target, force),
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
    println!("  - Install skills for your AI tool:");
    println!("      memcrate install claude-code");
    println!();
    println!("Docs: https://memcrate.dev");
    println!();
}

fn install(tool: Tool, target: Option<PathBuf>, force: bool) -> Result<()> {
    match tool {
        Tool::ClaudeCode => install_claude_code(target, force),
    }
}

fn install_claude_code(target: Option<PathBuf>, force: bool) -> Result<()> {
    let dest = resolve_claude_skills_dir(target)?;
    fs::create_dir_all(&dest)
        .with_context(|| format!("Failed to create {}", dest.display()))?;

    let skill_names: Vec<String> = SKILLS_CLAUDE_CODE
        .dirs()
        .filter_map(|d| {
            d.path()
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
        })
        .collect();

    let existing: Vec<String> = skill_names
        .iter()
        .filter(|n| dest.join(n).exists())
        .cloned()
        .collect();

    if !existing.is_empty() && !force {
        bail!(
            "Skills already installed in {}: {}. Pass --force to overwrite.",
            dest.display(),
            existing.join(", ")
        );
    }

    for name in &existing {
        let p = dest.join(name);
        fs::remove_dir_all(&p)
            .with_context(|| format!("Failed to remove existing {}", p.display()))?;
    }

    SKILLS_CLAUDE_CODE
        .extract(&dest)
        .with_context(|| format!("Failed to extract skills to {}", dest.display()))?;

    println!();
    println!(
        "Installed {} skill(s) for Claude Code to {}:",
        skill_names.len(),
        dest.display()
    );
    println!("  /load   load your vault context at the start of a session");
    println!("  /save   save the current session as a structured log");
    println!("  /pin    promote an insight into your permanent context files");
    println!();
    println!("Next:");
    println!("  1. Point Claude Code at your vault:");
    println!("       export MEMCRATE_VAULT_PATH=/path/to/your/vault");
    println!("     (Skip if your vault is at ~/vault — the skills default to that.)");
    println!();
    println!("  2. Start Claude Code in any directory:");
    println!("       claude");
    println!();
    println!("  3. Inside the session, run /load first to get oriented.");
    println!("     End the session with /save. Use /pin when something is");
    println!("     worth remembering forever.");
    println!();
    println!(
        "First-time note: Claude Code will ask permission to run a Bash command\n\
         the first time /load fires — that's the skill locating your vault.\n\
         Approve it once and the rest of the session runs clean."
    );
    println!();
    Ok(())
}

fn resolve_claude_skills_dir(target: Option<PathBuf>) -> Result<PathBuf> {
    match target {
        Some(p) => Ok(p),
        None => {
            let home = std::env::var("HOME").context(
                "Cannot resolve default Claude Code skills dir: HOME is not set. \
                 Pass --target <path> to override.",
            )?;
            Ok(PathBuf::from(home).join(".claude").join("skills"))
        }
    }
}
