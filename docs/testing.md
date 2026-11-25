# Testing this repository

Run `scripts/check` from a clean clone before publication. CI runs the same entry point on Ubuntu, without game assets or a desktop. A focused command is only partial evidence.

`scripts/check` runs, in order: formatting; a search that fails if code in `src` or `seat-client/src` spells a seat status as a string literal (tests keep the spelling as the wire's oracle); a search that fails if a `#[deprecated]` outside `vendor/` has no note saying `removed in #N`; strict Clippy on the workspace and on `unfolded-tabletop` with `render` and with `seat`; a check that no workspace member other than `unfolded-tabletop` reaches Bevy and that `unfolded-random` has no dependency, with each Bevy-free package linted alone, since a workspace build can supply a dependency feature that a consumer depending on that package alone would lack; rustdoc for the workspace and for `unfolded-tabletop` with `seat`, with warnings denied, so every doc link resolves; the workspace's tests and `unfolded-tabletop`'s with `render` and with `seat`; and the `vulkan::upload_memory` regression tests of `vendor/wgpu-hal`, from its own manifest and tracked lockfile.

These checks do not replace any consumer's own: assets, GPU, native acceptance, a real render through the seat, and saved-game continuity stay with each consumer (`docs/agents/consumers.md`).

## What the host needs

The tests need no graphics adapter, desktop, game assets or sibling checkout. Rustup selects the compiler and components in `rust-toolchain.toml`, and the committed lockfile fixes the dependencies. Beyond that:

- **Network** on a cold Cargo cache, to fetch the locked dependencies.
- **Bounded builds.** `.cargo/config.toml` sets four jobs and no debug info for every Cargo command run here, since unbounded parallel links of the Bevy test executables can exhaust a session's memory. The environment may override them; CI sets two jobs. Where several sessions share a machine, `scripts/build-slot` waits until no other build runs (its header says how it decides).
- **ptrace.** `record-replacement/tests/record_replacement_trace.rs` traces its own child's renames. Ordinary Linux hosts and GitHub's Ubuntu runners allow it; a container that forbids ptrace fails it.
- **Directory permissions that hold.** `tests/seat.rs` claims a socket in a directory its owner may write and search but not read, and expects opening that directory to be refused. Run the tests as an ordinary user: root, or a process with `CAP_DAC_OVERRIDE` or `CAP_DAC_READ_SEARCH`, as some containers grant, fails that case. GitHub's Ubuntu runners run as an ordinary user.
- **Unix-domain sockets** under the system's temporary directory, whose path must leave each socket within 107 bytes: `tests/seat.rs`, `tests/seat_start.rs`, `tests/agent_transport.rs`, `agent-players/tests/client.rs`, `agent-players/tests/waking.rs`, `seat-client/tests/seat_client.rs`, `seat-client/tests/hosting.rs` and `seat-client/tests/held_start.rs` listen on them. `waking.rs` and `held_start.rs` also start child processes, and `waking.rs` runs its own from a copy in that directory, which must let it run programs. `agent-players/tests/folder.rs` and `waking.rs` also need that directory's path to hold only ASCII letters, digits, `/`, `.`, `_` and `-`.
- **`git`**: `dev-tools/tests/cli` needs it to build a repository for the recorder's source fingerprints. `seat-client/tests/held_start.rs` needs it to build a repository for `--clean`, and `dev-tools/tests/move_pin.rs` to build the checkouts whose pins it moves.
- **Linux's `/proc`, `sh` and `sleep`**: `dev-tools/tests/process.rs` starts its children with `sh` and `sleep`, and the process support it tests reads `/proc`.
- **`sha256sum`**: `seat-client/tests/held_start.rs` checks the executable's digest in a record against coreutils' own.

`tests/seat.rs` takes about 30 seconds: one case waits out the capture limit for pipelines that are never ready.

A test that forks, traces or waits on a child has a test executable of its own (`CODING_STANDARDS.md`, "Isolated process tests"); several are their own harness, so they print their own `ok` lines.
