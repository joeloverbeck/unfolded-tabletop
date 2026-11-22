# Consumers

What each game needs from work that changes it: how it renders at the seat, how to verify it, how its pull requests merge, and its limits. Read a game's row before opening an issue or a branch in its repository. Checked against each game's main on 2026-11-16; each repository's own instructions win where they differ.

To find a consumer's use of an API here, search its main checkout with `--exclude-dir=target --exclude-dir=local --exclude-dir=.claude`; those hold build output, evidence snapshots and stale worktrees.

Commit messages and pull requests in a game say `For #N`. Closing an issue is the owner's acceptance, so leave closing keywords out. Each game deletes a branch on GitHub when its pull request merges.

**Changing shared text.** For each sentence a change rewords, search every game's main checkout for its first three words and its last three words (`grep -rnF --include=*.rs`), since a test may read either end. A changed error type changes text too where a game prints its `Debug`: search for `fn main() -> ` returning a `Result`, whose error Rust prints as `Debug`. Name each match in the pull request.

**Changing a game's test.** Search that game's Markdown for the test's name (`grep -rn --include=*.md`): its README and art notes describe what some tests check, numbers included. Update each match with the test.

**Adding a protocol request or status.** A new `unfolded_seat_protocol::Request` or `Status` variant breaks a game that matches either type exhaustively. In each game's main checkout, list the files that name the crate, `git grep -l unfolded_seat_protocol origin/main -- '*.rs'`, and read each `match` on `Request` or `Status` in them for a `_` arm. Record the search in the CHANGELOG entry.

**Changing a public item's signature.** A change to a public item's parameters or return type, such as `Seat::respond`'s (#79), breaks each game that calls it. In each game's main checkout, `git grep -n '\.<item>(' origin/main -- '*.rs'`, and read each match: a game's own method of the same name is no call. Record the callers in the CHANGELOG entry. Each caller's adoption is part of the work, checked against the branch before the change merges.

**Rewriting an adapter.** List the old file's user-facing strings (`git show main:<file> | grep -oE '"[A-Z][^"]+ [^"]+"'`) and account for each one in the pull request: kept, moved, or changed and why.

**Moving a pin.** Run `move-pin <game checkout> <full commit hash>` from this repository (`move-pin --help`): it moves every manifest's pin together, Arkham's two included, and refreshes the lockfiles that the game's `--locked` checks read. Run it again with the same hash to refresh a lockfile that drifted. When a manifest was left behind at an earlier move, it refuses and names each pin; `--align` moves them all.

**A change no game uses.** When a change here, such as removing a deprecated item, needs nothing of any game, check each game without a pull request there: move the pins of a throwaway worktree of its `origin/main` to the pushed branch's head, run its Verify command, and record each game's main revision, command and result in this repository's pull request. Each game adopts the change at its next pin move (#68).

**Native inspection.** The seat presses keys (`key`), drags a mouse button (`drag`) and turns the wheel (`scroll`) through Bevy's own input messages, so a seat test can drive a game's table camera through its real table: pan and Home by keys, orbit by a right drag and zoom by the wheel (#63). Cuba Libre's `native_right_drag_orbits_and_the_wheel_zooms_the_table_camera` drives a right drag, the wheel and Home through its real table. The seat cannot show that a real window delivers that input. That stays a person's inspection at a window: name it the owner's in a game's issue, and record in its pull request whether it was done.

**Held capture and comparison.** Each game's "Held capture" below is run from that game's checkout through `held-start` (`held-start --help`), so the picture sits beside a record of the game's revision, whether its tree is clean, and the SHA-256 of the executable that ran. `compare-captures` reads both records and says when the two captures came from one executable, which cannot show a change:

```sh
cargo run --manifest-path …/unfolded-tabletop/Cargo.toml -p unfolded-seat-client --bin held-start -- \
  <requests> <short out-dir> <case> -- <game command>
cargo run --manifest-path …/unfolded-tabletop/Cargo.toml -p unfolded-dev-tools --bin compare-captures -- \
  <out-dir>/<before case>.table.png <out-dir>/<after case>.table.png [--threshold 3]
```

Record the "before" case with `--clean` from `main`, then the "after" case from the branch. Keep `<out-dir>` short: the socket path inside it holds at most 107 bytes. Each game's held capture below was run on its real inputs on 2026-11-16 (#52).

## Arkham Unfolded

- **Seat host:** keeps Bevy's pipelined rendering. A change to the render world is exercised only here.
- **Verify:** `scripts/check`. It starts its own memory scope and checkout lock through `scripts/dev`, so run it bare: under another `systemd-run --scope` it fails at once with "Unit run-p… was already loaded", since `exec` keeps the process ID that names both scopes. Its root manifest and `tools/Cargo.toml` both pin unfolded-tabletop; move them together. The real-table seat tests are ignored by default. Run them with `ARKHAM_GATHERING_TEST_ASSETS` set to the `asset_root` in `~/.config/arkham-unfolded/gathering.toml` (README, "Evidence limits", lists the commands).
- **Held capture:** requests `--request '{"op":"settle"}' --request '{"op":"capture","path":"table.png"}'`, game command `target/debug/arkham-tabletop --experiment gathering --agent-seat --save <fresh path>`. Two starts of the unchanged table differed in about 200 pixels by at most 3 levels, so compare with `--threshold 3`. Batch form, with `capture` and `quit` requests on standard input, still works.
- **One-shot capture:** the ignored art test captures the pouch through `capture::when_ready`. Name the test after `--`, not before it, and set `ARKHAM_POUCH_RENDER_DIR` to a directory that does not exist yet, which the test creates: `ARKHAM_GATHERING_TEST_ASSETS=<collection> ARKHAM_POUCH_RENDER_DIR=<new dir> scripts/dev cargo test --locked --features tabletop --lib -- --ignored --exact gathering_tabletop::art::tests::real_collection_loads_every_verified_image_font_and_pouch`.
- **Merge:** after CI's "Rust checks" passes, use `scripts/tools merge-reviewed --pr <n> --head <sha> --base <main sha>`. Auto-close is off. Close an accepted issue with `scripts/tools publish comment --issue <n> --body-file <f> --close` (`docs/agents/github-administration.md`).
- **Public text:** issues, comments and PR bodies go through `scripts/tools publish` (`issue`, `comment`, `pr-body`), whose preflight refuses private path forms. Name an evidence file by its case or file name, never by a `local/…` path.
- **Tools in a new worktree:** the first `scripts/tools` call builds `arkham-tools` into `target/agent-tools`, which takes about two minutes. Run it after `scripts/build-slot`.
- **Work-item limit:** none of its own.
- **Pins a whole response:** `gathering_tabletop::seat::tests::nothing_hidden_appears_in_what_the_seat_reports` pins the response's top-level keys. A new response field changes it.
- **Motion captures:** the seat alone advances its presentation clock, so `settle` with a step count stops a card's motion at the same presentation time on every run. Its `gathering_draw_review` example writes saves whose first choice starts the opening draws, a mulligan, an action draw and an encounter draw; take that choice in `held-start`, then alternate `{"op":"settle","steps":13}` and `capture` to compare a motion before and after (Cuba Libre #141).

## Netrunner Unfolded

- **Seat host:** pipelined rendering disabled.
- **Verify:** `target/debug/netrunner-dev record local/verification/check-<rev> -- scripts/check`. Build the recorder with `cargo build --locked -p netrunner-dev` (`docs/agents/development.md`).
- **Held capture:** requests `--request '{"op":"activate","revision":0,"component":"play-corp"}' --request '{"op":"settle"}' --request '{"op":"capture","path":"table.png"}'` (or `play-runner`), game command `target/debug/netrunner-tabletop --assets <turns-assets> --agent-seat`. It opens on the seat choice, and the activation and `settle` reach a table with counters. A start without a save deals a fresh HQ hand, so two starts differ there and nowhere else; `--ignore` the hand's box, which `compare-captures` prints, or compare starts from one save. Batch form still works. The ignored `tabletop::tests::actual_renderer_uploads_required_images_and_refuses_changed_pixels`, with `NETRUNNER_ART_ROOT` set to the turns assets and `NETRUNNER_RENDER_CAPTURE` to a new PNG path, captures the real renderer.
- **Merge:** `gh pr merge --merge --match-head-commit <sha>`, after CI's "Rust checks" passes.
- **Work-item limit:** none of its own.

## Cuba Libre Unfolded

- **Seat host:** pipelined rendering disabled at the seat. Its own admission waits on its render-world pipeline flag.
- **Verify:** `scripts/check`. CI runs it on every pull request, and `main` requires it.
- **Held capture:** run from its checkout, since its art resolves against the working directory: requests `--request '{"op":"settle"}' --request '{"op":"capture","path":"table.png"}'`, game command `target/debug/cuba-libre --cards <cards> --save <fresh path> --agent-seat`. Two starts of the unchanged table were identical.
- **Hover dwell:** resting on a map space starts its status dwell, and the seat refuses input as busy until it settles. `point` and `settle` before a `drag`, `scroll` or `activate` there.
- **Kidnap die skip:** the die holds the table unsettled until it lands. While it is in the air, its adapter's `Game::interrupts` lets through an activation of the tray, `kidnap-die`, and Enter or Space naming `kidnap-die` in `"focused"`. All other input is refused as busy. `native_seat_skips_the_die_by_enter_space_or_a_click_on_the_tray` sends each of the three (#77).
- **Loops:** `native_controls_hover_camera_finish_and_process_restarts` and `native_terror_bells_loop_seamlessly_out_of_step_and_hover_their_space` cross the water and Terror loops with `advance`, two steps a request, and check that each reply took its steps on a settled table (#62).
- **Native capture:** the ignored native regression, `CUBA_TEST_CARDS=<cards> cargo test --locked --features tabletop --test native -- --ignored --test-threads=1`, captures through the seat. Set `CUBA_NATIVE_EVIDENCE` to a new directory.
- **Merge:** `gh pr merge --merge --match-head-commit <sha>`, after CI passes. `main` requires the branch to be up to date: when `main` moves, merge it in, run `scripts/check` again and wait for CI.
- **Record:** each issue or pull request names the revisions of Arkham, mudsong, Netrunner and this repository that it inspected, the source locations, and a disposition (its foundation 3, "Before every feature"). Its pull request also names the constitution rules consulted.
- **Work-item limit:** none of its own.

## mudsong (private)

- **Seat host:** compiles each pipeline before its frame.
- **Held capture:** `held-start` with a `settle` and a `capture` request, followed by its own seat command.
- **Verify and merge:** its README, "Working here". Its merge command verifies the exact head itself, and refuses when its main has moved.
- **Work-item limit:** set by the owner's newest ruling on its limit issue. Each adoption of a change from this repository needs a slot of its own: read that ruling before opening anything there.
- **Card motion:** its cards move on its fixed screen and turn there and in its scene by narrowing, with its own easing and staging, so it does not use `card_motion` (Cuba Libre #141).

Keep this public file to what kind of change each game needs. Leave out mudsong's paths, identifiers and content.
