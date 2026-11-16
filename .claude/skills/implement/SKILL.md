---
name: implement
description: "Implement a piece of work based on a spec or set of tickets."
disable-model-invocation: true
---

Implement the work described by the user in the spec or tickets.

If the user passes a ticket reference, fetch it from the issue tracker and state its title before starting. If the reference is ambiguous, ask.

Note the start time from `date` before the first substantive step. The ticket's appetite is measured from it, and the work record states it.

When the work spans repositories, read each game's row in `docs/agents/consumers.md` before opening an issue or a branch there: its seat host, verification, real capture, merge command and work-item limit. Open the game's own issue before its first commit, so that every commit there can say `For #N`. Record each real held start with `held-start` (`held-start --help`): the "before" run under its own case, from the game's clean `main`, before you edit its checkout.

Call the Skill tool with "tdd" where possible, at pre-agreed seams.

Run typechecking regularly, single test files regularly, and the full test suite once at the end.

Before writing a pull request's Evidence section, confirm each test it cites: it shows as `ok`, not `ignored`, in the log it cites, and it constructs the component the change touches (grep the test for it). A test that passes without reaching the changed path is not evidence for it; name what does reach the path, or write a test that does.

Once done, call the Skill tool with "code-review" to review the work. When the work spans repositories, review each repository's pull request before it merges, and check its Evidence section against the logs it cites.

Commit your work to the current branch.

To merge this repository's pull request: wait until `gh run list --branch <branch> --event pull_request` shows its run (`gh pr checks --watch` exits at once with "no checks reported" before the run exists), `gh run watch <id> --exit-status`, then `gh pr merge <n> --merge --match-head-commit <sha>`.
