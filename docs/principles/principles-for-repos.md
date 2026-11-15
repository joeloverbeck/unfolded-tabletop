# Principles for Repositories

**Constitution revision:** 2026-11-15.1
**Supersedes:** 2026-09-30.1.

## Purpose and requirements language

This constitution governs every adopting formal Git repository, including its foundations, work, representations, tests, agents, automation, publications and inter-repository contracts. It prevents work from acquiring authority through momentum, elegance, local correctness, enthusiasm, sunk cost or self-created demand, before repair or abandonment becomes difficult. A specialized rule is dormant when its trigger is absent; harmless inapplicability is not nonconformance.

**MUST** and **MUST NOT** are requirements; deviation needs an `ADOPT-2` waiver unless forbidden. **SHOULD** and **SHOULD NOT** are strong defaults; deviation records reason, accepted risk and bounded scope in an existing authoritative artifact, without a waiver unless a MUST is also relaxed. **MAY** grants permission, not preference.

All text is normative unless labelled **Informative**. The identified rules own detailed obligations. Examples are non-exhaustive and create no requirements by analogy. Identical obligations MUST have one canonical formulation; indexes and bootstraps route to it rather than restate competing rules. Required records MUST use an existing authoritative issue, PR, foundation, decision or governed state artifact unless a new artifact has an independently justified consumer. Compliance alone requires no registry, certification packet, recurring report or form.

## Required agent bootstrap

Copy this block into root `CLAUDE.md` under `ADOPT-1`. Only the `Adopted modules:` line is an adopter parameter: list that repository's adopted module filenames, or `none`. Every other byte is invariant. The core and module files themselves are always copied byte for byte. Repository-specific instructions may surround the block.

<!-- BEGIN PRINCIPLES-FOR-REPOS BOOTSTRAP 2026-11-15.1 -->
## Repository constitution

This repository adopts `docs/principles/principles-for-repos.md`, revision **2026-11-15.1**, and the modules below as binding. `CLAUDE.md` is the real root instruction file; `AGENTS.md` is its symlink.

Adopted modules: `fidelity.md`, `agency-simulation.md`, `search-adaptation.md`, `evaluators-scores.md`, `integration.md`, `longitudinal.md`.

### Mandatory reading

Reasonable doubt fires a trigger. Read the core's Modules table for applicability. A missing triggered module MUST be adopted under `ADOPT-1` before substantive dependent work. Read each applicable module's rules. Founding, material reorientation or foundation revision, adoption/reconciliation, and constitutional amendment require the complete adopted text: core and all adopted modules, plus any newly applicable module.

An exact quotation of a current-revision rule in a brief naming that revision satisfies the recipient's reading of that rule. Paraphrase, memory and earlier reading for another decision do not. The recipient reads anything else its own work triggers.

- Before accepting or activating work: read `PURP-3`, `WORK-1`, `WORK-2`, `WORK-3`, `WORK-5`, `EVID-5`, `OPS-1`, `OPS-4`, `AGENT-5`.
- Before using a test or other evidence for completion, adoption or expansion: read `EVID-3`, `EVID-4`, `EVID-6`, `REP-2`.
- Before merge, release or authoritative publication: read `AUTH-1`, `AUTH-2`, `AUTH-3`, `OPS-6`, `OPS-7`, `PUB-1`.
- Before founding, reorientation or selecting an outcome: read `PURP-1`, `PURP-2`, `PURP-4`, `GOV-1`.
- Before prerequisites, supply, substrate or a coordinated consumer iteration constrain work: read `PURP-5`, `PURP-6`, `WORK-6`.
- Before research or relying on an origin story: read `EVID-1`, `EVID-2`.
- Before broad exploration or a selected full sweep: read `EVID-7`.
- Before recurrent workflow adoption or manual evaluation: read `EVID-8`.
- Before narrowing accepted input or occurrence criteria: read `EVID-9`.
- Before interpreting meaning for a consequential decision: read `SEM-1`, `SEM-2`, `AUTH-3`.
- Before a representation, specification or structured prerequisite constrains work: read `REP-1`, `REP-2`, `REP-3`.
- Before adding or changing a capability or repository boundary: read `BOUND-1`, `BOUND-2`, `BOUND-3`, `BOUND-4`.
- Before dividing or mounting private content: read `BOUND-5`.
- Before composing or designing skills or semantic capabilities, or coordinating work across repositories: read `AGENT-1`, `AGENT-2`.
- Before a handoff, human wait or unattended continuation: read `AGENT-3`, `AGENT-4`, `WORK-3`, `WORK-5`, `OPS-4`, `OPS-7`.
- Before escalating missing inputs or uncertainty: read `AUTH-2`, `AUTH-3`, `AGENT-5`, `OPS-2`.
- Before repairing a test/review failure or starting exact review: read `OPS-3`, `OPS-5`.
- Before acting on consecutive real-input runs with no positive case: read `WORK-4`.
- Before creating work-control or governance surfaces: read `GOV-2`, `GOV-3`.
- Before adoption, reconciliation, waiver, specialization, identifier change or amendment: read `ADOPT-1`, `ADOPT-2`, `ADOPT-3`, `ADOPT-4`.

Record consulted IDs in the existing authoritative work artifact, not a compliance document. Bounded accepted work need not repeat readings or records until scope, premise, authority, publication set or triggered rule family changes.

### Authority and stop signals

Human intent → constrained proposal → checks → inspectable consequences → human acceptance → authoritative state. Human selection authorizes priority and bounded effort; it does not establish truth, fitness, reusability, demand or permission for unaccepted dependent work.

Stop discretionary expansion and read the controlling rules when inputs, oracles, lifecycle evidence or no-match outcomes are missing; an essential consumer handoff remains untested; claims outrun evidence or burden exceeds appetite; presentation, scores, replay or compliance substitute for a different claim, hide failed requirements or obscure causal credit; capability, boundary, fan-out, non-convergence or terminal-decision limits arise; or authority conflicts or a human request would transfer technical investigation to the human. Use the routes above and applicable modules.

If required text cannot be located or read, substantive work is unauthorized. Preserve potentially applicable safeguards while investigating. `AGENT-5` reserves only the human remainder; ordinary implementation uncertainty permits governed investigation and reversible work.
<!-- END PRINCIPLES-FOR-REPOS BOOTSTRAP 2026-11-15.1 -->

## Normative definitions

**Acceptance authority:** named human deciding scoped canonical state, not delegated technical choices.

**Acceptance-bearing increment:** independently inspectable/acceptable outcome part or earliest blocking premise/prerequisite; not scaffolding, repairs, tests or unfinished-work evidence alone.

**Behavioral robustness:** claimed qualitative effects within accepted bounds across nonessential changes/independent seeds, configurations or populations; not identical traces or suppressed diversity/causal sensitivity.

**Appetite:** accepted effort/exposure bounds in applicable time, work-item, review, compute/token, semantic-surface, money or human-burden units.

**Authoritative state:** human-accepted within scope; presence in artifacts, branches, tests or conversation grants none.

**Bottleneck:** earliest unresolved condition blocking primary-outcome acceptance.

**Capability class:** system/authority kind materially changing the solution surface: e.g. parser, model, framework, simulator, evaluator or governed service.

**Consumer:** current accepted use, decision, operational path or bounded decision experiment requiring declared fidelity; not supply's self-created demand.

**Distinctive activity loop:** smallest user-visible cycle distinguishing an effect and justifying material capability/state/recurrence/burden; availability, observation or adjacent use is not completion.

**Decision atom:** independently acceptable/rejectable human commitment consistent with other accepted commitments.

**Implementation judgment:** execution entailed/strongly constrained by authority/evidence, without selecting/materially changing reserved human consequences.

**Reserved human decision:** selecting/changing intent, outcome, materially different product/user semantics neither entailed nor strongly constrained by accepted authority, accepted source interpretation, authority, persistence, appetite, risk/waiver, capability/repository boundaries, irreversible/external effects, publication, oracle, terminal disposition or canonical state; not complexity, unfamiliarity, ambiguity or explanation difficulty alone.

**Independent pipeline:** invocable real-input-to-output path through authority/effects with its own acceptance/failure and release/park/remove/abandon decision; not just a step.

**Least-complex credible intervention:** smallest credible change addressing the same real case without substantial candidate machinery; need not be final/reusable, but losing must test the hypothesis.

**Primary outcome:** human-selected result justifying the repository/workstream. **Enabling work** enables it; **stewardship** maintains accepted products; **deferred possibilities** are inactive.

**Product-bearing artifact or release:** accepted primary-outcome part/named consumer input intended to survive within scope, beyond a bounded decision; not immutable/exhaustive.

**Primary path:** shortest evidence-bearing path from bottleneck to acceptance-bearing increment.

**Publication:** making production newly reachable through default/protected branches, releases, packages, deployments, accepted data or other authoritative paths.

**Qualifying opportunity run:** real-input exposure making positives plausible at claimed frequency, declared before interpretation.

**Real input:** accepted material at first-promise fidelity; synthetic/hypothetical/system-invented examples may test/falsify but do not qualify.

**Representative real-use run:** real input through actual authoring/configuration, runtime and presentation, letting a human finish/intentionally end a bounded task and inspect effects. It proves only activities, loops and recurrence/recovery/assurance/availability exposed at claimed fidelity, not adjacent enjoyment, awareness or subsystem success.

**Materially different second-use run:** representative use of a second accepted artifact/configuration/scenario, chosen for different real pressure on the same path; not synthetic/convenience-selected.

**Return-use run:** representative use after meaningful interruption/clean durable handoff exposing forgotten setup, hidden dependencies and recovery burden; no fixed delay.

**Semantic reviewer:** human/reasoning agent proposing revision-bound interpretations with warrant/uncertainty; authority needs constitutional/explicit human grant.

**Terminal decision:** adopt/reject/narrow/park/reframe/explicitly continue at appetite exhaustion or an authority gate; not a technical choice/finding.

**Completed work item:** bounded issue/PR/equivalent accepted, rejected, parked, superseded or closed terminally; not session, commit or effort.

**Work-control plane:** one system owning active work, priority, readiness, dependencies, acceptance and closure.

## Modules

Files below are complete modules beside this core, governed by the same revision identity. Every trigger stays here whether or not a repository has adopted its module. When a trigger fires, the complete module MUST be adopted before substantive dependent work. A module cannot waive core rules; overlapping triggers require every applicable module. Module rules carry their own triggers and references to other applicable rules. The module names are identifiers, not semantic restrictions on the triggers.

| File | Namespace | Trigger |
|---|---|---|
| `fidelity.md` | `FID` | Promising reproduction, conformance or compatibility with behavior governed by external rules, standards, protocols, published formats/interfaces or legacy systems. |
| `agency-simulation.md` | `SIM` | Runtime LLM narration, state changes, proposals or action selection; autonomous, agentic, adaptive or simulation claims relying on consequence/feedback loops or multiple causal scales; or any claimed causal tendency, emergent pattern, counterfactual effect or robustness across seeds/configurations/perturbations, whether or not labelled simulation. Ordinary coding-agent implementation alone does not trigger this module. |
| `search-adaptation.md` | `SEARCH` | Memory, search, retrieval or ranking materially bearing a slice; claims about cumulative search, learned operators or efficient exploration by search; improvement attributed to feedback, training, optimization or repeated selection; causal credit to a generator/upstream stage after material downstream transformation; or acceptance beyond an exposed guiding objective. |
| `evaluators-scores.md` | `EVAL` | Artificial evaluators materially support a claim whose interpretation depends on their competence/coverage, or a decision-bearing score aggregates materially different properties. |
| `integration.md` | `INT` | Cross-repository import/export, domain or pipeline handoff, feedback, scoped identifiers, private-data mounting, legacy migration or human-authorized coordinated execution. |
| `longitudinal.md` | `LONG` | A mechanism materially depends on later evidence, participation or action, and participant disappearance can strand accumulated state. |

## Semantic authority and acceptance

### AUTH-1 — Human authority

**Trigger:** Interpreted, generated or selected content entering persistent/canonical state or external action.

**Rule:** Apply the bootstrap authority chain. LLM outputs/changes remain proposals until humans accept exact surface. A mechanism or prerequisite selected as outcome needs inspectable benefit for named real use, or a bounded experiment with consumer effect, stop and removal conditions.

Runtime LLM behavior MAY avoid per-output approval only under accepted policy covering permissions, available state, persistence, side effects, refusal, failure, publication and intervention. Emission/display/action grants no canonical authority.

### AUTH-2 — Granular, non-transitive acceptance

**Trigger:** A proposal, patch, migration, repair, batch or derived consequence may change authoritative state.

**Rule:** Expose each decision atom as **entailed** by accepted premises, **required repair** of an invariant/contradiction, or **discretionary**. Entailed atoms MAY form a visible batch naming its premise. Repairs MUST expose the invariant and valid alternatives, including narrowing/rejecting the parent when applicable. Discretionary choices stay deselectable. Acceptance MUST NOT cover omitted/later descendants; localize atoms to protect unrelated accepted state on rejection.

Visibility concerns consequences, not approval per field/sentence/file. Related choices MAY share a coherent revision-bound account of behavior, alternatives, consequences and uncertainty. Link detail; independent discretionary commitments stay identifiable/rejectable.

Bounded activation delegates necessary implementation judgment, reviewable through patch/evidence, without extra gates: e.g. internal reversible naming, placement, decomposition, algorithms, data/fixtures, tests, refactoring and consistency. Alternatives do not reserve choices.

Agents, reviewers and artifacts MUST NOT reserve choices by labels, alternatives or approval requests. Only an accepted human instruction naming concrete human consequence, risk or authority reason can do so; buried agent-authored detail cannot. Delegation creates proposals (`AUTH-1`, `AGENT-5`).

### AUTH-3 — Exposed authority

**Trigger:** Results govern acceptance, rejection, promotion, scoring, publication, canonical mutation or consequential action.

**Rule:** Causality/state transitions MUST be inspectable/testable at their authority boundary. Separate reasoning, authoritative state and workflow control. Expose owner, input revisions, oracle, authority, evidence, uncertainty, validity limits, failure and recheck conditions.

A **mechanical validator** checks formal properties; a **semantic reviewer** proposes interpretations; an **acceptance authority** decides canonical state. “Validator” MUST NOT conceal roles; envelope validity proves no contents. Verify cited sources' existence, identity/revision and production/deterministic derivation of attributed values; otherwise label manual assertions, not verified provenance.

Decision surfaces MUST be intelligible without reconstructing the agent's technical context; bind exact revision; explain changed behavior/claims/risks/authority, recommendation, material alternatives, evidence/uncertainty, reversibility and blocked work; and separate demonstrated results, reasoned expectations and unresolved conditions. Concise evidence-linked explanations/examples/demonstrations suffice; raw records alone do not. Humans need not decode technical files.

Agents own competent investigation/verification; acceptance supplies no missing evidence or unexamined facts. Deficits persist until resolved or humans change the promise. Titles/qualifications MUST NOT claim satisfaction excluding defining requirements. Recommend changed promises plainly; `ADOPT-2` governs relaxed safeguards, `AGENT-5` requests.

## Purpose, activation, and demand

### PURP-1 — Purpose and portfolio fit

**Trigger:** A repository is founded, activated after dormancy, or materially reoriented.

**Rule:** The repository MUST name its purpose class—income-seeking, necessary support, personal utility, research or learning, or recreation—together with the benefit that would justify continued investment, its initial appetite, the event that permits expansion, and the work it displaces. Before founding, reactivation, or material reorientation, the acceptance authority MUST name every other repository it governs that has not reached an accepted primary artifact satisfying its selected primary outcome and is not explicitly parked or abandoned, then state which unfinished work this repository displaces and why the trade is intended. The classes are not a moral ranking. A recreational or speculative repository MUST NOT silently consume enterprise-scale resources, and an income-seeking label is not evidence of market value.

When a primary outcome's selected value lies materially in the experience of use, the benefit and acceptance evidence MUST satisfy the experiential-effect requirement in `PURP-2`. Absent an independently binding external, legal, safety, or stewardship obligation, loss of current appetite is sufficient for the acceptance authority to choose parking or abandonment under `WORK-5`; no confirming experiment is required merely to authorize stopping. That choice establishes current appetite and portfolio priority only. It does not by itself establish technical infeasibility or mechanism unfitness, and it does not erase narrower evidence already earned.

A research or learning purpose MUST name the transferable question, competency, or evidence sought and the condition that closes that outcome. Selecting an exercise, benchmark, port, replica, or borrowed product as the learning vehicle does not by itself select completion of that artifact as a primary outcome. The closure evidence MAY be a completed artifact when completion is expressly selected as proportionate proof of the learning outcome; otherwise, once the named learning outcome closes, unfinished vehicle work creates no residual completion obligation.

When a research or learning outcome and any non-learning primary outcome are both intended, each MUST be stated separately with its own consumer or human effect, acceptance evidence, appetite, expansion event, and abandonment decision. Success, failure, or closure of one does not decide the other. Continuing toward completion of the learning vehicle after the learning outcome closes is a material reorientation unless completion remains authorized by another separately selected primary outcome.

Founding a rewrite, spin-off, replacement, upstream extraction, or other repository that materially continues the same premise, selected primary outcome, product promise, or unresolved experiment MUST carry forward, insofar as they concern that continuity, the consumed appetite, every outstanding terminal decision, and the materially relevant surviving evidence state: supported observations, unresolved premises, material limitations, and what prior evidence explicitly did not establish. A successor MAY discard code, language, architecture, data, or presentation that no inherited evidence claim or accepted obligation requires and begin with a local first path, but a predecessor mechanism or product claim that remained untested or unsupported remains unproven and MUST NOT gain authority merely because it is reimplemented. When adequate predecessor evidence no longer exists, record the uncertainty under `EVID-1`; no historical reconstruction, migration, or repair is required. A new repository, codebase, language, presentation layer, renamed mechanism, or cosmetically narrower slice does not by itself reset the inherited state. The carryover ends only when the acceptance authority records a materially different primary claim or terminal decision; another locally successful increment does not discharge the old decision. A genuinely independent product or pipeline split under `BOUND-2`, including a prerequisite producer selected under `PURP-5`, MAY receive its own appetite when it has its own human-selected primary outcome and abandonment decision, but the originating or parked consumer's state remains recorded and is not silently discharged.

These facts SHOULD occupy a few sentences in an existing mission or founding issue. They MUST NOT create a separate portfolio-management system or recurring reporting duty.

### PURP-2 — Outcome before mechanism

**Trigger:** A mission, issue, experiment, architecture, schema, workflow, or repository boundary is proposed.

**Rule:** Unless the human explicitly selects a mechanism itself as part of the outcome or as a binding constraint, the primary outcome MUST be stated in terms of the accepted human or consumer effect so that materially different mechanisms could satisfy it. A candidate mechanism remains replaceable until representative evidence shows that it produces the named effect through the actual authoring, testing, runtime, and presentation path at acceptable operational cost.

When a primary outcome's selected value lies materially in the experience of use rather than in implementing an expressly selected mechanism or artifact as the outcome, a genre, medium, scenario, feature set, artifact form, or system capability is not yet the human effect. The outcome MUST name the activity or experience the human seeks and the qualitative or inspectable sign that would make completing, continuing, or returning to it worthwhile. That sign MAY be private, idiosyncratic, changing, and human-judged; it need not be reduced to retention, session length, engagement telemetry, or a promise that motivation will remain stable.

When an experiential label bundles materially separable activities or effects and one of them would independently justify a capability class, persistent state, recurrent path, repository boundary, or substantial human or maintenance burden, the primary outcome MUST separate those claims enough to identify the distinctive activity loop and acceptance evidence for each. It MUST distinguish, as applicable, value from performing the activity, choosing to repeat or return to it, occasional recovery or assurance, and the availability, visibility, or imaginative possibility of an option. Evidence for one mode MUST NOT be treated as evidence for another: enjoyment of adjacent prose, choices, consequences, or presentation; enthusiasm for the concept; or imagination sparked by seeing possibilities does not establish demand to execute, preserve, or revisit those possibilities.

Availability, possibility, recovery, assurance, accessibility, or another optional-use effect MAY itself be the selected human effect when stated explicitly and accepted at proportionate burden. Such acceptance does not establish recurrent activity demand or authorize separate persistent or generalized machinery beyond the minimum required to deliver the selected effect. This rule does not require bureaucratic decomposition of an inseparable coherent experience or of a cheap, local, reversible convenience whose independent burden is immaterial.

Unless the accepted primary outcome is itself a complete declared model of a finite or formally governed domain, the domain's potentially representable facts, entities, interactions, populations, mods, products, and future cases are not a requirements inventory. A possible state or behavior earns no authority merely because a world or system could contain it; current accepted human effects and consumers determine what must be represented.

An admired product, published paper or research architecture, reference implementation, benchmark result, research prototype, genre convention, corpus, legacy system, successful artifact, or familiar content form proves only the effect actually observed under its conditions. It does not grant authority to its domain structures, stages, schemas, ontologies, pipelines, simulation models, repository boundaries, or other machinery. A direct technology constraint MAY be binding without validating the domain mechanism built with it.

### PURP-3 — One selected stream and path

**Trigger:** Selecting, reprioritizing or challenging work.

**Rule:** Distinguish outcomes, enabling work, stewardship and deferred possibilities. Humans select stream/bottleneck; work MUST follow its primary path. Elegance, usefulness, sunk work and testability grant no adjacent priority. Agents MAY challenge with evidence, never redefine for architecture. Collectively leaving the path violates `WORK-3`.

Across repositories, name next actual consumer use and unresolved handoff risk. Local criteria MUST NOT exclude/discharge it. Producer value does not prove downstream promises (`EVID-3`, `INT-1`); repeatedly reporting missing consumers does not remove the need.

### PURP-4 — Incumbent and least-complex credible comparison

**Trigger:** A candidate capability, replacement, rewrite, redesign, migration, additive system, or allegedly new gap-filler is founded or materially activated.

**Rule:** Before work begins, name what currently happens in the candidate's absence. The incumbent may be an existing system, manual routine, skill, informal notes or memory, external product, ad hoc human judgment, deliberate non-action, or genuinely nothing. When the incumbent has relevant observed effects, declare which effects the candidate must preserve or improve and which dimensions bear the claim, such as accepted-output completeness, semantic quality, maintainer burden, elapsed effort, recovery, or cost.

When the candidate introduces or materially expands a capability class, persistent state model, simulator, ontology, DSL, framework, generalized evaluator, or other substantial machinery, also name the least-complex credible intervention that could plausibly produce the named effect or address the observed failure on the same representative real case. That intervention MUST be exercised before the candidate gains dependent infrastructure or materially expands its semantic surface. A bounded candidate probe MAY run first or in parallel when needed to make the comparison fair, but it does not waive the comparison.

A desired capability such as memory, retrieval, planning, search, autonomy, or simulation does not select its storage, indexing, ranking, planner, or state-model mechanism. Compare candidates at the level of the named consumer effect and current real case. A mechanism MAY be selected directly as a learning outcome or binding constraint under `PURP-2`, but implementing it does not establish fitness for a broader product claim.

The simpler intervention MUST NOT be a straw man, be denied inputs the real task supplies, or be held below the fidelity needed to exercise its causal hypothesis. One good-faith simpler comparator is the default; this rule does not require an exhaustive tournament of alternatives. A separate competitive comparator for choosing the mechanism is unnecessary only when the human selected the mechanism itself as part of the outcome, an external formal or safety constraint requires it, or no materially simpler credible intervention exists and the reason is recorded before substantial implementation. Selecting or requiring the mechanism waives only that competitive-adoption comparison: it does not establish that feedback, training, ranking, search, adaptation, optimization, or repeated selection caused the claimed improvement, and it does not permit the mechanism's guiding objective or exposed evaluation surface to validate a broader effect. Any such causal-effect claim remains subject to `EVID-4`.

Preserve lawful, safe, inspectable incumbent evidence even when it is quarantined from migration, ontology mining, or normative authority. At appetite exhaustion and before adoption or material extension, compare the candidate with the incumbent and the least-complex credible intervention on the declared dimensions. Material underperformance falsifies the comparison claim and triggers reject, park, narrow, or reframe; it MUST NOT manufacture an automatic repair backlog. When the incumbent is deliberate non-action or genuinely nothing, record its observable effects and costs rather than fabricating comparative evidence. If comparable evidence cannot be obtained, state the limitation and keep the claim experimental rather than claiming superiority.

### PURP-5 — Prerequisites are products when the promise depends on them

**Trigger:** A selected primary promise depends on an upstream product, design, specification, ruleset, policy, semantic contract, catalog, corpus, ontology, benchmark, evidence base, model, dataset, structured artifact, external capability, or other prerequisite that is unavailable, unaccepted, or materially volatile at the required fidelity and coverage. Volatility is material when unresolved or expected changes could alter the dependent scope's meaning, decisions, behavior, content, interfaces, presentation, tests, operation, or maintenance burden.

**Rule:** Judge sufficiency against the human-selected current product promise. A convenient probe, fixture, presentation sample, or renamed slice MUST NOT silently narrow that promise. Treat a missing prerequisite as a real research, design, data, content, or capability product with its own acceptance boundary. Before dependent production relies on it, name the accepted version and exact upstream decisions or semantic contract on which that production may rely.

The prerequisite release MUST be authoritative and complete for the declared consumer scope, including dependencies and interactions material to that scope. It must be the intended source for that work, not a sample or knowingly inadequate stand-in. It MUST expose coverage, provenance, known omissions, contamination status, authority, validity limits, failure behavior, supported consumer decisions, and maintenance burden. Use existing artifacts proportionately. An omission is compatible with readiness only when it cannot materially alter the accepted downstream scope. If the downstream promise still intends or expects to exercise a larger domain, coverage of a convenient subset does not establish readiness for that larger promise.

The human MAY explicitly select a smaller coherent current product whose defining effect and complete bounded task can be judged. A complete version of that product can become authoritative while larger ambitions remain unproven and inactive. Such selection changes the current promise; it is not an agent's implementation shortcut. It does not require permanent stability or completion of every future feature. Later rule or contract changes create a new version, identify the affected downstream behavior, and require proportionate revalidation. Expected iteration alone does not forbid implementation of an accepted complete bounded version.

Producer and consumer work MAY proceed in a coordinated bounded iteration. The iteration MUST name the real consumer, exact candidate or accepted source, current consumer scope, material unresolved questions, appetite, and next observable result. Where adequacy of the prerequisite or handoff is itself uncertain, a bounded implementation or probe in the intended consumer MAY test it before full adoption. Keep candidate inputs and unaccepted behavior explicit and isolated from authoritative production and unsupported dependent work. A runnable proposal can expose missing rules and demonstrate proposed resolutions; it cannot silently settle them. Apply `AGENT-5` to investigation, reversible proposals, and human decisions, and `INT-1`–`INT-2` to custody and feedback.

An accepted bounded source release permits production implementation of exactly that scope. A provisional implementation may be retained and promoted after the required source, behavior, evidence, and publication decisions are satisfied; no duplicate rewrite is required merely because the first implementation was an experiment. Consumer findings that alter upstream semantics return for upstream resolution and a new accepted version before adoption. A frozen source revision supplies a stable target; it does not itself prove adequacy. The actual-path evidence required by `EVID-3` remains due.

Park production, presentation, UI, assets, packaging, release, analytical work, and other dependents beyond the accepted scope or reliant on unresolved governing semantics. The coordinated iteration does not authorize general infrastructure, a portfolio of packages, or unrelated consumers. High-fidelity decision-only probes remain permissible under `EVID-2` and `EVID-4`, with their own limits; attractive output and motivation do not relax readiness requirements.

A producer MAY independently be active when its artifact is itself the human-selected primary outcome for a named next use. That selection does not establish the consumer's readiness or authorize speculative fields, services, or machinery. Keep the distinction between independent producer value and an unproved downstream promise visible under `PURP-3`.

### PURP-6 — Supply and demand close in both directions

**Trigger:** A shared or published capability, operation, schema field, evidence form, contract obligation, interface, ecosystem surface, or cross-repository dependency is proposed or maintained.

**Rule:** Every maintained supply MUST have at least one current accepted consumer at the fidelity supplied. Every accepted demand MUST resolve to an actual producer, representation, provenance or derivation path, authority, oracle, and failure behavior at the fidelity consumed. A roadmap, architecture, issue, schema, public ecosystem, or future pipeline created to exercise proposed supply does not establish demand.

An accepted catalog or corpus MAY be demanded at its declared coverage as a whole; each record does not need an independent consumer. Where identities and relationships are finite, use the smallest local mechanical check that closes both directions. A shared surface that loses its last accepted consumer MUST be removed, parked, made local and private, or explicitly adopted as a primary product in its own right.

## Work governance and stopping conditions

### WORK-1 — Governed work selection

**Trigger:** Activation/readiness, scope expansion, adjacent work, or foundational, shared, governance, migration, substrate or dependency-bearing work.

**Rule:** Before activation, name selected outcome, bottleneck/earliest missing prerequisite and smallest acceptance-bearing primary-path increment. Existing authority MAY cover direct work. Enabling, research, governance, migration, substrate or adjacent work MUST record in its issue/decision the bottleneck, primary-path consequence, current consumer/blocked human decision, displaced direct work and stop/return condition.

If these cannot be named, defer or park. Correctness, defects, testability, code, labels and detailed issues alone do not authorize selection.

### WORK-2 — Bounded, removable speculation

**Trigger:** An unproven premise, temporary bootstrap, candidate architecture or experiment.

**Rule:** Bound appetite by decision, semantic surface and human burden; artifact counts do not bound expanding obligations. Apply `EVID-4` before experiments look like primary progress/readiness. Constraining temporary bootstraps MUST name owner, purpose, allowed consumers, prohibited dependents, appetite, replacement/removal and expiry; inertia MUST NOT grant consumers/authority.

Implementation permits experiment, not adoption. External reliance requires human inspection of real end-to-end results and explicit adoption of the exact premise/scope. Human doubt permits only cheapest bounded falsification, without fan-out. Failure, abandonment or unadopted expiry removes the premise from active context, dependencies and default publication; independently justify survivors. Salvage is optional.

Before its defining effect is accepted, provisional behavior MUST NOT acquire product authority through broad golden/snapshot locks, mutation campaigns, certification, exhaustive conformance or preservation governance. Narrow tests, replay, diagnostics and invariants for safe, inspectable, reproducible decisions are allowed, as are proportionate security/privacy/corruption, irreversible-effect and authority protections. Later hardening follows demand, burden, primary path and governed lifetime.

### WORK-3 — Repository allocation tripwire

**Trigger:** Completed work accumulates without an acceptance-bearing primary-path increment.

**Rule:** A repository MAY accept its own human-observable unit and consecutive limit; otherwise **three completed work items** applies. New issues, lanes, cleanup, enabling success or crossing repositories with the same unresolved handoff/premise do not reset it or grant allowance.

Intermediate acceptance or acknowledged limitations do not reset an unexercised consumer effect. Retain the unmet premise; exclusions, deferred integration and “non-blocking” deficits do not discharge it. A producer can earn independent acceptance without proving its consumer.

For experiential, interactive or authoring outcomes, reset requires human-inspected representative use through the affected path, yielding the named effect or resolving its earliest blocking premise. It resets only what it exercises under `PURP-2`/`EVID-3`, not adjacent enjoyment, presentation, available possibilities or unrelated success. Isolated actions, components, checks and refactors do not establish unexercised loops, voluntary recurrence, return, recovery or assurance.

At the limit, select no further work until the human explicitly continues, reprioritizes, reframes or parks. Continuation MUST justify the direction and renew scope, appetite and allocation bound. Sessions are not an alternative count.

### WORK-4 — Demand-null tripwire

**Trigger:** A capability's accepted benefit claim depends on encountering positive cases, producing interventions, or causing consumer consequences, and real-input runs produce none.

**Rule:** Count only qualifying opportunity runs. Before interpreting a run, declare the exposure unit or basis that makes a positive case plausible at the frequency assumed by the claim. A negative result explicitly accepted as the product—such as assurance, validation, or a clean bill of health—is not demand-null merely because it causes no change. Rare-event capabilities SHOULD ratify a suitable exposure unit or threshold before runs begin.

Unless another threshold was accepted in advance, the **second consecutive demand-null qualifying run** stops further work on the capability and on substrate whose only accepted consumer is that capability. The existence or assumed frequency of demand becomes the claim under test and requires human reprioritization. Correct behavior and honest negative reporting do not reset the count; they make it informative.

### WORK-5 — Terminal decisions freeze the repository

**Trigger:** Appetite ends, or accepted work reaches a terminal-decision gate.

**Rule:** All discretionary repository work MUST stop until the exact decision is durably recorded; changing lanes, issues, cleanup or refactoring cannot evade it. Only necessary decision evidence, exact-state preservation/quarantine or minimal imminent data-loss, credential-exposure or immediately exploitable security containment MAY continue. Record emergency containment separately; the pending decision remains next. Extension requires fresh human reprioritization with scope and appetite.

Count extensions of the same pending decision, including successor repositories (`PURP-1`). Each MUST name missing evidence, why accumulated evidence cannot decide, and the exact scope to obtain it. At most **two consecutive extensions** follow the original appetite. After the second, no third: the human MUST adopt, reject, narrow, park or reframe on existing evidence. Only a materially different primary claim or terminal decision resets the count; new repositories, lanes, mechanisms, presentation, cosmetic narrowing or local success do not.

### WORK-6 — Consumers drive substrate; future compatibility stays cheap

**Trigger:** Storage, validation, rendering, audit, release, governance, or another substrate is proposed for a capability or consumer that has not yet executed its smallest real slice; or a future cross-cutting requirement is cited.

**Rule:** The first real capability execution or existing consumer contract MUST drive only the minimum local substrate needed for that slice. Disposable scaffolding MAY exist inside the slice. A general or shared substrate MUST NOT run first and alone; without the consumer it derives requirements from itself and becomes a different project requiring re-ratification, its own primary outcome, consumer, appetite, and removal path. A seam in storage, validation, rendering, audit, or release does not prove the upstream capability that never ran. Skipping a declared precondition or derivation path is a mechanism substitution and requires re-ratification.

A future requirement MAY justify a cheap, reversible compatibility choice or smoke test when later retrofit would be unusually costly. It MUST NOT justify exhaustive implementation, validation, or supporting infrastructure before real demand exists.

## Evidence, experiments, and generalization

### EVID-1 — Founding memories are premises, not authority

**Trigger:** A mission or acceptance program is motivated by a remembered episode, prior incident, recurring frustration, or origin story.

**Rule:** Check the account against surviving version history, artifacts, issues, logs, outputs, or other inspectable records before it authorizes dependent architecture. State what the record supports. A contradiction requires the mission to be re-derived before dependent work continues. If adequate evidence no longer exists, mark the claim unverified and keep work resting on it experimental. Memory may motivate investigation; it MUST NOT manufacture architectural authority.

### EVID-2 — Falsify dependencies in order; research serves a decision

**Trigger:** Important premises are uncertain, or research or experimentation is proposed.

**Rule:** Order uncertainties by dependency and attack the earliest unresolved premise whose failure would invalidate later work, using the cheapest evidence adequate to decide it. A downstream experiment MAY run ahead only when it is bounded and can materially change whether or how its prerequisite should be pursued. A risk gains no priority merely because it is concrete or satisfying to engineer.

Unless a research artifact is itself the accepted primary outcome, the experiment MUST name the blocked decision, claim, sufficient evidence, appetite, and possible terminal outcomes before work. Whenever an experiment reaches appetite exhaustion, is offered to justify adoption, continuation, narrowing, or reorientation, or is about to be relied upon by dependent work, durably record in an existing authoritative artifact the claim tested; the materially relevant revision, input, configuration, evaluation, and output identities as applicable; representative observations needed to support or challenge the conclusion; uncertainty and limitations; what the evidence does and does not establish; and the resulting adopt, reject, park, narrow, or reframe decision.

Preserve only the evidence and identities proportionate to that decision and a plausible next experiment. Bulk or generated outputs MAY remain outside Git when a representative sample or stable, retrievable, integrity-checkable binding is sufficient. A disposable decision-only probe need not become a maintained product, and this rule creates no repository-abandonment ritual, publication-grade report, experiment registry, recurring status duty, or obligation to reconstruct missing historical evidence. A fixed research-gate sequence, unanswered-question inventory, issue count, or closure history is not a value stream and creates no duty to continue.

### EVID-3 — Artifact before system

**Trigger:** General systems, workflows, frameworks, catalogs, large generated networks, semantic scale-up or external fidelity claims.

**Rule:** First produce/materially revise a real primary artifact or substantial risk-bearing slice through the proposed path. It MUST exercise relevant authoring, semantics, tests, runtime and presentation with fidelity adequate for material visual, motion, latency, readability and interaction judgments; not misleading debug abstractions.

Experiential, interactive and authoring slices MUST allow bounded actual task completion/intentional termination; isolated operations are probes unless selected outcomes. Each `PURP-2` loop justifying material machinery/burden MUST run before expansion. Observed options and adjacent enjoyment prove only experienced effects.

A diagnostic run MAY create credible rare/optional/unfamiliar opportunity. Recurrence requires later ordinary use with informed voluntary repetition/return; forced repetition is insufficient. Non-use/rejection is evidence against demand, not repair authority. For selected recovery, assurance, accessibility, availability or possibility value, name/expose the condition; humans MUST use it or, for availability/assurance, inspect sufficient real consequences/burden. This accepts that mode, not recurrence or broader persistence.

Supported effects remain evidence when others fail/remain unproved. Unsupported machinery MUST stop under `WORK-5`; agents MAY recommend simpler preservation without silently changing outcomes.

Before recurrent adoption/material expansion of experiential, interactive, authoring, configurable, reusable or engine-like paths, materially different second use MUST pass through the same accepted authoring/configuration, runtime and presentation. It MAY add ordinary content/configuration, bounded consumer adapters at existing seams or in-scope repairs. New capability classes, core ontology levels, case-only shared schemas, required services or synchronized unrelated changes do not prove recurrent fitness. Completion-blocking/burden-hiding defects require reruns after repair; lesser defects still count against fitness. Experiential recurrence requires voluntary election.

Periodic reuse, use beyond fresh memory, handoff or interruption recovery also needs return use from durable state without conversational/unauthored-memory reconstruction. Clean handoff MAY replace elapsed interruption with equivalent burden exposure. Single-use work without recurrence claims needs neither extra run and MUST NOT generalize merely to satisfy them. These runs prove only that path; generality needs `EVID-6`.

Authored specifications/configurations/packages promising downstream use MUST reach their intended consumer before bulk/repeated authoring or supporting machinery expands. The consumer MUST use actual pinned material as claimed, exercise difficult behavior against independent expectations and expose consequences. Interactive promises need real use; headless output proves only executed behavior. `INT-1` distinguishes specification implementation/runtime data; `EVID-7` allows bounded design exploration. Without consumers, next establish the path or obtain narrowing/parking/abandonment; independently selected producers may continue (`PURP-5`). Reviews/joins/models bypassing input or consumer prove no handoff.

Work MUST NOT scale to hundreds of interdependent records/rules/assets or equivalent semantic surface before representative effects at acceptable authoring/validation/maintenance cost. Legacy success proves its path only.

### EVID-4 — Evidence bears only its claim

**Trigger:** A test, prototype, benchmark, review, simulation, metric, component or probe is offered as evidence.

**Rule:** A **seam probe** proves real input crosses a boundary to output. An **acceptance-bearing vertical slice** proves expressibility, production, validation and operational sustainability of the smallest accepted demand, with required configurations, populations, controls, scale, provenance, refusal and verdict. Disconnected demos MUST NOT accumulate into wider fitness.

Source review, structure, coverage, expectations, consumer execution and human use each prove only their property. Joins are not entailment; models prove their behavior, not another consumer's; receipts record, not create evidence. Expectations MAY use source examples, calculations, invariants or justified models; implementation MUST NOT supply its own expected answer.

When priority/adoption depends on it, classify **product-bearing** versus **decision-only** in existing work before implementation or counting output as progress. Decision-only work MAY be disposable/high-fidelity; acceptance covers its named claim/decision only. Polish, extent, expense or resemblance MUST NOT imply downstream progress/readiness/reuse. Promotion separately requires real-input, authority, acceptance, boundary and publication compliance, not inertia.

Presentation/tangibility/enthusiasm/motivation/communication/emotional orientation proves only its selected effect. Proportionately bounded acceptance MAY resolve no product premise (`PURP-1`/`PURP-2`). Without separate design/acceptance for another claim, it MUST NOT imply upstream testing/stability/readiness or reusable machinery. State appetite, prohibited dependents and parking/disposal; promotion follows `PURP-4`/`PURP-5` too.

Replay proves pinned reproducibility, not causality/robustness/stability/diversity/generality; use proportionate controls, counterfactuals or independent variation (`SIM-2`, `EVID-6`). Distinguish desirable causal sensitivity from incidental variation. Statistics/simulation MAY diagnose/compare/falsify, never manufacture meaning, taste, fun, desirability or acceptance. Tests need valid oracles. Self-created evidence/consumers/obligations prove consistency at most, not need, demand, fitness, authority or causal credit.

### EVID-5 — Real inputs earn local paths

**Trigger:** Substantive architecture or implementation.

**Rule:** Start from accepted real input at consumed fidelity. Synthetic or hypothetical cases MAY test mechanics/falsify, but MUST NOT authorize canonical representations, general subsystems, architecture or backlogs.

One input earns only a local replaceable path; broader promises need `EVID-6`. Bounded, correct, safe, visible special cases, explicit branches and duplication MAY precede abstraction when cheaper to replace. Agents MUST NOT generalize merely to remove repetition before different real pressure earns it. This excuses neither inconsistent authority, hidden coupling, security/correctness defects nor duplicated logic whose current divergence is the bottleneck. Local clarity and repair remain required.

### EVID-6 — Generality follows different pressure

**Trigger:** Cross-product/domain/repository/independent-consumer reuse or a shared constraining contract.

**Rule:** Adoption MUST use **three materially different real design cases and one further holdout not used in design**; count alone is insufficient. Cross-product claims need different named products/independent consumers; other generality needs independently selected product-bearing artifacts or current accepted consumers. Synthetic variants, future or manufactured products and decision-only probes do not qualify. Target-specific language, fields, defaults, fixtures, sources or assumptions block adoption until removed/quarantined and the holdout re-evaluated.

Repositories MUST NOT become homes for imagined product families merely for reuse. Keep product work local and independently abandonable unless `BOUND-2` allows co-location. Repeated real pressure MAY earn an explicitly versioned shared extraction under boundary/integration rules. Visibility, licensing or confidentiality affect custody, not demand, generality, co-location or case requirements; justified shared code may be public or private.

Fewer cases require an intrinsically finite problem, fully governing external formal standard or scoped waiver. Ordinary local refactoring, helpers and deduplication with mechanically evident behavior and no broader/published reuse claim are exempt; they stay local/reversible until wider pressure.

### EVID-7 — Broad search is judged by omission resistance, not traversal volume

**Trigger:** The objective includes comprehensive consideration, broad search, novel combinations, resistance to obvious defaults, or an explicitly selected exploration process.

**Rule:** Define representative hard cases and judge the search by useful discoveries, omissions, diversity, decision value, and cost. Literal traversal, hierarchical search, retrieval, independent-agent search, adversarial sampling, and hybrids are candidate mechanisms. The human MAY make complete consideration of a finite catalog or another breadth commitment a binding part of the task. An agent MUST NOT replace that commitment with selective retrieval, sampling, familiar defaults, or early convergence because those methods are cheaper or already yield a plausible winner.

For a selected full sweep, bind the source snapshot and account for every entry at sufficient semantic depth to assess relevance to the accepted question, including supporting roles and potentially useful combinations. Read richer source material where a short record is insufficient. Explicit exclusions require grounds within the accepted search boundary; unfamiliarity, lack of a current implementation, provisional status, missing routing labels, or absence from guidance are not automatic grounds. Preserve pertinent poor-fit conditions and contrary evidence. A concise coverage account, including compact grouped reasons where justified, is sufficient; a separate essay or human approval for every rejected entry is not required. Mechanical census checks verify identity coverage only, not quality of consideration.

When combinations bear the goal, explore materially different coherent combinations and explain their division of labor, interactions, support needs, and risks. Individual fit does not establish joint fit, and an individually weak mechanism may have a useful supporting role. Include the incumbent and least-complex credible alternative where applicable. Full consideration of entries does not require exhaustive enumeration of every combination. Distinguish source-backed knowledge from author-owned novel or hybrid hypotheses; neither catalog pairings nor an impressive proposal establish experienced quality.

Use prospective comparison criteria and a proportionate evidence floor. State important trade-offs and preserve an inconclusive result when the evidence cannot support a verdict. A human may select a direction by judgment while its untested properties remain unproven. Agents MUST complete the accepted exploration and comparison commitment, or obtain an explicit change to it, before closing the stage or treating a preferred candidate as the accepted winner. A bounded task may contain these activities without a separate skill, file, ledger, or approval gate for each phase.

Touching every identifier establishes traversal only. Ledgers and report volume MUST NOT substitute for semantic consideration, omission resistance, or useful comparative evidence. The current representation or implementation MUST NOT silently filter the creative domain; a genuinely binding feasibility constraint may do so only as part of the accepted task.

### EVID-8 — Operational burden is part of fitness

**Trigger:** A recurrent workflow, tool, representation, test program, or manual review step is evaluated for adoption.

**Rule:** Fitness includes whether the actual maintainer can repeatedly author, inspect, debug, test, localize, release, recover, and maintain through the proposed path at acceptable burden. A workflow the maintainer will not accept is unfit even if isolated functions work.

Technical opacity of source files is not itself unfitness when their intended technical consumer can use them and the human decision surface satisfies `AUTH-3`. The relevant human burden is judging meaningful choices and consequences, not decoding every technical record. If the human reports abandoning review, approving merely to move past an incomprehensible packet, or being unable to understand a requested decision, the agent MUST simplify or demonstrate that decision surface before issuing another substantially equivalent request. It MUST NOT infer technical verification or experienced fitness from such approval. Preserve actual accepted authority while bringing unresolved factual claims and burdens back into the next intelligible decision; this creates no automatic historical re-audit.

For a recurrent authoring, configuration, activation, or return path, adoption or material expansion MUST declare, in an existing authoritative artifact, the human-burden units that matter for that product and an acceptable ceiling. Repository-specific units MAY include required records or fields, workflow transitions, services or credentials, manual generation or review passes, code changes, elapsed setup effort, information that must be remembered rather than discovered from durable state, and, for agent-mediated work, interruptions, independently governed judgments, and local technical context the human must reconstruct. No universal time, field-count, screen-count, service-count, or interruption threshold and no recurring measurement system are required. Representative use that exceeds the ceiling is appetite exhaustion under `WORK-5`; further expansion of the affected path remains frozen until the acceptance authority simplifies, narrows, splits, parks, or explicitly continues under `WORK-5` with a renewed scope and burden appetite.

A nominally optional field, subsystem, or workflow counts toward this burden when the standard path, schema, documentation, examples, quality claim, or downstream behavior reasonably presents omission as invalid, incomplete, materially inferior, or unsafe, or when representative use shows repeated completion, just-in-case authoring, postponement, or avoidance because omission cannot be judged confidently. Subjective preference, theoretical usefulness, or sunk cost alone is insufficient. The smallest accepted useful path SHOULD remain discoverable and operable without unrelated optional systems; when it does not, recheck operational fitness here and co-location under `BOUND-2` through `BOUND-4`.

A request for manual testing or evaluation MUST name the decision affected, why automation or a smaller proxy is inadequate, expected burden, and stopping condition. Where repeated human judgment is a binding oracle, the appetite MUST also declare and count a cumulative ceiling in the unit that actually consumes human attention, such as launches, playtests, listening passes, reading passes, review batches, selection rounds, or another declared unit. Accepted, rejected, and inconclusive evaluations all count; an appetite denominated only in developer time is insufficient when human judgment is the iteration loop. An agent that cannot adequately exercise or perceive the artifact MUST treat that human loop as a first-class resource constraint. A one-time acceptance that is not an iteration loop does not require a separate evaluation ceiling. A human decision request prohibited by `AGENT-5` is avoidable burden, not prudent oversight. Representation-created obligations count as cost under `REP-2`; they are not primary progress.

### EVID-9 — Narrowing rechecks the surviving real domain

**Trigger:** An accepted decision narrows what evidence, input, occurrence, consumer context, or subject matter qualifies.

**Rule:** Reapply the complete narrowed test before dependent work continues. For a finite declared inventory, test every member and record all surviving instances. For an open-ended domain, test all accepted real inputs plus a bounded representative challenge set, state exactly what was examined, and leave the remainder unproven. A synthetic example created merely to survive the narrowing is not real demand.

If no real instance survives, the premise that qualifying subject matter is presently available is falsified and dependent work stops pending new evidence or reprioritization. If instances survive, claims and work are limited to that scope. Individually reasonable refinements do not waive the recheck.

## Semantic evaluation and representation

### SEM-1 — Decision-bearing semantics must be decidable from the interface

**Trigger:** Deterministic code, a heuristic, or a model is asked to block, promote, reject, score, accept, or publish based on meaning-rich material.

**Rule:** Verify that the required category is decidable from the declared interface. Formally structured inputs or an explicitly ratified controlled language with a finite grammar and independent oracle MAY support a mechanical verdict. A prose criterion inside JSON, YAML, a database row, or another machine-shaped envelope remains prose.

Regexes, token rules, dictionaries, fuzzy matching, embeddings, classifiers, morphology, and handcrafted parsers MAY provide explicitly incomplete warnings or retrieval assistance. They MUST NOT silently control semantic authority over unrestricted prose. A probabilistic semantic classifier MAY be the selected product or experiment only with representative evaluation, uncertainty, calibration or failure behavior, and explicit authority limits; its output is not proof.

A top-ranked, nearest, highest-scoring, or otherwise preferred candidate proves only relative ordering among the candidates considered. It MUST NOT be treated as adequate, relevant, valid, legal, possible, or sufficient unless an independent accepted rule or oracle establishes that category. Where no candidate satisfies it, the interface and runtime MUST preserve the applicable no-match, abstention, clarification, refusal, deliberate non-action, or explicit failure outcome rather than force a winner.

If required information is absent from the interface, implementation MUST stop and propose an explicit field, semantic-review receipt, adapter, reduced promise, or controlled language. Heuristic reconstruction of missing information is not an ordinary implementation detail.

### SEM-2 — Semantic review is revision-bound and inspectable

**Trigger:** Unrestricted prose or other meaning-rich material must be interpreted for a decision-bearing purpose.

**Rule:** Place interpretation with the named human or reasoning-agent consumer and require a structured, revision-bound review receipt. The receipt MUST expose the exact source revision, review policy, reviewer or model identity, evidence, verdicts, reasons, uncertainty, authority label, validity limits, and recheck conditions. These may be concise named entries in an existing issue, review, or decision. A separate file, schema, approval form, or repeated record per source item is required only when a current consumer needs it. A review of implementation fidelity must state whether it examined the actual source-to-consumer path or only an intermediate representation.

Deterministic tooling MAY validate receipt shape, freshness, reference closure, coverage, and consequences formally entailed by explicit verdicts. It MUST NOT reinterpret the source to manufacture or overrule the semantic judgment. Provisional semantic proposals MAY feed further reversible proposals; they MUST NOT mutate canonical state, trigger destructive or external effects, close an authority gate, or become a publication claim without the named acceptance authority.

### REP-1 — Authoritative records and derived renders

**Trigger:** Structured records, prose documents, UIs, reports, or executable artifacts may carry the same decision-bearing meaning.

**Rule:** Choose representation for the actual consumer and task. Prose, structured content and parameters, examples, tables, or a deliberate combination MAY carry authoritative meaning. A compact structured record may itself be the accepted product; a duplicate human-facing rulebook or render is not required merely to make a pipeline appear complete. Human decisions still require the intelligible surface in `AUTH-3`. Long prose inside a structured envelope remains prose, and renaming it as tokens does not supply executable semantics.

Identify which source owns each material behavior or fact. Any derived render MUST preserve that meaning, remain reproducible from its source and declared transformation, and introduce no exclusive decision-bearing content. Views may summarize and link to technical detail; they must expose material choices and limits accurately. A rule, exception, or parameter discovered only in a presentation or implementation is a proposed source change when the source owns it. Resolve it through the owning authority before adoption.

An authored specification consumed by a human or coding agent MAY define behavior through precise language and worked examples, with product-specific code supplied by the execution consumer. This is a valid implementation contract and does not require a general interpreter, controlled language, or behavior encoded entirely as data. Distinguish it from a runtime-data contract in which software must evaluate supplied behavior without new product-specific implementation. Claim the latter only when the actual evaluator and its semantics are provided and exercised. Apply `INT-1` to evidence for either contract. A named procedure is adequate for specification use only when the declared source explains its behavior sufficiently; its name alone is not an implementation.

The consumer may choose internal algorithms, indices, identifiers, and data structures consistent with the accepted behavior. Those choices do not become upstream design questions merely because a fixture gives them concrete values. Missing or materially different domain behavior remains a source-semantic question under `INT-2` and `AGENT-5`. A source-driven implementation can be reproducible through pinned source, code, inputs, and build procedure; automatic code generation is not required unless it is the selected contract.

Begin with the minimum provisional structure needed for a coherent creation or revision. Legacy renders provide evidence, hypotheses, and cases; their organization does not define the new ontology. New creation, legacy migration, and downstream projection remain independent claims requiring independent justification and acceptance. Accepting one record-backed artifact does not adopt its schema or demote accepted legacy artifacts.

### REP-2 — Independent representation oracles

**Trigger:** Representation, specification, state/consequence model, migration or renderer constrains work/consumers, even with product-specific naming.

**Rule:** Before adoption, a risk-bearing slice MUST prove real authoring/revision → inspectable proposed record/state → human-accepted artifact/effect in consumer form: structured, rendered or executable. No extra representation without a consumer; claimed benefit MUST improve an actual task.

Explicit state/consequences require current accepted cases needing narration-independent persistence, coordination, deterministic or externally inspectable consequences with uncertainty/failure, recovery or another named consumer. Possible domain facts create no demand. Start with minimal local state/transition authority; generality needs `EVID-6`.

The candidate MUST NOT select/filter its validation inventory, scope, fixtures, hard cases or oracle. Derive these independently from accepted tasks, semantic/authority distinctions, preservation obligations and sources, regardless of expressibility. Before adoption, map every finite-target obligation/deficit; for open targets disclose challenged inventory/unproved remainder, including hard/breaking cases. Identity coverage/receipts prove no source acceptance, research, freedom from contamination or fitness.

A disposable local model MAY precede proof only without adequacy/reuse claims, adopted external dependents or supporting infrastructure; it stays replaceable and cannot choose its oracle/scope. `PURP-5` consumer probes gather evidence, not expansion authority. Representation-created synchronization, reconciliation, invalidation, migration, authoring and validation burden counts against fitness. Parsing old renders does not prove new authoring.

### REP-3 — Structural level and semantic prerequisite satisfaction

**Trigger:** A proposal is described as structural, or a named artifact is claimed to satisfy a prerequisite.

**Rule:** A structural proposal MUST name the level it structures: textual syntax, domain facts, evidence and authority, workflow state, transport, or storage. For a semantic decision system, domain facts and warrant structure MUST be considered before textual parsing machinery. Changing an accepted structural level, decision-bearing or shared interface, or authority or lifecycle ownership seam in a way that alters the accepted contract, consumer obligations, or capability boundary requires re-ratification. Local decomposition, internal interfaces, and ownership changes within those accepted boundaries are implementation judgment under `AGENT-5`, not separately ratified architecture.

A filename, title, directory, status token, schema type, passing validator, or nominal artifact class never satisfies a prerequisite by identity alone. At the fidelity the consumer uses, the candidate MUST provide the required content, coverage, semantic distinctions, provenance or derivation, authority state, evidence, validity limits, and failure behavior. A differently derived artifact placed under the expected name cannot borrow the prerequisite's authority. When the derivation path is part of the accepted consumer contract, substitution requires explicit re-ratification before dependent work activates.

## Capability and repository boundaries

### BOUND-1 — Pipeline inventory and founding stop

**Trigger:** A repository is founded, materially expanded, merged with another capability, or given a new independently invocable path.

**Rule:** Enumerate each candidate pipeline as:

> accepted real inputs → decision and semantic authority → primary output and side effects → acceptance and failure behavior → maintainer and consumer → release, parking, removal, and abandonment decision

Discovery of a second independent pipeline MUST stop implementation and require the agent to state plainly that the repository is probably taking on too much. Shared subject matter, schemas, storage, users, source material, final artifacts, code reuse, or a possible feedback loop do not rebut the split. Any proposal to rebut the separate-repository presumption MUST be decided under `BOUND-2`. A concise section in the existing founding issue or PRD is sufficient; no pipeline registry is required.

### BOUND-2 — Independent abandonment requires a separate Git repository by default

**Trigger:** A distinct pipeline has its own coherent consumer, acceptance boundary, failure behavior, maintenance cadence, release path, or abandonment decision.

**Rule:** The pipeline MUST live in a distinct Git repository unless one of the two co-location conditions below is satisfied. A directory, package, command, feature flag, skill family, separate CI job, or internal activation boundary is not sufficient isolation when shared foundations, schemas, identifiers, agent context, work control, release mechanics, or cleanup obligations still couple the lifecycle.

Co-location is permitted only when one genuinely indivisible product owns the lifecycle or repeated real work demonstrates that changes across the boundary must be atomic. A Rust core, TypeScript interface, CLI, persistence layer, and administrative editor MAY remain together when they jointly form one accepted product and one abandonment decision. An editor used only to maintain the repository's own data product MAY remain with it when the editor has no coherent acceptance, release, maintenance, or abandonment lifecycle apart from that product. Technical possibility, anticipated convenience, shared topic, or downstream feedback is not enough.

Co-location remains contingent on ordinary use of the smallest accepted useful path not requiring traversal or completion of unrelated optional systems. If setup, activation, debugging, or return to one capability materially depends on satisfying another capability with an independent lifecycle, re-evaluate the split under `BOUND-4` and the burden under `EVID-8`.

If removing one capability disables unrelated work or requires a repository-wide semantic scrub, the co-location decision is falsified.

### BOUND-3 — Boundaries follow authority and lifecycle, not workflow stages

**Trigger:** A capability, module, service, skill, or repository boundary is proposed.

**Rule:** A capability SHOULD hide a coherent authority, representation, policy, or decision that can change independently. Split when ownership, triggers, invalidation rules, retry behavior, side-effect classes, definitions of success, expertise, maintenance cadence, or lifecycles differ; keep work together when those are genuinely shared.

A chapter, guide section, workflow stage, repeated handoff, or artifact noun is not by itself a sound boundary. A coordinated task across existing repositories does not merge their authority or lifecycles. Conversely, those independent lifecycles do not require a fresh human prompt for every already authorized transfer; `AGENT-1` governs bounded coordination. Data or corpus stewardship, independently useful upstream decision support, downstream transformation or production, runtime execution, analytical feedback, and a release service with its own consumers or maintenance are presumptively separate lifecycles unless evidence demonstrates otherwise. Ordinary packaging and release of one coherent product remain technical layers of that product.

Long non-use of a capability is not by itself proof of an independent lifecycle. It becomes a boundary finding when the substantial capability also has its own reason for change, authoring or maintenance burden, acceptance behavior, or credible parking or abandonment decision, and repeated ordinary use of the active capability does not require it.

### BOUND-4 — Change amplification and removal test the boundary

**Trigger:** A substantial capability is proposed, or a supposedly local change unexpectedly spreads.

**Rule:** Inspect the complete footprint across production code, shared libraries, schemas, identifiers, storage, migrations, documentation, agent and skill context, work control, CI, releases, reports, generated data, and governance. An independent capability SHOULD be removable by deleting its owned surface plus a bounded set of explicit integration points.

If adding, changing, parking, or removing it requires synchronized edits across unrelated surfaces, a universal schema or identifier registry, repeated reconciliation, or a repository-wide residue sweep, treat the fan-out as an architecture finding. Stop expansion; derive secondary surfaces mechanically, narrow the contract, localize the representation, or split the repository. Work spent satisfying obligations created only by coupling is maintenance cost, not evidence of value.

### BOUND-5 — Private storage follows stewardship, not artifact nouns

**Trigger:** Private repositories, corpora, content roots, or artifact stores are divided or embedded.

**Rule:** Partition them according to coherent differences in semantic authority, access control, maintainer, retention, backup, release, security, and abandonment lifecycle—not merely because artifacts are briefs, proposals, accepted outputs, reports, or workflow stages. Artifacts sharing one stewardship lifecycle MAY occupy explicit states or directories in one private corpus even when their authority differs.

This storage rule does not override `BOUND-2` when independently invocable product lifecycles are present. A public tool repository SHOULD use declared import/export or a configured content-root contract rather than embed private content as an implicit subtree. Any legitimate mount, link, submodule, or external content root MUST be explicit, version-governed or pinned where appropriate, clone-visible, and unnecessary for unrelated repository operation.

## LLM capabilities and durable continuity

### AGENT-1 — LLM skills compose through a human-governed blackboard

**Trigger:** Multiple LLM skills, semantic agents, or independently invocable reasoning capabilities operate on shared domain state, or an authorized task coordinates more than one repository.

**Rule:** Capabilities compose through revision-identifiable artifacts and authoritative domain state, not peer dispatch or hidden conversational return points. The logical blackboard contains authoritative state, reviewable proposals, evidence, unresolved decisions, and only the status needed for applicability. It may be one file or several domain artifacts; an ungoverned shared directory or universal schema is insufficient. Each capability declares and enforces preconditions, bounded reads and proposal writes, postconditions, and failure behavior.

The human selects the work and may invoke one capability or explicitly authorize a bounded task spanning named results and repositories. In the latter case the executing agent MAY carry the task through its authorized phases and ordinary transfers without asking the human to launch each stage. The task must identify its outcome, participating ownership scopes, actual consumer, appetite, acceptance and stop boundaries, and next inspectable result. A single task can coordinate authoring, downstream implementation, verification, and proposed feedback without a separate workflow capability for each phase.

Each repository retains its own instructions, authoritative state, source versions, work-control plane, and acceptance decisions. The agent acts within the currently relevant repository's accepted scope. Link existing work artifacts rather than create a second or global status authority. `INT` governs all transfers; one repository's process does not gain authority over another's state. The human may review related proposals together while independently rejectable decisions remain visible under `AUTH-2`.

This permission does not authorize an unbounded autonomous pipeline, runtime LLM policy, reusable workflow controller, dynamic selection of independent peer capabilities, or acceptance on the human's behalf. Completion of a phase neither accepts its output nor authorizes missing scope. Outside the explicitly authorized task, the human chooses the next capability. A read-only advisor or deterministic command may report eligibility and missing preconditions, but cannot dispatch capabilities, mutate domain state, own return points, or acquire semantic authority.

### AGENT-2 — A capability owns one result and never controls peers

**Trigger:** An LLM skill or semantic capability is designed or revised.

**Rule:** It MUST own one coherent domain result and lifecycle. It MUST NOT invoke, dispatch, suspend itself for, transfer control to, require return from, or use peer names as routing instructions for another skill. A controller skill MUST NOT accumulate domain judgment, mutate peer blackboards, or own peer return points. Execution of human-selected phases under `AGENT-1` is not permission for a capability to call or control its peers. Keep coordination in the accepted task and durable artifacts; do not implement a new dispatcher merely to move between already authorized repositories.

A permanently required research, critique, or synthesis phase belongs inside the owning capability unless it is a deterministic tool or an independently useful capability selected by a separate human invocation or as an explicit phase under `AGENT-1`, with no authority to mutate the caller's state on its behalf. The owner MAY use subordinate reasoning workers when they have no independent invocation or return lifecycle and cannot mutate shared authoritative state on the owner's behalf. Deterministic validators, searches, formatting, atomic file operations, and implementation-local code are tools, not sibling workflows. Concurrent mutation of the same blackboard is forbidden unless adopted as a separate capability with proven conflict and acceptance semantics.

### AGENT-3 — Conversation is not durable project state

**Trigger:** Work crosses a human wait, context boundary, session, retry, invalidation, or authority gate.

**Rule:** Persist the current proposal and, when a reserved human decision is genuinely pending under `AGENT-5`, the exact question, authority being exercised, relevant input revision, allowed decision surface, and safe return point in an existing authoritative issue, branch, artifact, or state record. Recording an implementation judgment or uncertainty does not convert it into pending human authority. Conversation MUST NOT be the only place where current state, approval, pending work, or return position exists. Before relying on a new or materially changed session handoff, retry, invalidation, mutation, or independently changing capability boundary, prove on a narrow vertical slice that work resumes from revision-identifiable state without transcript reconstruction, stale application, or duplicated effects.

Control records MUST remain minimal and recoverable. They MUST NOT become a second source of domain truth or a hand-authored routing, status, provenance, or certification bureaucracy when ordinary version control and existing artifacts suffice.

### AGENT-4 — Unattended continuation requires bounded durable state

**Trigger:** Authorized work will continue without the human present, cross an overnight period, or hand off to another context.

**Rule:** It MAY continue only inside an already accepted bounded issue or an explicitly authorized coordinated task with linked repository scopes. Its durable state MUST record the current acceptance boundary and exact next acceptance-bearing increment, consumed and remaining appetite, review-cycle count, repository allocation streak, applicable demand-null streak, any outstanding terminal decision and its consecutive extension count, any active freeze, newly discovered capability classes or pipeline concerns, unresolved finding classifications that can cross a stop condition, publication limits, exact prospective publication set, and the authorized action on uncertainty or failure under `AGENT-5`.

The clock boundary alone does not force a stop when this state exists and no tripwire has fired. Missing state, required reasoning reconstruction, or a crossed tripwire MUST stop continuation before the unattended interval. Reconstruction from transcript or memory does not restore authority. Human absence never authorizes crossing an appetite, authority, scope, capability-class, repository-boundary, non-convergence, or publication boundary.

### AGENT-5 — Delegated judgment, evidenced escalation

**Trigger:** Uncertainty, alternatives, apparent ambiguity or a contemplated human request before the current acceptance surface.

**Rule:** Bounded activation delegates implementation judgment. Agents MUST proportionately investigate applicable work, foundations, sources, code, tests, history, conventions and authoritative external evidence, then choose/proceed with the best-supported option. Low confidence, novelty, complexity, alternatives, availability or responsibility transfer create no human gate. Do not ask technical questions the investigated agent can answer better.

A request requires a residual reserved decision currently blocking safe/proportionate progress. It MUST meet `AUTH-3`, identify the human consequence and why authority/investigation cannot decide, lead with a recommendation, explain why the strongest alternative loses, and state delay/error cost and safe return. Translate technical detail into human consequences.

For technical uncertainty, choose a local reversible consistent option, contain/test/proceed; record only materially relevant assumptions. For reserved uncertainty without an immediate stop requirement, agents MUST continue explicit local reversible proposals when isolated from false claims, adoption, dependents, irreversible effects and disproportionate waste; expose them at the next coherent acceptance. Stop at authority/risk/appetite/capability/repository/publication/irreversible boundaries, material user-semantic lock-in, disproportionate waste or other stop rules.

For delegated creative/domain proposals, agents SHOULD resolve ordinary gaps coherently and explain choices together. Proposals remain unaccepted; `PURP-5` may permit reversible demonstrations. Do not silently change accepted behavior/identity or reserve every provisional rule.

Batch genuine independent decisions when safe. Agents MUST NOT create serial micro-approvals, speculative pre-investigation questions or technical subgates. IDs do not reserve choices; interactive work follows the same rule.

Generic judgment/investigation/advice requests authorize technical judgment, not reserved choices. Silence, confusion, fatigue or advice requests MUST NOT count as acceptance. Clear authorization after an `AUTH-3` complete recommendation MAY accept it; MUST NOT reconfirm without materially changed facts, consequences or scope. Agent judgment remains a proposal, not canonical human authority.

## Implementation, review, and operational safety

### OPS-1 — Feasibility before readiness/TDD

**Trigger:** Risk-bearing TDD, implementation, exact review, execution readiness or unattended work.

**Rule:** Establish adequate input information for promised outputs; identify finite representation/oracle where available, semantic authority, uncertainty behavior, smallest acceptance-bearing increment, appetite, stops and publication boundary. Repeated human judgment also requires `EVID-8`'s cumulative unit/ceiling.

Producer-consumer readiness MUST separate candidate-construction authority from proved handoff. Name the execution owner and actual source-to-consumer check (`INT-1` where cross-repository); generalized evaluators/universal representations are not prerequisites for product-specific code. A `PURP-5` joint probe can resolve uncertainty, but its plan is not evidence.

Recurrent, reusable, configurable, engine-like or separable experiential claims also need `PURP-2`'s loop/value mode, `EVID-8`'s setup/return burden, and planned first, different second and applicable return use (`EVID-3`).

Material external dependencies owning primary-path state/decisions require a minimal real lifecycle probe and independent semantic oracle. Exercise applicable update/rebuild, IDs, score/distance meanings, persistence/reload, failure/recovery and replacement. Docs, types, example-repeating wrappers and plausible output do not prove correct use. Straightforward replaceable dependencies unable to alter accepted meaning, authority or consequential action need only ordinary proportionate checks.

Apply this gate to unrestricted prose, semantic gating/scoring, decision-bearing retrieval, relied-on lifecycle dependencies, external fidelity, shared abstractions, parsers/DSLs/models/frameworks/controllers, migrations, cross-cutting validators, generated scale, session-spanning work and default publication. Missing answers mean not ready; labels/detail confer no feasibility.

### OPS-2 — Missing inputs and new capability classes require re-ratification

**Trigger:** Implementation discovers that required information is absent, accepted scope is insufficient, or a substantial unapproved capability class is needed.

**Rule:** This trigger applies only when the gap affects the promised output, accepted authority, or another reserved human boundary. It does not fire merely because an implementation choice was unspecified or the agent is uncertain. Before stopping, the agent MUST investigate proportionately and verify that the missing information cannot be derived from accepted authority or safely localized as implementation judgment under `AGENT-5`.

When the trigger genuinely fires, stop before designing or building around the gap. Recommend an explicit field, adapter, semantic-review boundary, reduced promise, formal language, or revised scope and explain why it is preferred. Introduction of a parser, DSL, ontology, framework, controller, model, migration system, generalized evaluator, or comparable capability not named in accepted scope requires immediate human re-ratification with its consumer, appetite, authority, and removal path.

Heuristic reconstruction, hidden scope expansion, or quietly treating the new capability as an internal detail is forbidden.

### OPS-3 — Adjudicate tests, reviews, and infrastructure before repair

**Trigger:** A test, CI run, or reviewer reports a failure.

**Rule:** First determine whether the expectation is still valid, whether the defect lies in the system under test, the test or review oracle, or execution infrastructure, and whether the finding belongs to accepted scope. Read the actual failure annotation before diagnosing code or architecture; billing, permissions, unavailable services, dependency drift, environment drift, and runner failure are infrastructure claims until shown otherwise.

Classify the finding as an implementation defect, specified missing case, invalid or obsolete test, specification ambiguity, authority or ownership defect, infrastructure failure, novel requirement, or non-convergence signal. Only implementation defects and specified missing cases enter automatic repair. Uncertainty about classification requires proportionate investigation and does not itself authorize repair or human escalation. If the residual choice is implementation judgment, the agent chooses the best-supported reversible classification and proceeds; if it is a reserved human decision, `AGENT-5` governs the request. Reviewer-generated examples are observations, not requirements until their interpretation and scope are accepted.

### OPS-4 — Execution stops

**Trigger:** Nontrivial implementation or review.

**Rule:** Unless a stricter or explicitly accepted larger limit applies, stop at the earliest of:

- two active hours without inspectable acceptance-bearing increment;
- three failed implementation/review repair cycles at one acceptance boundary;
- three consecutive review passes retaining cases but adding members of an open semantic/paraphrase family;
- an unapproved capability class or independent pipeline;
- approximate doubling of expected production, test, evidence or semantic surface;
- exhaustion of token, compute, financial or human-burden limits;
- a context/session boundary requiring material reasoning reconstruction; or
- unattended continuation lacking `AGENT-4` state.

Open issues, AFK, sunk cost, passing legacy tests or reports override no stop. Extension needs fresh human priority, scope and appetite; `WORK-3`/`WORK-4` counts persist.

### OPS-5 — Exploration stabilizes the claim; exact review verifies a finite candidate

**Trigger:** Adversarial review, exact-SHA review, or repeated test expansion is proposed.

**Rule:** Exploratory review MAY discover and stabilize a draft contract, interface, and oracle before final implementation review. Once a finite candidate is declared, exact-candidate review verifies that candidate; it MUST NOT become an unlimited source of new requirements. Reviewer-generated examples remain in an evaluation corpus until ratified.

Permanent selectors, ledgers, red-green evidence, and reconciliation duties are reserved for stable decision-bearing behavior and MUST be proportional to its authority and expected lifetime. Three review passes that continue expanding an open semantic family trigger `OPS-4` rather than another repair cycle.

### OPS-6 — Bounded, fresh, recoverable effects

**Trigger:** External, destructive, irreversible, repeated or concurrency-sensitive effects.

**Rule:** Effects MUST have narrow accepted authority, expected-revision or equivalent freshness checks, idempotent or explicitly bounded retries, inspectable consequences, explicit failure behavior and recovery ownership. Local accepted edits need only ordinary version identity and atomic validated writes unless a real boundary proves extra machinery necessary. Do not build orchestration/receipt platforms before the smallest fault-tested slice proves need.

### OPS-7 — Clean-clone operational closure

**Trigger:** Activation, handoff, publication or reliance on a repository.

**Rule:** A clean clone MUST expose purpose/authority, work control, build/check and primary-outcome activation paths, external/private input contracts/locations, output ownership/location and absent-input behavior. Dependencies, services, stores, submodules and content roots need visible roles, access, version/pins and setup/recovery. Preserve root `AGENTS.md` → `CLAUDE.md` and the canonical constitution.

Ordinary use MUST NOT rely on remembered siblings, hidden gitlinks/mounts/services/topology or conversational setup. Missing private input MAY block its workflow but must fail early/honestly while preserving claimed input-independent checks. Public demo data is optional; synthetic fixtures MUST NOT claim real demand, representativeness or readiness.

## Publication

### PUB-1 — Publication

**Trigger:** Merge, release, deployment, package/data publication or another authoritative-path change.

**Rule:** Terminally failed, rejected, parked or unaccepted production MUST NOT become newly reachable on authoritative paths, even solo/private. Quarantined failure reports, fixtures and adversarial corpora MAY publish separately. Independent prerequisite/evidence work SHOULD exclude failed candidates.

Before publication, list exact commits/decision-bearing files and reconcile each acceptance. Names/status/tests/mergeability/intent do not suffice. Archiving exact failed production requires scoped waiver. Issue approval permits work, not all resulting bytes; proposal labels/custody records do not evade this. `PURP-5` provisional consumers stay isolated pending exact source/behavior/publication acceptance.

## Foundations and governance

### GOV-1 — Foundations are minimal and cannot manufacture a backlog

**Trigger:** Repository-specific foundations are created or expanded before the first accepted risk-bearing slice.

**Rule:** Foundations MUST identify only what current work needs: mission and purpose class, primary outcome and non-goals, authority, real inputs and named consumers, active bottleneck, blocking prerequisites, active uncertainty, bounded experiment, repository boundaries, work-control plane, and minimum contracts needed to run safely.

Foundations MUST NOT pre-author a complete future workflow map, workflow-to-skill conversion queue, speculative ADR family, template or schema family for unconsumed artifacts, shared implementation host, maintenance system, or architecture-derived backlog. Legacy guides, skills, corpora, and capability inventories MAY remain reference-only evidence; their chapters, edges, and handoffs do not become active boundaries or work items. Document presence is not work authorization.

More than roughly five repository-specific foundation documents or 5,000 repository-specific words before the first accepted slice—excluding this constitution and reference-only source evidence—MUST trigger an explicit recheck for speculative architecture. Every additional foundation document MUST have an immediate decision or execution consumer.

### GOV-2 — One work-control plane owns active work

**Trigger:** Work, readiness, priority, dependency state, acceptance, or closure is recorded.

**Rule:** Each repository MUST name one work-control plane, normally GitHub Issues and pull requests or another explicitly selected system. Local Markdown tickets, plans, specification directories, run-state files, labels, and archive folders MUST NOT form a second active queue or competing status authority.

Local documents MAY support an authoritative issue and preserve accepted reasoning or artifacts, but pending work, readiness, priority, and closure remain in the named plane and the document links back. Historical local records are evidence only. Changing the work-control plane requires an explicit cutover of active items; running two planes in parallel is not a migration strategy.

### GOV-3 — Governance is enabling work and shrinks with its subject

**Trigger:** Governance, conformance, provenance, orchestration, meta-tooling, control records, or policy automation is proposed or maintained.

**Rule:** Governance obeys the same consumer, real-input, bounded-commitment, primary-path, and foundation-minimization rules as production work. Each maintained gate or control surface MUST protect a current accepted invariant, decision, boundary, or recovery need. Prefer ordinary version control, focused tests, existing issues, and local checks before registries, ledgers, receipts, controllers, or generalized governance services.

Count the total active authoring, synchronization, verification, reading, and decision burden across artifacts. Moving process into data strings, appendices, generated reports, or another repository does not remove its cost. A disclosed burden is still a burden; documenting it does not establish fitness. Keep decision-facing explanations proportionate to the decision and expose underlying evidence on demand.

One exact commit or export identity may bind a coherent multi-file change. A single acceptance boundary does not itself require manual lockstep revision fields in every file, duplicated custody blocks, or copied joins and status records. Prefer derivation or an existing shared revision identity when it preserves the required invariant. Add finer records only for a named consumer or material recovery need. A requirement to record a fact creates no automatic requirement for a new document, workflow, schema, or checklist.

Constitutional application MUST protect the intended outcome and authority, not manufacture work explaining why an unmet defining requirement can count as met. Resolve conflicts at the governing promise under `AUTH-3` and `ADOPT-2`. Once governance obscures the actual product or repeatedly asks the human to approve technical bookkeeping, simplify it under `EVID-8` and return to the primary path. Keep necessary evidence and safeguards; remove redundant recitations and self-created obligations.

When the governed subject disappears or narrows, remove or shrink its policing surface. Governance MUST NOT preserve itself by inventing consumers, compliance work, or evidence of necessity. Reference-only workflows, legacy packages, and old decision inventories create no disposition, migration, or implementation backlog without independently selected current demand.

## Adoption, precedence, waivers, and amendment

### ADOPT-1 — Adoption is complete, exact, and progressively loaded

**Trigger:** A repository adopts this constitution, a module or a later revision, or a previously dormant module trigger fires.

**Rule:** The complete adopted text is the core plus every adopted module at one revision. An exact copy of the core MUST exist at `docs/principles/principles-for-repos.md`; each adopted module MUST be copied byte for byte beside it under its Modules-table filename. Every applicable module MUST be adopted in full before substantive dependent work. Optional means its trigger is absent, not permission to omit safeguards from an applicable domain. Adopting a module changes the complete adopted set and requires explicit human acceptance and applicable reconciliation; a pending adoption permits only investigation or provisional work authorized by the operative revision. Overlapping triggers require every applicable module. Module and core rules have the same force; modules cannot waive the core.

The revision's exact bootstrap MUST appear in root `CLAUDE.md`, and root `AGENTS.md` MUST be its symlink. Its sole substitution parameter is the entire `Adopted modules:` line: list the adopted module filenames in backticks, separated by commas and ending in a period, or `Adopted modules: none.`. This line MUST match the accepted adopted set; every other bootstrap byte remains exact. A clean clone MUST let Claude Code and Codex discover the bootstrap and all canonical adopted files. Root instructions SHOULD fit configured instruction limits; task-specific references belong in governed files/skills read on the triggers. The complete adopted text MUST NOT be pasted, imported with Claude Code's `@` syntax or otherwise loaded into every request merely for adoption; progressive loading is required by default.

Foundations MUST cite and specialize the constitution with concrete facts, never summarize, paraphrase, selectively reproduce or replace safeguards as competing normative text. A later revision is adopted as a complete core and complete adopted modules, or not adopted; selective alignment has no constitutional force. The previous complete adopted revision remains operative until human acceptance of reconciliation. Proposed replacement files, passing checks and implementation approval alone do not activate a revision.

Read the complete adopted text, applicable rules and modules exactly when the bootstrap requires. An exact quotation of a current-revision rule in a brief naming that revision satisfies the recipient's reading of that rule; it does not satisfy other triggered readings. Paraphrases and memory do not qualify. Absence from initial context excuses nothing; inability to locate/read required text blocks substantive work.

### ADOPT-2 — Precedence and waivers preserve the safeguard

**Trigger:** A local instruction specializes the constitution, two accepted instructions conflict, or deviation is requested.

**Rule:** A lower-level artifact MAY specialize a rule with repository-specific facts but MUST NOT silently weaken it. A narrower and stricter explicit human decision governs only for its stated scope. Ambiguity about whether a local instruction conflicts with or weakens this constitution preserves the applicable safeguard while the agent investigates. If the conflict remains and resolution requires a waiver, clarification of human intent, or another reserved human decision, request it under `AGENT-5`. Ordinary implementation uncertainty is outside this rule.

A waiver is valid only when the human-authored decision names the exact rule, bounded scope, reason and risk accepted, appetite, and event or date that ends it. Approval of an issue, plan, mechanism, pull request, label, silence, continued work, or a request to “make it work” is not a waiver. A waiver MAY relax process for a bounded case; it MUST NOT authorize a false claim, hidden semantic authority, unbounded continuation, or ambiguous publication. Those require a constitutional amendment or rejection of the claim.

Deviation from a **SHOULD** records the reason, risk, and scope in the existing authoritative artifact; it is not a waiver unless another **MUST** is also relaxed.

### ADOPT-3 — Existing repositories reconcile active authority, not historical prose

**Trigger:** This edition is added to an existing repository or a repository is materially reoriented.

**Rule:** Reconcile only surfaces that can currently authorize or constrain work: primary outcome, purpose class, portfolio displacement, and predecessor or successor appetite, evidence state, unresolved premises, limitations, and terminal state inherited under `PURP-1`; real inputs and consumers; active producer–consumer promises, actual-path evidence, complete bounded scope, source-to-code versus runtime-data contracts, authorized coordination, intelligible human decision surfaces, and selected search breadth; active experiential claims and their materially separable effects, distinctive activity loops, selected value modes, and applicable diagnostic and voluntary second-use evidence; active claims borne by artificial evaluators and their established competence, behavioral coverage, and validity limits; active autonomous or simulation claims and their causal-loop scale, counterfactual or behavioral-robustness evidence where claimed, and representation resolution; product-like locks, certification, or governance attached to still-provisional behavior; decision-bearing composite scores and their component, compensation, veto, and sensitivity authority; cumulative-search or learned-operator claims and their productive-neighborhood, lineage, and transformative-stage evidence; recurrent-use and applicable return-use claims and evidence; setup and return-burden appetite; bottleneck and prerequisites; appetite, allocation streak, demand-null state, consecutive terminal-decision extension count, and outstanding terminal decisions; accepted authority; local constitutional restatements; active issues and contracts; decision-bearing tests and production code; capability and repository boundaries; agent bootstrap; and exact default-path publication state.

Classify conflicts as accepted, provisional, parked, rejected, superseded, or evidence-only. Remove non-authoritative architecture from active dependency graphs and default agent context, and resume from the smallest justified primary-path increment. This is a conflict audit, not a historical documentation migration. Version history may preserve detail.

### ADOPT-4 — Amendments must reduce drift rather than accumulate it

**Trigger:** A constitutional change is proposed.

**Rule:** The amendment MUST:

1. name an observed failure, near miss, repeated agent mistake, or unusually compelling prospective hazard;
2. show why an existing rule cannot be clarified, strengthened, narrowed, or correctly triggered instead;
3. state both the prohibited failure and at least one legitimate safe path or counterexample;
4. remove or merge wording made redundant by the change;
5. recheck the bootstrap and reading triggers without automatically expanding always-loaded context;
6. preserve an identifier when its subject remains the same and create a new identifier only for a distinct obligation;
7. include an explicit supersession or migration mapping when rules are split, merged, deprecated, or reidentified; and
8. identify the reconciliation class and, when an existing authoritative portfolio inventory names affected repositories, identify those repositories. The amendment MUST NOT require creating a new central inventory merely for this purpose; each repository that adopts the revision identifies and reconciles itself under `ADOPT-3`.

The amendment's existing authoritative issue, pull request, or decision MUST carry the basis, safe paths, mapping, and reconciliation class required above. The complete replacement constitution MUST contain only current operative text and minimal revision metadata. Amendment narratives, repository-specific histories, affected-repository notes, revision-specific reconciliation instructions, and superseded migration guidance MUST NOT be carried forward unless they are themselves necessary to state a current operative obligation. Version history MAY preserve the diff; no changelog, amendment log, or other new artifact is required.

The reference ticket measures core-only reading for the bootstrap events accepting or activating an execution-ready work item, offering a test as completion evidence, and merging. Count the union of rules these three triggers name, each once, plus the complete Normative definitions section once. A rule or definitions section runs from its heading to the next Markdown heading, counted as stored UTF-8 bytes; approximate tokens are bytes divided by four, rounded up. Every amendment MUST run the canonical reading checker against the previous revision and report each event, the reference union, definitions, bootstrap and whole-text byte counts with approximate tokens and deltas. The bootstrap count includes its markers. Every core rule, including each new rule, MUST be named directly in a reading trigger. Ordinary reference reading MUST stay at or below 32,000 bytes and the bootstrap at or below 5,062 bytes. Moving text without preserving discoverable triggers and complete safeguards does not satisfy the budget. Module text is outside ordinary reference totals but remains mandatory when triggered; whole-text reporting MUST include core and modules.

Any increase in the reference figure since the previous revision, even within the ceiling, requires the amendment to name the failure needing the extra text and remove at least as much text elsewhere in the constitution. A proposed increase to either ceiling additionally MUST be explicit in the amendment issue and separately accepted by the human; convenience or accumulated prose grants no increase. An agent unable to meet the budget without dropping or weakening a safeguard MUST stop and expose each proposed loss, with its reason, as a separate human decision. The drafter MUST NOT silently settle that trade-off; human amendment and waiver authority remains governed by `AUTH-2` and `ADOPT-2`.

Semantic identifiers are stable within an edition line: later revisions may clarify or strengthen them while preserving their subject. A deliberate major replacement MAY restructure identifiers through an explicit migration mapping; it MUST NOT pretend old citations retain unchanged meaning. Unique safeguards belong in their controlling rule, not in a second normative list of inherited failures.

A complete amendment reconciliation MUST check for contradiction, duplicate authority, trigger discoverability, operational burden, and whether the new wording accidentally forbids a legitimate local implementation, bounded experiment, or independently accepted product.
