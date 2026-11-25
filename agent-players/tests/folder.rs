//! Seam: the files of a player's folders, and the installed program. Expectations come
//! from Cuba Libre #198's probe, whose folders held a Codex and a Claude Code session, and
//! #202's measurements and corrections. No tool runs here.

use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
};
use unfolded_agent_players::folder::{CODEX_WAIT, Folders, Unwritten, install};

/// The probe's own text, as its `AGENTS.md` ended.
const PROBED: &str = include_str!("fixtures/codex_wait.txt");

fn instructions(client: &Path) -> String {
    format!("Run `{} wait`, exactly. {PROBED}", client.display())
}

/// Every file in `directory`, by its path there.
fn files(directory: &Path) -> Vec<(PathBuf, String)> {
    let mut found = Vec::new();
    let mut left = vec![directory.to_owned()];
    while let Some(next) = left.pop() {
        for entry in fs::read_dir(next).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                left.push(path);
            } else {
                let text = fs::read_to_string(&path).unwrap();
                found.push((path.strip_prefix(directory).unwrap().to_owned(), text));
            }
        }
    }
    found.sort();
    found
}

fn names(files: &[(PathBuf, String)]) -> Vec<&str> {
    files
        .iter()
        .map(|(path, _)| path.to_str().unwrap())
        .collect()
}

/// Places of a game's players, beside each other.
struct Places {
    directory: tempfile::TempDir,
    claude_code: PathBuf,
    codex: PathBuf,
    client: PathBuf,
    instructions: String,
}

fn places() -> Places {
    let directory = tempfile::tempdir().unwrap();
    let client = directory.path().join("client/play");
    Places {
        claude_code: directory.path().join("claude-code"),
        codex: directory.path().join("codex"),
        instructions: instructions(&client),
        client,
        directory,
    }
}

impl Places {
    fn folders(&self) -> Folders<'_> {
        Folders {
            claude_code: &self.claude_code,
            codex: &self.codex,
            client: &self.client,
            instructions: &self.instructions,
        }
    }
}

#[test]
fn the_codex_wait_is_the_probes_text() {
    assert_eq!(CODEX_WAIT, PROBED.trim_end());
}

#[test]
fn each_tools_folder_holds_the_instructions_and_its_own_permission_for_the_client() {
    let places = places();
    places.folders().write().unwrap();
    let client = places.client.to_str().unwrap();

    let claude_code = files(&places.claude_code);
    assert_eq!(names(&claude_code), [".claude/settings.json", "CLAUDE.md"]);
    assert_eq!(
        serde_json::from_str::<Value>(&claude_code[0].1).unwrap(),
        json!({
            "disableClaudeAiConnectors": true,
            "enableArtifact": false,
            "permissions": {
                "allow": [format!("Bash({client}:*)")],
                "deny": [
                    "Edit(/CLAUDE.md)",
                    "Edit(/CLAUDE.local.md)",
                    "Edit(/AGENTS.md)",
                    "Edit(/.mcp.json)",
                    "Edit(/.claude/**)",
                    "Edit(//**/CLAUDE.md)",
                    "Edit(//**/CLAUDE.local.md)",
                    "Edit(//**/AGENTS.md)",
                    "Edit(//**/AGENTS.override.md)",
                    "Edit(//**/.claude.json)",
                    "Edit(//**/.mcp.json)",
                    "Edit(//**/.claude/**)",
                    "Edit(//**/.codex/**)",
                    "Edit(//**/.git/**)",
                    format!("Edit(/{}/client/**)", places.directory.path().display()),
                    format!("Edit(/{}/**)", places.codex.display()),
                ],
            },
            "sandbox": {
                "enabled": true,
                "failIfUnavailable": true,
                "autoAllowBashIfSandboxed": false,
                "allowUnsandboxedCommands": false,
                "excludedCommands": [format!("{client}:*")],
            },
        })
    );
    assert_eq!(claude_code[1].1, places.instructions);

    let codex = files(&places.codex);
    assert_eq!(
        names(&codex),
        [
            ".codex/config.toml",
            ".codex/rules/player.rules",
            "AGENTS.md"
        ]
    );
    assert_eq!(
        codex[0].1,
        "background_terminal_max_timeout = 7200000\nsandbox_mode = \"workspace-write\"\n\n[sandbox_workspace_write]\nwritable_roots = []\nexclude_slash_tmp = true\nexclude_tmpdir_env_var = true\n"
    );
    assert_eq!(
        codex[1].1,
        format!(
            "prefix_rule(\n    pattern = [\"{client}\"],\n    decision = \"allow\",\n    justification = \"The player's one client program.\",\n)\n"
        )
    );
    assert_eq!(codex[2].1, places.instructions);
}

/// A session's notes stay, its changes to what was written go, and anything else it left
/// is refused by name, with nothing written.
#[test]
fn writing_again_keeps_notes_replaces_its_own_files_and_refuses_the_rest() {
    let places = places();
    places.folders().write().unwrap();
    let first = files(places.directory.path());
    for folder in [&places.claude_code, &places.codex] {
        fs::create_dir(folder.join("notes")).unwrap();
        fs::write(folder.join("notes/a1b2.md"), "a plan").unwrap();
    }
    for changed in ["claude-code/.claude/settings.json", "codex/AGENTS.md"] {
        fs::write(places.directory.path().join(changed), "{}").unwrap();
    }
    places.folders().write().unwrap();
    let mut kept = files(places.directory.path());
    kept.retain(|(path, _)| path.parent().unwrap().file_name().unwrap() != "notes");
    assert_eq!(kept, first);
    assert_eq!(
        fs::read_to_string(places.codex.join("notes/a1b2.md")).unwrap(),
        "a plan"
    );

    for planted in [
        "codex/AGENTS.override.md",
        "codex/.claude/settings.json",
        "codex/.mcp.json",
        "codex/.codex/rules/more.rules",
        "codex/.git/config",
        "codex/notes/.codex",
        "codex/notes/AGENTS.md",
        "codex/notes/agents.override.md",
        "codex/notes/deep/note.md",
        "claude-code/.claude/settings.local.json",
        "claude-code/.claude/hooks/on.sh",
        "claude-code/.codex/config.toml",
        "claude-code/CLAUDE.local.md",
        "claude-code/notes/CLAUDE.md",
        "claude-code/plan.md",
    ] {
        let path = places.directory.path().join(planted);
        let parent = path.parent().unwrap();
        let made = !parent.exists();
        fs::create_dir_all(parent).unwrap();
        fs::write(&path, "planted").unwrap();
        fs::write(places.codex.join("AGENTS.md"), "changed").unwrap();
        let refused = places.folders().write().unwrap_err();
        let Unwritten::Foreign(named) = &refused else {
            panic!("{planted}: {refused}");
        };
        assert!(path.starts_with(named), "{planted}: {}", named.display());
        assert!(refused.to_string().contains(named.to_str().unwrap()));
        assert_eq!(
            fs::read_to_string(places.codex.join("AGENTS.md")).unwrap(),
            "changed",
            "{planted}: nothing is written"
        );
        fs::remove_file(&path).unwrap();
        if made {
            fs::remove_dir(parent).unwrap();
        }
    }
    // What Claude Code leaves: a directory with no file in it.
    fs::create_dir(places.claude_code.join(".claude/.cc-writes")).unwrap();
    places.folders().write().unwrap();

    // Links, one of them in the place of a file written here.
    let elsewhere = places.directory.path().join("elsewhere");
    fs::write(&elsewhere, "mine").unwrap();
    for link in [
        "codex/AGENTS.md",
        "codex/notes/link.md",
        "claude-code/.claude",
    ] {
        let link = places.directory.path().join(link);
        let aside = places.directory.path().join("aside");
        let moved = link.exists();
        if moved {
            fs::rename(&link, &aside).unwrap();
        }
        symlink(&elsewhere, &link).unwrap();
        let refused = places.folders().write().unwrap_err();
        assert!(
            matches!(&refused, Unwritten::Foreign(named) if *named == link),
            "{refused}"
        );
        fs::remove_file(&link).unwrap();
        if moved {
            fs::rename(&aside, &link).unwrap();
        }
    }
    let aside = places.directory.path().join("aside");
    fs::rename(&places.codex, &aside).unwrap();
    symlink(&aside, &places.codex).unwrap();
    assert!(matches!(
        places.folders().write().unwrap_err(),
        Unwritten::Foreign(named) if named == places.codex
    ));
    assert_eq!(fs::read_to_string(&elsewhere).unwrap(), "mine");
}

#[test]
fn folders_that_would_not_hold_are_refused_and_nothing_is_written() {
    let places = places();
    let refused = |folders: Folders| {
        let refused = folders.write().unwrap_err();
        assert_eq!(files(places.directory.path()), []);
        refused
    };
    // Each of the three places in another, and beside it in one directory.
    let (inside_claude, inside_codex) = (
        places.claude_code.join("bin/play"),
        places.codex.join("play"),
    );
    let in_client = places.directory.path().join("client/codex");
    for (claude_code, codex, client) in [
        (&places.claude_code, &places.codex, &inside_claude),
        (&places.claude_code, &places.codex, &inside_codex),
        (&places.claude_code, &in_client, &places.client),
        (&in_client, &places.codex, &places.client),
        (&places.codex, &places.codex, &places.client),
        (&places.codex.join("other"), &places.codex, &places.client),
        (
            &places.claude_code,
            &places.claude_code.join("other"),
            &places.client,
        ),
    ] {
        let refused = refused(Folders {
            claude_code,
            codex,
            client,
            instructions: &instructions(client),
        });
        assert!(matches!(refused, Unwritten::Nested(..)), "{refused}");
    }
    let with = |instructions: &str| {
        refused(Folders {
            instructions,
            ..places.folders()
        })
    };
    let text = &places.instructions;
    assert!(matches!(
        with(&text.replace("60 seconds", "a minute")),
        Unwritten::NoWait
    ));
    assert!(matches!(with(PROBED), Unwritten::NoClient));
    // Codex reads 32 KiB: the first length is the longest, the second one more.
    let long = |length: usize| format!("{text}{}", "x".repeat(length - text.len()));
    assert!(matches!(with(&long(32_769)), Unwritten::TooLong(32_769)));
    for path in [
        "play",
        "/a path",
        "/a/../play",
        "/a:b",
        "/a(b",
        "/a\"b",
        "/é",
    ] {
        let path = Path::new(path);
        for folders in [
            Folders {
                claude_code: path,
                ..places.folders()
            },
            Folders {
                codex: path,
                ..places.folders()
            },
            Folders {
                client: path,
                instructions: &instructions(path),
                ..places.folders()
            },
        ] {
            let refused = refused(folders);
            assert!(
                matches!(&refused, Unwritten::Path(named) if named == path),
                "{path:?}: {refused}"
            );
        }
    }
    Folders {
        instructions: &long(32_768),
        ..places.folders()
    }
    .write()
    .unwrap();
}

#[test]
fn an_installed_program_can_be_run_and_not_written_and_replaces_an_earlier_one() {
    let directory = tempfile::tempdir().unwrap();
    let built = directory.path().join("built");
    let client = directory.path().join("client/play");
    for text in ["one", "two"] {
        fs::write(&built, text).unwrap();
        // What an interrupted install leaves.
        if client.exists() {
            let left = directory.path().join("client/play.new");
            fs::write(&left, "half").unwrap();
            fs::set_permissions(&left, fs::Permissions::from_mode(0o555)).unwrap();
        }
        install(&built, &client).unwrap();
        assert_eq!(fs::read_to_string(&client).unwrap(), text);
        assert_eq!(
            fs::metadata(&client).unwrap().permissions().mode() & 0o777,
            0o555
        );
    }
    assert_eq!(files(&directory.path().join("client")).len(), 1);
    let missing = install(&directory.path().join("none"), &client).unwrap_err();
    assert!(matches!(missing, Unwritten::Failed(..)), "{missing}");
    assert_eq!(fs::read_to_string(&client).unwrap(), "two");
    assert!(matches!(
        install(&built, Path::new("play")).unwrap_err(),
        Unwritten::Path(_)
    ));
}
