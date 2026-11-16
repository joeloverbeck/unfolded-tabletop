# Coding standards

These are the review rules for this repository. They are judgement calls that no automated check holds; `scripts/check` enforces formatting, strict Clippy and the tests.

## Tests

- **Independent oracles.** A test's expected values come from recorded consumer values, published test vectors, source invariants or hand calculation. Constitution `EVID-4`, revision 2026-11-15.1: "Expectations MAY use source examples, calculations, invariants or justified models; implementation MUST NOT supply its own expected answer." Flag an expectation produced by calling the code under test, or by calling the same library that code uses to compute it.
- **Disclosed host requirements.** README's "Development checks" promises no graphics adapter, desktop, game assets or sibling checkout. Every other host requirement a test has must be stated in README next to the test it governs: ptrace permission, network on a cold Cargo cache, `git`, `sh`.
- **Isolated process tests.** A test that forks, traces with ptrace or waits on any child gets a test executable of its own. Threads in one executable share open file descriptions and children: a fork can inherit another test's lock, and a wait can reap another test's child.
- **Accurate claims.** README and module documentation say exactly what the tests hold. Counts, "every" and "all" claims, and statements about what a consumer keeps must match the code at this revision, not after a consumer's pending pull request.

## Bevy

- **Sub-apps at build time.** Code that adds to a sub-app (the render world, its extract) adds it while the app is built: in a plugin's `build`, or in a function a game calls before `finish`. `PipelinedRenderingPlugin::cleanup` moves the `RenderApp` to a thread of its own, so after `finish` and `cleanup` the app has no render sub-app, and code that looks for one silently finds none. Only Arkham's seat keeps pipelined rendering (`docs/agents/consumers.md`), so only its run exercises that case.
