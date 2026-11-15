# Inter-repository integration

The core constitution's requirements language, definitions and applicable rules govern this module. Its Modules table determines any additional applicable modules. One adopted revision binds the core and its modules.

### INT-1 — Semantic integration crosses a versioned artifact boundary

**Trigger:** Independent repositories exchange domain or pipeline artifacts during ordinary operation, or a proposed handoff is offered as evidence of downstream usability.

**Rule:** The producer MUST emit a bounded, immutable, versioned export. The consumer MUST acquire its own stable copy or integrity-checkable snapshot, retain source identity and provenance, translate through a consumer-owned adapter where needed, and exercise its own acceptance authority before authoritative mutation. The producer owns source semantics and the export contract; the consumer owns faithful implementation or local projection, consumer identifiers, validation, and its own state. A declared set of files at an immutable commit may be sufficient initially. Honor an existing accepted export interface; do not bypass it to mine producer internals. No registry, custom packaging platform, shared schema, or per-field custody system is implied.

State how the material is consumed:

- **Implementation from a specification:** a human or coding agent uses the exact authored source to implement product-specific behavior. The source must determine domain behavior for the accepted scope; code may supply internal representations and algorithms. Pin both source and implementation revisions, retain proportionate links from consequential rules to their implementation and independent behavioral checks, and expose source ambiguities. The program need not parse the rule specification at runtime.
- **Runtime data or interpretation:** the executing consumer reads the claimed parameters, content, or behavior representation. Parsing, typed tokens, field coverage, or a separate model is insufficient: demonstrate that the actual input governs the claimed behavior. A hybrid contract may use both modes, with their boundary explicit.

For the first real handoff, demonstrate representative difficult behavior through the intended consumer against source-derived expectations, then exercise a bounded source change and its intended downstream effect. For runtime data, change the consumed value or content and verify its use. For implemented rules, bind the source change to the corresponding implementation change and behavioral check. A reversible test revision may establish this dependency when no real correction is warranted; it does not justify changing canonical product rules merely to complete a ritual. Record the exact source, consumer revision, scope exercised, result, and material limits in existing work artifacts. No dedicated receipt schema is required. Apply `EVID-3` for actual use and expansion, and `EVID-4` for what this evidence establishes.

Proof of the promised downstream execution MUST exercise the repository and implementation that own the intended execution path. Producer-side structural checks and independently derived expectations remain appropriate. A scratch program, auxiliary repository, alternative-language model, or hard-coded parallel implementation MUST NOT substitute for consumer proof or evade a boundary that places runtime behavior downstream. An independent executable oracle requires a distinct justified verification question and proportionate scope; it does not become a second product implementation by default. A consumer that works from an ancestral source while bypassing the authored export does not prove that export's fitness.

Ordinary operation MUST NOT depend on live reads of the producer's working tree, internal database, private directory layout, skill definitions, issue tracker, implementation-owned records, or uncommitted state. Repository-owned automation MUST NOT mutate a peer's authoritative state. Human-authorized coordinated work follows `AGENT-1` and each repository's own authority. Ordinary package-manager code dependencies are permitted when they do not smuggle peer-owned semantic state or working-tree coupling.

### INT-2 — Feedback is proposal data, not a reverse authority transition

**Trigger:** A downstream result may influence an upstream repository.

**Rule:** Feedback MUST travel as an explicitly non-authoritative proposal package containing the source package identity, proposed claims or changes, evidence, scope, uncertainty, conflicts, and known omissions. The upstream repository alone accepts and mutates its state.

A consumer-owned adapter or projection MAY add consumer-specific representation, presentation, validation, policy, or state, but it MUST NOT silently amend, contradict, or replace producer-owned semantic truth or behavior that the consumer claims as imported authority. When the consumer's primary promise requires such a source-semantic change, adoption and production reliance on that changed behavior MUST remain blocked until the producer accepts the change, emits a new versioned package, and the consumer imports and accepts it. A bounded provisional implementation may demonstrate a proposed resolution under `PURP-5` and `AGENT-5`; keep it revision-bound and separate from accepted behavior. An implementation defect under an unchanged accepted rule is repaired downstream and does not require a new upstream design ruling. A consumer-specific interpretation that is explicitly local, does not claim to alter producer truth, and remains within the consumer's own authority is unaffected.

Repository-scoped identifiers remain scoped. The consumer preserves producer identities as provenance and allocates its own identities; independent repositories MUST NOT share a global identifier allocator or universal internal schema merely for convenience. Manual transfer or an ordinary file-import command SHOULD remain the default transport until repeated real exchanges justify automation.

### INT-3 — Direct legacy access is a bounded migration exception

**Trigger:** A one-time migration from a legacy or abandoned repository would otherwise require disproportionate new exporter work.

**Rule:** The bridge MUST be read-only, pin the exact source revision, name the accepted source scope and target mapping, preserve comparison and rollback evidence, prohibit ordinary runtime or authoring dependence, and expire on a named event or date. Output MUST become self-contained in the target's accepted custody before migration closes.

The source's schemas, directories, stages, and historical machinery are evidence, not target architecture. A bridge that writes back, follows a moving head, gains recurring consumers, or survives expiry is ongoing integration and MUST instead satisfy `INT-1` through `INT-4` as applicable or stop.

### INT-4 — Imported authority retains custody and derivation evidence

**Trigger:** Accepted consumer state depends on an imported package.

**Rule:** Retain enough immutable evidence to identify the exact package, producer and revision, content identity or digest, edits before acceptance, adapter or transformation revision, acceptance decision, and consumer artifacts derived from it. The package MAY live in Git or another stable private artifact store or attachment; it need not be duplicated when a durable integrity-checkable binding exists.

The evidence MUST remain retrievable for as long as the consumer claims reproducibility, source fidelity, or derivation. If it is lost, state the loss and downgrade the provenance claim rather than reconstructing it from memory. Prefer ordinary version control, content digests, and existing work artifacts over a provenance platform.

