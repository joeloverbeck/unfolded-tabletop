# Changelog

What a consumer needs to know when it moves its pin. Each entry names the issue that made the change; a revision is whatever `main` held after that issue's pull request merged. Changes that alter only this repository's process or documentation get no entry.

**Breaking** marks a change that can stop a consumer building or change what its game does. Everything else is additive.

## Unreleased

- `move-pin`, run from a checkout of this repository, moves every pin of it in a game's tracked manifests to one revision and refreshes the game's tracked lockfiles. It refuses pins that disagree (#37 retro).
- `camera::TableCameraPlugin` gathers the table camera's input itself: keys, mouse buttons, wheel, primary window and frame time, by Bevy's `Time` or `Time<Clock>` for a game clock of your own. It finds itself whether the pointer is over a UI node (the hover map) and whether keyboard focus is on a `ui_widgets::Button`; mark a UI node that stands for something in the scene, such as a count over a token, `InScene`, and the pointer over it still zooms and drags. To adopt it: insert your `Orbit`, mark your camera `TableCamera`, write `TableFacts` (`blocks_camera`, `takes_view_keys`, `takes_wheel`, and `scene`, a `Scene::Within` rectangle for a pointer region of your own) in a system before `TableCameraSystems`, read `Navigation` after it, and delete your own system. WASD pan stops only when the table blocks the camera, as in every game's system before. `Orbit::drive` takes the same input as plain values in a `camera::Input`, for tests. `Orbit::step`, `FrameInput` and `Capture` still work (#37).
- `camera::Framing` names a target and distance. `camera::PITCH`, `camera::DISTANCE`, `Pose::DEFAULT_YAW`, `Pose::DEFAULT_PITCH`, `Pose::at`, `Pose::direction` and `Pose::transform` make the limits and default pose readable, `Orbit::home` reads where Home returns, and `Orbit::set_home` changes it without moving the camera. `camera::KEYS`, `PAN_KEYS` and `VIEW_KEYS` are the keys the camera reads, and `camera::SEAT_KEYS`, under `seat`, the same keys for a seat key table; `SeatKey` and `Logical` are `Clone` and `Debug`, so a table can join them to its own (#37).
- **Breaking:** `Orbit::update_follow_target` takes an `Option<Framing>`; write `Some(framing.into())` for a `(Vec3, f32)`. `Orbit::new`, `with_home` and `focus` take anything that converts into a `Framing`, a `(Vec3, f32)` included. `unfolded-tabletop` with `render` now enables Bevy's `bevy_ui_widgets` feature (#37).

- `held-start --clean` refuses to launch from a working directory that is not a Git working tree or has changed tracked files, so a "before" case records only a clean revision (#36 retro).

- **Breaking:** `unfolded_record_replacement::save` owns a game's save record: its envelope, checksum, write baseline, unchanged shortcut, previous-record validation and recovery. To adopt it, implement `save::Format` (payload, identity, accepted identities, validity, limit), choose a `save::Write` for each write, word each `save::Error`, and delete your own envelope. Your saved format changes with it: saves written before it are refused and left on disk, and nothing migrates them. `replace`, `read` and `previous_path` are unchanged; `lock_path` and `archives` name the lock and restart archives, so tests need not spell them (#36).

- Deprecated `count_label`, and the seat client's `connect`, `exchange` and `read_line`, which no current consumer calls. They still work. Use `label_anchor::LabelAnchor` on a node of your own, and `Connection::open`, `Connection::send` and `Connection::request` (#56).
- `held-start --help` and `compare-captures --help` describe each tool in full (#56).

## Before 2026-11-16: what still matters

- **Breaking:** the re-exports `unfolded_tabletop::random`, `unfolded_tabletop::hover`, `unfolded_tabletop::record_replacement`, `unfolded_tabletop::assets` and `unfolded_tabletop::seat_client` were removed. Name each mechanism by its own package, such as `unfolded_random::SeededRandom`. The last revision of `main` that has them is `8080f29` (#31).
- `seat::start` runs a game's whole seat start in batch and held form. The separate calls (`detached`, `launch`, `detach`, `announce`, `Seat::open`, `Seat::admit`, `Seat::serve`, `Listener::serve`) still work, so a game that keeps its own sequence still builds. They are not deprecated: `start` is built from them, and games' tests call `Seat::admit` and `Seat::serve` (#32).
- `Game::pipelines_ready` is deprecated: the seat finds whether the render pipelines are ready itself, through `capture::Pipelines`. A capture also waits for the deprecated method, so a game that still answers it keeps working. Every response of a seated table gained `revision`, `settled` and `pending` fields; no other field or wording changed (#33).
- The seat protocol's vocabulary moved to `unfolded-seat-protocol`. The wire did not change, so a seat and a client at different revisions keep talking (#34).
- `label_anchor` keeps any caller-owned UI node over a world anchor. `count_label::spawn` now adds a `LabelAnchor`, and `CountLabelPlugin` installs `LabelAnchorPlugin`; a game may add both plugins, in either order (#35).
- **Contract:** the numbers a `SeededRandom` state produces are part of every consumer's saved games. `random/tests/random.rs` pins numbers recorded from Arkham; any change to them is breaking.
