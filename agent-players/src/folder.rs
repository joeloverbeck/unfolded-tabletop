//! The small folders an agent player's sessions run in, outside your checkout, one for
//! Claude Code and one for Codex: the player's instructions, that tool's permission for
//! one client program, and the settings under which a session waits without a model call.
//! A session is woken by its wait ending; why it does not poll is
//! `docs/adr/0001-agent-sessions-are-woken.md`.
//!
//! [`Folders`] holds your two folders, your client program and your instructions, and
//! writes the folders. [`install`] puts a built client program in its place. Which program
//! it is, what the instructions say besides [`CODEX_WAIT`], where everything lies and when
//! it is written stay yours. Give all three fixed places: each tool asks whether it may
//! trust a folder the first time, and the permission names the program by its path. A
//! path is ASCII letters, digits, `/`, `.`, `_` and `-` only.
//!
//! A tool runs in its own folder because Codex may write any file of the folder it runs in
//! but `.codex`, and so would write Claude Code's settings beside its own. A session may
//! keep notes in its folder's `notes`.
//!
//! **What was measured,** in Cuba Libre #198 and #202; no test here runs a tool. Codex
//! 0.172.0's sandbox, which needs no model, let a command write the Codex folder but
//! `.codex`, and nothing else on the machine: the temporary directories are excluded.
//! Claude Code 2.1.332's permission rules refused its file tools and a copy the program's
//! directory and what configures a session. Its sandbox, which these settings ask for,
//! is not measured to hold a limit: on the one machine it was tried on it could start no
//! command, so every command but the client program's failed. It needs `bubblewrap` and
//! `socat`, and without them these settings have Claude Code refuse to start, saying so.
//! Tell your player's human.

use serde_json::json;
use std::{
    error::Error,
    fmt,
    fs::{self, OpenOptions},
    io::{self, Write},
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
/// Where a session keeps its notes, in its folder.
const NOTES: &str = "notes";

/// The folders of a game's agent players, as [`Folders::write`] makes them.
#[derive(Clone, Copy, Debug)]
pub struct Folders<'a> {
    /// The folder Claude Code's sessions run in.
    pub claude_code: &'a Path,
    /// The folder Codex's sessions run in.
    pub codex: &'a Path,
    /// The one client program the sessions may run. Keep nothing else in its directory.
    pub client: &'a Path,
    /// The player's whole instructions, [`CODEX_WAIT`] among them, naming the client
    /// program by the path above: the permission knows no other spelling.
    pub instructions: &'a str,
}

impl Folders<'_> {
    /// Write both folders, making them where there are none. Claude Code's holds the
    /// instructions as `CLAUDE.md`, and `.claude/settings.json`; Codex's holds them as
    /// `AGENTS.md`, and `.codex/config.toml` and `.codex/rules/player.rules`. Files of
    /// those names are replaced.
    ///
    /// Anything else in a folder is refused, since a session could have put it there to
    /// change what a later one may do; but for plain files directly in `notes`, whose names
    /// begin with neither a dot, `CLAUDE` nor `AGENTS`. A link is refused everywhere. A refusal comes before
    /// anything is written; a file that could not be written may leave the folders written
    /// in part.
    ///
    /// In Codex the client program's commands are allowed and run outside its sandbox,
    /// which lets other commands write the folder and nothing else, and one poll may last
    /// as long as [`CODEX_WAIT`] asks. In Claude Code the client program's commands are
    /// allowed, and excluded from its sandbox, which every other command is to run in, by
    /// no permission of its own; its file tools are denied the program's directory,
    /// Codex's folder, both tools' settings in the home directory, and what configures a
    /// session in its own folder; and its connectors and artifacts are off, whose
    /// descriptions each model call would carry. The client program's is the only
    /// permission either tool is given.
    pub fn write(&self) -> Result<(), Unwritten> {
        let client = plain(self.client)?;
        let codex = plain(self.codex)?;
        plain(self.claude_code)?;
        let Some(beside) = self.client.parent() else {
            return Err(Unwritten::Path(self.client.to_owned()));
        };
        let places = [beside, self.claude_code, self.codex];
        for (index, inner) in places.into_iter().enumerate() {
            for outer in places[..index].iter().chain(&places[index + 1..]) {
                if inner.starts_with(outer) {
                    return Err(Unwritten::Nested(inner.to_owned(), (*outer).to_owned()));
                }
            }
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
        let beside = beside.display();
        let settings = json!({
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
                    "Edit(~/.claude/**)",
                    "Edit(~/.codex/**)",
                    // Two slashes begin a path that is not the folder's own.
                    format!("Edit(/{beside}/**)"),
                    format!("Edit(/{codex}/**)"),
                ],
            },
            "sandbox": {
                "enabled": true,
                "failIfUnavailable": true,
                "autoAllowBashIfSandboxed": false,
                "allowUnsandboxedCommands": false,
                "excludedCommands": [format!("{client}:*")],
            },
        });
        let claude_code = [
            ("CLAUDE.md", self.instructions.to_owned()),
            (".claude/settings.json", format!("{settings:#}\n")),
        ];
        let codex = [
            ("AGENTS.md", self.instructions.to_owned()),
            (
                ".codex/config.toml",
                format!(
                    "background_terminal_max_timeout = {CODEX_LIMIT}\nsandbox_mode = \"workspace-write\"\n\n[sandbox_workspace_write]\nwritable_roots = []\nexclude_slash_tmp = true\nexclude_tmpdir_env_var = true\n"
                ),
            ),
            (
                ".codex/rules/player.rules",
                format!(
                    "prefix_rule(\n    pattern = [\"{client}\"],\n    decision = \"allow\",\n    justification = \"The player's one client program.\",\n)\n"
                ),
            ),
        ];
        own(self.claude_code, &claude_code)?;
        own(self.codex, &codex)?;
        put(self.claude_code, &claude_code)?;
        put(self.codex, &codex)
    }
}

/// That `folder`, where there is one, holds nothing but `files` and notes.
fn own(folder: &Path, files: &[(&str, String)]) -> Result<(), Unwritten> {
    match fs::symlink_metadata(folder) {
        Ok(found) if found.is_dir() => {}
        Ok(_) => return Err(Unwritten::Foreign(folder.to_owned())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(failed(folder, error)),
    }
    let mut left = vec![PathBuf::new()];
    while let Some(within) = left.pop() {
        let directory = folder.join(&within);
        for entry in fs::read_dir(&directory).map_err(|error| failed(&directory, error))? {
            let entry = entry.map_err(|error| failed(&directory, error))?;
            let path = within.join(entry.file_name());
            let kind = entry
                .file_type()
                .map_err(|error| failed(&entry.path(), error))?;
            let ours = if kind.is_dir() {
                path == Path::new(NOTES)
                    || files
                        .iter()
                        .any(|(name, _)| Path::new(name).starts_with(&path))
            } else {
                kind.is_file()
                    && (within == Path::new(NOTES) && note(&entry.file_name())
                        || files.iter().any(|(name, _)| Path::new(name) == path))
            };
            if !ours {
                return Err(Unwritten::Foreign(entry.path()));
            }
            if kind.is_dir() {
                left.push(path);
            }
        }
    }
    Ok(())
}

/// Whether a file of this name in `notes` is a note, which neither tool reads by itself.
fn note(name: &std::ffi::OsStr) -> bool {
    let name = name.to_string_lossy().to_lowercase();
    ![".", "claude", "agents"]
        .iter()
        .any(|start| name.starts_with(start))
}

fn put(folder: &Path, files: &[(&str, String)]) -> Result<(), Unwritten> {
    for (name, text) in files {
        let path = folder.join(name);
        let directory = path.parent().unwrap_or(folder);
        fs::create_dir_all(directory).map_err(|error| failed(directory, error))?;
        // A file of its own each time, so that nothing is written through a link.
        match fs::remove_file(&path) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => {
                return Err(failed(&path, error));
            }
            _ => {}
        }
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .and_then(|mut file| file.write_all(text.as_bytes()))
            .map_err(|error| failed(&path, error))?;
    }
    Ok(())
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

/// Why the folders or the client program were not written.
#[derive(Debug)]
#[non_exhaustive]
pub enum Unwritten {
    /// A path is not absolute, contains `..`, or holds more than ASCII letters, digits,
    /// `/`, `.`, `_` and `-`.
    Path(PathBuf),
    /// The first lies in the second or is it: two of the folders and the client program's
    /// directory, as they are spelled; links are not followed.
    Nested(PathBuf, PathBuf),
    /// The instructions lack [`CODEX_WAIT`].
    NoWait,
    /// The instructions do not name the client program by its path.
    NoClient,
    /// The instructions are this many bytes long, more than Codex reads.
    TooLong(usize),
    /// A folder holds this, which is not among what was written there and is no note, or
    /// the folder itself is something else, such as a link.
    Foreign(PathBuf),
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
            Self::Nested(inner, outer) => write!(
                f,
                "{} lies in {}, which must be a place of its own",
                inner.display(),
                outer.display()
            ),
            Self::NoWait => f.write_str("the instructions lack how a Codex session waits"),
            Self::NoClient => {
                f.write_str("the instructions do not name the client program by its path")
            }
            Self::TooLong(bytes) => write!(
                f,
                "the instructions are {bytes} bytes long, and Codex reads {LONGEST}"
            ),
            Self::Foreign(path) => write!(
                f,
                "{} is not part of a player's folder and could change what a session may do: look at it, then remove it",
                path.display()
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
