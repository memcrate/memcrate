use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use include_dir::{include_dir, Dir};
use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use time::macros::format_description;
use time::OffsetDateTime;

static REFERENCE_VAULT: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/reference-vault");
static AGENT_SKILLS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/skills/agent");

const OPTIONAL_FOLDERS: &[&str] = &["Projects", "Daily", "Tasks", "Inbox"];

// Written into each installed skill folder so a later --force can tell which
// skills Memcrate owns. Without it we would happily delete a same-named skill
// the user wrote themselves.
const SKILL_MARKER: &str = ".memcrate-skill";

#[derive(Parser)]
#[command(
    name = "memcrate",
    version,
    about = "Set up a markdown context vault your AI tools can read.",
    long_about = "Memcrate creates a portable, local-first markdown vault and installs the /load, /save, and /pin skills for Claude Code, Claude Desktop, and Codex. Run it with no arguments for the guided setup."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Where the vault should live. Skips the location prompt.
    #[arg(long, value_name = "PATH")]
    vault: Option<PathBuf>,

    /// Take every default and ask nothing. Implied when there is no terminal.
    #[arg(long)]
    yes: bool,

    /// Also create the optional folders (Projects/, Daily/, Tasks/, Inbox/).
    #[arg(long)]
    full: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Answer a few questions to fill in your Profile and Projects files.
    Profile {
        /// Path to the vault. Found automatically when omitted.
        path: Option<PathBuf>,

        /// Rewrite the files even if they already have content.
        #[arg(long)]
        force: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum Tool {
    /// Claude Code (Anthropic's CLI). Installs to ~/.claude/skills/, which
    /// Claude Desktop's local agent mode also reads, so this covers both.
    ClaudeCode,
    /// Codex (OpenAI's CLI). Installs to ~/.codex/skills/.
    Codex,
    /// Every supported tool.
    All,
}

impl Tool {
    fn label(self) -> &'static str {
        match self {
            Tool::ClaudeCode => "Claude Code",
            Tool::Codex => "Codex",
            Tool::All => "all tools",
        }
    }

    /// Where this tool loads user-level skills from.
    fn skills_dir(self) -> Result<PathBuf> {
        let home = home_dir()?;
        Ok(match self {
            Tool::ClaudeCode => home.join(".claude").join("skills"),
            Tool::Codex => home.join(".codex").join("skills"),
            Tool::All => bail!("Tool::All has no single skills directory"),
        })
    }

    fn expand(self) -> Vec<Tool> {
        match self {
            Tool::All => vec![Tool::ClaudeCode, Tool::Codex],
            t => vec![t],
        }
    }
}

const DEFAULT_VAULT_DIR: &str = "memcrate-vault";

/// The whole product in one command: pick a vault location, create it, seed
/// Profile and Projects, and install the skills for every supported tool.
fn run(cli: Cli) -> Result<()> {
    let ask = !cli.yes && io::stdin().is_terminal();

    if ask {
        println!("Memcrate creates a markdown vault your AI tools can read, then installs");
        println!("the /load, /save, and /pin skills for Claude Code, Claude Desktop, and Codex.");
        println!("Press Enter to accept a default, or Ctrl-C to stop.");
        println!();
    }

    let default = home_dir()?.join(DEFAULT_VAULT_DIR);
    let vault = match cli.vault {
        Some(p) => expand_home(&p.to_string_lossy())?,
        None if ask => ask_vault_path(&default)?,
        None => default,
    };

    let reused = vault.join(".memcrate").exists();
    if reused {
        println!("Using the existing vault at {}.", vault.display());
    } else {
        ensure_writable(&vault, false)?;
        fs::create_dir_all(&vault)
            .with_context(|| format!("Failed to create {}", vault.display()))?;
        REFERENCE_VAULT
            .extract(&vault)
            .with_context(|| format!("Failed to extract reference vault to {}", vault.display()))?;
        println!("Created your vault at {}.", vault.display());
    }

    if cli.full {
        for folder in OPTIONAL_FOLDERS {
            let dir = vault.join(folder);
            fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create {}", dir.display()))?;
            fs::write(dir.join(".gitkeep"), "")
                .with_context(|| format!("Failed to write .gitkeep in {}", dir.display()))?;
        }
        println!("Added the optional folders: Projects, Daily, Tasks, Inbox.");
    }

    println!();

    // One tool failing should not cost the user the other one.
    let mut installed: Vec<(Tool, SkillSet)> = Vec::new();
    let mut problems: Vec<String> = Vec::new();
    for tool in Tool::All.expand() {
        let dest = tool.skills_dir()?;
        match install_skills(tool, &dest, true) {
            Ok(set) => installed.push((tool, set)),
            Err(e) => problems.push(format!("{}: {}", tool.label(), first_line(&e.to_string()))),
        }
    }

    for p in &problems {
        println!("Skipped {}", p);
    }
    if !problems.is_empty() {
        println!();
        println!("Memcrate never replaces a skill it did not install. Rename or remove");
        println!("the listed skill(s) and re-run if you want Memcrate's versions.");
    }

    println!();
    if let Some(hint) = type_hint(&installed) {
        println!(
            "You now have three verbs in {}:",
            tool_list(installed.len())
        );
        println!("  load   read your vault and get oriented. Run this first.");
        println!("  pin    promote a fact into your permanent context files.");
        println!("  save   write a session log before you finish.");
        println!();
        println!("{}", hint);
        println!();
    }
    println!("Optional: `memcrate profile` answers a few questions about you so");
    println!("your first load has something to read.");
    println!();
    Ok(())
}

/// `memcrate profile`: fill in Profile.md and Projects.md from a few questions.
/// Optional, and separate from setup on purpose. Everything works without it.
fn profile(path: Option<PathBuf>, force: bool) -> Result<()> {
    let vault = find_vault(path)?;
    let profile_path = vault.join("Core").join("Context").join("Profile.md");
    let projects_path = vault.join("Core").join("Context").join("Projects.md");

    if !profile_path.exists() || !projects_path.exists() {
        bail!(
            "{} does not look like a Memcrate vault: Core/Context/Profile.md is missing.\n\
             Run `memcrate` first to create one.",
            vault.display()
        );
    }

    let profile_text = fs::read_to_string(&profile_path)
        .with_context(|| format!("Failed to read {}", profile_path.display()))?;
    let projects_text = fs::read_to_string(&projects_path)
        .with_context(|| format!("Failed to read {}", projects_path.display()))?;

    let untouched =
        profile_text.contains(IDENTITY_PLACEHOLDER) || projects_text.contains("## Example Project");
    if !untouched && !force {
        bail!(
            "Profile.md and Projects.md already have content. Edit them directly, or \
             pass --force to answer the questions again and overwrite."
        );
    }

    println!("Filling in {}", vault.display());
    println!("Press Enter to skip any question.");
    println!();

    let name = prompt_line("Your name")?;
    let what_you_do =
        prompt_line("What do you build? (e.g. \"Full-stack web apps, mostly React and Node\")")?;
    let tools = prompt_line(
        "Tools you use daily, comma-separated (e.g. \"VS Code, TypeScript, Postgres\")",
    )?;
    let projects = prompt_multiline(
        "Projects you're working on, one per line (e.g. \"Acme Dashboard: internal analytics tool\").\nBlank line when done",
    )?;

    if name.is_empty() && what_you_do.is_empty() && tools.is_empty() && projects.is_empty() {
        println!("Nothing entered, left your files alone.");
        return Ok(());
    }

    let today = today_iso();
    fs::write(
        &profile_path,
        update_profile(&profile_text, &name, &what_you_do, &tools, &today),
    )
    .with_context(|| format!("Failed to write {}", profile_path.display()))?;
    fs::write(
        &projects_path,
        update_projects(&projects_text, &projects, &today),
    )
    .with_context(|| format!("Failed to write {}", projects_path.display()))?;

    println!();
    println!("Saved to Profile.md and Projects.md.");
    println!("Your tools will read these on the next /load.");
    Ok(())
}

/// Locate a vault: explicit path, then the current directory or a parent with a
/// marker, then a single vault in the home directory, then the default.
fn find_vault(path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = path {
        return expand_home(&p.to_string_lossy());
    }

    if let Ok(cwd) = std::env::current_dir() {
        let mut walk: Option<&Path> = Some(cwd.as_path());
        while let Some(dir) = walk {
            if dir.join(".memcrate").exists() {
                return Ok(dir.to_path_buf());
            }
            walk = dir.parent();
        }
    }

    if let Ok(home) = home_dir() {
        let mut found: Vec<PathBuf> = Vec::new();
        if let Ok(entries) = fs::read_dir(&home) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && p.join(".memcrate").exists() {
                    found.push(p);
                }
            }
        }
        found.sort();
        match found.len() {
            1 => return Ok(found.into_iter().next().unwrap()),
            n if n > 1 => {
                let list: Vec<String> =
                    found.iter().map(|p| format!("  {}", p.display())).collect();
                bail!(
                    "Found more than one vault:\n{}\n\nPick one:\n  memcrate profile <path>",
                    list.join("\n")
                );
            }
            _ => {}
        }

        let default = home.join(DEFAULT_VAULT_DIR);
        if default.exists() {
            return Ok(default);
        }
    }

    bail!("No vault found. Run `memcrate` to create one, or pass the path:\n  memcrate profile <path>")
}

fn type_hint(installed: &[(Tool, SkillSet)]) -> Option<String> {
    let parts: Vec<String> = installed
        .iter()
        .map(|(tool, set)| match tool {
            Tool::Codex => format!("${} in Codex", set.name("load")),
            _ => format!("/{} in Claude Code and Claude Desktop", set.name("load")),
        })
        .collect();
    if parts.is_empty() {
        return None;
    }
    Some(format!("Type them as {}.", parts.join(", ")))
}

fn and_list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{} and {}", a, b),
        [rest @ .., last] => format!("{}, and {}", rest.join(", "), last),
    }
}

fn tool_list(installed: usize) -> &'static str {
    match installed {
        2 => "Claude Code, Claude Desktop, and Codex",
        1 => "your tool",
        _ => "no tools",
    }
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or(s).to_string()
}

fn ask_vault_path(default: &Path) -> Result<PathBuf> {
    loop {
        let answer = prompt_line(&format!(
            "Where should your vault live? [{}]",
            default.display()
        ))?;
        let chosen = if answer.is_empty() {
            default.to_path_buf()
        } else {
            expand_home(&answer)?
        };

        if chosen.exists() && !chosen.join(".memcrate").exists() {
            let empty = fs::read_dir(&chosen)
                .map(|mut d| d.next().is_none())
                .unwrap_or(false);
            if !empty {
                println!(
                    "{} already exists and is not empty, and it is not a Memcrate vault.",
                    chosen.display()
                );
                println!("Pick a different path.");
                println!();
                continue;
            }
        }
        return Ok(chosen);
    }
}

/// Shells expand `~` before we ever see it, but a typed answer keeps it literal.
fn expand_home(input: &str) -> Result<PathBuf> {
    let trimmed = input.trim();
    if let Some(rest) = trimmed.strip_prefix('~') {
        if rest.is_empty() {
            return home_dir();
        }
        // is_separator accepts `\` as well as `/` on Windows only.
        if rest.starts_with(std::path::is_separator) {
            return Ok(home_dir()?.join(&rest[1..]));
        }
    }
    Ok(PathBuf::from(trimmed))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Profile { path, force }) => profile(path, force),
        None => run(cli),
    }
}

fn home_dir() -> Result<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .context(
            "Cannot resolve home directory: neither HOME nor USERPROFILE is set. \
             Pass an explicit path.",
        )
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

fn skill_names() -> Vec<String> {
    let mut names: Vec<String> = AGENT_SKILLS
        .dirs()
        .filter_map(|d| {
            d.path()
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
        })
        .collect();
    names.sort();
    names
}

const SKILL_PREFIX: &str = "memcrate-";

/// Which names a tool's skills went in under.
#[derive(Debug, PartialEq, Eq)]
enum SkillSet {
    Plain,
    /// The user owns skills with these plain names, so ours carry SKILL_PREFIX.
    Prefixed(Vec<String>),
}

impl SkillSet {
    fn name(&self, verb: &str) -> String {
        match self {
            SkillSet::Plain => verb.to_string(),
            SkillSet::Prefixed(_) => format!("{}{}", SKILL_PREFIX, verb),
        }
    }
}

fn blocks_before(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '.' | '~' | '/' | '_' | '-')
}

fn blocks_after(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '-' | '_' | '/')
}

/// Point `/load`, `$save` and friends at the prefixed names, leaving paths and words alone.
fn prefix_commands(text: &str) -> String {
    let verbs = skill_names();
    let mut out = String::with_capacity(text.len() + 64);
    let mut prev: Option<char> = None;
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        if matches!(c, '/' | '$') && !matches!(prev, Some(p) if blocks_before(p)) {
            let verb = verbs.iter().find(|v| {
                rest[1..].starts_with(v.as_str())
                    && !matches!(rest[1 + v.len()..].chars().next(), Some(n) if blocks_after(n))
            });
            if let Some(verb) = verb {
                out.push(c);
                out.push_str(SKILL_PREFIX);
                out.push_str(verb);
                rest = &rest[1 + verb.len()..];
                prev = verb.chars().last();
                continue;
            }
        }
        out.push(c);
        rest = &rest[c.len_utf8()..];
        prev = Some(c);
    }
    out
}

fn prefix_skill_md(text: &str) -> String {
    let verbs = skill_names();
    let mut out = String::with_capacity(text.len() + 64);
    let mut fences = 0;
    for (i, line) in text.split_inclusive('\n').enumerate() {
        let body = line.trim_end_matches(['\n', '\r']);
        if body == "---" && (i == 0 || fences == 1) {
            fences += 1;
        } else if fences == 1 {
            let name = body.strip_prefix("name:").map(str::trim);
            if let Some(verb) = name.filter(|n| verbs.iter().any(|v| v == n)) {
                out.push_str(&format!("name: {}{}", SKILL_PREFIX, verb));
                out.push_str(&line[body.len()..]);
                continue;
            }
        }
        out.push_str(line);
    }
    prefix_commands(&out)
}

fn write_skill(dir: &Dir, root: &Path, target: &Path, prefixed: bool) -> Result<()> {
    fs::create_dir_all(target).with_context(|| format!("Failed to create {}", target.display()))?;
    for sub in dir.dirs() {
        write_skill(sub, root, target, prefixed)?;
    }
    for file in dir.files() {
        let rel = file.path().strip_prefix(root).unwrap_or(file.path());
        let out = target.join(rel);
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        let contents = match file.contents_utf8() {
            Some(text) if prefixed && rel == Path::new("SKILL.md") => {
                prefix_skill_md(text).into_bytes()
            }
            _ => file.contents().to_vec(),
        };
        fs::write(&out, contents).with_context(|| format!("Failed to write {}", out.display()))?;
    }
    Ok(())
}

/// A skill folder belongs to Memcrate if we marked it, or (for installs from
/// versions before the marker existed) if its SKILL.md names Memcrate.
fn is_memcrate_skill(dir: &Path) -> bool {
    if dir.join(SKILL_MARKER).exists() {
        return true;
    }
    fs::read_to_string(dir.join("SKILL.md"))
        .map(|s| s.to_lowercase().contains("memcrate"))
        .unwrap_or(false)
}

fn owns_skill(dest: &Path, name: &str) -> bool {
    // Pre-marker installs only ever used the plain names, so prefixed ones need the marker.
    if name.starts_with(SKILL_PREFIX) {
        return dest.join(name).join(SKILL_MARKER).exists();
    }
    is_memcrate_skill(&dest.join(name))
}

fn install_skills(tool: Tool, dest: &Path, force: bool) -> Result<SkillSet> {
    fs::create_dir_all(dest).with_context(|| format!("Failed to create {}", dest.display()))?;

    let verbs = skill_names();
    let foreign = |n: &String| dest.join(n).exists() && !owns_skill(dest, n);
    let conflicts: Vec<String> = verbs.iter().filter(|n| foreign(n)).cloned().collect();
    let (set, other) = if conflicts.is_empty() {
        (SkillSet::Plain, SkillSet::Prefixed(Vec::new()))
    } else {
        (SkillSet::Prefixed(conflicts.clone()), SkillSet::Plain)
    };
    let names: Vec<String> = verbs.iter().map(|v| set.name(v)).collect();

    let existing: Vec<String> = names
        .iter()
        .filter(|n| dest.join(n).exists())
        .cloned()
        .collect();

    let (ours, theirs): (Vec<String>, Vec<String>) =
        existing.iter().cloned().partition(|n| owns_skill(dest, n));

    // Never delete a skill someone else wrote, whatever flags we were given.
    if !theirs.is_empty() {
        let mut blocking = conflicts.clone();
        blocking.extend(theirs);
        bail!(
            "{} already has skill(s) Memcrate did not install: {}.\n\n\
             Memcrate will not overwrite skills it does not own, even with --force.\n\
             Move or rename them first.",
            dest.display(),
            blocking.join(", ")
        );
    }

    if !ours.is_empty() && !force {
        bail!(
            "Memcrate skills are already installed in {}: {}. Pass --force to update them.",
            dest.display(),
            ours.join(", ")
        );
    }

    for name in &ours {
        let p = dest.join(name);
        fs::remove_dir_all(&p)
            .with_context(|| format!("Failed to remove existing {}", p.display()))?;
    }

    let prefixed = matches!(set, SkillSet::Prefixed(_));
    for skill in AGENT_SKILLS.dirs() {
        let verb = skill
            .path()
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let target = dest.join(set.name(&verb));
        fs::create_dir_all(&target)
            .with_context(|| format!("Failed to create {}", target.display()))?;
        // Marker first, so a write that fails halfway still leaves a folder we own.
        let marker = target.join(SKILL_MARKER);
        fs::write(
            &marker,
            "Installed by Memcrate. Safe for memcrate to replace.\n",
        )
        .with_context(|| format!("Failed to write {}", marker.display()))?;
        write_skill(skill, skill.path(), &target, prefixed)?;
    }

    // Marker only here: a user's own plain skill may mention memcrate.
    for verb in &verbs {
        let p = dest.join(other.name(verb));
        if p.join(SKILL_MARKER).exists() {
            fs::remove_dir_all(&p)
                .with_context(|| format!("Failed to remove old {}", p.display()))?;
        }
    }

    println!(
        "Installed {} skills for {} to {}",
        names.len(),
        tool.label(),
        dest.display()
    );
    if !conflicts.is_empty() {
        println!("{}", prefixed_notice(tool, &conflicts, &names));
    }
    Ok(set)
}

fn prefixed_notice(tool: Tool, conflicts: &[String], names: &[String]) -> String {
    format!(
        "You already have your own {} skill{} for {}, so Memcrate installed its verbs there as {}.",
        and_list(conflicts),
        if conflicts.len() == 1 { "" } else { "s" },
        tool.label(),
        and_list(names)
    )
}

const IDENTITY_PLACEHOLDER: &str = "<!-- Who you are professionally. One paragraph. -->";
const TOOLS_PLACEHOLDER: &str = "<!-- Editor, languages, runtimes, CLIs, default services. -->";
const PROJECTS_DATE_PLACEHOLDER: &str = "last_updated: YYYY-MM-DD";

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
    now.format(fmt).unwrap_or_else(|_| "0000-00-00".to_string())
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
    // Matches what a user might type, so the em dash stays: macOS substitutes
    // one automatically when you type a hyphen surrounded by spaces.
    let (name, desc) = if let Some((n, d)) = line.split_once(" \u{2014} ") {
        (n.trim(), Some(d.trim()))
    } else if let Some((n, d)) = line.split_once(" - ") {
        (n.trim(), Some(d.trim()))
    } else if let Some((n, d)) = line.split_once(": ") {
        (n.trim(), Some(d.trim()))
    } else {
        // "MyApp:" or "MyApp -" with nothing after: drop the dangling separator.
        (
            line.trim_end_matches([':', '-', '\u{2014}']).trim_end(),
            None,
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("memcrate-test-{}-{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn project_section_em_dash_separator() {
        let s = project_to_section("MyApp \u{2014} a thing I build");
        assert_eq!(s, "## MyApp\n\n- **Type**: a thing I build\n\n");
    }

    #[test]
    fn project_section_hyphen_separator() {
        let s = project_to_section("MyApp - a thing I build");
        assert_eq!(s, "## MyApp\n\n- **Type**: a thing I build\n\n");
    }

    #[test]
    fn project_section_colon_separator() {
        let s = project_to_section("MyApp: a thing I build");
        assert_eq!(s, "## MyApp\n\n- **Type**: a thing I build\n\n");
    }

    #[test]
    fn project_section_bare_name_gets_placeholder() {
        let s = project_to_section("MyApp");
        assert!(s.starts_with("## MyApp\n\n<!--"));
    }

    #[test]
    fn project_section_dangling_separator_gets_clean_placeholder() {
        for input in ["MyApp: ", "MyApp:", "MyApp -", "MyApp \u{2014}"] {
            let s = project_to_section(input);
            assert!(s.starts_with("## MyApp\n\n<!--"), "input: {:?}", input);
        }
    }

    #[test]
    fn tools_section_splits_and_skips_blanks() {
        assert_eq!(
            build_tools_section("vim, rust,  ,go"),
            "- vim\n- rust\n- go"
        );
        assert_eq!(build_tools_section(""), "");
    }

    #[test]
    fn identity_section_variants() {
        assert_eq!(build_identity_section("Brad", ""), "**Brad**");
        assert_eq!(build_identity_section("", "I teach"), "I teach");
        assert_eq!(
            build_identity_section("Brad", "I teach"),
            "**Brad**\n\nI teach"
        );
        assert_eq!(build_identity_section("", ""), "");
    }

    #[test]
    fn update_profile_fills_placeholders_and_date() {
        let text = format!(
            "---\n{}\n---\n\n{}\n\n{}\n",
            PROJECTS_DATE_PLACEHOLDER, IDENTITY_PLACEHOLDER, TOOLS_PLACEHOLDER
        );
        let out = update_profile(&text, "Brad", "I teach", "vim, rust", "2026-08-06");
        assert!(out.contains("last_updated: 2026-08-06"));
        assert!(out.contains("**Brad**\n\nI teach"));
        assert!(out.contains("- vim\n- rust"));
        assert!(!out.contains(IDENTITY_PLACEHOLDER));
        assert!(!out.contains(TOOLS_PLACEHOLDER));
    }

    #[test]
    fn update_profile_skipped_answers_keep_placeholders() {
        let text = format!("{}\n{}\n", IDENTITY_PLACEHOLDER, TOOLS_PLACEHOLDER);
        let out = update_profile(&text, "", "", "", "2026-08-06");
        assert!(out.contains(IDENTITY_PLACEHOLDER));
        assert!(out.contains(TOOLS_PLACEHOLDER));
    }

    #[test]
    fn update_projects_replaces_example_and_keeps_next_section() {
        let text = format!(
            "{}\n\n## Example Project\n\n- **Type**: sample\n\n## Keep Me\n\ncontent\n",
            PROJECTS_DATE_PLACEHOLDER
        );
        let projects = vec!["App One: first".to_string(), "App Two".to_string()];
        let out = update_projects(&text, &projects, "2026-08-06");
        assert!(out.contains("last_updated: 2026-08-06"));
        assert!(!out.contains("## Example Project"));
        assert!(out.contains("## App One\n\n- **Type**: first"));
        assert!(out.contains("## App Two"));
        assert!(out.contains("## Keep Me\n\ncontent"));
    }

    #[test]
    fn update_projects_replaces_example_when_last_section() {
        let text = format!(
            "{}\n\n## Example Project\n\n- **Type**: sample\n",
            PROJECTS_DATE_PLACEHOLDER
        );
        let out = update_projects(&text, &["App".to_string()], "2026-08-06");
        assert!(!out.contains("## Example Project"));
        assert!(out.contains("## App"));
    }

    #[test]
    fn update_projects_appends_when_no_example_heading() {
        let out = update_projects("# Projects", &["App".to_string()], "2026-08-06");
        assert!(out.starts_with("# Projects\n"));
        assert!(out.contains("## App"));
    }

    #[test]
    fn update_projects_empty_list_only_updates_date() {
        let text = format!("{}\n\n## Example Project\n", PROJECTS_DATE_PLACEHOLDER);
        let out = update_projects(&text, &[], "2026-08-06");
        assert!(out.contains("last_updated: 2026-08-06"));
        assert!(out.contains("## Example Project"));
    }

    #[test]
    fn today_iso_shape() {
        let d = today_iso();
        assert_eq!(d.len(), 10);
        assert_eq!(&d[4..5], "-");
        assert_eq!(&d[7..8], "-");
    }

    #[test]
    fn typed_tilde_paths_expand_to_the_home_dir() {
        let home = match home_dir() {
            Ok(h) => h,
            Err(_) => return,
        };
        assert_eq!(expand_home("~").unwrap(), home);
        assert_eq!(expand_home("~/notes").unwrap(), home.join("notes"));
        assert_eq!(expand_home("  ~/notes  ").unwrap(), home.join("notes"));
        assert_eq!(expand_home("/tmp/x").unwrap(), PathBuf::from("/tmp/x"));
        // A leading "~" that is not a path separator is a real directory name.
        assert_eq!(expand_home("~notes").unwrap(), PathBuf::from("~notes"));
        #[cfg(windows)]
        assert_eq!(expand_home("~\\notes").unwrap(), home.join("notes"));
        #[cfg(not(windows))]
        assert_eq!(expand_home("~\\notes").unwrap(), PathBuf::from("~\\notes"));
    }

    #[test]
    fn default_vault_lives_in_the_home_dir() {
        if home_dir().is_err() {
            return;
        }
        let default = home_dir().unwrap().join(DEFAULT_VAULT_DIR);
        assert!(default.ends_with(DEFAULT_VAULT_DIR));
        assert!(default.starts_with(home_dir().unwrap()));
    }

    #[test]
    fn marker_file_marks_a_skill_as_ours() {
        let dir = tmp("owned");
        fs::write(dir.join("SKILL.md"), "unrelated content").unwrap();
        assert!(!is_memcrate_skill(&dir));
        fs::write(dir.join(SKILL_MARKER), "").unwrap();
        assert!(is_memcrate_skill(&dir));
    }

    #[test]
    fn pre_marker_installs_are_recognized_by_content() {
        let dir = tmp("legacy");
        fs::write(dir.join("SKILL.md"), "reads their Memcrate vault").unwrap();
        assert!(is_memcrate_skill(&dir));
    }

    #[test]
    fn a_users_own_skill_is_never_ours() {
        let dir = tmp("foreign");
        fs::write(
            dir.join("SKILL.md"),
            "---\nname: load\n---\nLoad from my Obsidian vault.",
        )
        .unwrap();
        assert!(!is_memcrate_skill(&dir));
    }

    #[test]
    fn missing_skill_md_is_not_ours() {
        assert!(!is_memcrate_skill(&tmp("empty-skill")));
    }

    #[test]
    fn all_expands_to_every_tool() {
        assert_eq!(Tool::All.expand().len(), 2);
        assert_eq!(Tool::Codex.expand(), vec![Tool::Codex]);
    }

    #[test]
    fn each_tool_has_its_own_skills_dir() {
        if std::env::var("HOME").is_err() && std::env::var("USERPROFILE").is_err() {
            return;
        }
        let claude = Tool::ClaudeCode.skills_dir().unwrap();
        let codex = Tool::Codex.skills_dir().unwrap();
        assert!(claude.ends_with(".claude/skills"));
        assert!(codex.ends_with(".codex/skills"));
        assert_ne!(claude, codex);
        assert!(Tool::All.skills_dir().is_err());
    }

    #[test]
    fn bundled_skills_are_the_three_verbs() {
        let mut names = skill_names();
        names.sort();
        assert_eq!(names, vec!["load", "pin", "save"]);
    }

    fn bundled(verb: &str) -> String {
        AGENT_SKILLS
            .get_file(format!("{}/SKILL.md", verb))
            .and_then(|f| f.contents_utf8())
            .unwrap()
            .to_string()
    }

    fn own_skill(dest: &Path, name: &str) -> String {
        // Mentioning "memcrate" here would make it look like a pre-marker install of ours.
        let text = "---\nname: mine\n---\nA skill I wrote myself.\n".to_string();
        fs::create_dir_all(dest.join(name)).unwrap();
        fs::write(dest.join(name).join("SKILL.md"), &text).unwrap();
        text
    }

    fn owned_skill(dest: &Path, name: &str) {
        fs::create_dir_all(dest.join(name)).unwrap();
        fs::write(dest.join(name).join("SKILL.md"), "old").unwrap();
        fs::write(dest.join(name).join(SKILL_MARKER), "").unwrap();
    }

    fn snapshot(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        let mut out = Vec::new();
        for entry in fs::read_dir(dir).unwrap().flatten() {
            let p = entry.path();
            if p.is_dir() {
                out.push((p.clone(), Vec::new()));
                out.extend(snapshot(&p));
            } else {
                out.push((p.clone(), fs::read(&p).unwrap()));
            }
        }
        out.sort();
        out
    }

    #[test]
    fn clean_dir_gets_the_plain_set() {
        let dest = tmp("skills-clean");
        assert_eq!(
            install_skills(Tool::ClaudeCode, &dest, true).unwrap(),
            SkillSet::Plain
        );
        for verb in ["load", "pin", "save"] {
            let dir = dest.join(verb);
            assert!(dir.join(SKILL_MARKER).exists());
            assert_eq!(
                fs::read_to_string(dir.join("SKILL.md")).unwrap(),
                bundled(verb)
            );
            assert!(!dest.join(format!("memcrate-{}", verb)).exists());
        }
    }

    #[test]
    fn a_users_own_save_moves_memcrate_to_prefixed_names() {
        let dest = tmp("skills-own-save");
        let theirs = own_skill(&dest, "save");
        assert_eq!(
            install_skills(Tool::ClaudeCode, &dest, true).unwrap(),
            SkillSet::Prefixed(vec!["save".to_string()])
        );
        assert_eq!(
            fs::read_to_string(dest.join("save").join("SKILL.md")).unwrap(),
            theirs
        );
        assert!(!dest.join("save").join(SKILL_MARKER).exists());
        assert!(!dest.join("load").exists());
        assert!(!dest.join("pin").exists());
        for verb in ["load", "pin", "save"] {
            let dir = dest.join(format!("memcrate-{}", verb));
            assert!(dir.join(SKILL_MARKER).exists());
            let text = fs::read_to_string(dir.join("SKILL.md")).unwrap();
            // Windows checkouts embed the bundled skills with CRLF.
            let name = format!("name: memcrate-{}", verb);
            assert!(text.lines().any(|l| l == name));
            for token in ["/load", "/pin", "/save", "$load", "$pin", "$save"] {
                assert!(!text.contains(token), "{} still has {}", verb, token);
            }
        }
        let save = fs::read_to_string(dest.join("memcrate-save").join("SKILL.md")).unwrap();
        assert!(save.contains("`/memcrate-load`"));
    }

    #[test]
    fn switching_to_prefixed_removes_memcrate_plain_skills() {
        let dest = tmp("skills-to-prefixed");
        owned_skill(&dest, "pin");
        owned_skill(&dest, "save");
        let theirs = own_skill(&dest, "load");
        assert_eq!(
            install_skills(Tool::Codex, &dest, true).unwrap(),
            SkillSet::Prefixed(vec!["load".to_string()])
        );
        assert!(!dest.join("pin").exists());
        assert!(!dest.join("save").exists());
        assert_eq!(
            fs::read_to_string(dest.join("load").join("SKILL.md")).unwrap(),
            theirs
        );
        for verb in ["load", "pin", "save"] {
            assert!(dest
                .join(format!("memcrate-{}", verb))
                .join(SKILL_MARKER)
                .exists());
        }
    }

    #[test]
    fn an_unmarked_load_mentioning_memcrate_survives_a_switch_to_prefixed() {
        let dest = tmp("skills-unmarked-load");
        let theirs = own_skill(&dest, "save");
        let load = "---\nname: load\n---\nRead my memcrate vault.\n";
        fs::create_dir_all(dest.join("load")).unwrap();
        fs::write(dest.join("load").join("SKILL.md"), load).unwrap();
        assert_eq!(
            install_skills(Tool::ClaudeCode, &dest, true).unwrap(),
            SkillSet::Prefixed(vec!["save".to_string()])
        );
        assert_eq!(
            fs::read_to_string(dest.join("load").join("SKILL.md")).unwrap(),
            load
        );
        assert!(!dest.join("load").join(SKILL_MARKER).exists());
        assert_eq!(
            fs::read_to_string(dest.join("save").join("SKILL.md")).unwrap(),
            theirs
        );
        for verb in ["load", "pin", "save"] {
            assert!(dest
                .join(format!("memcrate-{}", verb))
                .join(SKILL_MARKER)
                .exists());
        }
    }

    #[test]
    fn switching_back_to_plain_removes_memcrate_prefixed_skills() {
        let dest = tmp("skills-to-plain");
        for verb in ["load", "pin", "save"] {
            owned_skill(&dest, &format!("memcrate-{}", verb));
        }
        assert_eq!(
            install_skills(Tool::ClaudeCode, &dest, true).unwrap(),
            SkillSet::Plain
        );
        for verb in ["load", "pin", "save"] {
            assert!(dest.join(verb).join(SKILL_MARKER).exists());
            assert!(!dest.join(format!("memcrate-{}", verb)).exists());
        }
    }

    #[test]
    fn a_foreign_skill_at_a_prefixed_name_blocks_the_install() {
        let dest = tmp("skills-blocked");
        own_skill(&dest, "save");
        let theirs = dest.join("memcrate-load");
        fs::create_dir_all(&theirs).unwrap();
        fs::write(
            theirs.join("SKILL.md"),
            "---\nname: memcrate-load\n---\nMy own memcrate loader.\n",
        )
        .unwrap();
        owned_skill(&dest, "pin");
        owned_skill(&dest, "memcrate-pin");
        let before = snapshot(&dest);
        let err = install_skills(Tool::ClaudeCode, &dest, true)
            .unwrap_err()
            .to_string();
        assert!(first_line(&err).contains("save, memcrate-load"));
        assert_eq!(snapshot(&dest), before);
    }

    #[test]
    fn prefix_rewrites_frontmatter_name_and_commands() {
        let text = "---\nname: save\ndescription: runs /load or $load, or asks.\n---\n\
                    - `/save <label>` - scope\n(same rule as `/load`)\n\
                    `/pin` is the only verb\nRun `/load` in your next session.\n\"/pin\" alone\n";
        let out = prefix_skill_md(text);
        assert!(out.starts_with("---\nname: memcrate-save\n"));
        assert!(out.contains("runs /memcrate-load or $memcrate-load, or asks."));
        assert!(out.contains("`/memcrate-save <label>`"));
        assert!(out.contains("(same rule as `/memcrate-load`)"));
        assert!(out.contains("`/memcrate-pin` is the only verb"));
        assert!(out.contains("Run `/memcrate-load` in your next session."));
        assert!(out.contains("\"/memcrate-pin\" alone"));
        assert_eq!(prefix_skill_md(&out), out);
    }

    #[test]
    fn prefix_handles_crlf_frontmatter() {
        let out = prefix_skill_md("---\r\nname: load\r\n---\r\nRun /save.\r\n");
        assert_eq!(
            out,
            "---\r\nname: memcrate-load\r\n---\r\nRun /memcrate-save.\r\n"
        );
    }

    #[test]
    fn prefix_leaves_paths_and_plain_words_alone() {
        for text in [
            "Write to Core/Sessions/ and foo/save/ today.",
            "a/load, x./pin, ~/save, _/pin, -/load, //save, a$load",
            "/loads /load-x /load_y /save/ $pins",
            "reload the page, pin this, save this session",
            "name: load outside frontmatter",
        ] {
            assert_eq!(prefix_skill_md(text), text);
        }
        assert_eq!(
            prefix_skill_md("---\ntitle: x\n---\nname: load\n"),
            "---\ntitle: x\n---\nname: load\n"
        );
        assert_eq!(prefix_commands("/load"), "/memcrate-load");
        assert_eq!(prefix_commands("($save)"), "($memcrate-save)");
    }

    #[test]
    fn type_hint_matches_each_tools_outcome() {
        let clean = vec![
            (Tool::ClaudeCode, SkillSet::Plain),
            (Tool::Codex, SkillSet::Plain),
        ];
        assert_eq!(
            type_hint(&clean).unwrap(),
            "Type them as /load in Claude Code and Claude Desktop, $load in Codex."
        );
        let mixed = vec![
            (
                Tool::ClaudeCode,
                SkillSet::Prefixed(vec!["load".to_string()]),
            ),
            (Tool::Codex, SkillSet::Plain),
        ];
        assert_eq!(
            type_hint(&mixed).unwrap(),
            "Type them as /memcrate-load in Claude Code and Claude Desktop, $load in Codex."
        );
        let codex_only = vec![(Tool::Codex, SkillSet::Prefixed(vec!["save".to_string()]))];
        assert_eq!(
            type_hint(&codex_only).unwrap(),
            "Type them as $memcrate-load in Codex."
        );
        assert_eq!(type_hint(&[]), None);
    }

    #[test]
    fn prefixed_notice_names_the_users_skills() {
        let names: Vec<String> = ["load", "pin", "save"]
            .iter()
            .map(|v| format!("memcrate-{}", v))
            .collect();
        assert_eq!(
            prefixed_notice(Tool::ClaudeCode, &["load".to_string()], &names),
            "You already have your own load skill for Claude Code, so Memcrate installed \
             its verbs there as memcrate-load, memcrate-pin, and memcrate-save."
        );
        assert!(prefixed_notice(
            Tool::Codex,
            &["load".to_string(), "save".to_string()],
            &names
        )
        .starts_with("You already have your own load and save skills for Codex,"));
    }

    #[test]
    fn ensure_writable_ok_for_missing_or_empty() {
        let dir = tmp("empty");
        assert!(ensure_writable(&dir.join("does-not-exist"), false).is_ok());
        assert!(ensure_writable(&dir, false).is_ok());
    }

    #[test]
    fn ensure_writable_rejects_non_empty_without_force() {
        let dir = tmp("nonempty");
        fs::write(dir.join("file.txt"), "x").unwrap();
        assert!(ensure_writable(&dir, false).is_err());
        assert!(ensure_writable(&dir, true).is_ok());
    }

    #[test]
    fn ensure_writable_rejects_existing_vault_without_force() {
        let dir = tmp("vault");
        fs::write(dir.join(".memcrate"), "").unwrap();
        let err = ensure_writable(&dir, false).unwrap_err().to_string();
        assert!(err.contains("--force"));
        assert!(ensure_writable(&dir, true).is_ok());
    }
}
