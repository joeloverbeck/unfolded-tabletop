# Changelog

What a consumer needs to know when it moves its pin. Each entry names the issue that made the change; a revision is whatever `main` held after that issue's pull request merged. Changes that alter only this repository's process or documentation get no entry.

**Breaking** marks a change that can stop a consumer building or change what its game does. Everything else is additive.

## Unreleased

- **Breaking:** `unfolded_record_replacement::save` owns a game's save record: one envelope for every game (`{"record":"unfolded-save-1","identity":…,"payload":…,"checksum":…}`), its checksum, the write baseline, the shortcut for an unchanged write, validation of the `.previous` record and recovery from it. A game implements `save::Format` (payload, identity, validity, limit), chooses `Write::Save`, `Write::Create` or `Write::Archive`, and words each `save::Error`. Moving to it changes your saved format: saves written before it are refused as unsupported and left on disk, and nothing migrates them. `replace`, `read` and `previous_path` are unchanged; `lock_path` and `archives` name the lock and restart archives, so tests need not spell them (#36).

- Deprecated `count_label`, and the seat client's `connect`, `exchange` and `read_line`, which no current consumer calls. They still work. Use `label_anchor::LabelAnchor` on a node of your own, and `Connection::open`, `Connection::send` and `Connection::request` (#56).
- `held-start --help` and `compare-captures --help` describe each tool in full (#56).

## Before 2026-11-16: what still matters

- **Breaking:** the re-exports `unfolded_tabletop::random`, `unfolded_tabletop::hover`, `unfolded_tabletop::record_replacement`, `unfolded_tabletop::assets` and `unfolded_tabletop::seat_client` were removed. Name each mechanism by its own package, such as `unfolded_random::SeededRandom`. The last revision of `main` that has them is `8080f29` (#31).
- `seat::start` runs a game's whole seat start in batch and held form. The separate calls (`detached`, `launch`, `detach`, `announce`, `Seat::open`, `Seat::admit`, `Seat::serve`, `Listener::serve`) still work, so a game that keeps its own sequence still builds. They are not deprecated: `start` is built from them, and games' tests call `Seat::admit` and `Seat::serve` (#32).
- `Game::pipelines_ready` is deprecated: the seat finds whether the render pipelines are ready itself, through `capture::Pipelines`. A capture also waits for the deprecated method, so a game that still answers it keeps working. Every response of a seated table gained `revision`, `settled` and `pending` fields; no other field or wording changed (#33).
- The seat protocol's vocabulary moved to `unfolded-seat-protocol`. The wire did not change, so a seat and a client at different revisions keep talking (#34).
- `label_anchor` keeps any caller-owned UI node over a world anchor. `count_label::spawn` now adds a `LabelAnchor`, and `CountLabelPlugin` installs `LabelAnchorPlugin`; a game may add both plugins, in either order (#35).
- **Contract:** the numbers a `SeededRandom` state produces are part of every consumer's saved games. `random/tests/random.rs` pins numbers recorded from Arkham; any change to them is breaking.
