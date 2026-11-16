# unfolded-tabletop

Concrete Rust mechanisms shared by the games that consume them: verified asset
reads, card and token geometry, labels, a table camera, screen capture, seeded
randomness, recoverable file replacement, and an agent seat that lets an agent
play a game's real table with no window. Most of them were extracted from Arkham
Unfolded.

This is a bounded candidate with real consumers, not a general tabletop framework.
It makes no promise of general reuse or API stability, and it is not published to
crates.io. Its current consumers are Arkham Unfolded, Netrunner Unfolded, Cuba
Libre Unfolded and `mudsong` (a private repository). `GLOSSARY.md` defines the
terms used here.

## Mechanisms

| Package | Mechanism | Bevy |
| --- | --- | --- |
| `unfolded-random` | A seeded SplitMix64 generator, its index and shuffle, and a fresh seed from `/dev/urandom` | never |
| `unfolded-hover` | When a pointer-driven reading view opens, switches and closes | never |
| `unfolded-record-replacement` | A game's save record, on a locked, staged replacement that keeps the previous record | never |
| `unfolded-assets` | Bounded and SHA-256-verified reads, bounded PNG crops, and publication of a prepared directory | never |
| `unfolded-seat-protocol` | The agent seat's wire vocabulary: requests, statuses and the `refused` start line | never |
| `unfolded-seat-client` | A client for a held agent seat, on Unix, and the `held-start` tool | never |
| `unfolded-dev-tools` | Bounded context readers, a recorder for checks, and the `compare-captures` tool | never |
| `unfolded-tabletop::camera`, feature `render` | Table camera orbit, pan, wheel, scene-drag capture and follow | Bevy 0.20.0 |
| `unfolded-tabletop::capture`, feature `render` | Writing a captured picture, and a windowless surface whose image can be captured | Bevy 0.20.0 |
| `unfolded-tabletop::card`, feature `render` | Rounded physical-card meshes: face, body and outline | Bevy 0.20.0 |
| `unfolded-tabletop::token`, feature `render` | Solid token bodies with a textured face | Bevy 0.20.0 |
| `unfolded-tabletop::loading`, feature `render` | Whether every required asset and its dependencies loaded | Bevy 0.20.0 |
| `unfolded-tabletop::printed`, feature `render` | Text rows printed as ink on flat planes in the world | Bevy 0.20.0 |
| `unfolded-tabletop::label_anchor`, feature `render` | Keeping a UI node of your own over a world point or entity | Bevy 0.20.0 |
| `unfolded-tabletop::count_label`, feature `render` | Deprecated numeric labels; use `label_anchor` | Bevy 0.20.0 |
| `unfolded-tabletop::seat`, feature `seat` | The agent seat, in batch form or held on a local socket; enables `render` | Bevy 0.20.0 |

"Never" holds under every feature set, and `scripts/check` fails if Bevy appears
below any of those packages. `unfolded-random` has no dependencies at all.

Each mechanism's contract is in its rustdoc: what it does, what it refuses, what
it leaves to you, and what no test holds. Run `cargo doc --open` in your own
workspace, so that you read the revision you pin.

## Using it

Depend on each package you use, and patch `wgpu-hal`, all at one immutable `rev`:

```toml
[dependencies]
unfolded-random = { git = "https://github.com/joeloverbeck/unfolded-tabletop", rev = "<sha>" }
unfolded-tabletop = { git = "https://github.com/joeloverbeck/unfolded-tabletop", rev = "<sha>", features = ["seat"] }

[patch.crates-io]
wgpu-hal = { git = "https://github.com/joeloverbeck/unfolded-tabletop", rev = "<sha>" }
```

- **The patch goes in your workspace's root manifest.** A Git dependency's patch
  does not propagate. `vendor/wgpu-hal` keeps Arkham's narrowly scoped NVIDIA
  Vulkan staging workaround; `vendor/wgpu-hal/ARKHAM_PATCH.md` records it.
- **One `rev` for everything**, including `unfolded-dev-tools` if you pin it. Every
  package here changes revision together; none has a lifecycle of its own.
- **Keep Bevy out of crates without an interface.** A crate with no interface in it
  depends only on the Bevy-free packages it uses, so Cargo can never link Bevy into
  it. The crate that holds your interface also depends on `unfolded-tabletop`, which
  brings none of the other packages except, under `seat`, `unfolded-seat-protocol`.
- **Name each mechanism by its own package**: `unfolded_random::SeededRandom`, not
  a path through `unfolded_tabletop`.
- **Moving your pin:** read `CHANGELOG.md` for what changed between the two
  revisions, and run your own checks. Rolling back means reverting your adapter and
  your pin together (`docs/provenance.md`).

## What stays in your game

This repository holds mechanisms only. Your game keeps its rules, observations,
layout, materials, resource meanings, required artwork, renderer admission and
native-operation policy; what its randomness decides; what it saves, what identifies
a compatible save and the words for each refused one;
when a capture is taken and what a failed one means; and what an agent observes
and names at its table. You connect each mechanism to that policy in an adapter of
your own, and your checks, not this repository's, show that your game still works:
assets, GPU, native acceptance, a real render through the seat, and saved-game
continuity.

## Tools

Both tools run from a checkout of this repository, and a game need not pin them.
Run either with `--help` for its full description.

- **`held-start`** records one held start of a game's agent seat as pull-request
  evidence, with the game's revision beside any captures. The held seat is Linux
  only. Run it from the game's checkout:

  ```sh
  cargo run --manifest-path …/unfolded-tabletop/Cargo.toml -p unfolded-seat-client --bin held-start -- \
    --request '{"op":"settle"}' --request '{"op":"capture","path":"table.png"}' \
    <out-dir> <case> -- <game executable> <its seat options>
  ```

- **`compare-captures`** compares two captures of the same table:

  ```sh
  cargo run --manifest-path …/unfolded-tabletop/Cargo.toml -p unfolded-dev-tools --bin compare-captures -- \
    <before.png> <after.png> [--ignore x,y,width,height] [--threshold <level>] [--limit <count>]
  ```

## Working on this repository

Work is tracked in this repository's GitHub issues. Run `scripts/check` before
publishing.

- `CLAUDE.md`: the repository's constitution and the rules it adopts.
- `CODING_STANDARDS.md`: the review rules, including what belongs in this README.
- `docs/testing.md`: what `scripts/check` runs, and what the tests need from a host.
- `docs/governance.md`: the evidence boundaries under which each mechanism is shared.
- `docs/provenance.md`: where each mechanism came from, and how to roll one back.
- `docs/agents/consumers.md`: what each consumer needs from work that changes it.
