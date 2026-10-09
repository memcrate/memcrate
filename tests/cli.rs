use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_memcrate");
const VERBS: [&str; 3] = ["load", "pin", "save"];

struct Home {
    root: PathBuf,
}

impl Home {
    // A space and a non-ASCII letter, like many real Windows user folders.
    fn new(name: &str) -> Home {
        let root = std::env::temp_dir().join(format!(
            "memcrate cli \u{e9} {}-{}",
            std::process::id(),
            name
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        Home { root }
    }

    fn cmd(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(BIN);
        cmd.args(args)
            .env("HOME", &self.root)
            .env("USERPROFILE", &self.root)
            .current_dir(&self.root);
        cmd
    }

    fn run(&self, args: &[&str]) -> Output {
        self.cmd(args).stdin(Stdio::null()).output().unwrap()
    }

    fn run_with_input(&self, args: &[&str], input: &str) -> Output {
        let mut child = self
            .cmd(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    fn own_skill(&self, tool: &str, name: &str) -> String {
        let dir = self.path(tool).join("skills").join(name);
        fs::create_dir_all(&dir).unwrap();
        let text = format!("---\nname: {}\n---\nA skill I wrote myself.\n", name);
        fs::write(dir.join("SKILL.md"), &text).unwrap();
        text
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

// Same text on every OS: the scratch home becomes ~ and separators become /.
fn normalize(text: &str, home: &Path) -> String {
    text.replace(&home.display().to_string(), "~")
        .replace('\\', "/")
}

fn bundled_skill(verb: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("skills/agent")
        .join(verb)
        .join("SKILL.md");
    fs::read_to_string(path).unwrap()
}

fn assert_skills(home: &Home, tool: &str, prefix: &str) {
    for verb in VERBS {
        let dir = home
            .path(tool)
            .join("skills")
            .join(format!("{}{}", prefix, verb));
        assert!(dir.join("SKILL.md").exists(), "missing {}", dir.display());
        assert!(
            dir.join(".memcrate-skill").exists(),
            "no marker in {}",
            dir.display()
        );
    }
}

// Folders are recorded too (with no bytes), so a stray empty folder shows up.
fn snapshot(dir: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    let mut entries = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).unwrap().flatten() {
            let p = entry.path();
            if p.is_dir() {
                entries.push((p.clone(), None));
                stack.push(p);
            } else {
                entries.push((p.clone(), Some(fs::read(&p).unwrap())));
            }
        }
    }
    entries.sort();
    entries
}

#[test]
fn clean_install_matches_the_readme_sample() {
    let home = Home::new("readme");
    let out = home.run(&["--yes"]);
    assert!(out.status.success(), "{}", stderr(&out));

    let readme = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md"))
        .unwrap()
        .replace("\r\n", "\n");
    let section = &readme[readme.find("## Getting started").unwrap()..];
    let sample: Vec<&str> = section
        .lines()
        .skip_while(|l| !l.starts_with("Created your vault"))
        .take_while(|l| !l.starts_with("```"))
        .collect();
    let sample = sample.join("\n");
    assert!(sample.contains("Type them as"), "README sample not found");

    let got = normalize(&stdout(&out), &home.root);
    assert!(
        got.contains(&sample),
        "README sample:\n{}\n\nactual output:\n{}",
        sample,
        got
    );

    for f in [
        ".memcrate",
        "Core/Context/Profile.md",
        "Core/Context/Projects.md",
        "Core/Context/Current State.md",
        "Core/Sessions/_README.md",
    ] {
        assert!(
            home.path("memcrate-vault").join(f).exists(),
            "vault is missing {}",
            f
        );
    }
    assert_skills(&home, ".claude", "");
    assert_skills(&home, ".codex", "");
}

#[test]
fn second_run_reuses_the_vault_and_refreshes_skills() {
    let home = Home::new("rerun");
    assert!(home.run(&["--yes"]).status.success());
    let stale = home.path(".claude/skills/load/SKILL.md");
    fs::write(&stale, "stale copy").unwrap();
    let profile = home.path("memcrate-vault/Core/Context/Profile.md");
    fs::write(&profile, "My own notes.\n").unwrap();

    let out = home.run(&["--yes"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("Using the existing vault at"));
    assert_eq!(fs::read_to_string(&stale).unwrap(), bundled_skill("load"));
    assert_eq!(fs::read_to_string(&profile).unwrap(), "My own notes.\n");
}

#[test]
fn full_adds_the_optional_folders() {
    let home = Home::new("full");
    let out = home.run(&["--yes", "--full"]);
    assert!(out.status.success(), "{}", stderr(&out));
    for folder in ["Projects", "Daily", "Tasks", "Inbox"] {
        assert!(home
            .path("memcrate-vault")
            .join(folder)
            .join(".gitkeep")
            .exists());
    }
}

#[test]
fn vault_flag_expands_a_typed_tilde() {
    let home = Home::new("tilde");
    let out = home.run(&["--yes", "--vault", "~/notes"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(home.path("notes/.memcrate").exists());
    assert!(!home.path("~").exists());
}

#[test]
fn vault_flag_refuses_a_non_empty_folder() {
    let home = Home::new("nonempty");
    let target = home.path("stuff");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("keep.txt"), "mine").unwrap();

    let out = home.run(&["--yes", "--vault", &target.display().to_string()]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("exists and is not empty"),
        "{}",
        stderr(&out)
    );
    assert!(!target.join(".memcrate").exists());
    assert_eq!(fs::read_to_string(target.join("keep.txt")).unwrap(), "mine");
}

#[test]
fn own_skill_moves_that_tool_to_prefixed_names() {
    let home = Home::new("conflict");
    let theirs = home.own_skill(".claude", "save");

    let out = home.run(&["--yes"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("You already have your own save skill for Claude Code"),
        "{}",
        text
    );
    assert!(
        text.contains(
            "Type them as /memcrate-load in Claude Code and Claude Desktop, $load in Codex."
        ),
        "{}",
        text
    );

    assert_eq!(
        fs::read_to_string(home.path(".claude/skills/save/SKILL.md")).unwrap(),
        theirs
    );
    assert_skills(&home, ".claude", "memcrate-");
    assert!(!home.path(".claude/skills/load").exists());
    assert!(!home.path(".claude/skills/pin").exists());
    assert_skills(&home, ".codex", "");
}

#[test]
fn blocked_tools_are_skipped_without_touching_anything() {
    let home = Home::new("blocked");
    for tool in [".claude", ".codex"] {
        home.own_skill(tool, "save");
        home.own_skill(tool, "memcrate-save");
    }
    let claude = snapshot(&home.path(".claude"));
    let codex = snapshot(&home.path(".codex"));

    let out = home.run(&["--yes"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("Skipped Claude Code"), "{}", text);
    assert!(text.contains("Skipped Codex"), "{}", text);
    assert!(!text.contains("You now have"), "{}", text);
    assert_eq!(snapshot(&home.path(".claude")), claude);
    assert_eq!(snapshot(&home.path(".codex")), codex);
}

#[test]
fn profile_fills_in_the_vault_from_piped_answers() {
    let home = Home::new("profile");
    assert!(home.run(&["--yes"]).status.success());

    let answers = "Ada Lovelace\nAnalytical engines\nVS Code, Rust\nApp One: first app\n\n";
    let out = home.run_with_input(&["profile"], answers);
    assert!(out.status.success(), "{}", stderr(&out));
    let context = home.path("memcrate-vault/Core/Context");
    let profile = fs::read_to_string(context.join("Profile.md")).unwrap();
    assert!(profile.contains("Ada Lovelace"), "{}", profile);
    assert!(profile.contains("- Rust"), "{}", profile);
    let projects = fs::read_to_string(context.join("Projects.md")).unwrap();
    assert!(projects.contains("## App One"), "{}", projects);

    // Exits before reading stdin, so no pipe: writing to it could race the exit.
    let again = home.run(&["profile"]);
    assert_eq!(again.status.code(), Some(1));
    assert!(
        stderr(&again).contains("already have content"),
        "{}",
        stderr(&again)
    );
}

#[test]
fn profile_outside_a_vault_fails_cleanly() {
    let home = Home::new("novault");
    let out = home.run(&["profile", &home.path("nothing").display().to_string()]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("does not look like a Memcrate vault"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn unknown_flags_exit_with_usage_error() {
    let home = Home::new("badflag");
    let out = home.run(&["--nope"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn version_matches_the_crate() {
    let home = Home::new("version");
    let out = home.run(&["--version"]);
    assert_eq!(
        stdout(&out).trim(),
        format!("memcrate {}", env!("CARGO_PKG_VERSION"))
    );
}
