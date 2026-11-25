---
status: accepted
---

# Agent sessions are woken and do not poll

An agent player's session waits inside one command, which ends when its seat has
something to decide. Until then no model runs, and the command's end is what wakes it.
No session looks in at intervals.

Joel accepted this with Cuba Libre's spec
[#197](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197),
[activated](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197#issuecomment-7156330255)
on 2026-11-25: an idle session costs nothing, each event costs at most one waiting call,
and a game command needs no approval model. A poll costs a model call for each period in
which nothing happened, and a game's other players can think for an hour.

[Cuba Libre #198](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197#issuecomment-7157575553)
measured each way in both tools before anything was built:

- **Codex** waits in one long poll. Its own instructions hold a blocking wait to 60
  seconds, so the player's instructions must give the exact call, and the folder's
  settings must raise the limit. With both, a wait of 15 minutes took no model call, in
  both sessions that tried. `folder::CODEX_WAIT` is that wording.
- **Claude Code** runs the wait in the background and ends its turn. It is woken when
  the command ends, at three calls for each decision.

The session runs in a small folder that holds its instructions and each tool's permission
for the one client program. That permission is what keeps the approval model out of play.
The program lies outside the folder, and each tool's sandbox keeps a session from writing
it. Claude Code's is off by default, so the folder turns it on.

What this gives up: a Codex session is woken only while its window is open and its turn
is not interrupted. Waits longer than 16 minutes were not measured.

## Considered options

- **A waiter for Codex**, a process that queues one message for each event to a session
  that has ended its turn, resuming one whose window is closed. It worked. #197 chose the
  long poll if that met every waiting limit, and it did, so no waiter was built. It is the
  known fallback if real play misses a limit.
- **Waits that end at a period**, as Arkham's Codex partners used. Each costs a model call
  while nothing happens.
- **Codex instructions without the exact call.** Told only that the wait returns when it is
  needed, a session took about three model calls a minute. Told to wait in one long poll,
  with the limit raised, it took one a minute.
- **Claude Code's foreground wait.** It took one call for each decision, in the one
  session that tried it. It has a longest time, and a message the human types waits for the
  event. Joel had accepted the background wait, and it stays.
- **Claude Code's sandbox with its sockets allowed, and the client program inside it.** Its
  file limits held in Cuba Libre #202's measurement, but a command asked the user's
  session bus to write outside the folder, and it did. So the client program is excluded
  by name, and every other command stays inside with its sockets blocked. On the machine
  of that measurement, an Ubuntu that restricts user namespaces, the sandbox then starts
  no command at all: every command but the client's fails.
- **Claude Code's Monitor.** It ends after 30 minutes at most, so it would wake a session
  for nothing.
