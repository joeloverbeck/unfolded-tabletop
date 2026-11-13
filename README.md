# unfolded-tabletop

Concrete Rust mechanisms extracted from Arkham Unfolded for named consumers:
presentation mechanisms for its Gathering table and Netrunner Unfolded's first
Brân run, and seeded randomness, screen capture and the agent seat for Arkham and
`mudsong`. This is a bounded candidate with real consumers, not a general
tabletop framework.

The library owns verified file reads, bounded PNG crops, atomic prepared-directory
publication, rounded card geometry, solid textured-token assembly, asset-load
state reporting, a seeded generator with its index and shuffle, screen capture
with its windowless surface, and the agent seat's protocol, input synthesis and
held session. Consumers own rules, observations, layout, materials, resource
meanings, required artwork, renderer admission policy, native-operation policy,
what their randomness decides and how its state is saved, when a capture is
taken and what a failed one means, and what an agent observes and names at their
table. Enable `render` for the Bevy 0.20.0 modules, and `seat`, which implies
it, for the agent seat; preparation and `random` work without Bevy.

## Agent seat

`seat`, behind the feature of the same name, holds the game-neutral half of
Arkham's agent seat. Origin: `joeloverbeck/arkham-unfolded` at
`0b8e6fb3425174110446d6956cfa9bc38008e279`, `src/agent_seat.rs` and
`src/agent_seat/socket.rs`. Arkham's migration, in its own pull request, sits
its Gathering table at it. `mudsong`, a private repository, pins it for its
application. No game's domain enters the module, and a consumer that does not
enable the feature compiles nothing of it.

An agent sits at a game's real table with no window, through one JSON request
per line and one JSON response per line: on standard input and output, or held
across shell commands on a local socket.

- `Seat` alone steps the table, in fixed steps of one sixtieth of a second. It
  writes pointer and key input where Bevy's window-event conversion writes it,
  settles the table, captures its image through `capture`, and answers
  `observe`, `activate`, `key`, `settle`, `capture` and `quit`.
- The guards are the protocol's, so both forms have them. An activation or a key
  acts only on the revision it names. `Enter` and `Space` press only the control
  the request names. A connection stops acting after its first refusal or missed
  activation.
- `socket` holds one session on a Unix-domain socket that only its owner can
  reach: the launcher that starts the seat as a process of its own, the
  listener, and the session's idle end.

Each game implements `Game` in its own adapter: its observation and revision,
the names it resolves to screen positions and back, what has keyboard focus,
whether its table is settled, its clock, its admission to play, and whether its
render pipelines are ready. These are each game's too: the words of an admission
that ran out of time, the plugins of its windowless host, its launch options,
the environment variable that marks a launched seat, and how long a held
session may idle.

`tests/seat.rs` pins the protocol's responses and refusals as Arkham worded
them, on a table that belongs to no game and without a graphics adapter: its
picking and its renderer are stand-ins. Arkham's own tests hold the same
protocol on its real table. A real render through the seat is each consumer's
own check.

## Screen capture

`capture` holds the part of Arkham's screen capture that both consumers use.
Origin: `joeloverbeck/arkham-unfolded` at
`617904319a7b260e496af61cc5390458c2b1ff6b`, `write_image` of
`src/tabletop_capture.rs` and the windowless surface and capture loop of
`src/agent_seat.rs`. Arkham's migration, in its own pull request, writes its
native captures and its agent seat's pictures through it. `mudsong`, a private
repository, pins it for its application. No game's domain enters the module.

- `write_image` writes a captured picture as 8-bit RGB: alpha is dropped, and a
  floating-point picture is converted from linear light.
- `no_window`, `install` and `Surface` give a host with no operating-system
  window a focused logical primary window and an image for one camera to render,
  as the default UI camera.
- `image_to_file` asks Bevy for a capture of an image and returns once the file
  is written. The caller supplies the step that runs one update of its host and
  the deadline; the module waits two milliseconds between steps.

What is not here stays with its one consumer. Arkham keeps its screenshot
observers, their timed diagnostics and its exits, and frame pacing, on which no
capture here depends. The module does not enable Bevy's window event loop, so
the application that enabled it leaves it out of a windowless host itself. What
must be ready before a capture, the clock the step advances, the time allowed
and the wording of a failure belong to each consumer.

`tests/capture.rs` pins the pixels, the surface and the capture as Arkham had
them before the move, without a graphics adapter. One behaviour differs on
purpose: once a capture has returned, a picture that arrives for it is not
written. In Arkham a capture given up as incomplete still wrote its file when
the picture came later, and a picture handed back twice was written twice,
which Bevy does not do. A real render is each consumer's own check.

## Seeded randomness

`random` holds Arkham's SplitMix64 generator, its rejection-sampled index and its
shuffle. Origin: `joeloverbeck/arkham-unfolded` at
`4e7a014541ef46d8dc8185e2759a139480ea60e3`, `src/gathering.rs` and
`src/gathering/runtime.rs`. Arkham draws its setup and play randomness from it.
`mudsong`, a private repository, pins it for its rules library. No game's domain
enters the module.

A generator's whole state is one `u64`, and the numbers a state produces are a
contract: a consumer's saved game stores that state and continues only while it
keeps producing the same numbers. `tests/random.rs` pins numbers recorded from
Arkham before the move, so changing them is a breaking change for every consumer.

Each consumer's pull request records its own migration, checks and acceptance.
Reverting a consumer's dependency commit returns it to its prior producer;
Arkham's prior commit contains its original generator.

## Development checks

Run `scripts/check` from a clean clone before publication. It checks full-workspace
formatting first, then strict Clippy and tests with the default, `render` and
`seat` features. Rustup selects the compiler/components in `rust-toolchain.toml`; the
committed lockfile fixes this repository's verification dependencies. CI runs the
same entry point on Ubuntu without game assets or a desktop. A focused command is
only partial evidence. These checks do not replace any consumer's own checks:
asset, GPU, native acceptance or saved-game continuity.

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

## Third-consumer waiver

The specialization above covers code used by both Arkham and Netrunner, and for
any surface it ends when a third consumer appears. On 2026-11-11 the owner
accepted a further scoped `EVID-6` waiver under `ADOPT-2`, which lets `mudsong`
be that consumer. It does not edit the constitution. Its authoritative text is
`docs/foundation/04-engineering-and-shared-code.md` in that private repository;
this is its record here, carried by the first extraction made under it, the
seeded randomness above.

| Field | Accepted boundary |
| --- | --- |
| Exact rule | `EVID-6`: three materially different real cases and an independent holdout before cross-product adoption |
| Scope | Concrete mechanisms, not specific to any game, owned in `unfolded-tabletop` and really used by `mudsong` and by at least one of `arkham-unfolded` and `netrunner-unfolded` |
| Reason | These three are the only real products. Duplicating mechanisms or inventing nominal consumers would defeat the intended reuse |
| Accepted risk | Two consumers can miss an incompatible assumption; a later case may force an interface change or a reversal |
| Required evidence | A real path in each consuming application, each repository's own checks, game policy preserved in adapters, and a coordinated rollback |
| Appetite | Two active hours per extraction attempt, then reassess. No extension into a framework project |
| Ends | For a given mechanism: a consumer outside these three games, publication to crates.io or a promise of general reuse, or loss of its second real consumer. There is no milestone or date expiry. After that, further sharing needs the normal evidence or a new waiver |

The waiver establishes neither general library fitness nor acceptance of an
untested migration.
