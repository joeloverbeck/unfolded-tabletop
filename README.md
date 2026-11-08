# unfolded-tabletop

Concrete Rust presentation mechanisms extracted from Arkham Unfolded for its
Gathering table and Netrunner Unfolded's first Brân run. This is a bounded
two-consumer candidate, not a general tabletop framework.

The library owns verified file reads, bounded PNG crops, atomic prepared-directory
publication, rounded card geometry, solid textured-token assembly, and asset-load
state reporting. Consumers own rules, observations, layout, materials, resource
meanings, required artwork, renderer admission policy and native-operation policy.
Enable `render` for the Bevy 0.20.0 modules; preparation works without Bevy.

## Development checks

Run `scripts/check` from a clean clone before publication. It checks full-workspace
formatting first, then strict Clippy and tests with both default and `render`
features. Rustup selects the compiler/components in `rust-toolchain.toml`; the
committed lockfile fixes this repository's verification dependencies. CI runs the
same entry point on Ubuntu without game assets or a desktop. A focused command is
only partial evidence. These checks do not replace either application's asset,
GPU or native acceptance checks.

The workspace's local Cargo patch exercises its owned `vendor/wgpu-hal` copy.
Consumers still select their own immutable root patch and dependency lockfile;
this verification setup does not change their pins.

`vendor/wgpu-hal` retains Arkham's narrowly scoped NVIDIA Vulkan staging workaround
and its upstream licensing. Applications must configure their own Cargo root
`[patch.crates-io]`; a dependency's patch does not propagate. Pin both the library
and patched package to the same immutable repository revision.

Origin: `joeloverbeck/arkham-unfolded` at
`83bd1264b7962d844cfa2417d7f50be642575c9a`, principally `src/assets.rs`,
`src/tabletop_pieces/card_shape.rs`, `src/tabletop_counters.rs`,
`src/tabletop_health.rs` and `vendor/wgpu-hal`. Existing consumer tests stay with
their respective adapters. There are no bundled game assets or live sibling
checkout dependencies in the intended published contract.

The original coordinated extraction and consumer acceptance belong to
[Netrunner issue #1](https://github.com/joeloverbeck/netrunner-unfolded/issues/1).
Each consumer retains independent acceptance authority. Reverting both consumer
dependency commits returns them to their prior producer; Arkham's prior commit
contains its original in-repository implementations and GPU patch.

The accepted, task-scoped `EVID-6` specialization permits extraction on these two
actual consumers instead of requiring three contexts. It does not establish
broad reuse fitness. Production adoption requires both consumer checks and the
bounded native inspection recorded in the issue.
