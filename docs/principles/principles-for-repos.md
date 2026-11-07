# Principles for Repositories

**Constitution revision:** 2026-09-30.1  
**Supersedes:** 2026-08-27.2. Existing rule identifiers are retained.

## 1. Purpose, scope, and requirements language

This constitution governs every formal Git repository that adopts it. It applies to repository foundations, issues, plans, schemas, tests, reviewers, agents, skills, automation, branches, releases, data products, and inter-repository contracts. A specialized rule is dormant when its trigger does not exist; harmless inapplicability is not nonconformance.

Its purpose is to prevent expensive work from acquiring authority through momentum, elegance, local correctness, agent enthusiasm, sunk cost, or architecture that manufactures its own demand. It is designed to stop work before a repository becomes difficult to repair or abandon.

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative:

- **MUST** and **MUST NOT** state constitutional requirements. Deviation requires a valid waiver under `ADOPT-2`, unless the rule expressly forbids waiver.
- **SHOULD** and **SHOULD NOT** state strong defaults. Deviation requires a recorded reason, accepted risk, and bounded scope in an existing authoritative artifact, but not a formal waiver.
- **MAY** states permission, not preference.

All text in this constitution is normative unless explicitly labeled **Informative**. Semantic rule identifiers contain the controlling detailed obligations. Examples are non-exhaustive and do not create requirements by analogy. Identical obligations MUST use one canonical formulation; indexes and bootstraps route readers to that formulation rather than restating it competitively.

When this constitution requires a fact or decision to be recorded, the repository MUST use an existing authoritative issue, pull request, foundation, decision record, or governed state artifact unless a new artifact has an independently justified consumer. No registry, certification packet, recurring report, or form is required merely to prove constitutional compliance.

## 2. Required agent bootstrap

The following block is normative routing text. Every adopting repository MUST copy it verbatim into the repository-root `CLAUDE.md`. `CLAUDE.md` is the real file; repository-root `AGENTS.md` MUST be a symlink to it. Repository-specific instructions may surround the block but MUST NOT alter it.

<!-- BEGIN PRINCIPLES-FOR-REPOS BOOTSTRAP 2026-09-30.1 -->
## Repository constitution

This repository adopts `docs/principles/principles-for-repos.md`, revision **2026-09-30.1**, as binding. `CLAUDE.md` is the real root instruction file and `AGENTS.md` is its symlink. A namespace name below means every rule whose identifier begins with that namespace and a hyphen.

### Mandatory reading

Reasonable doubt activates a reading trigger. Memory, a previous session, an earlier reading for a different decision, and paraphrases do not satisfy it.

- Before founding, materially reorienting, or reconciling a repository; materially revising foundations; or adopting a constitutional revision: read the complete constitution.
- Before accepting or activating work, applying execution-ready or AFK status, opening an adjacent lane, expanding scope, or selecting a coordinated producer–consumer iteration: read `PURP`, `WORK`, `OPS`, and `AGENT-5`.
- Before turning uncertainty into a human request or blocker: read `AUTH-2`, `AUTH-3`, `AGENT-5`, and `OPS-2`.
- Before adopting or expanding a representation, specification, recurrent authoring path, schema, ontology, state model, simulator, corpus, evaluator, semantic gate, migration, generated surface, shared infrastructure, or general capability: read `EVID`, `SEM`, `REP`, `GOV`, and `OPS`.
- Before implementing or accepting a claim about feedback, training, ranking, search, adaptation, optimization, or repeated selection: read `PURP` and `EVID`; also read `SEM` and `REP` when semantic decisions or actions depend on it.
- Before using a test, prototype, review, simulation, score, or experience as grounds for completion, adoption, or expansion: read `EVID-4` and the applicable proof requirements in `EVID-3`, `EVID-6`, and `REP-2`.
- Before promising fidelity to external rules, standards, protocols, legacy behavior, or governed workflows: read `EVID`, `REP`, and `OPS`.
- Before adding or redesigning a capability, pipeline, skill family, repository boundary, or human/agent workflow: read `BOUND`, `AGENT`, and `INT`.
- Before cross-repository import, export, feedback, migration, shared identifiers, private-data mounting, or coordinated execution: read `INT` and `BOUND`; read `AGENT` when execution spans capabilities or repositories.
- Before unattended continuation, handoff, repeated repair or review, or continuation near an appetite or allocation limit: read `AGENT`, `WORK`, and `OPS`.
- Before merge, release, or making production work newly reachable from an authoritative path: read `AUTH` and `PUB`.
- Before a waiver, specialization, adoption reconciliation, identifier change, or amendment: read `ADOPT`.

Name the consulted rule identifiers in the existing authoritative work artifact. Do not create a compliance document. An accepted bounded task may continue without rereading or recording the same consultation until its scope, premise, authority, publication set, or triggered rule family changes.

### Universal authority chain

Human intent → constrained proposal → mechanical or semantic checks → inspectable consequences → human acceptance → authoritative state

Human selection authorizes priority and bounded effort. It does not establish truth, fitness, reusability, demand, or permission for unaccepted dependent work.

### Immediate stop-and-read signals

Stop further discretionary expansion and read the controlling rules when:

- required information, an independent oracle, dependency lifecycle evidence, or a valid no-match outcome is missing (`SEM`, `REP`, `OPS`);
- local completion excludes an essential consumer effect, repeated artifacts leave the actual handoff untested, or another implementation is offered as proof of the intended path (`PURP-3`, `PURP-5`, `EVID-3`, `EVID-4`, `INT-1`);
- experiential, recurrent, autonomous, simulation, search, generality, fidelity, or causal claims outrun their applicable evidence, or burden exceeds appetite (`PURP-2`, `EVID`, `REP`);
- presentation, enthusiasm, evaluator scores, exact replay, or compliance records stand in for a different claim; a composite score conceals a failed requirement; or transformation conceals causal credit (`EVID-4`);
- an unapproved capability, independent pipeline, uncontrolled change fan-out, non-convergence, exhausted appetite, or outstanding terminal decision appears (`BOUND`, `WORK`, `OPS`);
- acceptance requires the human to reconstruct technical work, approval is being used to repair missing evidence, or authority, publication, or governing instructions conflict (`AUTH`, `AGENT-5`, `ADOPT`).

If applicable text cannot be located or read, substantive work is not authorized. Preserve a potentially applicable safeguard while investigating. Escalate only the reserved human remainder under `AGENT-5`; ordinary implementation uncertainty and several reasonable technical options do not themselves require a stop. Authorized evidence-gathering and provisional work remain governed by the applicable rules.
<!-- END PRINCIPLES-FOR-REPOS BOOTSTRAP 2026-09-30.1 -->

## 3. Normative definitions

**Acceptance authority** is the named human authority that decides what becomes canonical for a stated scope. It is not the default oracle for implementation choices that accepted authority and repository evidence permit the executing agent to resolve.

**Acceptance-bearing increment** is a coherent part of a named primary outcome, or the earliest premise or prerequisite blocking it, that can be inspected and accepted on its own. More scaffolding, internal repair, tests, ledgers, reports, or evidence about unfinished work do not qualify merely by existing.

**Behavioral robustness** is the degree to which a claimed qualitative tendency, causal effect, or family of outcomes remains within declared acceptable bounds across nonessential changes, independently selected seeds, configurations, or population variations. It does not require identical trajectories, suppress meaningful diversity, or protect the result from causally relevant interventions.

**Appetite** is the accepted bound on effort and exposure, expressed through the units that matter for the work: implementation time, completed work items, review cycles, LLM or compute use, semantic surface, financial cost, and human burden as applicable.

**Authoritative state** is state that the acceptance authority has accepted for the declared scope. Presence in a repository, branch, issue, plan, test, schema, generated artifact, or agent transcript does not make state authoritative.

**Bottleneck** is the earliest unresolved condition currently preventing the selected primary outcome from reaching acceptance.

**Capability class** is a substantial kind of system or authority whose introduction materially changes the accepted solution surface, such as a parser, DSL, ontology, framework, controller, persistent state model, simulator, migration system, generalized evaluator, shared registry, or new independently governed service.

**Consumer** is a current accepted use, human decision, operational path, or bounded decision-bearing experiment that requires an output at a declared fidelity. A roadmap, architecture diagram, schema, issue, or future pipeline authored merely to exercise proposed supply is not a consumer.

**Distinctive activity loop** is the smallest user-visible cycle whose completion distinguishes one claimed experiential effect from adjacent effects and whose existence materially justifies a separate capability, persistent state, recurrent path, or substantial human or maintenance burden. A merely available option, adjacent activity, or observation that the loop could be performed is not completion of the loop.

**Decision atom** is a human-governed semantic or governance commitment that can be accepted or rejected independently without contradicting another accepted commitment.

**Implementation judgment** is a choice among ways to execute an accepted contract whose human-governed consequences are entailed or strongly constrained by accepted authority and repository evidence, and that does not select or materially change the primary outcome, product promise, accepted source interpretation, authority, persistence contract, capability or repository boundary, appetite, irreversible or external effect, publication scope, or acceptance oracle.

**Reserved human decision** is a choice that requires the acceptance authority because it selects or changes human intent, the primary outcome, materially different user-facing or product semantics not entailed or strongly constrained by accepted authority, an accepted source-fidelity interpretation, appetite, risk acceptance, waiver, capability or repository boundary, irreversible or external effect, publication scope, terminal disposition, or authoritative state. A question is not reserved merely because it is technically complex, unfamiliar to the human, ambiguous at first inspection, or difficult for the agent to explain.

**Externally governed fidelity promise** is a claim that a repository will reproduce, conform to, or remain compatible with behavior controlled by a source set outside the repository's own design authority, such as a rulebook, protocol, formal standard, law or regulation, published interface or file format, or legacy system behavior.

**Fidelity obligation topology** is a coarse, source-derived map of the behavioral and adjudication families that determine an externally governed fidelity promise: controlling sources and precedence, base behaviors, exception classes, timing and lifecycle boundaries, persistent modifiers, optional or compound continuations, cross-rule interactions, unresolved ambiguity classes, and the oracles needed to decide them. It estimates obligation shape and burden; it is not an exhaustive state-space or pairwise-interaction matrix unless the accepted claim independently requires one.

**Independent pipeline** is an invocable path from accepted real input through decision or semantic authority and side effects to a primary output, with its own acceptance or failure behavior and a coherent release, parking, removal, or abandonment decision. A function, phase, UI, library, or handoff is not an independent pipeline merely because it is a step.

**Least-complex credible intervention** is the smallest good-faith change that could plausibly produce the named effect or address the observed failure on the same representative real case without adopting the candidate's substantial new machinery. It need not be production-final or globally reusable, but it MUST exercise the causal hypothesis seriously enough that losing to it would matter.

**Primary outcome** is the human-selected result whose acceptance justifies the repository or current workstream. **Enabling work** exists only to make a primary outcome possible. **Stewardship** maintains an already accepted product. **Deferred possibilities** are not active work.

**Product-bearing artifact or release** is accepted as part of the selected primary outcome or as authoritative input to a named consumer, and is intended to survive for its declared scope rather than exist only to answer a bounded decision. Product-bearing does not mean immutable, exhaustive beyond that scope, or protected from later replacement.

**Primary path** is the shortest evidence-bearing path from the current bottleneck to an acceptance-bearing increment of the selected primary outcome.

**Publication** is any action that makes a production delta newly reachable from a default or protected branch, release, package, deployed environment, accepted data release, or other authoritative path.

**Qualifying opportunity run** is a real-input execution that exposes enough relevant opportunity that a positive case should plausibly occur at the frequency assumed by the capability's accepted benefit claim. Its exposure basis is declared before interpreting the result.

**Real input** is accepted material at the fidelity the first promise actually consumes. A synthetic fixture, hypothetical payload, imagined API, or example authored by the proposed system is not a real input, although it may test mechanics or falsify a candidate mechanism.

**Representative real-use run** is an execution using accepted real input through the actual authoring or configuration, runtime, and presentation path far enough for the human to complete or intentionally terminate a bounded current task and inspect the material effect. A run is representative only for the activities, consequence-bearing loops, recurrence, recovery, assurance, or availability effects it actually exposes at the fidelity claimed. Enjoyment of an adjacent activity or awareness of an unexercised option does not make the run representative for that option's use or recurrence. A seam probe, synthetic fixture, isolated mechanic, subsystem demonstration, or internal report is not a representative real-use run merely because it succeeds.

**Materially different second-use run** is a representative real-use run using a second accepted real artifact, scenario, or configuration selected independently enough to exert meaningfully different content, authoring, configuration, or operational pressure on the same accepted path. A synthetic variant or case selected merely because the current machinery already handles it does not qualify.

**Return-use run** is a representative real-use run resumed or newly created after a meaningful interruption or clean handoff from durable repository state, with enough separation to expose forgotten setup, hidden dependencies, and recovery burden. No universal interruption duration is implied.

**Semantic reviewer** is a human or reasoning agent that interprets meaning and produces a revision-bound proposal with warrant and uncertainty. A semantic reviewer is not an acceptance authority unless the constitution or an explicit human decision grants that authority.

**Terminal decision** is a reserved human disposition—adopt, reject, narrow, park, reframe, or explicitly continue—required because an appetite ended or an accepted authority gate was reached. An ordinary implementation judgment, reviewer finding, or unresolved technical preference is not a terminal decision.

**Completed work item** is a bounded issue, pull request, or equivalent unit that has reached a terminal disposition—accepted, rejected, parked, superseded, or otherwise closed. A session, commit, status update, or amount of elapsed time is not a completed work item.

**Work-control plane** is the one named system that owns active work, priority, readiness, dependency state, acceptance, and closure for the repository.

## 4. Semantic authority and acceptance

### AUTH-1 — Human semantic authority and bounded runtime behavior

**Trigger:** Any generated, interpreted, selected, migrated, or inferred content that may enter persistent repository state, canonical domain state, or an externally consequential action.

**Rule:** The universal authority chain and human-selection limitation in the bootstrap MUST govern the transition. LLM-produced repository changes, foundations, canonical facts, accepted datasets, specifications, and persistent semantic state remain proposals until the acceptance authority accepts the exact decision surface. When a mechanism or prerequisite is itself selected as the immediate primary outcome, it MUST still yield an inspectable benefit for a named real use or remain a bounded experiment with a named consumer effect, stop condition, and removal path.

A product MAY execute runtime LLM behavior without per-output human approval only inside a human-accepted policy covering permissions, available state, persistence boundaries, side-effect classes, refusal behavior, failure handling, publication limits, and means of human intervention. Runtime output does not become canonical merely because the product emitted, displayed, or acted upon it.

### AUTH-2 — Acceptance is granular and non-transitive

**Trigger:** A proposal, patch, migration, batch, repair, derived consequence, or plan may mutate authoritative state.

**Rule:** Every independently rejectable decision atom MUST be visible within the accepted decision surface and classified as one of:

- **entailed:** mechanically or logically follows from an already accepted premise;
- **required repair:** necessary to preserve an accepted invariant or resolve a contradiction; or
- **discretionary:** an optional extrapolation, integration, retcon, addition, or creative choice.

Entailed atoms MAY be accepted in a visible batch that names their premise. A required repair MUST expose the protected invariant and materially valid alternatives, including narrowing or rejecting the parent premise when applicable. Discretionary atoms remain separately deselectable. Approval of a premise, issue, plan, mechanism, token, batch label, or patch envelope MUST NOT authorize omitted or subsequently generated descendants. Atoms MUST be localized so rejecting one does not require disentangling unrelated accepted state.

Visibility is about consequential commitments, not a separate approval for every field, rule sentence, data row, identifier, or file. A coherent design proposal MAY expose related choices together through a concise account of behavior, alternatives, consequences, and remaining uncertainty, bound to its exact technical revision. Materially independent discretionary commitments remain identifiable and rejectable. Supporting files may carry detail without becoming required human reading; concealed decisions and blanket approval of unknown descendants remain forbidden.

Acceptance or activation of a bounded execution-ready work item delegates to the implementing agent all implementation judgment necessary to satisfy its accepted contract. Such choices remain reviewable through the resulting patch and evidence, but they are not separately human-governed decision atoms, approval gates, or stop conditions. When they remain internal and reversible, this includes naming, local identifiers, file placement, decomposition, algorithms, internal data shapes, fixture or replay-witness shape, test strategy, refactoring, and consistency choices. The existence of several defensible alternatives does not change this classification.

A work item, agent, reviewer, or decision record MUST NOT convert an implementation judgment into a reserved human decision merely by assigning it an identifier, enumerating alternatives, or asking the human to accept it separately. An otherwise ordinary implementation choice MAY be explicitly reserved only by an accepted human instruction whose acceptance surface names the concrete human consequence, risk, or authority reason requiring human selection; burying a reservation in agent-authored implementation detail is insufficient. Delegated implementation judgment produces a proposal, not authoritative state; final acceptance and reserved human decisions remain governed by `AUTH-1` and `AGENT-5`.

### AUTH-3 — Decision-bearing results expose their authority

**Trigger:** A result can block, promote, reject, score, accept, publish, mutate canonical state, or trigger an irreversible or external action.

**Rule:** Decision-bearing causality and state transitions MUST be explicit, inspectable, and testable at the authority boundary they affect. Domain reasoning, authoritative state, and workflow control remain distinct responsibilities. The result MUST expose its owner, input revisions, decision rule or oracle, authority level, evidence, relevant uncertainty, validity limits, failure behavior, and recheck conditions. Use authority-specific names:

- a **mechanical validator** checks syntax, shape, identity, reference closure, arithmetic, freshness, or another formally stated property;
- a **semantic reviewer** interprets meaning and produces a proposal; and
- an **acceptance authority** decides what becomes canonical.

The unqualified word `validator` MUST NOT conceal these roles. Mechanical validity of an envelope does not establish the truth or acceptance of its contents. A decision-bearing claim naming a source MUST verify its existence, identity or revision, and production or deterministic derivation of the attributed values. Otherwise label the value as a manual assertion; do not present it as provenance-verified evidence.

A human decision surface MUST be intelligible without reconstructing the agent's technical context. It MUST explain the behavior, claim, risk, or authority that acceptance changes; the recommendation and material alternatives; evidence and uncertainty; reversibility; and any blocked-work consequence. It MUST bind the exact proposed revision and separate demonstrated results, reasoned expectations, and unresolved conditions. A concise explanation linked to evidence, selected examples, or an actual demonstration is sufficient. Raw schemas, large data files, identifiers, evidence packets, and exhaustive records do not by themselves constitute an intelligible decision surface. Technical source files need not be personally readable when the human can judge the material commitments and consequences through an adequate view.

The agent remains responsible for technical investigation and verification within its competence. Human acceptance authorizes the declared commitment; it does not supply missing execution evidence, certify unexamined technical facts, or discharge an unmet requirement. A disclosed deficit remains unmet until resolved or an explicit human decision changes the promise. A title, success token, interpretation, or recommendation MUST NOT call the original requirement satisfied while exclusions or qualifications remove its defining property. Recommend the changed promise plainly and state its consequences; apply `ADOPT-2` when a constitutional safeguard would be relaxed. Requests for choices also satisfy `AGENT-5`.

## 5. Purpose, activation, and demand

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

### PURP-3 — One selected value stream and primary path

**Trigger:** Work is selected, reprioritized, or challenged.

**Rule:** The repository MUST distinguish primary outcomes, enabling work, stewardship, and deferred possibilities. The human selects or accepts the active value stream and bottleneck. Work MUST follow the primary path from that bottleneck to an acceptance-bearing increment. An adjacent opportunity gains no priority merely because it is elegant, useful, partly implemented, testable, or present in the repository.

An agent MAY challenge the stated bottleneck with evidence, but MUST NOT redefine it to fit a preferred architecture. Several locally legitimate work items that collectively leave the primary path constitute an allocation failure under `WORK-3`.

When the selected outcome spans repositories, the primary path MUST name the next actual consumer use and the unresolved handoff that could invalidate the investment. Repository-local acceptance criteria MUST NOT exclude that risk and then treat local acceptance as discharging it. An upstream artifact may have independent value, but continued investment in its downstream promise follows the evidence required by `EVID-3` and `INT-1`. Reporting a missing consumer repeatedly does not make the consumer unnecessary.

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

## 6. Work governance and stopping conditions

### WORK-1 — Work selection is a governed transition

**Trigger:** An issue or work item is accepted or activated; an execution-ready or AFK label is applied; work begins outside accepted scope; an adjacent lane opens; scope materially expands; or foundational, shared, governance, migration, substrate, or dependency-bearing work is proposed.

**Rule:** Before activation, identify the selected primary outcome, current bottleneck or earliest unresolved prerequisite, and smallest acceptance-bearing primary-path increment. Direct work on that increment MAY proceed under existing accepted authority. Enabling, research, governance, migration, substrate, or adjacent work MUST state, in the existing authoritative issue or decision, the bottleneck, primary-path consequence, current consumer or blocked human decision, more direct work displaced, and stopping or return condition.

If those facts cannot be named, the work MUST be deferred or parked. Local correctness, a real defect, testability, issue presence, a label, available code, or a detailed plan does not authorize selection.

### WORK-2 — Speculation is bounded, removable, and not adopted by implementation

**Trigger:** Work rests on an unproven premise, temporary bootstrap, candidate architecture, or experiment.

**Rule:** Give the premise an appetite proportionate to its decision, including semantic surface and human burden. An artifact count is not an appetite when internal obligations can expand without bound. When an experimental artifact could be mistaken for primary progress or prerequisite readiness, classify it under `EVID-4` before implementation. A temporary bootstrap that may constrain later work MUST name its owner, bounded purpose, allowed consumers, prohibited dependents, appetite, replacement or removal condition, and expiry event or date. It MUST NOT gain consumers or become canonical through inertia.

Approval to implement authorizes an experiment, not adoption. Outside work MAY treat the premise as binding only after the human inspects a real end-to-end result and explicitly adopts the exact premise and scope. Human doubt narrows the work to the cheapest bounded falsifying experiment and blocks dependency fan-out. If the premise fails, is abandoned, or remains unadopted when its appetite ends, remove it from active agent context, active dependency graphs, and default publication paths; rejustify any survivor independently. There is no duty to salvage it.

Before the defining human, consumer, or behavioral effect of an experimental premise is accepted, the repository MUST NOT grant its provisional behavior product-like authority through broad golden-trajectory or snapshot locks, mutation-testing campaigns, certification programs, exhaustive conformance matrices, or governance whose primary purpose is preserving that behavior. Narrow tests, replay, diagnostics, and invariants needed to make the experiment safe, inspectable, reproducible, or decidable are permitted, as are proportionate protections for security, privacy, corruption prevention, irreversible side effects, and explicit authority boundaries. After the exact effect and scope are accepted, further hardening remains subject to current demand, operational burden, the primary path, and the governed lifetime of the behavior.

### WORK-3 — Repository allocation tripwire

**Trigger:** Completed work accumulates without an acceptance-bearing primary-path increment.

**Rule:** Each active repository MAY set a human-observable allocation unit and maximum consecutive streak. In the absence of an accepted repository-specific limit, the default is **three completed work items**. Opening another issue, changing lanes, performing cleanup, or completing locally correct enabling work does not reset the streak. Where repositories share one unresolved handoff or premise, moving the next task across that boundary does not reset the governing streak or create a fresh allowance for the same uncertainty.

A status change, approval of an intermediate artifact, or acceptance of its disclosed limitations does not reset a streak whose defining consumer effect remains unexercised. Report the artifact's narrower result accurately and retain the unmet premise. Reclassifying missing proof as an exclusion, deferred integration, or a non-blocking deficit does not discharge it. An independently useful producer outcome may be accepted on its own terms without establishing the dependent outcome.

For an experiential, interactive, or authoring-facing primary outcome, only a human-inspected representative real-use run through the affected path can reset the streak. The run MUST yield an acceptance-bearing human effect or resolve the earliest premise or prerequisite currently blocking one. It resets the streak only for the named premise, effect, or path it actually exercises under `PURP-2` and `EVID-3`. A pleasant adjacent activity, polished presentation, enthusiasm for available possibilities, or positive result on another effect does not reset a streak whose governing premise concerns an unexercised distinctive activity loop, voluntary recurrence, return use, recovery, assurance, or another materially separable claim. An isolated action, state transition, renderer, schema, validator, simulation, diagnostic, coverage increase, refactor, or successful subsystem demonstration does not reset the streak merely because it works.

When the tripwire fires, further work items MUST NOT be selected until the human explicitly continues, reprioritizes, reframes, or parks the repository. Continuation MUST state why the same direction remains justified and renew the scope, appetite, and allocation bound. No session-count alternative applies.

### WORK-4 — Demand-null tripwire

**Trigger:** A capability's accepted benefit claim depends on encountering positive cases, producing interventions, or causing consumer consequences, and real-input runs produce none.

**Rule:** Count only qualifying opportunity runs. Before interpreting a run, declare the exposure unit or basis that makes a positive case plausible at the frequency assumed by the claim. A negative result explicitly accepted as the product—such as assurance, validation, or a clean bill of health—is not demand-null merely because it causes no change. Rare-event capabilities SHOULD ratify a suitable exposure unit or threshold before runs begin.

Unless another threshold was accepted in advance, the **second consecutive demand-null qualifying run** stops further work on the capability and on substrate whose only accepted consumer is that capability. The existence or assumed frequency of demand becomes the claim under test and requires human reprioritization. Correct behavior and honest negative reporting do not reset the count; they make it informative.

### WORK-5 — Terminal decisions freeze the repository

**Trigger:** An appetite expires or is exhausted, or accepted work reaches a point requiring a terminal decision.

**Rule:** All discretionary work across the repository MUST stop until the exact decision is durably recorded. Changing lanes, filing or repairing another issue, cleanup, refactoring, architecture work, and allegedly small improvements do not evade the freeze.

Only work strictly necessary to assemble evidence for the outstanding decision, preserve or quarantine the exact state, or minimally contain imminent data loss, credential exposure, or immediately exploitable security harm MAY continue. Emergency containment remains narrow, is recorded separately, and does not displace the outstanding decision as the next repository action. Continuation or extension requires fresh human reprioritization with explicit scope and appetite.

Count consecutive extensions of the same outstanding terminal decision, including extensions carried across a successor repository under `PURP-1`. Each extension MUST name the specific evidence the decision still lacks, why the evidence already accumulated cannot decide it, and the exact scope intended to obtain it. At most **two consecutive extensions** are permitted after the original appetite. When the second extension expires or is exhausted, a third is unavailable: the acceptance authority MUST decide on the evidence in hand by adopting, rejecting, narrowing, parking, or reframing. The count resets only when the acceptance authority records a materially different primary claim or terminal decision. Changing repository, lane, mechanism, or presentation; renaming or cosmetically narrowing the slice; or producing another locally successful increment does not discharge the decision or reset the count.

### WORK-6 — Consumers drive substrate; future compatibility stays cheap

**Trigger:** Storage, validation, rendering, audit, release, governance, or another substrate is proposed for a capability or consumer that has not yet executed its smallest real slice; or a future cross-cutting requirement is cited.

**Rule:** The first real capability execution or existing consumer contract MUST drive only the minimum local substrate needed for that slice. Disposable scaffolding MAY exist inside the slice. A general or shared substrate MUST NOT run first and alone; without the consumer it derives requirements from itself and becomes a different project requiring re-ratification, its own primary outcome, consumer, appetite, and removal path. A seam in storage, validation, rendering, audit, or release does not prove the upstream capability that never ran. Skipping a declared precondition or derivation path is a mechanism substitution and requires re-ratification.

A future requirement MAY justify a cheap, reversible compatibility choice or smoke test when later retrofit would be unusually costly. It MUST NOT justify exhaustive implementation, validation, or supporting infrastructure before real demand exists.

## 7. Evidence, experiments, and generalization

### EVID-1 — Founding memories are premises, not authority

**Trigger:** A mission or acceptance program is motivated by a remembered episode, prior incident, recurring frustration, or origin story.

**Rule:** Check the account against surviving version history, artifacts, issues, logs, outputs, or other inspectable records before it authorizes dependent architecture. State what the record supports. A contradiction requires the mission to be re-derived before dependent work continues. If adequate evidence no longer exists, mark the claim unverified and keep work resting on it experimental. Memory may motivate investigation; it MUST NOT manufacture architectural authority.

### EVID-2 — Falsify dependencies in order; research serves a decision

**Trigger:** Important premises are uncertain, or research or experimentation is proposed.

**Rule:** Order uncertainties by dependency and attack the earliest unresolved premise whose failure would invalidate later work, using the cheapest evidence adequate to decide it. A downstream experiment MAY run ahead only when it is bounded and can materially change whether or how its prerequisite should be pursued. A risk gains no priority merely because it is concrete or satisfying to engineer.

Unless a research artifact is itself the accepted primary outcome, the experiment MUST name the blocked decision, claim, sufficient evidence, appetite, and possible terminal outcomes before work. Whenever an experiment reaches appetite exhaustion, is offered to justify adoption, continuation, narrowing, or reorientation, or is about to be relied upon by dependent work, durably record in an existing authoritative artifact the claim tested; the materially relevant revision, input, configuration, evaluation, and output identities as applicable; representative observations needed to support or challenge the conclusion; uncertainty and limitations; what the evidence does and does not establish; and the resulting adopt, reject, park, narrow, or reframe decision.

Preserve only the evidence and identities proportionate to that decision and a plausible next experiment. Bulk or generated outputs MAY remain outside Git when a representative sample or stable, retrievable, integrity-checkable binding is sufficient. A disposable decision-only probe need not become a maintained product, and this rule creates no repository-abandonment ritual, publication-grade report, experiment registry, recurring status duty, or obligation to reconstruct missing historical evidence. A fixed research-gate sequence, unanswered-question inventory, issue count, or closure history is not a value stream and creates no duty to continue.

### EVID-3 — Produce the artifact before the system around it

**Trigger:** A general system, workflow, framework, catalog, large generated network, semantic scale-up, or externally governed fidelity promise is proposed.

**Rule:** First produce or materially revise a real primary artifact or substantial risk-bearing slice through the proposed path. The slice MUST exercise the actual authoring, semantic, testing, runtime, and presentation risks that bear the claim. When look, feel, motion, latency, readability, interaction, or another presentation quality is material, the slice MUST reach enough fidelity to judge it; a misleading debugging abstraction is not evidence.

For an externally governed fidelity promise, the first acceptance-bearing slice MUST be selected from independently derived hard cases and be hostile to the proposed representation or execution model. Where the fidelity obligation topology contains material cross-cutting interactions, the slice MUST exercise at least one such interaction through the real authoring, authority, execution, and oracle path. A tutorial, known script, simple scalar adjustment, isolated rule, per-item transcription milestone, or happy-path example is a seam probe unless it is itself the selected primary outcome. This requirement MUST NOT manufacture interactions absent from the governing source; the hostile slice is proportionate to the actual topology.

For an experiential, interactive, or authoring-facing primary outcome, the slice MUST permit the human to complete or intentionally terminate a bounded current task through the actual path. An isolated mechanic, one content node, one state transition, or one successful subsystem operation is a seam probe unless that operation is itself the selected primary outcome.

For an experiential claim decomposed under `PURP-2`, each distinctive activity loop that materially justifies separate capability, persistent state, recurrent operation, or substantial burden MUST be exercised in an acceptance-bearing slice before that machinery or burden materially expands. Merely presenting or observing the option, completing an adjacent path, or enjoying related prose, choices, consequences, or presentation is evidence only for the effects actually experienced.

A deliberately diagnostic run MAY create a credible need or opportunity to ensure that a rare, optional, or unfamiliar loop is genuinely experienced. When recurrence is part of the claim, diagnostic completion alone is insufficient: after the human understands the loop, a later representative ordinary-use run MUST give the human a genuine choice whether to repeat or return to it, and recurrent fitness requires that the human elect to do so. Voluntary non-use or rejection is evidence against recurrent demand, not a defect that authorizes repair.

An optional or rare-event mechanism need not recur spontaneously when the selected effect is occasional recovery, assurance, accessibility, availability, or possibility value rather than recurrent activity. The slice MUST explicitly name that value and credibly create or expose the relevant condition. The human MUST either use the mechanism or, when the selected effect lies in availability or assurance rather than activation, inspect enough of the real consequence and burden to judge that effect. Acceptance authorizes only that mode of value; it does not establish desire to repeat the activity or demand for broader persistent machinery.

When evidence accepts one separable effect but rejects or leaves unproven another effect that justifies material machinery or scope, the supported effect remains valid evidence. Further work on the unsupported machinery MUST stop pending a terminal decision under `WORK-5` to narrow, reframe, reject, park, or explicitly continue that claim with a renewed appetite. An agent MAY recommend the least-complex path that preserves supported effects but MUST NOT silently substitute a materially different primary outcome.

Before an experiential, interactive, authoring-facing, configurable, reusable, or engine-like path may be adopted as recurrent or materially expanded, it MUST pass a materially different second-use run through the same accepted authoring or configuration, runtime, and presentation path. For a recurrent experiential claim, this run MUST include the later ordinary-use election described above; a second forced diagnostic repetition does not qualify. The run MAY add ordinary content and configuration, use a bounded consumer-owned adapter at an existing seam, and repair a defect in behavior already within accepted scope. It does not establish recurrent fitness when completion depends on introducing a new capability class, a new level in the core ontology, a shared-schema expansion solely to admit the case, a new required external service, or synchronized changes across unrelated capabilities. A defect that prevents the bounded task from completing or conceals its material burden requires the run to be repeated after repair; smaller defects remain evidence against operational fitness even when the repaired run succeeds.

When the selected outcome or product promise includes periodic reuse, continued use beyond the maintainer's fresh memory, handoff, or recovery after interruption, recurrent fitness additionally requires a return-use run from durable repository state. The run MUST NOT depend on reconstructing material setup, state, or procedure from conversation or unauthored memory. A clean handoff MAY substitute for elapsed interruption when it exposes the same discoverability and recovery burden. A bounded single-use experiment or artifact that makes no recurrence claim need not pass these runs and MUST NOT be expanded into a general system merely to satisfy them. Passing these runs authorizes only the declared recurrent path and does not replace the materially different cases and holdout required by `EVID-6` for a broader generalization claim.

For an autonomous, agentic, adaptive, or simulation claim whose value depends on decisions producing consequences and feedback, the first acceptance-bearing slice MUST complete the smallest causal loop that bears the claim: a candidate decision is valid under current authority; execution produces an inspectable consequence, explicit failure, or deliberate non-action; relevant perception or feedback occurs; any claimed memory, belief, plan, or adaptation persists at the fidelity relied upon; and a later decision or human effect materially depends on that result. Planning, retrieval, movement, state mutation, event delivery, or narration demonstrated separately are seam probes unless one is itself the selected primary outcome. The loop MUST cross the causal scale and semantic boundary named by the selected effect. An individual action loop does not bear a claim about household, organizational, settlement, regional, ecological, economic, or long-horizon consequences unless the slice reaches that boundary and the later decision or human effect depends on it. The slice need not prove every population, final scale, or final architecture; it MUST establish the exact claimed loop and effect before broader scale.

When the accepted claim concerns a causal tendency, emergent pattern, counterfactual wake, or behavior expected across seeds, configurations, populations, or nearby model variants, exact replay of one trajectory is insufficient. Before material scale or product-like hardening, compare the relevant intervention with a control, prior state, or other credible counterfactual and exercise bounded independently selected seeds, cases, or nonessential perturbations proportionate to the claim. The evidence MUST show what direction or family of outcomes is attributable to the intervention, what diversity is legitimate, which changes are intended to matter, and whether the claimed effect survives declared nonessential variation within accepted qualitative bounds. No universal replication count, statistical threshold, or demand for trajectory similarity applies. A deterministic one-off whose complete accepted claim is the exact trace is exempt. A product that deliberately seeks chaotic divergence, fragile equilibria, or butterfly effects MAY accept sensitivity as part of the human effect, but it MUST distinguish causally legible sensitivity from arbitrary dependence on incidental identifiers, ordering, fixtures, or unexamined implementation details.

When memory, search, retrieval, or ranking materially bears the slice, independently selected cases MUST distinguish a relevant result from a merely nearest or highest-scoring result and exercise confusable alternatives, absent-answer behavior, and changed or superseded information when those are part of the accepted claim. Judge the downstream decision or human effect as well as retrieval mechanics; a plausible neighbor list or aggregate hit metric alone does not prove downstream memory or retrieval fitness unless that metric is exactly the complete selected outcome.

When a search, evolutionary, repeated-generation, automated-variation, or adaptive-operator mechanism claims cumulative improvement, learned operator effectiveness, or efficient exploration rather than merely bounded sampling, the first acceptance-bearing slice MUST exercise the representation, candidate generation or variation, validation, material repair or post-processing, evaluation, and selection stages actually relied upon. Before material scale, that path MUST demonstrate a productive candidate neighborhood: enough valid, materially distinct, inspectable candidates reach the accepted evaluation surface at acceptable cost to support the claimed mode of search, and downstream transformation does not silently erase the lineage on which improvement or operator learning is attributed. The repository MUST state the observed limitations and leave unsupported operators, regions, or generality unproven. No universal yield threshold or search-space metric is required. Cheap exhaustive enumeration, fuzzing, random sampling, one-off creative generation, or another bounded search that makes no cumulative-improvement, operator-learning, or search-efficiency claim MAY proceed under the ordinary evidence and appetite rules.

For an authored specification, definition, configuration, or package whose selected promise includes downstream implementation or operation, the first risk-bearing slice MUST reach the intended consumer before bulk authoring, repeated complete packages, or supporting machinery materially expand. Bounded exploration of alternative designs remains permitted under `EVID-7`. The consumer MUST use the actual revision-bound authored material in the manner claimed, exercise representative difficult behavior against independently derived expectations, and expose the material consequence. An interactive promise also requires the bounded real-use task already specified above. A headless run can establish the behavior it executes; it does not establish interaction or user experience. `INT-1` distinguishes implementation from a specification from runtime interpretation of data.

If the consumer does not exist, the next bounded work must establish that first consuming path or obtain a human decision to narrow, park, or abandon the downstream promise. A separately selected producer product may proceed for its independent use under `PURP-5`; naming a future consumer does not prove the handoff. Review claims, reference joins, and executable models that bypass the authored input or intended consumer cannot satisfy this requirement.

A completed accepted work is stronger evidence than a comprehensive process. Work MUST NOT scale to hundreds of interdependent records, content nodes, rules, generated assets, or equivalent semantic surface before a small representative slice produces the intended effect at acceptable authoring, validation, and maintenance cost. A legacy success proves its own path, not the fitness of a replacement path.

### EVID-4 — Evidence is limited to the claim it bears

**Trigger:** A test, prototype, benchmark, component, review, simulation, metric, or end-to-end probe is offered as evidence.

**Rule:** Distinguish a **seam probe**, which proves that real input can cross a boundary and produce some real output, from an **acceptance-bearing vertical slice**, which proves that the path can express, produce, validate, and operationally sustain the smallest accepted real demand with its required configurations, populations, controls, scale, provenance, refusal behavior, and verdict path. Disconnected demonstrations MUST NOT accumulate into proof of stack, platform, framework, product-line, or product fitness.

For a specification or data handoff, distinguish source review, structural validity, declared coverage, worked expectations, execution in the intended consumer, and representative human use. Each supports only its stated property. An identifier join establishes a declared relationship, not that rules entail an outcome. A reference model establishes behavior of that model under its inputs, not behavior of another implementation or successful consumption of an authored package. A receipt records evidence; its existence does not create it. Independent expectations MAY come from source-derived examples, calculations, invariants, or an independently justified model, but implementation output MUST NOT define its own expected answer.

Evidence of source fidelity and evidence of consumer utility are separate claims. Reproducing selected source behavior does not establish that the implementation is useful, usable, maintainable, or worth its burden; helping a consumer does not establish fidelity to the governing source. When both claims justify the repository, each MUST have its own acceptance evidence and failure disposition. The acceptance authority MAY accept one and reject, narrow, park, or abandon the other without creating a duty to continue.

When the distinction affects priority or adoption, classify a prototype or experiment in its existing authoritative issue or decision as **product-bearing** or **decision-only** before implementation or, for an existing artifact, before treating its output as progress. A decision-only experiment exists to answer a named decision. It MAY be disposable and MAY require high presentation or operational fidelity, but its acceptance authorizes only the claim and decision it was designed to bear. It MUST NOT be counted as progress toward a downstream primary artifact, prerequisite readiness, or a reusable product merely because it is polished, extensive, expensive, or shaped like the eventual product. Later promotion of its implementation requires separate satisfaction of the applicable real-input, authority, acceptance, boundary, and publication rules; implementation inertia is not promotion.

Presentation quality, visual or physical tangibility, enthusiasm, motivation, communication value, emotional orientation, or resemblance to the eventual product is evidence only of that explicitly selected human effect. When such an effect is explicitly selected under `PURP-1` and stated under `PURP-2`, a bounded artifact MAY be accepted for it at an appetite proportionate to that effect even when it resolves no product premise. Unless separately designed and accepted to bear another named claim, the payoff MUST NOT be represented as testing or advancing upstream design, stabilizing requirements, satisfying a prerequisite, establishing downstream production readiness, or justifying reusable production machinery. The artifact's appetite, prohibited dependents, and parking or disposal condition MUST be explicit; any later promotion remains subject to `PURP-4`, `PURP-5`, and the applicable real-input, authority, acceptance, boundary, and publication rules.

When the accepted claim attributes improvement to feedback, training, ranking, search, adaptation, optimization, repeated selection, or another mechanism that changes candidates in response to an evaluation signal, the evidence MUST distinguish that mechanism's contribution through a credible simpler, non-adaptive, null, ablated, or prior-state comparator, or a held-out evaluation, under materially comparable inputs, opportunity, and relevant resource budget. Use the cheapest matched comparison adequate to the claim. The `PURP-4` comparator MAY satisfy this obligation when it tests the same causal claim; no second tournament, universal random control, formal statistical study, or publication-grade protocol is required unless the accepted claim needs that strength. A mechanism selected as a learning exercise, implementation exercise, exploratory instrument, or recreational medium MAY be built and used without proving superiority, but any claim that its adaptation caused cumulative improvement remains experimental until this comparison bears it. If no meaningful comparison can be obtained, state the limitation rather than claiming the effect.

When a generator, search operator, model, policy, or earlier stage receives causal credit for a final candidate that is materially changed by repair, retry, normalization, compiler or test feedback, filtering, reranking, human editing, or another downstream transformation, the evidence MUST preserve lineage and use an adequate ablation, comparator, or stage-specific measurement to isolate the credited contribution. A transformative stage is a separate mechanism and cost. Improvement by the combined pipeline MAY be claimed without isolation when that combined pipeline is exactly the claim, but the result MUST NOT be attributed to an upstream stage alone. Routine formatting or semantics-preserving normalization does not trigger this obligation.

A reward, fitness function, training objective, benchmark score, optimization target, user-selection signal, or repeatedly consulted evaluation set MAY guide candidate generation or selection. Improvement on that guiding signal establishes only the narrow declared objective on that exposed evaluation surface. It does not by itself establish generalization, causal improvement, consumer utility, superiority, interesting behavior, or acceptance beyond that narrow signal. A broader claim requires a held-out or otherwise independently constituted evaluation surface, representative case, or human comparison derived from the accepted effect rather than selected by what the candidate already optimizes. The guiding signal MAY serve as acceptance evidence when satisfying it is exactly the complete accepted claim and no broader effect is asserted.

When a decision-bearing score aggregates materially different properties, its component values, directions, normalizations, weights, uncertainty where relevant, and aggregation or compensation policy MUST remain inspectable. For the stated decision, the acceptance authority MUST distinguish properties that may trade off from requirements or invariants that are non-compensable. Failure of a non-compensable requirement MUST remain a gate, veto, abstention, or explicit failure; improvement elsewhere MUST NOT conceal it. A composite score MAY guide search, ranking, triage, or summary, and MAY be the acceptance oracle only when the exact aggregation and trade-offs are themselves the complete accepted claim. When unresolved but plausible choices of weights, normalization, or aggregation can materially reverse a decision, expose that sensitivity rather than present one scalar as determinate.

When simulation, self-play, synthetic users, model judges, surrogate models, or other artificial evaluators materially bear a claim whose interpretation depends on their competence or behavioral coverage, independently establish that relevant competence or coverage on known, planted, held-out, or otherwise independently adjudicated cases at the level the claim requires. A weak, deliberately restricted, or uncalibrated evaluator MAY provide a baseline, conditional result, diagnostic, or valid counterexample; its failure to find a behavior or defect MUST NOT establish absence beyond its demonstrated coverage. If no adequate competence or coverage oracle exists, report behavior conditional on the named evaluator population, configuration, and validity limits rather than attributing the property generally to the evaluated system.

A fixed-seed replay, golden trace, snapshot, or deterministic trajectory establishes reproducibility only under its pinned conditions. It does not by itself establish causal attribution, behavioral robustness, stability, diversity, generalization, or that the same qualitative effect occurs after nonessential changes. Those claims require the proportionate control, counterfactual, perturbation, or independently selected case evidence in `EVID-3` and, when broader reuse or generality is claimed, `EVID-6`. Conversely, sensitivity to a deliberately changed causal input is not a robustness failure; the accepted claim must distinguish relevant interventions from nonessential perturbations and state what variation is desirable.

Simulation and statistics MAY diagnose behavior, compare declared properties, or falsify a measurable claim. They MUST NOT manufacture meaning, taste, fun, desirability, or human acceptance. More passing tests increase confidence only relative to a valid oracle. Evidence, fixtures, consumers, or obligations created solely by the candidate architecture to exercise itself establish internal consistency at most; they do not prove necessity, demand, fitness, authority, or the causal contribution of the mechanism.

### EVID-5 — Real inputs authorize only local first paths

**Trigger:** Substantive architecture or implementation is about to begin.

**Rule:** Work MUST begin from at least one accepted real input at the fidelity the first promise consumes. Synthetic fixtures and hypothetical payloads MAY test mechanics or falsify a mechanism, but MUST NOT authorize a canonical representation, generalized subsystem, repository-wide architecture, or backlog.

One real input authorizes only a deliberately local, replaceable first path. Broader promises require materially different cases and `EVID-6`. Early cases MAY use scenario-specific code, explicit branches, duplicated data, or other deliberately local special cases when they are bounded, correct, safe, visible, and cheaper to replace than a speculative abstraction. Agents MUST NOT introduce a generalized subsystem merely to remove repetition before materially different real pressure earns it under `EVID-6`.

This permission does not excuse inconsistent authoritative state, hidden coupling, security or correctness defects, or duplicated logic whose current divergence is itself the bottleneck. A clean local first implementation is preferable to a speculative reusable layer, but local clarity and repair remain required.

### EVID-6 — Generalization follows materially different pressure

**Trigger:** A representation or capability is claimed to be reusable across products, domains, repositories, or independent consumers; or it will constrain them as a shared contract.

**Rule:** Adoption MUST use at least **three materially different real cases** during design and **one further holdout case** not used to design it, unless one of the exceptions below applies. Count alone is insufficient: cases MUST exercise different pressures. For a cross-product or product-family claim, the cases MUST come from materially different named products or independent consumers. For another generalization claim, they MUST arise from independently selected product-bearing artifacts or current accepted consumers. Synthetic variants, planned future products, decision-only probes, and nominal products created merely to manufacture reusable demand do not qualify. Target-specific language, fields, defaults, fixtures, source choices, or assumptions discovered in supposedly general material block adoption until removed or quarantined and the holdout is re-evaluated.

A repository MUST NOT be founded or expanded as the permanent home of an imagined family of future products merely to maximize reuse. Product-specific work remains local and separately abandonable unless `BOUND-2` independently permits co-location. Once materially different real products create repeated pressure, observed commonality MAY be extracted under `BOUND` and `INT` into an explicitly versioned shared component or repository. Publication status, confidentiality, licensing, and the wish to keep reusable domain material private affect custody and distribution; they do not establish demand, earn generality, justify a multi-product repository, or waive the required cases. A justified shared capability MAY be public or private.

A smaller set is permissible only for an intrinsically finite problem, an external formal standard that fully governs the representation, or a scoped waiver. This rule does not apply to ordinary implementation-local refactoring, helper extraction, or deduplication whose behavioral contract is mechanically evident and which makes no cross-product, domain-neutral, or published reuse claim. Such code remains local and reversible until broader pressure exists.

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

### EVID-10 — Longitudinal mechanisms must outlive their dependencies

**Trigger:** A mechanism's value materially depends on evidence, participation, or action arriving later, and accumulated state can be stranded by participant disappearance.

**Rule:** State the expected wait and compare it with observed participant lifetime or availability when measurable. If survival cannot be measured, declare the uncertainty and keep the mechanism a bounded experiment. When the required wait is material relative to observed survival, viability MUST be tested before adoption; likely participant disappearance is not merely an operational edge case.

Whenever one participant exclusively owns authoritative accumulated state, define how that state and authority are transferred, recovered, retired, or explicitly lost when the participant disappears. Ordinary deferral that remains valuable and usable without the participant does not trigger this rule.

## 8. Semantic evaluation and representation

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

### REP-2 — Representation and state-model fitness use an independent oracle

**Trigger:** A record format, authored specification, schema, ontology, persistent state model, simulator, deterministic consequence model, migration system, or renderer may constrain other work or an intended consumer. A product-specific label does not exempt a representation whose adequacy bears a downstream promise.

**Rule:** Before adoption, a risk-bearing slice MUST prove the intended direction:

> real authoring or genuine revision → inspectable proposed record or state → human-accepted primary artifact or effect in the form its real consumer uses

The primary artifact MAY be the structured record itself, a derived render, or an executable artifact. A second representation MUST NOT be required without a real consumer, and claimed consumer benefit MUST improve an actual consumer task.

A category of explicit state or deterministic consequence becomes authoritative only when a current accepted real case requires it to persist independently of narration, coordinate multiple actors, turns, or systems, drive a deterministic or externally inspectable consequence—including uncertainty or failure—support recovery, or serve another named accepted consumer. The possibility that the domain or fictional world could contain the fact, object, relation, body part, garment, action, or edge case is not demand. Begin with the smallest local state and transition authority needed by the current case; broader physical or semantic simulation must earn generality under `EVID-6`.

When a simulation spans more than one causal scale, no resolution level has presumptive authority. The representation used for each current phenomenon MUST be justified by the selected effect and the distinctions its accepted consumers actually use. Individual people, households, organizations, settlements, cohorts, aggregates, scheduled processes, and summary events are all legitimate authority levels. Lower-level completeness MUST NOT gain authority merely because it appears more fundamental, realistic, reusable, or capable of eventually generating higher-level behavior. Individual detail is required when the accepted case depends on identity, embodiment, custody, knowledge, timing, relationship, consent, or interaction that aggregation would erase; aggregate or summarized behavior cannot substitute for a promise about those distinctions. Conversely, an individual-level substrate MUST NOT be treated as a prerequisite for a household-, market-, settlement-, ecological-, or regional effect that can be tested credibly at a coarser level. Any promotion, demotion, aggregation, or disaggregation relied upon by a consumer MUST preserve the authoritative continuity and causal ancestry that consumer needs, without transferring knowledge, rights, or detail that the source representation did not establish.

When an LLM participates, the accepted boundary MUST distinguish facts it may narrate freely, state changes it may authoritatively produce under the accepted runtime policy, changes it may only propose, and consequence classes governed by deterministic or human authority. The LLM MUST NOT cross that boundary merely by narrating a convenient success, failure, discovery, injury, movement, or other governed outcome. Conversely, deterministic state MUST NOT expand to control narrative detail that no current accepted consequence or consumer requires.

Where a controlled system defines or mechanically derives legal actions, valid targets, preconditions, timing windows, resource limits, or consequence classes, those constraints are authoritative. An LLM MAY select, rank, sequence, or parameterize within the accepted boundary, and MAY propose a novel action for separate validation, but it MUST NOT make an unavailable action or target executable by naming, scoring, or narrating it. Any semantic matching or ranking at this boundary remains subject to `SEM-1`. Free-form LLM action authority remains a legitimate product or bounded experiment only when explicitly accepted and when its risk-bearing slice proves the required validity, failure, consequence, recovery, and intervention behavior as applicable.

The candidate representation MUST NOT choose or filter the target inventory, evaluation scope, fixtures, hard cases, or oracle used to validate its own adequacy. Derive them independently from accepted consumer tasks, semantic distinctions, authority states, preservation obligations, and source material selected without regard to what the representation already expresses. A representation-selected slice cannot validate that representation.

For a finite target, map the complete declared inventory and record every unmapped obligation as a representation deficit before adoption. For an externally governed fidelity promise, that inventory MUST describe behavioral obligations, not only named source items. It MUST include the controlling sources, base behavior families, exception classes, precedence, timing and lifecycle boundaries, persistent modifiers, optional or compound continuations, and material interaction classes identified by the fidelity obligation topology; unresolved interpretations and untested interaction classes remain explicit deficits. A complete transcription or one test or receipt per card, endpoint, command, section, file, or other named item does not establish fidelity while cross-cutting obligations remain unrepresented or lack an independent oracle. The fidelity obligation topology MUST remain coarse but sufficient to expose material semantic, adjudication, authoring, and verification burden; it MUST NOT become an exhaustive pairwise matrix or state-space enumeration unless the accepted claim, safety case, or formal standard independently requires that stronger proof.

For an open-ended target, declare the bounded inventory actually challenged and the unproven remainder. Include known hard cases and material expected to strain or break the representation. Identity coverage or a per-record receipt does not establish that the underlying source material is accepted, adequately researched, uncontaminated, or fit for the decisions claimed.

A disposable local representation MAY precede this proof only when it makes no adequacy or reuse claim, has no adopted external dependents or supporting infrastructure, remains explicitly replaceable, and cannot define its own oracle or validation scope. A bounded candidate-consumer iteration under `PURP-5` may exercise it before adoption; that use is evidence-gathering, not a reuse claim or license to expand. Reconciliation, synchronization, invalidation, migration, authoring, and validation obligations created by the representation count against its fitness. Parsing a legacy render does not prove the new authoring direction.

### REP-3 — Structural level and semantic prerequisite satisfaction

**Trigger:** A proposal is described as structural, or a named artifact is claimed to satisfy a prerequisite.

**Rule:** A structural proposal MUST name the level it structures: textual syntax, domain facts, evidence and authority, workflow state, transport, or storage. For a semantic decision system, domain facts and warrant structure MUST be considered before textual parsing machinery. Changing an accepted structural level, decision-bearing or shared interface, or authority or lifecycle ownership seam in a way that alters the accepted contract, consumer obligations, or capability boundary requires re-ratification. Local decomposition, internal interfaces, and ownership changes within those accepted boundaries are implementation judgment under `AGENT-5`, not separately ratified architecture.

A filename, title, directory, status token, schema type, passing validator, or nominal artifact class never satisfies a prerequisite by identity alone. At the fidelity the consumer uses, the candidate MUST provide the required content, coverage, semantic distinctions, provenance or derivation, authority state, evidence, validity limits, and failure behavior. A differently derived artifact placed under the expected name cannot borrow the prerequisite's authority. When the derivation path is part of the accepted consumer contract, substitution requires explicit re-ratification before dependent work activates.

## 9. Capability and repository boundaries

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

## 10. Inter-repository integration

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

## 11. LLM capabilities and durable continuity

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

### AGENT-5 — Delegated judgment is the default; escalation is evidence-bound

**Trigger:** During accepted work, an agent encounters uncertainty, multiple plausible paths, an apparent ambiguity, or considers asking a human to clarify, choose, rule, ratify, approve, or confirm before the current acceptance surface.

**Rule:** Acceptance or activation of bounded work delegates implementation judgment as defined in this constitution. The agent MUST investigate proportionately using the accepted issue, repository foundations, controlling sources, relevant code, tests, version history, established conventions, and authoritative external material as applicable. It MUST then choose and proceed with the best-supported option. Low confidence, novelty, complexity, several defensible alternatives, human availability, or the convenience of transferring responsibility is not a human decision boundary. The agent MUST NOT ask the human to answer a technical question that this investigation makes the agent better positioned to answer.

A decision request is permitted only when the residual choice is a reserved human decision and currently blocks safe or proportionate progress. Before requesting it, the agent MUST identify the exact human-governed consequence; explain why accepted authority and proportionate investigation cannot decide it; recommend one option; name the strongest material alternative and why it loses; state evidence, uncertainty, reversibility, and the cost of delay or error; and name the work blocked and safe return point. The request MUST lead with the recommendation rather than an unranked menu and MUST translate technical detail into consequences for the primary outcome, behavior, authority, risk, appetite, reversibility, or publication.

When the residual uncertainty is not reserved, the agent MUST select a local, reversible, consistency-preserving option, contain its consequences, test it, and proceed. It records the assumption only when it materially affects behavior or future interpretation. When a reserved human decision exists, no accepted instruction requires an immediate stop, and a provisional choice can remain explicit, local, reversible, and quarantined from false claims, authoritative adoption, dependent work, irreversible effects, and disproportionate waste, the agent MUST continue provisionally and surface the choice in the next coherent acceptance surface rather than interrupt immediately. It MUST stop when proceeding would cross an authority, risk, appetite, capability, repository, publication, or irreversible-effect boundary; materially lock in user-facing semantics; create disproportionate waste; or violate another constitutional stop condition.

When an accepted brief delegates creative or domain proposal work, the agent SHOULD resolve ordinary gaps into its best-supported coherent proposal and explain the material choices together. Such work remains a proposal until accepted. Where `PURP-5` permits it, a local reversible implementation may make the consequences inspectable before the human chooses. Do not silently change accepted behavior, the product's identity, or another reserved boundary, and do not ask for a separate ruling on each provisional rule merely because it was newly authored.

Independent genuine decisions MUST be batched when safe. Agents MUST NOT create serial micro-ratifications, ask speculative questions before investigation, or turn final acceptance into a sequence of internal design rulings. Internal identifiers MAY track analysis but do not make a choice human-governed. Interactive work follows the same rule as unattended work.

A generic human request to use the agent's judgment, investigate, or recommend authorizes implementation judgment but is not by itself acceptance of a reserved human decision. Silence, confusion, fatigue, or a request for a recommendation MUST NOT be recorded as acceptance. When such an instruction follows a decision-complete surface under `AUTH-3` and clearly authorizes the recommended option, it MAY constitute acceptance and MUST NOT be reconfirmed. Once the human clearly selects or accepts an option, the agent MUST NOT seek duplicate confirmation unless the relevant facts, consequences, or scope materially change. The agent's exercise of delegated judgment produces a proposal; it does not displace human acceptance authority over canonical state.

## 12. Implementation, review, and operational safety

### OPS-1 — Feasibility precedes TDD and execution readiness

**Trigger:** Risk-bearing work is about to enter TDD, implementation, exact-candidate review, `ready-for-agent`, AFK, or unattended execution.

**Rule:** Establish that supplied inputs contain the information required for the promised outputs. Identify the finite representation or oracle where one exists, semantic authority, uncertainty behavior, smallest acceptance-bearing increment, appetite, stop conditions, and safe publication boundary. Where repeated human judgment will be a binding oracle, execution readiness also requires the cumulative unit and ceiling mandated by `EVID-8`.

For a producer–consumer task, readiness MUST distinguish authority to construct a bounded candidate from evidence that the handoff works. Name the intended execution owner and the actual source-to-consumer check under `INT-1`; do not require a generalized evaluator or final universal representation before product-specific implementation can begin. A jointly bounded verification step under `PURP-5` can resolve the uncertainty. Its planned result is not evidence already obtained.

Where recurrent, reusable, configurable, engine-like, or materially separable experiential fitness is part of the accepted claim, execution readiness also requires the applicable distinctive activity loop and selected value mode under `PURP-2`, the setup and return-burden appetite under `EVID-8`, and the planned first-use, materially different second-use, and, when triggered, return-use evidence under `EVID-3`.

When a materially relied-upon external dependency owns persistence, indexing, retrieval, ranking, planning, migration, or another primary-path state or decision lifecycle, execution readiness additionally requires a minimal real lifecycle probe and an independent oracle for the semantics actually relied upon. Exercise update or rebuild behavior, identifier mapping, score or distance interpretation, persistence and reload, failure, recovery, and replacement boundaries only as applicable. Documentation, type signatures, wrapper tests that repeat examples, and plausible outputs do not establish correct use. A mechanically straightforward, locally replaceable dependency whose misuse cannot alter accepted meaning, authoritative state, or an externally consequential action needs only ordinary proportionate verification.

When the promised output includes an externally governed fidelity promise, execution readiness additionally requires the exact controlling source set and revisions or equivalent immutable identities; precedence among those sources; the named authority or decision rule for ambiguity; failure behavior when no supported interpretation can be established; and an independent fidelity oracle. Derive and accept a fidelity obligation topology independently of the candidate architecture, then estimate implementation, authoring, adjudication, testing, review, and maintenance burden against appetite. Independently select the first hostile slice and its acceptance criteria under `EVID-3`. Counts of cards, rules, sections, endpoints, commands, files, or other named items are not scope estimates when exceptions or interactions multiply obligations.

Unresolved source ambiguity blocks an unqualified fidelity claim. Before escalation, the agent MUST exhaust the controlling source set, precedence, reconciliation rules, and proportionate independent evidence, then recommend the best-supported local interpretation, variant, or reduced claim under `AGENT-5` even when no option can be presented as source fact. The acceptance authority MAY ratify that clearly labeled choice and continue within the narrower claim; it MUST NOT present the choice as source-verified fidelity. A bounded provisional implementation MAY proceed before ratification as decision-only work only when the interpretation remains explicit, local, reversible, quarantined from authoritative source fact and dependent production, and is surfaced in the next coherent acceptance surface. If those conditions do not hold, the blocker MUST name the human-governed consequence, recommendation, evidence limit, and safe return point under `AUTH-3` and `AGENT-5`.

Before the source set and fidelity obligation topology are accepted and the hostile slice is selected, only bounded source reconciliation, research, disposable local experiments, and decision-only probes MAY proceed. Their implementation MUST NOT gain dependent infrastructure, become canonical product machinery, authorize a generalized engine, DSL, framework, or inventory-scale product transcription or implementation, or count as product progress. Decision artifacts such as accepted source identities, ambiguity rulings, and the topology MAY become authoritative for the feasibility decision. This reconnaissance MAY be as deep as necessary within its appetite to discover the obligation surface; its implementation remains replaceable and subordinate to the decision.

Apply this gate whenever work involves unrestricted language, semantic scoring or gating, decision-bearing search or retrieval, a materially relied-upon external dependency for a primary-path state or decision lifecycle, an externally governed fidelity promise, a shared abstraction, parser, DSL, ontology, framework, controller, model, migration, cross-cutting validator, large generated surface, session-spanning work, or intended default-branch publication. If any required answer is missing, the task is not ready. A label or detailed issue body cannot manufacture feasibility.

### OPS-2 — Missing inputs and new capability classes require re-ratification

**Trigger:** Implementation discovers that required information is absent, accepted scope is insufficient, or a substantial unapproved capability class is needed.

**Rule:** This trigger applies only when the gap affects the promised output, accepted authority, or another reserved human boundary. It does not fire merely because an implementation choice was unspecified or the agent is uncertain. Before stopping, the agent MUST investigate proportionately and verify that the missing information cannot be derived from accepted authority or safely localized as implementation judgment under `AGENT-5`.

When the trigger genuinely fires, stop before designing or building around the gap. Recommend an explicit field, adapter, semantic-review boundary, reduced promise, formal language, or revised scope and explain why it is preferred. Introduction of a parser, DSL, ontology, framework, controller, model, migration system, generalized evaluator, or comparable capability not named in accepted scope requires immediate human re-ratification with its consumer, appetite, authority, and removal path.

Heuristic reconstruction, hidden scope expansion, or quietly treating the new capability as an internal detail is forbidden.

### OPS-3 — Adjudicate tests, reviews, and infrastructure before repair

**Trigger:** A test, CI run, or reviewer reports a failure.

**Rule:** First determine whether the expectation is still valid, whether the defect lies in the system under test, the test or review oracle, or execution infrastructure, and whether the finding belongs to accepted scope. Read the actual failure annotation before diagnosing code or architecture; billing, permissions, unavailable services, dependency drift, environment drift, and runner failure are infrastructure claims until shown otherwise.

Classify the finding as an implementation defect, specified missing case, invalid or obsolete test, specification ambiguity, authority or ownership defect, infrastructure failure, novel requirement, or non-convergence signal. Only implementation defects and specified missing cases enter automatic repair. Uncertainty about classification requires proportionate investigation and does not itself authorize repair or human escalation. If the residual choice is implementation judgment, the agent chooses the best-supported reversible classification and proceeds; if it is a reserved human decision, `AGENT-5` governs the request. Reviewer-generated examples are observations, not requirements until their interpretation and scope are accepted.

### OPS-4 — Non-convergence and resource tripwires stop implementation

**Trigger:** Nontrivial implementation or review is underway.

**Rule:** Unless a stricter or explicitly accepted larger limit exists, stop at the earliest of:

- two hours of active work without an inspectable acceptance-bearing increment;
- three failed implementation or review repair cycles on the same acceptance boundary;
- three consecutive review passes that preserve earlier cases but add new members of an open-ended semantic or paraphrase family;
- discovery of an unapproved capability class or independent pipeline;
- approximate doubling of the expected production, test, evidence, or semantic surface;
- exhaustion of the declared token, compute, financial, or human-burden ceiling;
- a context or session boundary that would require reconstruction of material reasoning; or
- attempted unattended continuation without the state required by `AGENT-4`.

An open issue, AFK label, sunk cost, green legacy suite, or persuasive completion report does not override a stop. Extension requires fresh human reprioritization with a new scope and appetite. These issue-level limits do not reset `WORK-3` or `WORK-4`.

### OPS-5 — Exploration stabilizes the claim; exact review verifies a finite candidate

**Trigger:** Adversarial review, exact-SHA review, or repeated test expansion is proposed.

**Rule:** Exploratory review MAY discover and stabilize a draft contract, interface, and oracle before final implementation review. Once a finite candidate is declared, exact-candidate review verifies that candidate; it MUST NOT become an unlimited source of new requirements. Reviewer-generated examples remain in an evaluation corpus until ratified.

Permanent selectors, ledgers, red-green evidence, and reconciliation duties are reserved for stable decision-bearing behavior and MUST be proportional to its authority and expected lifetime. Three review passes that continue expanding an open semantic family trigger `OPS-4` rather than another repair cycle.

### OPS-6 — Consequential side effects are narrow, revision-checked, and recoverable

**Trigger:** Work may cause external, destructive, irreversible, repeated, or concurrency-sensitive effects.

**Rule:** The effect MUST have narrow accepted authority, an expected-revision or equivalent freshness check, idempotent or explicitly bounded retry behavior, and an inspectable consequence. Failure behavior and recovery ownership MUST be explicit.

Ordinary version-control identity and atomic validated writes are sufficient for local accepted edits unless a real retry, interoperability, concurrency, provenance, or external-side-effect boundary proves that additional machinery is needed. An orchestration or receipt platform MUST NOT be built before the smallest fault-tested slice proves it necessary.

### OPS-7 — A clean clone exposes operational closure

**Trigger:** A repository is activated, handed off, published, or depended upon.

**Rule:** From a clean clone, a maintainer or agent MUST be able to determine the repository's purpose and authority, work-control plane, build or verification path, activation path for the current primary outcome, contracts and expected locations for external or private inputs, output ownership and location, and behavior when inputs are absent. Required repositories, services, artifact stores, submodules, and content roots MUST be clone-visible with their role, access expectation, version or pinning rule, and setup or recovery path. The clean clone MUST also preserve the root `AGENTS.md` symlink to `CLAUDE.md` and the canonical constitution at its declared path.

Ordinary operation MUST NOT depend on a remembered sibling checkout, hidden gitlink, undocumented mount, local-only service, untracked directory topology, or conversational setup knowledge. Missing private inputs MAY prevent a private workflow from running, but the repository MUST fail early and honestly while preserving any claimed input-independent verification. A public demonstration corpus is optional; synthetic fixtures MUST NOT be fabricated to claim demand, representativeness, or product readiness.

## 13. Publication

### PUB-1 — Publication scope follows exact acceptance scope

**Trigger:** A merge, release, deployment, package publication, accepted data release, or other default/protected-path change is proposed.

**Rule:** A terminally failed, rejected, parked, or still-unaccepted production delta MUST NOT become newly reachable from a default or protected branch or other authoritative path, including in a solo private repository. Failure reports, fixtures, and adversarial corpora MAY be published separately when clearly quarantined. Independently publishable prerequisite or evidence work SHOULD remain on a path that does not include a failed candidate.

Before publication, enumerate the exact commits and decision-bearing files that will become newly reachable and reconcile each with its acceptance state. Branch names, issue status, test counts, mergeability, and intent are insufficient. Publishing an exact failed delta for archival reasons requires a scoped waiver. Acceptance of an issue authorizes bounded work; it does not automatically accept every resulting production byte for default-path publication. Calling production bytes a proposal or enclosing them in an accepted custody record does not evade this rule. A provisional consumer implementation permitted by `PURP-5` stays isolated from authoritative production until its exact source, behavior, and publication scope have been accepted.

## 14. Foundations and governance

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

## 15. Adoption, precedence, waivers, and amendment

### ADOPT-1 — Adoption is complete, exact, and progressively loaded

**Trigger:** A repository adopts this constitution or a later revision.

**Rule:** An exact copy of the complete adopted text MUST exist at `docs/principles/principles-for-repos.md`. The exact bootstrap for that revision MUST appear in root `CLAUDE.md`, and root `AGENTS.md` MUST be a symlink to `CLAUDE.md`. A clean clone MUST allow Claude Code and Codex to discover that bootstrap and read the canonical file. Root instructions SHOULD remain concise enough to fit the configured agent instruction limits; task-specific reference material belongs in governed files or skills that are read on the bootstrap triggers. The complete constitution MUST NOT be pasted, imported through Claude Code's `@` syntax, or otherwise pulled into every request merely to satisfy adoption; progressive loading through the bootstrap is the default required model.

Repository foundations MUST cite and specialize this constitution with concrete facts; they MUST NOT summarize, paraphrase, selectively reproduce, or replace its safeguards as competing normative text. A later revision is adopted as a complete text or not adopted. Partial alignment has no constitutional force, and the previously adopted complete revision remains in force until reconciliation is accepted.

The complete constitution or applicable rule namespaces MUST be read exactly as the bootstrap's triggers specify. Its absence from initial prompt context is not permission to ignore it; failure to locate or read applicable text blocks substantive work.

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

Semantic identifiers are stable within an edition line: later revisions may clarify or strengthen them while preserving their subject. A deliberate major replacement MAY restructure identifiers through an explicit migration mapping; it MUST NOT pretend old citations retain unchanged meaning. Unique safeguards belong in their controlling rule, not in a second normative list of inherited failures.

A complete amendment reconciliation MUST check for contradiction, duplicate authority, trigger discoverability, operational burden, and whether the new wording accidentally forbids a legitimate local implementation, bounded experiment, or independently accepted product.
