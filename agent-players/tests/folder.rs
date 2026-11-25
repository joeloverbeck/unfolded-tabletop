//! Seam: the files a player's folder holds, and the installed program. Expectations come
//! from Cuba Libre #198's probe, whose folders held a Codex and a Claude Code session, and
//! #202's measurement of Claude Code's sandbox. No tool runs here.

use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use unfolded_agent_players::folder::{CODEX_WAIT, Player, Unwritten, install};

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

#[test]
fn the_codex_wait_is_the_probes_text() {
    assert_eq!(CODEX_WAIT, PROBED.trim_end());
}

#[test]
fn the_folder_holds_the_instructions_and_each_tools_permission_for_the_client() {
    let directory = tempfile::tempdir().unwrap();
    let folder = directory.path().join("player");
    let client = directory.path().join("client/play");
    let text = instructions(&client);
    let player = Player {
        folder: &folder,
        client: &client,
        instructions: &text,
    };
    player.write().unwrap();
    // A session's notes, and settings of an earlier game revision.
    fs::create_dir(folder.join("notes")).unwrap();
    fs::write(folder.join("notes/mine.md"), "a plan").unwrap();
    fs::write(folder.join(".claude/settings.json"), "{}").unwrap();
    player.write().unwrap();

    let files = files(&folder);
    let names: Vec<_> = files
        .iter()
        .map(|(path, _)| path.to_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            ".claude/settings.json",
            ".codex/config.toml",
            ".codex/rules/player.rules",
            "AGENTS.md",
            "CLAUDE.md",
            "notes/mine.md",
        ]
    );
    let client = client.to_str().unwrap();
    let beside = directory.path().join("client");
    assert_eq!(
        serde_json::from_str::<Value>(&files[0].1).unwrap(),
        json!({
            "disableClaudeAiConnectors": true,
            "enableArtifact": false,
            "permissions": {
                "allow": [format!("Bash({client}:*)"), "Edit(/notes/**)"],
                "deny": [
                    "Edit(/AGENTS.md)",
                    "Edit(/CLAUDE.md)",
                    "Edit(/.claude/**)",
                    "Edit(/.codex/**)",
                    format!("Edit(/{}/**)", beside.display()),
                ],
            },
            "sandbox": {
                "enabled": true,
                "failIfUnavailable": true,
                "allowUnsandboxedCommands": false,
                "excludedCommands": [format!("{client}:*")],
            },
        })
    );
    assert_eq!(files[1].1, "background_terminal_max_timeout = 7200000\n");
    assert_eq!(
        files[2].1,
        format!(
            "prefix_rule(\n    pattern = [\"{client}\"],\n    decision = \"allow\",\n    justification = \"The player's one client program.\",\n)\n"
        )
    );
    assert_eq!(files[3].1, text);
    assert_eq!(files[4].1, text);
    assert_eq!(files[5].1, "a plan");
}

#[test]
fn a_folder_that_would_not_hold_is_refused_and_nothing_is_written() {
    let directory = tempfile::tempdir().unwrap();
    let folder = directory.path().join("player");
    let client = directory.path().join("client/play");
    let text = instructions(&client);
    let refused = |folder: &Path, client: &Path, instructions: &str| {
        let refused = Player {
            folder,
            client,
            instructions,
        }
        .write()
        .unwrap_err();
        assert_eq!(files(directory.path()), []);
        refused
    };
    let inside = folder.join("play");
    assert!(matches!(
        refused(&folder, &inside, &instructions(&inside)),
        Unwritten::ClientInside
    ));
    assert!(matches!(
        refused(&folder, &client, &text.replace("60 seconds", "a minute")),
        Unwritten::NoWait
    ));
    assert!(matches!(
        refused(&folder, &client, PROBED),
        Unwritten::NoClient
    ));
    // Codex reads 32 KiB: the first length is the longest, the second one more.
    let long = |length: usize| format!("{text}{}", "x".repeat(length - text.len()));
    assert!(matches!(
        refused(&folder, &client, &long(32_769)),
        Unwritten::TooLong(32_769)
    ));
    for path in ["play", "/a path", "/a/../play", "/a:b", "/a(b", "/a\"b", "/é"] {
        let path = Path::new(path);
        assert!(
            matches!(
                refused(&folder, path, &instructions(path)),
                Unwritten::Path(named) if named == path
            ),
            "{path:?}"
        );
        assert!(
            matches!(
                refused(path, &client, &text),
                Unwritten::Path(named) if named == path
            ),
            "{path:?}"
        );
    }
    Player {
        folder: &folder,
        client: &client,
        instructions: &long(32_768),
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
