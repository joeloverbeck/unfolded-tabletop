//! The small folder an agent player's session runs in, outside your checkout: the
//! player's instructions and, for Claude Code and Codex, the permission for one client
//! program, so that a game command needs no prompt and no approval model, with the
//! settings under which a session waits without a model call. A session is woken by its
//! wait ending; why it does not poll is `docs/adr/0001-agent-sessions-are-woken.md`.
//!
//! A [`Player`] holds your folder, your client program and your instructions, and writes
//! the folder. [`install`] puts a built client program in its place. Which program it is,
//! what the instructions say besides [`CODEX_WAIT`], where the folder and the program lie
//! and when they are written stay yours. Give both fixed places: each tool asks whether it
//! may trust a folder the first time, and the permission names the program by its path.
//!
//! What keeps a session from writing the program is its tool's own sandbox, and no test
//! here runs either tool. Codex's lets a session write its folder and the temporary
//! directories, so keep the program out of both. Claude Code's is turned on by the
//! folder's settings for every command but the client program's, and its file tools are
//! denied the program's directory. Cuba Libre #198 and #202 measured what Codex 0.172.0
//! and Claude Code 2.1.332 do with these settings.

use serde_json::json;
use std::{
    error::Error,
    fmt, fs, io,
    os::unix::fs::PermissionsExt,
    path::{Component, Path, PathBuf},
};

/// How a Codex session waits without a model call, in the words Cuba Libre #198 measured:
/// neither a raised limit alone nor "one long poll" without this call held there. Say
/// which of your commands wait and how to run them, "exactly", then this as it stands.
/// The folder's settings raise the limit it names.
pub const CODEX_WAIT: &str = "\
This replaces your usual limit of 60 seconds for a blocking wait and your usual progress updates: I ask for one blocking wait of up to two hours, I need no communication from you while it runs, and I can interrupt it by typing. This session's background_terminal_max_timeout is 7200000 ms, so the long poll is allowed. Every extra check costs me money. Make each of them this one exec call, with the command in place of COMMAND:

// @exec: {\"yield_time_ms\": 7200000}
let r = await tools.exec_command({cmd:\"COMMAND\",yield_time_ms:30000});
while (r.session_id !== undefined) r = await tools.write_stdin({session_id:r.session_id,chars:\"\",yield_time_ms:7200000});
text(r);

If that cell yields anyway, wait on it with yield_time_ms 7200000.";

/// The milliseconds [`CODEX_WAIT`] names.
const CODEX_LIMIT: u32 = 7_200_000;
/// What Codex reads of its instructions by its default `project_doc_max_bytes`.
const LONGEST: usize = 32 * 1024;

/// A player's folder, as [`Player::write`] makes it.
#[derive(Clone, Copy, Debug)]
pub struct Player<'a> {
    /// The folder the sessions run in.
    pub folder: &'a Path,
    /// The one client program the sessions may run, outside the folder.
    pub client: &'a Path,
    /// The player's whole instructions, [`CODEX_WAIT`] among them, naming the client
    /// program by the path above: the permission knows no other spelling.
    pub instructions: &'a str,
}

impl Player<'_> {
    /// Write the folder, making it where there is none: `AGENTS.md` and `CLAUDE.md`, the
    /// same instructions for Codex and for Claude Code, and each tool's settings. Files
    /// of those names are replaced, and nothing else in the folder is touched. A refusal
    /// comes before anything is written; a file that could not be written may leave the
    /// folder written in part.
    ///
    /// In Codex the client program's commands are allowed, and one poll may last as long
    /// as [`CODEX_WAIT`] asks. In Claude Code they are allowed and excluded from its
    /// sandbox, where every other command runs; its file tools may write the folder's
    /// `notes` directory unasked, and are denied the instructions, the settings and the
    /// program's directory; and its connectors and artifacts are off, whose descriptions
    /// each model call would carry.
    pub fn write(&self) -> Result<(), Unwritten> {
        let client = plain(self.client)?;
        plain(self.folder)?;
        if self.client.starts_with(self.folder) {
            return Err(Unwritten::ClientInside);
        }
        if !self.instructions.contains(CODEX_WAIT) {
            return Err(Unwritten::NoWait);
        }
        if !self.instructions.contains(client) {
            return Err(Unwritten::NoClient);
        }
        if self.instructions.len() > LONGEST {
            return Err(Unwritten::TooLong(self.instructions.len()));
        }
        let beside = self
            .client
            .parent()
            .and_then(Path::to_str)
            .ok_or_else(|| Unwritten::Path(self.client.to_owned()))?;
        let claude = json!({
            "disableClaudeAiConnectors": true,
            "enableArtifact": false,
            "permissions": {
                "allow": [format!("Bash({client}:*)"), "Edit(/notes/**)"],
                "deny": [
                    "Edit(/AGENTS.md)",
                    "Edit(/CLAUDE.md)",
                    "Edit(/.claude/**)",
                    "Edit(/.codex/**)",
                    // Two slashes begin a path that is not the folder's own.
                    format!("Edit(/{beside}/**)"),
                ],
            },
            "sandbox": {
                "enabled": true,
                "failIfUnavailable": true,
                "allowUnsandboxedCommands": false,
                "excludedCommands": [format!("{client}:*")],
            },
        });
        let files = [
            ("AGENTS.md", self.instructions.to_owned()),
            ("CLAUDE.md", self.instructions.to_owned()),
            (
                ".codex/config.toml",
                format!("background_terminal_max_timeout = {CODEX_LIMIT}\n"),
            ),
            (
                ".codex/rules/player.rules",
                format!(
                    "prefix_rule(\n    pattern = [\"{client}\"],\n    decision = \"allow\",\n    justification = \"The player's one client program.\",\n)\n"
                ),
            ),
            (".claude/settings.json", format!("{claude:#}\n")),
        ];
        for (name, text) in files {
            let path = self.folder.join(name);
            let directory = path.parent().unwrap_or(self.folder);
            fs::create_dir_all(directory).map_err(|error| failed(directory, error))?;
            fs::write(&path, text).map_err(|error| failed(&path, error))?;
        }
        Ok(())
    }
}

/// Copy the `built` client program to `client`, the path your permission names. The copy
/// may be run and not written, and takes an earlier one's place at once, so a session that
/// waits in the earlier one goes on waiting.
pub fn install(built: &Path, client: &Path) -> Result<(), Unwritten> {
    plain(client)?;
    let (Some(directory), Some(name)) = (client.parent(), client.file_name()) else {
        return Err(Unwritten::Path(client.to_owned()));
    };
    let mut new = name.to_owned();
    new.push(".new");
    let new = directory.join(new);
    fs::create_dir_all(directory).map_err(|error| failed(directory, error))?;
    // What an interrupted install left may not be written.
    match fs::remove_file(&new) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(failed(&new, error)),
        _ => {}
    }
    fs::copy(built, &new).map_err(|error| failed(built, error))?;
    fs::set_permissions(&new, fs::Permissions::from_mode(0o555))
        .map_err(|error| failed(&new, error))?;
    fs::rename(&new, client).map_err(|error| failed(client, error))
}

/// `path` as text, when it is absolute and a permission or a setting reads all of it as a
/// path.
fn plain(path: &Path) -> Result<&str, Unwritten> {
    path.to_str()
        .filter(|text| {
            path.is_absolute()
                && !path.components().any(|part| part == Component::ParentDir)
                && text
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "/._-".contains(c))
        })
        .ok_or_else(|| Unwritten::Path(path.to_owned()))
}

fn failed(path: &Path, error: io::Error) -> Unwritten {
    Unwritten::Failed(path.to_owned(), error)
}

/// Why a player's folder or its client program was not written.
#[derive(Debug)]
#[non_exhaustive]
pub enum Unwritten {
    /// A path is not absolute, contains `..`, or holds more than ASCII letters, digits,
    /// `/`, `.`, `_` and `-`.
    Path(PathBuf),
    /// The client program's path lies in the folder, which its sessions write.
    ClientInside,
    /// The instructions lack [`CODEX_WAIT`].
    NoWait,
    /// The instructions do not name the client program by its path.
    NoClient,
    /// The instructions are this many bytes long, more than Codex reads.
    TooLong(usize),
    /// A file or directory could not be read or written.
    Failed(PathBuf, io::Error),
}

impl fmt::Display for Unwritten {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(path) => write!(
                f,
                "{} must be an absolute path of ASCII letters, digits, \"/\", \".\", \"_\" and \"-\" only, without \"..\"",
                path.display()
            ),
            Self::ClientInside => f.write_str("the client program lies in the player's folder"),
            Self::NoWait => f.write_str("the instructions lack how a Codex session waits"),
            Self::NoClient => {
                f.write_str("the instructions do not name the client program by its path")
            }
            Self::TooLong(bytes) => write!(
                f,
                "the instructions are {bytes} bytes long, and Codex reads {LONGEST}"
            ),
            Self::Failed(path, error) => write!(f, "{}: {error}", path.display()),
        }
    }
}

/// Each cause's text is this error's, so only the cause's own source is passed on.
impl Error for Unwritten {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Failed(_, error) => error.source(),
            _ => None,
        }
    }
}
