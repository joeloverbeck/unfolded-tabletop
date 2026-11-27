---
name: implement
description: "Implement a piece of work based on a spec or set of tickets."
disable-model-invocation: true
---

Implement the work described by the user in the spec or tickets.

If the user passes a ticket reference, fetch it from the issue tracker and state its title before starting. If the reference is ambiguous, ask.

Before activating the work, read the activation rules: `scripts/rule PURP-3 WORK-1 WORK-2 WORK-3 WORK-5 EVID-5 OPS-1 OPS-4 AGENT-5`. Before the first merge, read the merge rules: `scripts/rule AUTH-1 AUTH-2 AUTH-3 OPS-6 OPS-7 PUB-1`. A list of rule IDs in a ticket is not a reading. The work record names the IDs you read.

Note the start time from `date` before the first substantive step. The ticket's appetite is measured from it, and the work record states it.

When the work spans repositories, read each game's row in `docs/agents/consumers.md` before opening an issue or a branch there: its seat host, verification, real capture, merge command and work-item limit. Open the game's own issue before its first commit, so that every commit there can say `For #N`. Work in each game in a worktree of its `origin/main` under `~/.cache/agent-tmp/`, with `CARGO_TARGET_DIR` set to the worktree's own `target`: the game's main checkout belongs to whoever else is working there, even when it looks clean, and a shared target lets two checkouts overwrite each other's `target/debug/<game>`, which the other task's tests and your captures then run. Run its first, cold build and every test in a fresh scope: `systemd-run --user --scope --collect -q -p MemoryHigh=6G -p MemoryMax=8G -p MemorySwapMax=1G -- <command>`, unless the game's row says its command starts its own. From the consumer worktree, run each heavy command through the shared checkout's `scripts/build-slot -- <command>`. It reports queue time and command time separately. Run one recorded check command per game, in sequence. Keep its revision, command, log and terminal result before starting the next, and tell the user each result as it arrives. After an interruption, read the retained records and resume from the first incomplete command. A game's main moves while you work: fetch and bring in its `origin/main` before you push its branch and again before you merge it, and run its check again when main moved. Read a game's work-item limit issue just before you record a ruling there, and again just after, so a ruling another session records meanwhile shows at once.

Choose evidence before editing. Exercise the changed behavior on its production path. Use the held capture procedure in `docs/agents/consumers.md` when the change can affect rendering, input, admission or behavior at the held seat. Otherwise use focused tests at the normal entry point alongside the full repository checks.

Before editing render scheduling or readiness, name each changed sampler and the command or test that exercises it in the issue. Record where tests inject status and what they prove about downstream handling. Use evidence from the actual renderer for claims about sampler execution or renderer readiness. Record whether windowed acceptance is required and how it will be obtained or deferred. Keep the commands and renderer settings with the results.

When changing queue admission, deadlines or shutdown in concurrent code, run the focused regression and review the ordering before starting full consumer checks. Trace when a request is admitted, when an observer can see it and which outcome wins when operations race. Resolve findings that could invalidate consumer evidence before running the full checks.

When a producer change affects consumers, commit and check the producer, then push the candidate so Cargo can fetch its revision. Wait for that check to pass before moving consumer pins, including when Cargo can fetch the candidate locally. Before merging the producer, use `move-pin` to pin each affected consumer's worktree to that revision and pass its checks there. Follow [Compatibility checks](../../../docs/agents/consumers.md#compatibility-checks) for consumers that need no adoption PR. After the producer merges, follow [Adopting the merged pin](../../../docs/agents/consumers.md#adopting-the-merged-pin) for consumer PRs.

Call the Skill tool with "tdd" where possible, at pre-agreed seams.

Run typechecking regularly, single test files regularly, and the full test suite once at the end.

Before writing a pull request's Evidence section, confirm each test it cites: it shows as `ok`, not `ignored`, in the log it cites, and it constructs the component the change touches (grep the test for it). A test that passes without reaching the changed path is not evidence for it; name what does reach the path, or write a test that does.

Once done, call the Skill tool with "code-review" to review the work. Reuse earlier review for the parts of the final diff it still covers. When the work spans repositories, review each repository's pull request before it merges, and check its Evidence section against the logs it cites.

Commit your work to the current branch.

Commit locally, then run the check at that commit, so its log names the revision it passed. Publish in a call of its own: read the check's exit status, then push or merge.

To merge this repository's pull request: wait until `gh run list --branch <branch> --event pull_request` shows its run (`gh pr checks --watch` exits at once with "no checks reported" before the run exists), `gh run watch <id> --exit-status`, then `gh pr merge <n> --merge --match-head-commit <sha>`.
