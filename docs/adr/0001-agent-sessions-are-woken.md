---
status: accepted
---

# Agent sessions are woken and do not poll

An agent player's session waits inside one command, which ends when its seat has
something to decide. Until then no model runs, and the command's end is what wakes it.
No session looks in at intervals.

The owner accepted, with Cuba Libre's spec
[#197](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197),
[activated](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197#issuecomment-7156330255)
on 2026-11-25, that sessions are woken and do not poll: an idle session costs nothing, each
event costs at most one waiting call, and a game command needs no approval model. A poll
costs a model call for each period in which nothing happened, and a game's other players
can think for an hour. How each tool does it is measured, not accepted one by one:

- **Codex** waits in one long poll, which #197's own rule takes from
  [Cuba Libre #198](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197#issuecomment-7157575553).
  Its own instructions hold a blocking wait to 60 seconds. The player's instructions lift
  that and say in words how to poll, and the folder's settings raise the limit. One session
  given the words of `folder::CODEX_WAIT` waited 15 minutes without a
  [model call](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197#issuecomment-7173116900),
  as two had that were given code to run. Cuba Libre #205 and Arkham #297 measure the words
  in play; a limit missed there goes to the owner.
- **Claude Code** runs the wait in the background and ends its turn. It is woken when
  the command ends, at three calls for each decision.

A session runs in a small folder that holds its instructions and its tool's permission for
one client program, which lies outside. Each tool has a folder of its own, by a correction
to #197: Codex may write its folder, and in a shared one it wrote Claude Code's settings.

What this gives up: a Codex session is woken only while its window is open and its turn
is not interrupted. Waits longer than 16 minutes were not measured.

## Considered options

- **A waiter for Codex**, a process that queues one message for each event to a session
  that has ended its turn, resuming one whose window is closed. It worked. #197 chose the
  long poll if that met every waiting limit, and it did, so no waiter was built. It is the
  known fallback if real play misses a limit.
- **Waits that end at a period**, as Arkham's Codex partners use. Each costs a model call
  while nothing happens.
- **Codex instructions that do not lift the 60 seconds.** Told only that the wait returns when it is
  needed, a session took about three model calls a minute. Told to wait in one long poll,
  with the limit raised, it took one a minute.
- **Claude Code's foreground wait.** It took one call for each decision, in the one
  session that tried it. It has a longest time, and a message the human types waits for the
  event. The owner had accepted the background wait, and it stays.
- **Claude Code's sandbox with its sockets allowed, and the client program inside it.** A
  command asked the user's session bus to write outside the folder, and it did. So the
  client program is excluded by name, and every other command is to stay inside, its
  sockets blocked.
- **Claude Code's Monitor.** It ends after 30 minutes at most, so it would wake a session
  for nothing.
