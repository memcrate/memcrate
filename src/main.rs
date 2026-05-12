use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use include_dir::{include_dir, Dir};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use time::macros::format_description;
use time::OffsetDateTime;

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

    /// Populate Profile.md and Projects.md with a quick interactive wizard.
    Setup {
        /// Path to the vault. If omitted, looks for a vault in the current
        /// directory, then walks up looking for a `.memcrate` marker, then
        /// scans your home directory for a single vault.
        path: Option<PathBuf>,

        /// Overwrite Profile.md and Projects.md even if they've been hand-edited.
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
        Commands::Setup { path, force } => setup(path, force),
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
    println!("  1. (Optional) Seed your Profile and Projects from a few prompts:");
    println!("       memcrate setup");
    println!();
    println!("  2. Install skills for your AI tool:");
    println!("       memcrate install claude-code");
    println!();
    println!("  3. Start your AI tool. Run /load first to get oriented;");
    println!("     /pin facts as you work; /save the session at the end.");
    println!();
    println!("You can also hand-edit the scaffolded files (they include");
    println!("section guidance inline), but you should never *need* to.");
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
    println!("  1. Scaffold a vault if you don't have one yet:");
    println!("       memcrate init ~/vault");
    println!();
    println!("  2. Start Claude Code:");
    println!("       claude");
    println!();
    println!("  3. Inside the session, run /load first to get oriented.");
    println!("     End the session with /save. Use /pin when something is");
    println!("     worth remembering forever.");
    println!();
    println!(
        "First-time note: Claude Code will ask permission to read your vault's\n\
         Profile.md the first time /load fires. The prompt will show the path\n\
         (Read ~/vault/Core/Context/Profile.md). Approve it once and the rest\n\
         of the session runs clean."
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

fn resolve_setup_vault(path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = path {
        return Ok(p);
    }

    if let Ok(cwd) = std::env::current_dir() {
        if cwd.join(".memcrate").exists() {
            return Ok(cwd);
        }
        let mut walk = cwd.as_path();
        while let Some(parent) = walk.parent() {
            if parent.join(".memcrate").exists() {
                return Ok(parent.to_path_buf());
            }
            walk = parent;
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let home_path = PathBuf::from(&home);
        let mut found: Vec<PathBuf> = Vec::new();
        if let Ok(entries) = fs::read_dir(&home_path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && p.join(".memcrate").exists() {
                    found.push(p);
                }
            }
        }
        found.sort();

        match found.len() {
            0 => {}
            1 => return Ok(found.into_iter().next().unwrap()),
            _ => {
                let list: Vec<String> = found.iter().map(|p| format!("  {}", p.display())).collect();
                bail!(
                    "Multiple Memcrate vaults found in your home directory:\n{}\n\n\
                     Pick one explicitly:\n  memcrate setup <path>",
                    list.join("\n")
                );
            }
        }

        let default = home_path.join("vault");
        if default.exists() {
            return Ok(default);
        }
    }

    bail!(
        "No Memcrate vault found. Pass the vault path explicitly:\n  \
         memcrate setup <path>\n\n\
         Or scaffold a new vault first:\n  memcrate init ~/vault"
    );
}

const IDENTITY_PLACEHOLDER: &str = "<!-- Who you are professionally. One paragraph. -->";
const TOOLS_PLACEHOLDER: &str = "<!-- Editor, languages, runtimes, CLIs, default services. -->";
const PROJECTS_DATE_PLACEHOLDER: &str = "last_updated: YYYY-MM-DD";

fn setup(path: Option<PathBuf>, force: bool) -> Result<()> {
    let vault = resolve_setup_vault(path)?;
    let profile_path = vault.join("Core").join("Context").join("Profile.md");
    let projects_path = vault.join("Core").join("Context").join("Projects.md");

    if !profile_path.exists() || !projects_path.exists() {
        bail!(
            "Vault at {} is malformed: Core/Context/Profile.md or Projects.md \
             is missing. Re-run `memcrate init {}` to repair the scaffold.",
            vault.display(),
            vault.display()
        );
    }

    let profile_text = fs::read_to_string(&profile_path)
        .with_context(|| format!("Failed to read {}", profile_path.display()))?;
    let projects_text = fs::read_to_string(&projects_path)
        .with_context(|| format!("Failed to read {}", projects_path.display()))?;

    let profile_pristine = profile_text.contains(IDENTITY_PLACEHOLDER);
    let projects_pristine = projects_text.contains("## Example Project");

    if (!profile_pristine || !projects_pristine) && !force {
        bail!(
            "Profile.md or Projects.md has already been modified. \
             Pass --force to overwrite, or hand-edit instead."
        );
    }

    println!("Memcrate setup — populates Profile.md and Projects.md from your answers.");
    println!("(Press Enter on any question to skip it. Ctrl-C aborts.)");
    println!();
    println!("Vault: {}", vault.display());
    println!();

    let name = prompt_line("Your name (or how you'd like to be referred to)")?;
    let what_you_do = prompt_line("What do you do? (one short paragraph)")?;
    let tools = prompt_line("Tools you always use (comma-separated)")?;
    let projects = prompt_multiline("Active projects (one per line, blank line to finish)")?;

    let today = today_iso();
    let updated_profile = update_profile(&profile_text, &name, &what_you_do, &tools, &today);
    let updated_projects = update_projects(&projects_text, &projects, &today);

    fs::write(&profile_path, updated_profile)
        .with_context(|| format!("Failed to write {}", profile_path.display()))?;
    fs::write(&projects_path, updated_projects)
        .with_context(|| format!("Failed to write {}", projects_path.display()))?;

    println!();
    println!("Updated:");
    println!("  {}", profile_path.display());
    println!("  {}", projects_path.display());
    println!();
    println!("Next:");
    println!("  memcrate install claude-code");
    println!("  claude");
    println!("  /load   # your vault now has real context to load");
    println!();

    Ok(())
}

fn prompt_line(label: &str) -> Result<String> {
    print!("{}:\n> ", label);
    io::stdout().flush().ok();
    let stdin = io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .context("Failed to read from stdin")?;
    println!();
    Ok(line.trim().to_string())
}

fn prompt_multiline(label: &str) -> Result<Vec<String>> {
    println!("{}:", label);
    let stdin = io::stdin();
    let mut lines = Vec::new();
    loop {
        print!("> ");
        io::stdout().flush().ok();
        let mut line = String::new();
        let read = stdin
            .lock()
            .read_line(&mut line)
            .context("Failed to read from stdin")?;
        if read == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        lines.push(trimmed.to_string());
    }
    println!();
    Ok(lines)
}

fn today_iso() -> String {
    let now = OffsetDateTime::now_utc();
    let fmt = format_description!("[year]-[month]-[day]");
    now.format(fmt)
        .unwrap_or_else(|_| "0000-00-00".to_string())
}

fn update_profile(text: &str, name: &str, what: &str, tools: &str, today: &str) -> String {
    let mut out = text.to_string();

    let identity = build_identity_section(name, what);
    if !identity.is_empty() {
        out = out.replace(IDENTITY_PLACEHOLDER, &identity);
    }

    let tools_block = build_tools_section(tools);
    if !tools_block.is_empty() {
        out = out.replace(TOOLS_PLACEHOLDER, &tools_block);
    }

    out = out.replacen(
        PROJECTS_DATE_PLACEHOLDER,
        &format!("last_updated: {}", today),
        1,
    );

    out
}

fn build_identity_section(name: &str, what: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !name.is_empty() {
        parts.push(format!("**{}**", name));
    }
    if !what.is_empty() {
        parts.push(what.to_string());
    }
    parts.join("\n\n")
}

fn build_tools_section(tools: &str) -> String {
    if tools.is_empty() {
        return String::new();
    }
    tools
        .split(',')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(|t| format!("- {}", t))
        .collect::<Vec<_>>()
        .join("\n")
}

fn update_projects(text: &str, projects: &[String], today: &str) -> String {
    let mut out = text.replacen(
        PROJECTS_DATE_PLACEHOLDER,
        &format!("last_updated: {}", today),
        1,
    );

    if projects.is_empty() {
        return out;
    }

    let new_sections: String = projects
        .iter()
        .map(|p| project_to_section(p))
        .collect::<Vec<_>>()
        .join("");

    if let Some(start) = out.find("## Example Project") {
        let next_h2 = out[start + 1..]
            .find("\n## ")
            .map(|i| start + 1 + i + 1)
            .unwrap_or(out.len());
        let before = &out[..start];
        let after = &out[next_h2..];
        out = format!("{}{}{}", before, new_sections, after);
    } else {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
        out.push_str(&new_sections);
    }

    out
}

fn project_to_section(line: &str) -> String {
    let line = line.trim();
    let (name, desc) = if let Some((n, d)) = line.split_once(" — ") {
        (n.trim(), Some(d.trim()))
    } else if let Some((n, d)) = line.split_once(" - ") {
        (n.trim(), Some(d.trim()))
    } else if let Some((n, d)) = line.split_once(": ") {
        (n.trim(), Some(d.trim()))
    } else {
        (line, None)
    };

    match desc {
        Some(d) if !d.is_empty() => {
            format!("## {}\n\n- **Type**: {}\n\n", name, d)
        }
        _ => {
            format!(
                "## {}\n\n<!-- /pin will add status, stack, decisions as you work. -->\n\n",
                name
            )
        }
    }
}
