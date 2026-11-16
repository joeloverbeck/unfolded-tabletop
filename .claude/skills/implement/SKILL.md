---
name: implement
description: "Implement a piece of work based on a spec or set of tickets."
disable-model-invocation: true
---

Implement the work described by the user in the spec or tickets.

If the user passes a ticket reference, fetch it from the issue tracker and state its title before starting. If the reference is ambiguous, ask.

When the work spans repositories, read each game's row in `docs/agents/consumers.md` before opening an issue or a branch there: its seat host, verification, real capture, merge command and work-item limit. Record each real held start with `held-start` (README, "Held-seat client"): the "before" run under its own case, before the change.

Call the Skill tool with "tdd" where possible, at pre-agreed seams.

Run typechecking regularly, single test files regularly, and the full test suite once at the end.

Once done, call the Skill tool with "code-review" to review the work. When the work spans repositories, review each repository's pull request before it merges, and check its Evidence section against the logs it cites.

Commit your work to the current branch.
