# unfolded-tabletop

Concrete Rust mechanisms, most of them extracted from Arkham Unfolded, for four
named games: Arkham Unfolded, Netrunner Unfolded, Cuba Libre Unfolded and
`mudsong`. This is a bounded candidate with real consumers, not a general
tabletop framework.

The library owns verified file reads, bounded PNG crops, atomic prepared-directory
publication, rounded card geometry, solid textured-token assembly, asset-load
state reporting, a seeded generator with its index and shuffle, screen capture
with its windowless surface, the agent seat's protocol, input synthesis and
held session, recoverable file replacement, and table camera motion and input
capture. Consumers own rules, observations, layout, materials, resource
meanings, required artwork, renderer admission policy, native-operation policy,
what their randomness decides and how its state is saved, when a capture is
taken and what a failed one means, and what an agent observes and names at their
table.

## Packages

Every package below lives in this one repository and changes revision with it.
None has a lifecycle of its own.

| Package | Mechanism | Bevy | Direct dependencies |
| --- | --- | --- | --- |
| `unfolded-random` | seeded randomness | never | none |
| `unfolded-hover` | hover opening lifecycle | never | none |
| `unfolded-record-replacement` | recoverable file replacement | never | `tempfile` |
| `unfolded-assets` | verified reads, PNG crops, prepared-directory publication | never | `image` (PNG), `rustix`, `serde_json`, `sha2` |
| `unfolded-seat-protocol` | the agent seat's wire vocabulary: requests, statuses, the `refused` start line | never | `serde`, `serde_json` |
| `unfolded-seat-client` | held-seat client and its `held-start` recorder, on Unix | never | `anyhow`, `serde_json`, `unfolded-seat-protocol` |
| `unfolded-dev-tools` | development readers and recorded checks | never | see its manifest |
| `unfolded-tabletop` | `camera`, `capture`, `card`, `count_label`, `label_anchor`, `loading`, `printed`, `token` behind `render`; `seat` behind `seat` | Bevy 0.20.0 under `render` | none without features; Bevy and `image` with `render`; `serde_json`, `tempfile`, `rustix` and `unfolded-seat-protocol` with `seat` |

"Never" holds under every feature set: `scripts/check` fails if `cargo tree`
finds a `bevy` package below any of them, and fails if `unfolded-random` brings
any crate at all. A game crate with no interface in it should depend only on the
Bevy-free packages it uses, so Cargo can never link Bevy into it, whatever else
its workspace builds. The crate that holds the game's interface also depends
on `unfolded-tabletop`, which depends on none of them except, under `seat`,
`unfolded-seat-protocol`. Arkham, Netrunner
and Cuba Libre keep their interface behind a `tabletop` feature, which enables
`unfolded-tabletop/seat`.

Pin every package a game takes from this repository, its root
`[patch.crates-io]` entry for `wgpu-hal` and `unfolded-dev-tools` at one
immutable `rev`. A Git dependency's patch does not propagate, so the game's own
root must name it.

A game names each mechanism by its own package: `unfolded_random::SeededRandom`,
not `unfolded_tabletop::random::SeededRandom`. The old paths were re-exports
kept while the games moved (issue #31). The last revision that has them is
`138e050`.

## Printed mat labels

`printed`, behind `render`, extracts the transparent row atlas and flat label
planes from Arkham `d0212e5eecf21c358472a62c93c8992607497263`,
`src/gathering_tabletop/printed.rs`. [Issue #25](https://github.com/joeloverbeck/unfolded-tabletop/issues/25)
coordinates [Arkham #247](https://github.com/joeloverbeck/arkham-unfolded/issues/247)
and the printed labels selected in [Netrunner #26](https://github.com/joeloverbeck/netrunner-unfolded/issues/26).
The existing two-consumer EVID-6 exception applies; this is not a typography
framework. The inspected third consumer has no corresponding mat-label atlas.

Keep an `Atlas` across scene rebuilds. `render` takes caller-owned text bundles
and pixel dimensions, creates a transparent camera and UI root, and returns a
`Rendered` row set. `spawn` maps one row onto an unlit blended XZ plane at the
caller’s exact transform. The caller supplies positive dimensions and UI scale,
a nonempty row set, valid row indices and positive finite plane heights; invalid
inputs panic. Images are reused at the same dimensions and replaced when the
row count or dimensions change. Row geometry compensates for UI scale; callers
divide their own font sizes by the same scale. Cameras retain Bevy’s defaults
apart from their supplied order and transparent clear color.

Consumers own content, fonts, colors, layout, orientation, names, actions and
updates. They also despawn each returned camera, root and plane on rebuild;
the root owns its descendants. No plugin or camera-facing behavior is added.
The shared tests exercise this interface without a graphics adapter. Each game
retains its own checks and rendered inspection, including live text changes.
Revert consumer adapters and immutable pins together before removing the shared
addition; Arkham's previous revision retains the original implementation. No
saved state or artwork changes.

## Hover opening lifecycle

`hover::Hover` owns the transient opening lifecycle extracted from mudsong at
`b6c05ac561bf52f4dabb7d77fa313889a768b628`, `crates/app/src/pointing.rs::grow`.
[Issue #18](https://github.com/joeloverbeck/unfolded-tabletop/issues/18) coordinates
its mudsong migration and Cuba Libre contribution-view consumer under the scoped
EVID-6 extension recorded there. This candidate is not a general tooltip system.

Call `update` with the picked identity, eligibility, elapsed time and local dwell
duration. A new identity starts at zero elapsed dwell; the first eligible view
waits, an already open view switches immediately, and absent/ineligible targets
dismiss it. Eligibility changes on the same identity retain its elapsed dwell.
`visible` identifies what to show and `waiting` reports pending dwell. Consumers
own picking, content, actions, geometry, timing, dimming and animation readiness.
Its package cannot depend on Bevy. Reset the lifecycle with its `Default` on a local
view reset. It changes no persisted records. Revert consumer adapters and pins
together before removing this producer module; each consumer still needs its
own actual interaction and image checks.

## World-anchored labels

`label_anchor`, behind `render`, keeps a UI node a caller owns over a world
anchor. [Issue #35](https://github.com/joeloverbeck/unfolded-tabletop/issues/35)
generalized it from the token count labels below and Netrunner's board labels at
`df604944d3c7ad96b786df5edf6c0959e79733f2`, `src/tabletop/turns.rs`.

Install `LabelAnchorPlugin`, spawn the node with `PositionType::Absolute`, and add
a `LabelAnchor`: `LabelAnchor::entity(token, offset)` or `LabelAnchor::point(at)`.
Each frame, in `PostUpdate` after transform propagation and the camera's update and
before UI layout, it writes the node's `left`, `top` and `Visibility`, and nothing
else:

- An entity anchor reads the entity's own `Transform`, as for a root token: the
  offset is scaled by its scale but not rotated, and a parent is not followed. The
  node is removed once the entity has no `Transform`.
- A point anchor is a fixed world point and is never removed.
- The node's top edge is centred on the projected point by its measured width, or,
  with `left_edge()`, starts there. A margin the caller sets still shifts it.
- The node is hidden when projection fails and, with `within_viewport()`, while
  the point falls outside the camera's logical viewport rectangle.

Tracking requires one `Camera3d`; with none, or several, it changes nothing. It
makes no promise about several views or offsets that rotate with a parent. The
node's text, font, colour, padding, accessibility name, pickability, stacking and
lifetime are the caller's, as are when a count updates and whether a zero shows.

## Token count labels

`count_label`, behind `render`, holds numeric label creation from Arkham
`d7a6f2ea46fbf97719cc352ee8b94270857b9bcd`,
`src/tabletop_counters.rs`. [Issue #11](https://github.com/joeloverbeck/unfolded-tabletop/issues/11)
coordinated Arkham's counter migration with Netrunner's compact credit
and click totals under [Netrunner #12](https://github.com/joeloverbeck/netrunner-unfolded/issues/12).
Since issue #35 its `spawn` adds a `LabelAnchor` to the non-pickable, centred
digit string it makes, and `CountLabelPlugin` installs `LabelAnchorPlugin` unless
a game has already installed it; a game that installs both installs
`LabelAnchorPlugin` first. They remain so that a game can return to them by
reverting its adapter and pin together; new code spawns its own node with a
`LabelAnchor`. Mudsong has no corresponding counter-label consumer to migrate.

Each consumer retains its own checks and rendered inspection. Revert consumer
adapters and dependency pins before withdrawing either producer addition; Arkham's
prior revision contains the original implementation. No saved state changes.

## Recoverable file replacement

`record_replacement` owns Arkham's bounded reads, persistent sidecar lock,
staged and synced replacement, previous-record retention, exclusive creation,
and restart archiving. Origin: `joeloverbeck/arkham-unfolded` at
`89be726a81f73f3d5828683a72ee16eb3de2f66a`, `src/record_replacement.rs`.
The coordinated extraction belongs to
[Netrunner issue #9](https://github.com/joeloverbeck/netrunner-unfolded/issues/9).
Arkham's adapter keeps its actual save, reload and restart paths. Netrunner's
saved state (`src/saved_state.rs`) and Cuba Libre's save (`src/save.rs`) replace
their records through it. Mudsong has no saved-game consumer and needs no migration.

Consumers supply their record-size bound, error conversion, replacement admission
and recovery validation. Schemas, checksums, content identities, freshness and
replay policy stay in each game. Admission and recovery validation run under the
same lock as installation; the lock's inode is never truncated or removed.
An unchanged decision writes no record bytes and does not validate recovery.

The lock is explicitly released when replacement returns or unwinds, so an
unrelated fork waiting to exec cannot keep it held through its inherited file
description. An independent writer is still refused while admission, recovery
validation or installation is active. Unlock errors, like close errors, are
best-effort cleanup: they do not mask the operation's error, report an installed
record as failed, or panic during unwinding. Closing the file remains the fallback;
if the OS refuses unlock, an inherited descriptor can still retain the lock until
it closes. [Issue #13](https://github.com/joeloverbeck/unfolded-tabletop/issues/13)
owns this repair and its coordinated consumer verification.

This preserves the original replacement behavior and diagnostics. It is not a
multi-file transaction: a later failure can retain a restart archive or rotate
the previous record. Staged files are synced, but parent-directory metadata is
not; this does not promise universal power-loss durability or automatic repair
of corrupt records. `record-replacement/tests/record_replacement.rs` retains the original mechanism
cases at the public interface, and since issue #30 every mechanism case Arkham's
continuity tests held: archives, recovery validation, size limits, non-regular
destinations, lock contention and recovery-install cleanup.
`record_replacement_fork.rs` beside it holds lock release before a forked child's
inherited descriptor closes, and `record_replacement_trace.rs` traces both
renames of one replacement under the same sidecar lock, each in an executable of
its own. The trace needs permission to ptrace the test's own child, which
ordinary Linux hosts and GitHub's Ubuntu runners grant; a container that forbids
ptrace fails it. Arkham owns its admission, session wording and real driver restart
tests, and both consumers retain their required checks.

Reverting the consumer adapter and dependency pin restores Arkham's prior local
implementation. Revert the producer addition only after every adopting consumer
has returned to its previous producer; no personal saves are rewritten by that
rollback.

## Table camera

`camera`, behind `render`, owns the orbit, camera-relative WASD pan, wheel
normalization, scene-drag capture and follow interpolation extracted from
`joeloverbeck/arkham-unfolded` at
`89be726a81f73f3d5828683a72ee16eb3de2f66a`, `src/tabletop_camera.rs`.
The consumer supplies its home target and distance, focus/follow targets, and
UI capture facts through `FrameInput`. Named views, table framing, visibility,
modal lifecycle and when an action changes the follow target stay local.

The existing motion settings remain: pitch 0.35–1.35 radians, manual distance
6–90 world units, wheel distance multiplied by 0.95 per line, and follow
interpolation after the manual distance clamp. Focus and follow targets are
consumer-supplied and are not admission-validated here. These are the inherited
camera's bounds, not a promise to fit arbitrary tables.

Arkham's framing tests (its startup home and named initial views) and
real-table navigation stay around its adapter. `tests/camera.rs` holds the motion
Arkham's camera tests held before issue #30: orbit, camera-relative pan, wheel
normalization and limits, scene-drag capture and interruption, follow
interpolation, focus and Home, with the expectations Arkham recorded. Netrunner's
table (`src/tabletop.rs`, under [Netrunner #11](https://github.com/joeloverbeck/netrunner-unfolded/issues/11))
and Cuba Libre's (`src/tabletop.rs`) move their cameras through it. Mudsong's fixed orthographic presentation does not consume this mechanism.
Reverting Arkham's adapter and dependency pin restores its original local
camera. This candidate makes no general camera-platform claim.

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

The protocol's words are spelled once, in `unfolded-seat-protocol`, which needs no
Bevy: the requests the seat reads, the statuses its responses carry and the one
`refused` line of a start that fails before a table is seated. The seat, its
launcher and the held-seat client below use it; the wire did not change when they
moved to it ([issue #34](https://github.com/joeloverbeck/unfolded-tabletop/issues/34)),
so a seat and a client at different revisions keep talking. To withdraw it, first
revert each game's tools and its pin together.

- `Seat` alone steps the table, in fixed steps of one sixtieth of a second. It
  writes pointer and key input where Bevy's window-event conversion writes it,
  settles the table, captures its image through `capture`, and answers
  `observe`, `activate`, `point`, `scroll`, `key`, `settle`, `capture` and `quit`.
  `point` rests the pointer on a named component or choice as `activate` does,
  and leaves it there without pressing; its response says where it rests and
  what is under it. `mudsong`, a private repository that pins the library for
  its application, is the consumer that sends it; a game that does not send it
  changes nothing.
- `scroll` names `revision` and exactly one `component` or `choice`, as `point`
  does, plus `y` and optional `x` (default zero), in wheel lines. Each axis must
  be finite and between -32 and 32, with at least one nonzero axis. For example,
  `{"op":"scroll","revision":9,"choice":"GainCredit","y":-4}` scrolls down
  over that visible control. Positioning and actual picking precede both normal
  `MouseWheel` and `PointerInput::Scroll` messages; no handler is called directly.
  The `scrolled` response confirms input delivery and reports position and delta,
  not whether a particular panel moved. Observe or capture the consumer effect.
  Each game keeps its scroll policy, camera capture and modal restrictions.
  [Issue #23](https://github.com/joeloverbeck/unfolded-tabletop/issues/23) coordinates
  Arkham's existing menu and Netrunner's overflowing action dock. No adapter API,
  saved format or artwork changes. Revert consumer pins before withdrawing it.
- The guards are the protocol's, so both forms have them. An activation, a point,
  scroll or key acts only on the revision it names. A key that the game marks as able
  to commit presses only the control the request names. A connection stops
  acting after its first refusal or miss: its later activations, keys and scrolls are
  skipped. A point presses nothing, so it is still served, and a missed point
  stops the later activations and keys as a missed activation does.
- `socket` holds one session on a Unix-domain socket that only its owner can
  reach: the launcher that starts the seat as a process of its own, the
  listener, and the session's idle end. The launcher reads the starting seat's
  first line, which is its `ready` response or the one `refused` response that
  `seat::refuse` prints. A game that answers a protocol of its own on such a
  socket claims the path with `Listener::claim` and accepts through
  `Listener::unix_listener` with a loop of its own instead of the seat's. The
  path is removed when its `Listener` drops, as with the seat's; `tests/seat.rs`
  holds both. The claim's refusals still speak of a seat.
- `start` carries a table from "this process was asked to seat an agent" to the
  end of its session, in either form. The game hands `Start::run` its socket, if
  any, and a function that builds its table with the windowless surface installed.
  `Start` holds the game's marker, its words for an admission that ran out of time
  and, unless the game changes them, a 120-second admission limit and a
  30-minute idle period. Before this module existed, Arkham, Netrunner and Cuba
  Libre admitted within 120 seconds and `mudsong` within 30, and all four idled
  for 30 minutes. In batch form the seat builds the table, opens the seat, admits
  the table, announces `ready` and answers standard input. Held, the command an
  agent runs claims the socket and launches the seat, which leaves the launching
  shell's session, takes the listener, builds, opens and admits the table,
  announces `ready` and serves. Every failure before `ready` prints exactly one
  `refused` line, and a launching command relays the seat's as its own. A table can
  keep something that must outlive its app, such as a lease on its artwork; it is
  dropped after the app, whether the start succeeds or fails. `Start::admit`
  opens and admits a built table in the
  calling process, so a game's tests sit an agent at its table with no launch.
  The separate calls (`detached`, `launch`, `detach`, `announce`, `Seat::open`,
  `Seat::admit`, `Seat::serve`, `Listener::serve`) still work, so a game that pins
  this revision and keeps its own sequence still builds.
  [Issue #32](https://github.com/joeloverbeck/unfolded-tabletop/issues/32) moves
  the four games onto `start`. To withdraw it, first revert each game's adapter
  and its pin together.

Each game implements `Game` in its own adapter: its observation and revision,
the names it resolves to screen positions and back, what is still unsettled
and its admission to play. The seat answers the rest unless the game says
otherwise. Nothing has focus. A step of the seat moves Bevy's manual time
(`TimeUpdateStrategy::ManualDuration`) by `seat::STEP`. No key may be
pressed. A game overrides `focus`, `advance` or `keys` to answer them itself,
as Arkham does for its controls' focus, its presentation clock and its keys,
with those among them that can commit. The seat itself decides whether the
render pipelines are ready before a capture. `capture::install`, which builds
the windowless surface, adds a system after Bevy's `render_system` that finds
whether any pipeline is queued or not compiled (`capture::Pipelines`). It does
so while the table is built, before pipelined rendering moves the render world
to a thread of its own. A capture waits for ready on two consecutive frames.
`Game::pipelines_ready` is deprecated and stays only so that a game that has
not moved still builds; a capture also waits for it. Every response of a
seated table states, beside the game's `observation` and `table`, the
`revision` a request must name, whether the table is `settled` and what is
`pending`, as `revision` and `unsettled` answer them. The one `refused` line
of a start that fails before a table is seated (`seat::refuse`) has none of
them. These fields are additions: no other field or wording changed.
`seat::Unshown` is a table that shows nothing and is admitted as its function
says, for a game's test of its admission alone. [Issue #33](https://github.com/joeloverbeck/unfolded-tabletop/issues/33)
coordinates the four games' move to these defaults. To withdraw them, first
revert each game's adapter and its pin together. These are each game's too: the words of an admission that
ran out of time, the plugins and contents of its windowless table, its launch
options, the environment variable that marks a launched seat, its admission limit,
how long a held session may idle, and whatever must outlive its app. A game that
starts through `start` hands it all of them except its launch options.

`ui_target` reads a visible, nonempty Bevy UI node's logical center and whether
that center lies in its calculated clip and the supplied logical viewport. It
preserves clipped/offscreen positions with `on_screen: false`, and omits nodes
without layout or inherited visibility. This is geometry only: the seat still
checks the actual picking hit before pressing. Control membership, labels,
ordering, focus and modal rules remain in each game's adapter.
Origin: Arkham `71281007a41905bf25a95a29486e10fb11b0a83f`,
`src/gathering_tabletop/seat.rs`. [Issue #15](https://github.com/joeloverbeck/unfolded-tabletop/issues/15)
coordinates its Arkham migration with Netrunner's #18 diagnostic adapter.
Mudsong uses its own rectangular hotspots and needs no geometry migration.
Revert adopting consumers' adapters and pins before removing this additive helper;
it changes no saves or artwork.

Some things are fixed here and were sized or worded by Arkham, the first table
to use the seat. No second use has shaped them yet. A test holds some of them
and not others, so a change to the rest would pass every check:

- The numbers: a step of one sixtieth of a second, settling for 1,800 steps
  unless a request asks otherwise and for 7,200 at most, and 30 seconds for a
  capture. Admission settles the admitted table for at most 1,800 steps and
  then returns, settled or not. `tests/seat.rs` holds the step, which a game
  reads as `seat::STEP`. The settling limits and the capture's 30 seconds are
  fixed in `src/seat.rs` and held by no test, here or in Arkham. A capture
  whose pipelines are never ready two frames running is refused once the 30
  seconds pass, so that test in `tests/seat.rs` takes 30 seconds to run; it
  does not hold their length.
- The protocol's vocabulary and wording: an agent names a `component` or a
  `choice`, responses carry `"mode": "seat"`, and refusals speak of a choice
  identity, a reading, a menu row and the attempt's revision. Arkham's own seat
  tests activate by both request fields and hold the mode; `tests/seat.rs`
  activates by `component` and holds the mode. `tests/seat.rs` holds, word for
  word, the refusals of an outdated revision, a committing key, an unknown key,
  a busy table, a skipped request, a missed activation, a missed point and a
  capture on a host with no image, and Arkham's tests assert parts of some of
  those. No Arkham test points. The other
  refusals of a capture, and the wording for a focus that is no control or menu
  row, are held by no test.
- The held session is Linux only. Its launcher starts the running executable
  again through `/proc/self/exe`, a socket path holds at most 107 bytes, and the
  held seat's standard error goes to a log at the socket's path with `.log`
  added. `tests/seat.rs` holds the 107 bytes, and Arkham's tests read the log at
  that path. `tests/seat_start.rs` starts a launched seat end to end on a table
  of no game: it runs its own executable as a game's command, through the real
  launcher, and reads `ready` and a session ended by `quit` in both forms. For
  each failure before `ready` it reads the one line the command prints: the only
  line in batch form, and in held form the launching command's only line, which
  relays the seat's refusal. In both forms these failures are a table that cannot
  be built, one with no camera for the seat to open, a refused admission and an
  unfinished one. Only in held form are they a socket that cannot be claimed and a
  marked seat handed no listener. It also holds that a table's kept value drops
  after its app when admission succeeds and when it is refused. It starts child
  processes of its own and listens on Unix-domain sockets under the system's
  temporary directory, whose path must leave the sockets within 107 bytes. It
  needs no graphics adapter. A real render
  through a held start is still each consumer's own check.

`tests/seat.rs` pins the protocol's responses and refusals as Arkham worded
them, on a table that belongs to no game and without a graphics adapter: its
picking, its renderer and its render world's finding of the pipelines are
stand-ins, and its keys are its own. Arkham's own tests hold the same
protocol on its real table. A real render through the seat is each consumer's
own check.

## Held-seat client

`seat_client`, in `unfolded-seat-client`, is available on Unix. It extracts Arkham's
`tools/src/seat.rs` at `8301466b5bbc20b99411901cfcd6db2b29865431`, and since
[issue #34](https://github.com/joeloverbeck/unfolded-tabletop/issues/34) also the
single-request exchange of Arkham's `seat-wait` at
`c73485c3f066442664073d276fa4d419a3e75cf3`.

`Connection::open` connects, and says when no seat listens there (no socket, or
one nothing answers on) apart from every other failure. A connection then serves
one call; each sends, half-closes its input and reads:

- `send` sends request lines, copies each response line unchanged to the caller's
  writer (`io::sink()` keeps none) and says how it went: every request answered and
  none failed; every request answered and one refused, missed, skipped or with no
  status (not JSON, none, or not a string); or fewer answers than requests, which
  it says first. A string status the protocol does not name is no failure, as in
  Arkham.
- `request` sends one `Request` and returns its parsed answer with its status and
  line, or says how the session ended: the connection closed before it could be
  sent, was reset, or closed unanswered, or the answer was `bye` or `exited`. An
  answer that is not JSON or has no status the protocol names is an error. Which
  answers the caller accepts stays with the caller.

While the seat says nothing, both wake about five times a second to run the
caller's check, whose error ends the wait: the caller's deadline or interrupt,
without setting socket timeouts itself. A final complete line without a newline is
accepted. There is no response-line size limit or write timeout, and a writer
blocked on a large send is not interrupted. Signal installation, launch guidance,
exit statuses, wording and partner-wait decisions stay in each game.
`requests`, `connect`, `exchange` and `read_line` remain from before, so a game that
pins this revision and keeps its own loop still builds: with them the caller sets
the read timeout and compares the answer count with its requests.

The protocol's statuses, its request shapes, the `refused` start line and the
launcher's reading of a start's first line are `unfolded-seat-protocol`'s, which
the seat, this client and `held-start` share, and which a game's tools can name. Its
`seat-protocol/tests/protocol.rs` holds every status's and request's wire spelling.

[Issue #17](https://github.com/joeloverbeck/unfolded-tabletop/issues/17) and
[Arkham #241](https://github.com/joeloverbeck/arkham-unfolded/issues/241) coordinate
this candidate for [Cuba Libre #10](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/10).
Those records carry the owner's 2026-11-14 bounded EVID-6 extension for mechanisms
used by Cuba and an existing game, the two-hour extraction cap, actual-consumer
verification and rollback. Cuba's `cuba-tools` sends its requests through it.
Revert Arkham's adapter and dependency pins before removing
the producer addition; no saves or artwork change.

The package's `held-start` binary records one held start of any game's seat as
evidence for a pull request. Run it from the game's checkout, so that it records
that checkout's revision:

```sh
cargo run --manifest-path …/unfolded-tabletop/Cargo.toml -p unfolded-seat-client --bin held-start -- \
  <out-dir> <case> -- <game executable> <its seat options>
```

It appends `--socket <out-dir>/<case>.sock`, with `<out-dir>` made absolute since a
game may refuse a relative socket path as Arkham does, and runs the command with no
standard input. It keeps that command's standard output and error as `<case>.stdout` and
`<case>.stderr`, and copies the seat's log to `<case>.seat.log`. `<case>.record`
names:

- the command, the working directory, and that directory's Git revision with
  whether its tracked files are unchanged;
- the launching command's exit status, its standard-output line count, and the
  first line's status and any error;
- when that status is `ready`, the statuses that `observe` and `quit` answered;
- whether the socket was removed within ten seconds, and the log's last two lines.

The record is also printed. A case already recorded in the directory is refused,
so a later run cannot replace an earlier one. Record a "before" run under its own
case before changing the game. `seat-client/tests/held_start.rs` holds a ready
start, a refused one, the refusal of a recorded case and a relative `<out-dir>`
handed to a game that refuses a relative socket. Its stand-in game is the
test executable itself, so it has no test harness of its own. It starts child
processes, listens on a Unix-domain socket under the system's temporary directory,
and runs `git` when it is installed, which finds no repository there.

## Screen capture

`capture` holds the part of Arkham's screen capture that Arkham and `mudsong` both
used when it moved; Netrunner and Cuba Libre now capture through it too.
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
  as the default UI camera. Where the host has a render world, `install` also
  adds `Pipelines`, the render world's finding after each frame of whether every
  pipeline is compiled. Call it after the render plugins are added. A host with
  no render world finds them ready.
- `image_to_file` asks Bevy for a capture of an image and returns once the file
  is written. The caller supplies the step that runs one update of its host and
  the deadline; the module waits two milliseconds between steps.

What is not here stays with its one consumer. Arkham keeps its screenshot
observers, their timed diagnostics and its exits, and frame pacing, on which no
capture here depends. The module does not enable Bevy's window event loop, so
the application that enabled it leaves it out of a windowless host itself. What
must be ready before a capture, the clock the step advances, the time allowed
and the wording of a failure belong to each consumer; the agent seat, one of
them, waits for `Pipelines`.

`tests/capture.rs` pins the pixels, the surface and the capture as Arkham had
them before the move, without a graphics adapter. One behaviour differs on
purpose: once a capture has returned, a picture that arrives for it is not
written. In Arkham a capture given up as incomplete still wrote its file when
the picture came later, and a picture handed back twice was written twice,
which Bevy does not do. A real render is each consumer's own check.

## Seeded randomness

`random::fresh_seed` reads eight bytes from the operating system's
`/dev/urandom` device and returns their little-endian `u64`, or the unchanged
open/read error. It extracts `Session::new` from Arkham at
`9f4bbf6050df9dc4f675ab42c0f5fcb879c1a827`, `src/gathering.rs`.
[Issue #21](https://github.com/joeloverbeck/unfolded-tabletop/issues/21)
coordinates Arkham's existing startup migration with ordinary new-game setup
in [Netrunner #22](https://github.com/joeloverbeck/netrunner-unfolded/issues/22).
When a fresh game is allowed, failure wording and seed persistence remain with
each consumer. No fallback seed is supplied. Mudsong has no corresponding
entropy acquisition to migrate. Revert consumer adapters and pins before
removing this helper; no saved format or generator sequence changes.

`random` holds Arkham's SplitMix64 generator, its rejection-sampled index and its
shuffle. Origin: `joeloverbeck/arkham-unfolded` at
`4e7a014541ef46d8dc8185e2759a139480ea60e3`, `src/gathering.rs` and
`src/gathering/runtime.rs`. Arkham draws its setup and play randomness from it.
`mudsong`, a private repository, pins it for its rules library. No game's domain
enters the module.

A generator's whole state is one `u64`, and the numbers a state produces are a
contract: a consumer's saved game stores that state and continues only while it
keeps producing the same numbers. `random/tests/random.rs` pins numbers recorded from
Arkham before the move, so changing them is a breaking change for every consumer.

Each consumer's pull request records its own migration, checks and acceptance.
Reverting a consumer's dependency commit returns it to its prior producer;
Arkham's prior commit contains its original generator.

## Development checks

Run `scripts/check` from a clean clone before publication. `.cargo/config.toml`
bounds every Cargo command run here, not only the check's: four jobs and no debug
info, since unbounded parallel links of the Bevy test executables can exhaust a
session's memory. The environment may override them; CI sets two jobs. The check
runs full-workspace formatting first. It then fails if code outside
`unfolded-seat-protocol` spells a seat status as a string literal; tests keep the
spelling as the wire's oracle. Next comes strict Clippy, on the workspace and on
`unfolded-tabletop` with `render` and with `seat`. It asserts that no workspace member other than
`unfolded-tabletop` reaches Bevy, and that `unfolded-random` has no dependency, and lints each Bevy-free package
alone, since a workspace build can supply a dependency feature that a game
depending on that package alone would lack. Then it runs the workspace's tests
and `unfolded-tabletop`'s with `render` and with `seat`. Rustup selects the compiler/components in `rust-toolchain.toml`; the
committed lockfile fixes this repository's verification dependencies. CI runs the
same entry point on Ubuntu without game assets or a desktop. A focused command is
only partial evidence. These checks do not replace any consumer's own checks:
asset, GPU, native acceptance or saved-game continuity.

The workspace's local Cargo patch exercises its owned `vendor/wgpu-hal` copy.
`scripts/check` also runs that copy's `vulkan::upload_memory` regression tests
from its own manifest and tracked lockfile; they need no GPU. `assets/tests/assets.rs`,
`tests/card.rs`, `tests/token.rs` and `tests/loading.rs` hold bounded and verified
reads, PNG crop bounds, publication without replacement, card meshes, token
assembly and load-state reports with test-made inputs. No test registers a real
font, so `loading::font_registered` is held only for a font without a family.
Consumers still select their own immutable root patch and dependency lockfile;
this verification setup does not change their pins.

`vendor/wgpu-hal` retains Arkham's narrowly scoped NVIDIA Vulkan staging workaround
and its upstream licensing. Applications must configure their own Cargo root
`[patch.crates-io]`; a dependency's patch does not propagate. Pin the patched package
at the same immutable revision as the library packages, as **Packages** says.

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

On 2026-11-13 the owner read the waiver as covering the agent seat as one
mechanism: an operation of the seat that one consumer uses, such as `point`,
is covered while the seat itself is used by two.

## Development readers and recorded checks

The separate `unfolded-dev-tools` workspace crate owns the bounded context readers,
check recorder/resource summaries and process/file support extracted from Arkham
`d0b57009ba037ca8756a4f2756dcb550d8634773`. Shared issue #27 coordinates Arkham #249
and Netrunner #37. The existing two-consumer EVID-6 specialization applies; this
is development support for those consumers, not a general agent platform.

Arkham keeps its CLI and publication/native-operation policies. Netrunner keeps
its fixture/preflight selection. Consumers may embed `reader`, `check`, `process`
and `digest`; `run_cli` supplies the same reading/recording commands in a dedicated
process and installs its interrupt handler there. Reader arguments retain their
explicit byte limits, paging cursors and source hashes. Recording retains complete
logs, source fingerprints, interruption and child-cleanup semantics. These are
mechanical evidence, not authority to accept or publish their result.

The runtime crate does not depend on this crate or on its command-line/signal
handling dependencies. Consumers pin immutable Git revisions; no sibling checkout
or separately built Arkham executable is needed. `dev-tools/tests/cli` runs, through
`run_cli` and one process per invocation, the reader, search and recorder cases
Arkham's tool tests held before issue #30, and a relative command run from the
working directory. The test executable is its own harness and the child its
recorded checks run. `dev-tools/tests/process.rs` holds process ownership and the
digest. Issue #30 links Arkham's pull request that removes its copies; Arkham
keeps the tests of its own command wiring, including its nested `scripts/dev`
and `scripts/check` record.
Revert consumer adapters/pins before removing the shared crate; the baseline
above retains the original implementation. Game behavior, saves and assets are
outside this extraction.
