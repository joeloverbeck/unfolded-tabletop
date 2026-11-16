# Consumers

What each game needs from work that changes it: how it renders at the seat, how to verify it, how its pull requests merge, and its limits. Read a game's row before opening an issue or a branch in its repository. Checked against each game's main on 2026-11-16; each repository's own instructions win where they differ.

Commit messages and pull requests in a game say `For #N`. Closing an issue is the owner's acceptance, so leave closing keywords out.

**Held capture and comparison.** Each game's "Held capture" below is run from that game's checkout through `held-start` (`held-start --help`), so the picture sits beside a record of the game's revision and whether its tree is clean:

```sh
cargo run --manifest-path …/unfolded-tabletop/Cargo.toml -p unfolded-seat-client --bin held-start -- \
  <requests> <short out-dir> <case> -- <game command>
cargo run --manifest-path …/unfolded-tabletop/Cargo.toml -p unfolded-dev-tools --bin compare-captures -- \
  <out-dir>/<before case>.table.png <out-dir>/<after case>.table.png [--threshold 3]
```

Record the "before" case from clean `main`, then the "after" case from the branch. Keep `<out-dir>` short: the socket path inside it holds at most 107 bytes. Each game's held capture below was run on its real inputs on 2026-11-16 (#52).

## Arkham Unfolded

- **Seat host:** keeps Bevy's pipelined rendering. A change to the render world is exercised only here.
- **Verify:** `scripts/check`. The real-table seat tests are ignored by default. Run them with `ARKHAM_GATHERING_TEST_ASSETS` set to the `asset_root` in `~/.config/arkham-unfolded/gathering.toml` (README, "Evidence limits", lists the commands).
- **Held capture:** requests `--request '{"op":"settle"}' --request '{"op":"capture","path":"table.png"}'`, game command `target/debug/arkham-tabletop --experiment gathering --agent-seat --save <fresh path>`. Two starts of the unchanged table differed in about 200 pixels by at most 3 levels, so compare with `--threshold 3`. Batch form, with `capture` and `quit` requests on standard input, still works.
- **Merge:** after CI's "Rust checks" passes, use `scripts/tools merge-reviewed --pr <n> --head <sha> --base <main sha>`. Auto-close is off. Close an accepted issue with `scripts/tools publish comment --issue <n> --body-file <f> --close` (`docs/agents/github-administration.md`).
- **Public text:** issues, comments and PR bodies go through `scripts/tools publish` (`issue`, `comment`, `pr-body`), whose preflight refuses private path forms. Name an evidence file by its case or file name, never by a `local/…` path.
- **Work-item limit:** none of its own.
- **Pins a whole response:** `gathering_tabletop::seat::tests::nothing_hidden_appears_in_what_the_seat_reports` pins the response's top-level keys. A new response field changes it.

## Netrunner Unfolded

- **Seat host:** pipelined rendering disabled.
- **Verify:** `target/debug/netrunner-dev record local/verification/check-<rev> -- scripts/check`. Build the recorder with `cargo build --locked -p netrunner-dev` (`docs/agents/development.md`).
- **Held capture:** requests `--request '{"op":"activate","revision":0,"component":"play-corp"}' --request '{"op":"settle"}' --request '{"op":"capture","path":"table.png"}'` (or `play-runner`), game command `target/debug/netrunner-tabletop --assets <turns-assets> --agent-seat`. It opens on the seat choice, and the activation and `settle` reach a table with counters. A start without a save deals a fresh HQ hand, so two starts differ there and nowhere else; `--ignore` the hand's box, which `compare-captures` prints, or compare starts from one save. Batch form still works. The ignored `tabletop::tests::actual_renderer_uploads_required_images_and_refuses_changed_pixels`, with `NETRUNNER_ART_ROOT` set to the turns assets and `NETRUNNER_RENDER_CAPTURE` to a new PNG path, captures the real renderer.
- **Merge:** `gh pr merge --merge --match-head-commit <sha>`, after CI's "Rust checks" passes.
- **Work-item limit:** none of its own.

## Cuba Libre Unfolded

- **Seat host:** pipelined rendering disabled at the seat. Its own admission waits on its render-world pipeline flag.
- **Verify:** the five commands in README, "Checks and development seat". No CI runs them.
- **Held capture:** run from its checkout, since its art resolves against the working directory: requests `--request '{"op":"settle"}' --request '{"op":"capture","path":"table.png"}'`, game command `target/debug/cuba-libre --cards <cards> --save <fresh path> --agent-seat`. Two starts of the unchanged table were identical.
- **Native capture:** the ignored native regression, `CUBA_TEST_CARDS=<cards> cargo test --locked --features tabletop --test native -- --ignored --test-threads=1`, captures through the seat. Set `CUBA_NATIVE_EVIDENCE` to a new directory.
- **Merge:** `gh pr merge --merge --match-head-commit <sha>`. Merged branches are not deleted on GitHub; delete the remote branch once its head is in `main`.
- **Work-item limit:** none of its own.

## mudsong (private)

- **Seat host:** compiles each pipeline before its frame.
- **Held capture:** `held-start` with a `settle` and a `capture` request, followed by its own seat command.
- **Verify and merge:** its README, "Working here". Its merge command verifies the exact head itself, and refuses when its main has moved.
- **Work-item limit:** set by the owner's newest ruling on its limit issue. Each adoption of a change from this repository needs a slot of its own: read that ruling before opening anything there.

Keep this public file to what kind of change each game needs. Leave out mudsong's paths, identifiers and content.
