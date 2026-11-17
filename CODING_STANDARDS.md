# Coding standards

These are the review rules for this repository. They are judgement calls that no automated check holds; `scripts/check` enforces formatting, strict Clippy, the tests and the searches `docs/testing.md` lists.

## Tests

- **Independent oracles.** A test's expected values come from recorded consumer values, published test vectors, source invariants or hand calculation. Constitution `EVID-4`, revision 2026-11-15.1: "Expectations MAY use source examples, calculations, invariants or justified models; implementation MUST NOT supply its own expected answer." Flag an expectation produced by calling the code under test, or by calling the same library that code uses to compute it.
- **Disclosed host requirements.** `docs/testing.md` promises no graphics adapter, desktop, game assets or sibling checkout. Every other host requirement a test has must be stated there, naming the test it governs: ptrace permission, network on a cold Cargo cache, `git`, `sh`. Confirm each named list by searching every `tests/` directory for the requirement's use (`UnixListener::bind`, `Command::new`, `ptrace`, `/proc`), and flag a test that uses one but is not listed.
- **Isolated process tests.** A test that forks, traces with ptrace or waits on any child gets a test executable of its own. Threads in one executable share open file descriptions and children: a fork can inherit another test's lock, and a wait can reap another test's child.
- **Accurate claims.** Rustdoc, README and the files under `docs/` say exactly what the tests hold. Counts, "every" and "all" claims, and statements about what a consumer keeps must match the code at this revision, not after a consumer's pending pull request. A producer's docs name a consumer's new test or pull request in the retro, after that consumer merges; a brief asks for it there. This covers every behaviour a changed README passage or doc comment describes, not only claims about tests: name the test that reaches it, or trace it through the code, and flag a described behaviour that neither confirms.

## Documentation

- **README is a consumer's front door.** It changes only when what a consumer does to adopt, pin or run something changes. A new mechanism adds one row to its Mechanisms table. Everything else has one home: a mechanism's contract, what it leaves to the consumer and what no test holds go in its rustdoc; what a consumer must do when moving its pin goes in `CHANGELOG.md`; a mechanism's origin and coordinating issues go in `docs/provenance.md`; its evidence boundary goes in `docs/governance.md`; a test's host requirements go in `docs/testing.md`; a tool's full behaviour goes in its `--help`. Flag a README passage that explains how a mechanism works, records history, or lists what tests hold.
- **Changelog entries.** A change a consumer can see (a new or changed public item, a deprecation, a changed wire, number or wording) adds one entry under `CHANGELOG.md`'s "Unreleased", naming its issue, and marks it **Breaking** when it can stop a consumer building or change what its game does.
- **Deprecations.** A deprecation is a removal scheduled by an issue. `scripts/check` refuses a `#[deprecated]` whose note does not say `removed in #N`; flag one whose `#N` is not an open issue for removing that item, or whose issue lacks the condition for removing it.

## Consumers

- **Newly derived facts.** When a mechanism starts deriving a fact that a consumer used to supply itself, or never had, the pull request names each consumer that gains the fact and what in that consumer it now covers. Search each consumer for the entities the fact matches, such as pickable UI nodes for "pointer over UI", and flag a consumer whose behaviour changes unnamed.
- **Changed wording.** When a change alters text a game can display, or the error type behind it, the pull request names each game test that reads that text, from the search in `docs/agents/consumers.md`, "Changing shared text". Flag a claim such as "no game's test reads it" that names no search.

## Bevy

- **Sub-apps at build time.** Code that adds to a sub-app (the render world, its extract) adds it while the app is built: in a plugin's `build`, or in a function a game calls before `finish`. `PipelinedRenderingPlugin::cleanup` moves the `RenderApp` to a thread of its own, so after `finish` and `cleanup` the app has no render sub-app, and code that looks for one silently finds none. Only Arkham's seat keeps pipelined rendering (`docs/agents/consumers.md`), so only its run exercises that case.
