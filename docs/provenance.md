# Provenance and rollback

Where each mechanism came from, and the issues that coordinated its move. Use it to find the original implementation when a consumer rolls back.

## Rollback

To withdraw a change here, first revert each adopting consumer's adapter and its pin together, then revert the change in this repository once every adopting consumer is back on its previous pin. Reverting a consumer's pin returns it to the producer it had before; for an extraction, the consumer's commit before its migration still holds its original implementation, at or after the origin below.

## Origins

| Mechanism | Origin | Coordinated by |
| --- | --- | --- |
| `unfolded-assets`, `card`, `token`, `loading`, `vendor/wgpu-hal` | Arkham `83bd1264b7962d844cfa2417d7f50be642575c9a`: `src/assets.rs`, `src/tabletop_pieces/card_shape.rs`, `src/tabletop_counters.rs`, `src/tabletop_health.rs`, `vendor/wgpu-hal` | [Netrunner #1](https://github.com/joeloverbeck/netrunner-unfolded/issues/1) |
| `unfolded-record-replacement` | Arkham `89be726a81f73f3d5828683a72ee16eb3de2f66a`: `src/record_replacement.rs` | [Netrunner #9](https://github.com/joeloverbeck/netrunner-unfolded/issues/9); lock release #13; tests #30; the save record #36, from Arkham `c73485c3f066442664073d276fa4d419a3e75cf3`: `src/saved_state.rs`, Netrunner `df604944d3c7ad96b786df5edf6c0959e79733f2`: `src/saved_state.rs` and Cuba Libre `c5f9cb8720e0f0e9068faea09d5579671be77e38`: `src/save.rs` |
| `camera` | Arkham `89be726a81f73f3d5828683a72ee16eb3de2f66a`: `src/tabletop_camera.rs` | [Netrunner #11](https://github.com/joeloverbeck/netrunner-unfolded/issues/11); tests #30; input gathering #37, from Arkham `c73485c3f066442664073d276fa4d419a3e75cf3`: `camera_controls` in `src/gathering_tabletop.rs`, Netrunner `df604944d3c7ad96b786df5edf6c0959e79733f2`: `camera_controls` in `src/tabletop/turns.rs` and Cuba Libre `c5f9cb8720e0f0e9068faea09d5579671be77e38`: `camera_controls` in `src/tabletop.rs` |
| `count_label` | Arkham `d7a6f2ea46fbf97719cc352ee8b94270857b9bcd`: `src/tabletop_counters.rs` | #11, [Netrunner #12](https://github.com/joeloverbeck/netrunner-unfolded/issues/12) |
| `label_anchor` | `count_label`, with Netrunner `df604944d3c7ad96b786df5edf6c0959e79733f2`: `src/tabletop/turns.rs` | #35 |
| `printed` | Arkham `d0212e5eecf21c358472a62c93c8992607497263`: `src/gathering_tabletop/printed.rs` | #25, [Arkham #247](https://github.com/joeloverbeck/arkham-unfolded/issues/247), [Netrunner #26](https://github.com/joeloverbeck/netrunner-unfolded/issues/26) |
| `unfolded-hover` | mudsong (private) | #18 |
| `random::fresh_seed` | Arkham `9f4bbf6050df9dc4f675ab42c0f5fcb879c1a827`: `Session::new` in `src/gathering.rs` | #21, [Netrunner #22](https://github.com/joeloverbeck/netrunner-unfolded/issues/22) |
| `unfolded-random`'s generator, index and shuffle | Arkham `4e7a014541ef46d8dc8185e2759a139480ea60e3`: `src/gathering.rs`, `src/gathering/runtime.rs` | `docs/governance.md`, third-consumer waiver |
| `capture` | Arkham `617904319a7b260e496af61cc5390458c2b1ff6b`: `write_image` of `src/tabletop_capture.rs`, and the windowless surface and capture loop of `src/agent_seat.rs` | |
| `seat` | Arkham `0b8e6fb3425174110446d6956cfa9bc38008e279`: `src/agent_seat.rs`, `src/agent_seat/socket.rs` | `scroll` #23; `start` #32; defaults #33 |
| `seat::ui_target` | Arkham `71281007a41905bf25a95a29486e10fb11b0a83f`: `src/gathering_tabletop/seat.rs` | #15 |
| `unfolded-seat-protocol` | Split from `seat` here | #34 |
| `unfolded-seat-client` | Arkham `8301466b5bbc20b99411901cfcd6db2b29865431`: `tools/src/seat.rs`; and `seat-wait`'s single-request exchange at `c73485c3f066442664073d276fa4d419a3e75cf3` | #17, [Arkham #241](https://github.com/joeloverbeck/arkham-unfolded/issues/241), [Cuba Libre #10](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/10); #34 |
| `held-start` | Written here | `fcdda27`; captures #52 |
| `unfolded-dev-tools` | Arkham `d0b57009ba037ca8756a4f2756dcb550d8634773` | #27, [Arkham #249](https://github.com/joeloverbeck/arkham-unfolded/issues/249), [Netrunner #37](https://github.com/joeloverbeck/netrunner-unfolded/issues/37); tests #30 |
| `compare-captures` | Written here | #52 |
