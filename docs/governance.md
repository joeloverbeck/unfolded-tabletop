# Governance records

The evidence boundaries under which mechanisms are shared here, and which one covers each mechanism. Read this before extracting a mechanism, adopting a consumer, or anything that could end a boundary below. The constitution is `docs/principles/principles-for-repos.md`; `CLAUDE.md` names its revision and adopted modules.

None of these boundaries establishes general library fitness or accepts an untested migration. Each requires every affected consumer's own checks through its actual changed path, the game's policy kept in its adapter, and a coordinated rollback (`docs/provenance.md`).

## Two-consumer specialization

The accepted, task-scoped `EVID-6` specialization lets a mechanism be extracted on its two actual consumers, Arkham Unfolded and Netrunner Unfolded, instead of requiring three contexts. It does not establish broad reuse fitness. Production adoption requires both consumers' checks and the bounded native inspection recorded in [Netrunner issue #1](https://github.com/joeloverbeck/netrunner-unfolded/issues/1), which holds the original coordinated extraction and consumer acceptance. Each consumer keeps independent acceptance authority. For any surface, the specialization ends when a third consumer appears.

## Third-consumer waiver

On 2026-11-11 the owner accepted a further scoped `EVID-6` waiver under `ADOPT-2`, which lets `mudsong` be that third consumer. It does not edit the constitution. Its authoritative text is in that private repository; this is its record here, carried by the first extraction made under it, seeded randomness.

| Field | Accepted boundary |
| --- | --- |
| Exact rule | `EVID-6`: three materially different real cases and an independent holdout before cross-product adoption |
| Scope | Concrete mechanisms, not specific to any game, owned in `unfolded-tabletop` and really used by `mudsong` and by at least one of `arkham-unfolded` and `netrunner-unfolded` |
| Reason | These three are the only real products. Duplicating mechanisms or inventing nominal consumers would defeat the intended reuse |
| Accepted risk | Two consumers can miss an incompatible assumption; a later case may force an interface change or a reversal |
| Required evidence | A real path in each consuming application, each repository's own checks, game policy preserved in adapters, and a coordinated rollback |
| Appetite | Two active hours per extraction attempt, then reassess. No extension into a framework project |
| Ends | For a given mechanism: a consumer outside these three games, publication to crates.io or a promise of general reuse, or loss of its second real consumer. There is no milestone or date expiry. After that, further sharing needs the normal evidence or a new waiver |

On 2026-11-13 the owner read the waiver as covering the agent seat as one mechanism: an operation of the seat that one consumer uses, such as `point`, is covered while the seat itself is used by two.

## Cuba Libre extension

On 2026-11-14 the owner accepted a bounded `EVID-6` extension, recorded in [Cuba Libre foundation 3](https://github.com/joeloverbeck/cuba-libre-unfolded/blob/38e988442dfa6ac287baf0a373e77fde5ac77764/docs/foundation/03-engineering-and-reuse.md#accepted-evid-6-waiver), for concrete mechanisms owned here and actually used by Cuba Libre Unfolded and at least one existing game. Reason: avoid duplication and invented consumers. Risk: two cases may miss incompatible assumptions. Appetite: two active hours per extraction attempt. It ends on a fifth game, a crates.io or general-framework promise, or the loss of the second real consumer.

On 2026-11-22 the owner read the extension as covering a mechanism whose Cuba Libre consumer is already ticketed in an active, owner-approved spec: card motion, whose consumer is ticket [#145](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/145) of Cuba Libre spec [#137](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/137). If #137 ends without #145 using the mechanism, it moves back to Arkham.

On 2026-11-23 the owner authorized `label_anchor::OtherView` for Cuba Libre's card tray, ticket [#143](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/143) of the same spec: a marker only Cuba Libre is to use, on a mechanism Arkham, Netrunner and Cuba Libre use. If #137 ends without #143 using it, it is removed.

On 2026-11-25 the owner [activated](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197#issuecomment-7156330255) Cuba Libre spec [#197](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197), which names the extraction of the transport and client of Arkham's agent partners and their Cuba Libre consumer, ticket [#203](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/203), so that reading covers them. If #197 ends without #203 using them, they move back to Arkham.

## Which boundary covers each mechanism

| Mechanism | Boundary | Record |
| --- | --- | --- |
| `unfolded-assets`, `card`, `token`, `loading`, the `wgpu-hal` patch | Two-consumer specialization | [Netrunner #1](https://github.com/joeloverbeck/netrunner-unfolded/issues/1) |
| `card::pile` | Two-consumer specialization | #92, [Cuba Libre #140](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/140) |
| `card::face_with_corners`, `card::pile_with_corners` | Cuba Libre extension, under the 2026-11-22 reading of a consumer ticketed in an active, owner-approved spec. Neither Arkham nor Netrunner calls them, and Cuba Libre does not use them yet: its consumer is the card tray, ticket [#143](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/143) of its spec [#137](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/137), not yet built. If #137 ends without #143 using them, they are removed | #92, [Cuba Libre #140](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/140) |
| `unfolded-record-replacement` | Two-consumer specialization; also the Cuba Libre extension, since Cuba Libre saves through it beside Arkham and Netrunner | [Netrunner #9](https://github.com/joeloverbeck/netrunner-unfolded/issues/9), #13, #36 |
| `camera` | Two-consumer specialization; also the Cuba Libre extension, since Cuba Libre moves its camera through it beside Arkham and Netrunner | [Netrunner #11](https://github.com/joeloverbeck/netrunner-unfolded/issues/11), #37 |
| `printed` | Two-consumer specialization | #25 |
| `random::fresh_seed` | Two-consumer specialization | #21 |
| `seat::ui_target` | Two-consumer specialization | #15 |
| `unfolded-dev-tools` | Two-consumer specialization; `compare-captures` does not widen it, since Cuba Libre and mudsong run it from a checkout of this repository and neither pins nor depends on it | #27, #52 |
| `unfolded-random`'s generator, index and shuffle | Third-consumer waiver | this file |
| `capture` | Third-consumer waiver | this file |
| `slab` | Third-consumer waiver | #89 |
| `seat`, `unfolded-seat-protocol` | Third-consumer waiver, read on 2026-11-13 as covering the seat as one mechanism | this file, #34 |
| `unfolded-hover` | Cuba Libre extension | #18 |
| `unfolded-seat-client` | Cuba Libre extension | #17, [Arkham #241](https://github.com/joeloverbeck/arkham-unfolded/issues/241) |
| `label_anchor` | Cuba Libre extension, beside the two-consumer specialization | #35 |
| `label_anchor::OtherView` | Cuba Libre extension, as the owner authorized on 2026-11-23: a marker on `label_anchor`, which Arkham, Netrunner and Cuba Libre use. No game marks a camera with it: its consumer is Cuba Libre's card tray, ticket [#143](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/143) of its spec [#137](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/137), not yet built. If #137 ends without #143 using it, it is removed | [Cuba Libre #143](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/143) |
| `window_size` | Cuba Libre extension, for its adoption by Cuba Libre beside Arkham | #83, [Arkham #279](https://github.com/joeloverbeck/arkham-unfolded/issues/279), [Cuba Libre #111](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/111) |
| `card_motion` | Cuba Libre extension, read on 2026-11-22 as covering a consumer ticketed in an active, owner-approved spec. Cuba Libre does not use it yet: its consumer is the card advance, ticket [#145](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/145) of its spec [#137](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/137), not yet built | [Cuba Libre #141](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/141) |
| `agent_transport`, `unfolded-agent-players`, `unfolded-seat-client`'s `Connection::ask` | Cuba Libre extension, under the 2026-11-22 reading of a consumer ticketed in an active, owner-approved spec, [#197](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197), [activated](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197#issuecomment-7156330255) on 2026-11-25. No game uses them at this revision. Arkham's adoption is part of [Cuba Libre #199](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/199). `Reply::hung_up` is for the waits without a period of ticket [#201](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/201), not yet built. Cuba Libre's consumer is the faction seat over the socket, ticket [#203](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/203), not yet built. If #197 ends without #203 using them, they move back to Arkham | [Cuba Libre #199](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/199) |
