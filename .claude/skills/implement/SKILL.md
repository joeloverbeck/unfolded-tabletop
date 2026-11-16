---
name: implement
description: "Implement a piece of work based on a spec or set of tickets."
disable-model-invocation: true
---

Implement the work described by the user in the spec or tickets.

If the user passes a ticket reference, fetch it from the issue tracker and state its title before starting. If the reference is ambiguous, ask.

Before activating the work, read the activation rules: `scripts/rule PURP-3 WORK-1 WORK-2 WORK-3 WORK-5 EVID-5 OPS-1 OPS-4 AGENT-5`. Before the first merge, read the merge rules: `scripts/rule AUTH-1 AUTH-2 AUTH-3 OPS-6 OPS-7 PUB-1`. A list of rule IDs in a ticket is not a reading. The work record names the IDs you read.

Note the start time from `date` before the first substantive step. The ticket's appetite is measured from it, and the work record states it.

When the work spans repositories, read each game's row in `docs/agents/consumers.md` before opening an issue or a branch there: its seat host, verification, real capture, merge command and work-item limit. Open the game's own issue before its first commit, so that every commit there can say `For #N`. Record each real held start with `held-start` (`held-start --help`): the "before" run under its own case with `--clean`, from the game's `main`, before you edit your worktree. Work in each game in a worktree of its `origin/main` under `~/.cache/agent-tmp/`, with `CARGO_TARGET_DIR` set to the worktree's own `target`: the game's main checkout belongs to whoever else is working there, even when it looks clean, and a shared target lets two checkouts overwrite each other's `target/debug/<game>`, which the other task's tests and your captures then run. Run its first, cold build and every test in a fresh scope: `systemd-run --user --scope --collect -q -p MemoryHigh=6G -p MemoryMax=8G -p MemorySwapMax=1G -- <command>`.

When the work changes a producer interface that consumers adopt, pin each consumer to the pushed producer branch's head and pass its own checks there before merging the producer. Each consumer's tests encode rules its code does not show. Then move every pin to the producer's merge commit and check again. When `git diff --quiet <pinned head> <merge commit>` holds, the pin and lockfile are the only change: the consumer's CI on the moved pin is that check. Move each pin with `move-pin` (`docs/agents/consumers.md`, "Moving a pin").

Call the Skill tool with "tdd" where possible, at pre-agreed seams.

Run typechecking regularly, single test files regularly, and the full test suite once at the end.

Before writing a pull request's Evidence section, confirm each test it cites: it shows as `ok`, not `ignored`, in the log it cites, and it constructs the component the change touches (grep the test for it). A test that passes without reaching the changed path is not evidence for it; name what does reach the path, or write a test that does.

Once done, call the Skill tool with "code-review" to review the work. When the work spans repositories, review each repository's pull request before it merges, and check its Evidence section against the logs it cites.

Commit your work to the current branch.

Commit locally, then run the check at that commit, so its log names the revision it passed. Publish in a call of its own: read the check's exit status, then push or merge.

To merge this repository's pull request: wait until `gh run list --branch <branch> --event pull_request` shows its run (`gh pr checks --watch` exits at once with "no checks reported" before the run exists), `gh run watch <id> --exit-status`, then `gh pr merge <n> --merge --match-head-commit <sha>`.
