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
//! **What was measured,** in Cuba Libre #198 and #202, on one machine; no test here runs a
//! tool, and no Codex session has run in such a folder since it was one for each tool.
//!
//! - Codex 0.172.0's sandbox, tried without a model, let a command write the Codex folder
//!   but `.codex`, and no other file that the rest of the machine sees. The folder's
//!   approval policy, `never`, stands over a user's own: the session is told that a
//!   command which asks to leave the sandbox is rejected, whoever reviews the user's. That
//!   the rule then still runs the client program outside the sandbox was read in Codex's
//!   source at that version, not run.
//! - Claude Code 2.1.332 in `auto` mode, with these settings but for `.agents`, ran the
//!   client program by the rule, with no approval model, in the background too, and earlier
//!   settings' interactive session showed no prompt. Its deny rules refused its file tools
//!   and a copy what they name. Its file tools wrote the folder's other files unasked, and
//!   files elsewhere that no rule denies, with the approval model's consent. In `default`
//!   mode the client program ran by the rule with the sockets of its sandbox allowed, and
//!   in an earlier folder of both tools; a note was asked for.
//! - Claude Code's sandbox, with these settings, could start no command on that machine, an
//!   Ubuntu that restricts user namespaces. So every command but the client program's
//!   failed, whatever it was. With its sockets allowed, which these settings do not, it
//!   started, and refused each write that was tried to a place outside the folder that is
//!   not a temporary directory, and to what the deny rules spell out inside it. A pattern,
//!   `**`, does not bind a command there: one that is approved could write
//!   `notes/CLAUDE.md` or `AGENTS.override.md`, until the folder is written again. One that
//!   asked the session bus to write outside had it done. Claude Code's sandbox needs
//!   `bubblewrap` and `socat`; without `bubblewrap`, Claude Code ended at its start with
//!   the reason, and without `socat` it was not tried.
//!
//! The sessions of one tool share its folder: until it is written again, one Codex session
//! may leave another an `AGENTS.override.md`. The MCP servers of a user's own settings
//! stay with the tool in any folder.
//!
//! Tell your player's human what of this they meet.

use serde_json::json;
use std::{
    error::Error,
    fmt,
    fs::{self, OpenOptions},
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::{Component, Path, PathBuf},
};

/// How a Codex session waits without a model call. Say which of your commands wait and
/// how to run them, "exactly", then this as it stands. The folder's settings raise the
/// limit it names.
///
/// These words, with the code in them, are what Cuba Libre #198 measured. It did not try
/// the words without the code, which is being measured, so this text may change: quote it
/// nowhere else, and write your folders from it.
pub const CODEX_WAIT: &str = include_str!("codex_wait.txt");

/// The longest poll, in milliseconds, that [`CODEX_WAIT`] asks for.
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
    /// Any other file in a folder is refused, since a session could have put it there to
    /// change what a later one may do; but for plain files directly in `notes`, whose names
    /// begin with neither a dot, `claude` nor `agents`, in any case. A link is refused
    /// everywhere. A refusal comes before anything is written; a file that could not be
    /// written may leave the folders written in part.
    ///
    /// In Codex the client program's commands are allowed and to run outside its sandbox.
    /// That lets other commands write the folder but `.codex` and nothing else, and none
    /// may ask to leave it. One poll may last as long as [`CODEX_WAIT`] asks. In Claude Code the client program's commands are
    /// allowed, and excluded from its sandbox, which every other command is to run in, without
    /// a permission of its own; its file tools are denied the program's directory,
    /// Codex's folder and, wherever they lie, `CLAUDE.md`, `CLAUDE.local.md`, `AGENTS.md`,
    /// `AGENTS.override.md`, `.mcp.json`, `.claude.json`, `.claude`, `.codex`, `.agents` and
    /// `.git`;
    /// and its connectors and artifacts are off, whose
    /// descriptions each model call would carry. The client program's is the only
    /// permission either tool is given; what else a tool does unasked is its own.
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
                    // In the folder, spelled out: the sandbox reads no pattern.
                    "Edit(/CLAUDE.md)",
                    "Edit(/CLAUDE.local.md)",
                    "Edit(/AGENTS.md)",
                    "Edit(/.mcp.json)",
                    "Edit(/.claude/**)",
                    // Two slashes begin a path that is not the folder's own: what
                    // instructs or configures a session of either tool, anywhere.
                    "Edit(//**/CLAUDE.md)",
                    "Edit(//**/CLAUDE.local.md)",
                    "Edit(//**/AGENTS.md)",
                    "Edit(//**/AGENTS.override.md)",
                    "Edit(//**/.claude.json)",
                    "Edit(//**/.mcp.json)",
                    "Edit(//**/.claude/**)",
                    "Edit(//**/.codex/**)",
                    "Edit(//**/.agents/**)",
                    "Edit(//**/.git/**)",
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
                    "approval_policy = \"never\"\napprovals_reviewer = \"user\"\nbackground_terminal_max_timeout = {CODEX_LIMIT}\nsandbox_mode = \"workspace-write\"\n\n[sandbox_workspace_write]\nwritable_roots = []\nexclude_slash_tmp = true\nexclude_tmpdir_env_var = true\n"
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

/// That `folder`, where there is one, holds no file but `files` and notes. An empty
/// directory configures nothing, and Claude Code leaves one in `.claude`.
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
            if kind.is_dir() {
                left.push(path);
                continue;
            }
            let ours = kind.is_file()
                && (within == Path::new(NOTES) && note(&entry.file_name())
                    || files.iter().any(|(name, _)| Path::new(name) == path));
            if !ours {
                return Err(Unwritten::Foreign(entry.path()));
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
