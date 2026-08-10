# Systean Implementation Roadmap to 1.0

Status: **authoritative implementation roadmap**  
Architecture source: [`FINAL_ARCHITECTURE.md`](FINAL_ARCHITECTURE.md)

This roadmap deliberately separates architecture from implementation. The architecture may be documented before code exists; however, a phase is not considered implemented until its executable invariants and regression/property tests pass.

---

## Current baseline

Already implemented foundations:

- [x] Fixed alphabet and orthography ↔ pronunciation mapping
- [x] Deterministic syllabification and first-root-syllable stress
- [x] Manual root validation and spoken segmentation checks
- [x] Bare-root morphology (`WORD = ROOT`)
- [x] Typed semantic IR and `.semsys` package compiler
- [x] Named semantic roles and type checking
- [x] Dictionary as the single lexical-root source of truth
- [x] Structural surface parser/generator
- [x] Canonical primary-participant/predicate/rest frame order
- [x] `ki ... ku` scope grouping
- [x] Quantifier scope by appearance
- [x] `AND > OR` precedence
- [x] Core particles `ke/ne/va/zo/ra/mu/da/me`
- [x] CLI/WASM/site consuming the same Rust language package

Architecture-only work does not require executable tests. Every implementation phase below does.

---

## Phase 1 — Freeze end-state architecture

Goal: eliminate meta-architectural uncertainty before continuing feature implementation.

- [x] Define final processing pipeline
- [x] Define source-of-truth ownership
- [x] Define discourse/reference architecture
- [x] Define structured-literal architecture
- [x] Define names/quotation architecture
- [x] Define number/quantity/time architecture
- [x] Define aspect/event architecture
- [x] Define uncertainty/generic architecture
- [x] Define speech-act/affect/focus/repair architecture
- [x] Define whole-language compiler responsibilities
- [x] Define versioning/compatibility rules
- [x] Define analyzer/generator end state

No code or tests are required for this phase.

---

## Phase 2 — Typed elaboration + discourse core

Goal: make surface syntax capable of carrying unresolved references safely into a deterministic resolver.

Implementation:

- [ ] Add `TypedSurfaceAst` / equivalent elaboration representation
- [ ] Infer expected semantic types and roles for reference slots
- [ ] Represent explicit unresolved references without guessing
- [ ] Represent omitted arguments as unresolved reference slots
- [ ] Add runtime `DiscourseState`
- [ ] Add stable internal referent IDs
- [ ] Track introduction provenance and semantic type
- [ ] Add deterministic accessibility scopes
- [ ] Implement 0/1/many candidate resolution
- [ ] Return candidate diagnostics on ambiguity
- [ ] Ensure no recency/salience/world-knowledge fallback exists

Required validation:

- [ ] Unique compatible referent resolves
- [ ] Zero candidates fails as unresolved
- [ ] Multiple candidates fail as ambiguous
- [ ] Wrong-type candidates are excluded structurally
- [ ] Resolution result is independent of candidate insertion order
- [ ] Omitted argument uses the same resolver as an explicit shorthand reference

Completion result: multi-utterance state exists and no reference is resolved heuristically.

---

## Phase 3 — Aliases, boundaries, and safe omission

Goal: make long discourse manageable without ambiguous pronoun heuristics.

Implementation:

- [ ] Add explicit local alias binding
- [ ] Add alias lexical scope/lifetime
- [ ] Allow aliases for non-Entity semantic values
- [ ] Add discourse/section boundaries
- [ ] Retire ordinary shorthand candidates deterministically at boundaries
- [ ] Permit alias lifetime to outlive ordinary shorthand scope when declared
- [ ] Enable required-argument omission only through unique reference resolution
- [ ] Add canonical generator rules for explicit vs. shorthand references

Required validation:

- [ ] Alias resolves exactly regardless of other compatible referents
- [ ] Alias cannot escape its lexical scope
- [ ] Boundary removes only candidates declared inaccessible
- [ ] Omission becomes invalid as soon as a second compatible candidate exists
- [ ] Generator never emits an ambiguous shorthand

Completion result: safe cross-sentence reference and omission are usable.

---

## Phase 4 — Proper names + external quotation

Goal: allow real people, places, projects, titles, and foreign text in ordinary Systean discourse.

Implementation:

- [ ] Choose one audible/visible proper-name marker
- [ ] Define deterministic name payload representation
- [ ] Define canonical pronunciation/adaptation rules
- [ ] Keep native/foreign names in one grammatical class
- [ ] Route same-name collisions through normal reference disambiguation
- [ ] Add explicit external quotation boundaries
- [ ] Support nested quotation deterministically
- [ ] Preserve quoted payload as opaque text
- [ ] Prevent quoted foreign text from entering the ordinary root parser

Required validation:

- [ ] Proper-name status round-trips in writing and speech
- [ ] Two same-named referents remain distinguishable only through explicit context/aliasing
- [ ] Foreign text never parses as Systean accidentally inside opaque quotation
- [ ] Nested quotes preserve exact boundaries

Completion result: names and arbitrary external text can appear in real conversations.

---

## Phase 5 — First playable vocabulary and conversation slice

Goal: stop testing only engine mechanics and make Systean usable for small real conversations.

Language authoring:

- [ ] Manually author approximately 30–50 high-value content roots
- [ ] Cover people/entities, perception, possession, movement, communication, location, basic properties, and everyday actions
- [ ] Give every predicate/relation an explicit semantic frame
- [ ] Avoid contextual polysemy and vague derivations
- [ ] Add only roots chosen manually by the language author

Tooling:

- [ ] Add a discourse playground accepting several utterances
- [ ] Show resolved references and ambiguity candidates
- [ ] Show canonical semantic IR and regenerated surface form

Required validation:

- [ ] Small greeting/introduction dialogue
- [ ] Refer to two different people across several turns
- [ ] Ask and answer a truth question
- [ ] Give a command and request
- [ ] State and negate propositions
- [ ] Use existential and universal quantification
- [ ] Use coordination with `va/zo` precedence
- [ ] Demonstrate an intentional ambiguity error and explicit repair

Completion result: **Playable Systean milestone**.

---

## Phase 6 — Structured literals and numbers

Goal: support formal numeric content without allocating a root for every value.

Implementation:

- [ ] Add generic structured-literal codec interface
- [ ] Implement `Number`
- [ ] Implement `Digit`
- [ ] Implement `DigitSequence`
- [ ] Choose canonical written numeric notation
- [ ] Choose canonical spoken digit forms
- [ ] Define integer composition
- [ ] Define negative-number realization
- [ ] Define exact decimal/rational realization
- [ ] Define scale/exponent realization
- [ ] Keep approximation outside exact numeric parsing

Required validation:

- [ ] Written number → semantic value → canonical written number round-trip
- [ ] Spoken number → same semantic value
- [ ] Number and digit sequence cannot silently coerce into each other
- [ ] Very large/small supported forms remain deterministic
- [ ] Alternative accepted forms, if any, regenerate canonically

Completion result: exact numeric expressions are first-class language values.

---

## Phase 7 — Quantities and units

Goal: express measurements compositionally and without conflating values with units.

Implementation:

- [ ] Define semantic `Unit`/`Quantity` representation
- [ ] Declare unit dimensions
- [ ] Add canonical unit identities
- [ ] Add deterministic written/spoken unit forms
- [ ] Add exact conversion metadata where applicable
- [ ] Represent measurement uncertainty/precision explicitly
- [ ] Keep conversion/evaluation separate from parsing

Required validation:

- [ ] `Number` and `Quantity` remain type-distinct
- [ ] Incompatible unit dimensions fail type checking where required
- [ ] Exact conversions preserve exact values
- [ ] Approximate measurement remains explicitly approximate

Completion result: measurements are usable in ordinary and technical language.

---

## Phase 8 — Time, dates, duration, and temporal relations

Goal: express when events occur without obligatory tense morphology.

Implementation:

- [ ] Finalize `Instant` / `Interval` / `Duration` / calendar type split
- [ ] Add calendar/date literal codec if compact notation is used
- [ ] Add deterministic spoken date/time generation
- [ ] Add explicit before/after/during relations
- [ ] Add duration relations
- [ ] Add deterministic context-bound values such as `now`
- [ ] Define context input contract for speaker/addressee/time/place when used
- [ ] Keep temporal relation independent from aspectual target identity

Required validation:

- [ ] Absolute date/time round-trip
- [ ] Relative time uses an explicit/context-bound anchor
- [ ] Same sentence under different explicit `now` contexts resolves deterministically to different intended values without changing parse rules
- [ ] No hidden tense inference from word order

Completion result: normal scheduling, history, duration, and temporal description are possible.

---

## Phase 9 — Event/aspect completion

Goal: make start/stop/continue/finish/repeat usable on explicit semantic targets.

Implementation:

- [ ] Finalize concrete `Event` / `Process` / `State` / `Activity` construction patterns
- [ ] Finalize habitual/repeated activity representation
- [ ] Surface-realize `start`
- [ ] Surface-realize `cease`
- [ ] Surface-realize `continue`
- [ ] Surface-realize `finish`
- [ ] Surface-realize `interrupt`
- [ ] Surface-realize `repeat`
- [ ] Ensure target selection is structural, never guessed

Required validation:

- [ ] Stop one concrete occurrence differs from stop a habit
- [ ] Finish an object-targeted process differs from cease an activity
- [ ] Repetition count is explicit
- [ ] Parser does not check whether the event actually happened
- [ ] Parser does not require prior speaker knowledge

Completion result: aspect is expressive without lexical sense guessing.

---

## Phase 10 — Unknown/unspecified/withheld/approximate information

Goal: make incomplete knowledge explicit rather than pragmatically ambiguous.

Implementation:

- [ ] Surface-realize explicit unspecified values
- [ ] Surface-realize unknown-to-agent claims
- [ ] Surface-realize withheld values
- [ ] Keep existential quantification distinct
- [ ] Add approximation construction
- [ ] Add explicit tolerance/range where required
- [ ] Add contextual-standard mechanism only with declared context parameters

Required validation:

- [ ] Unknown ≠ unspecified ≠ existential ≠ withheld
- [ ] Approximate 10 ≠ exact 10
- [ ] Contextual standards expose their context dependency
- [ ] No world-knowledge inference decides missing parameters

Completion result: uncertainty and deliberate underspecification are first-class.

---

## Phase 11 — Generic and statistical claims

Goal: express non-universal generalizations without relying on ordinary-language vagueness.

Implementation:

- [ ] Define semantics for any accepted generic operator
- [ ] Define `most`/majority semantics if included
- [ ] Define frequency/typicality operators if included
- [ ] Define probability claims if included
- [ ] Choose surface forms only after semantic signatures are fixed

Required validation:

- [ ] Generic ≠ universal
- [ ] Majority ≠ existential
- [ ] Typical/frequency claims expose their measure/domain
- [ ] No exception-tolerant pragmatic reinterpretation of universal claims

Completion result: statistical/general claims remain literal and typed.

---

## Phase 12 — Speech acts beyond the current core

Goal: complete ordinary conversational intent while keeping proposition and utterance distinct.

Implementation:

- [ ] Formalize unmarked/default assertion behavior
- [ ] Keep `ke` truth-question semantics
- [ ] Add explicit value-question construction
- [ ] Add choice-question construction if needed
- [ ] Define answer structures where they carry semantic content
- [ ] Keep `da` command semantics
- [ ] Keep `me` request semantics
- [ ] Add other speech acts only when exact semantics are justified

Required validation:

- [ ] Question type is recoverable without intonation
- [ ] Requested value slot is explicit
- [ ] Command/request distinction remains semantic
- [ ] Prosody cannot turn assertion into question normatively

Completion result: normal conversational acts are deterministic.

---

## Phase 13 — Affect, emotion, focus, and topic

Goal: preserve human expressivity without allowing prosody or word order to rewrite literal meaning.

Implementation:

- [ ] Define affect target model
- [ ] Define affect/intensity semantics
- [ ] Manually choose core expressive particles/interjections
- [ ] Define standalone affect utterances
- [ ] Define focus operator
- [ ] Define topic operator if useful
- [ ] Keep canonical core argument order unchanged
- [ ] Keep prosody semantically non-rewriting

Required validation:

- [ ] Same proposition with different explicit affect remains the same proposition but different utterance structure
- [ ] Focus does not change semantic roles
- [ ] Topic does not change scope unless explicitly defined to do so
- [ ] Sarcasm never maps a proposition to its negation implicitly

Completion result: Systean can sound human without sacrificing literal semantics.

---

## Phase 14 — Repair, correction, and discourse editing

Goal: support natural conversation when speakers make mistakes or refine previous statements.

Implementation:

- [ ] Add explicit retract act
- [ ] Add explicit correction/replace act
- [ ] Add clarification of a prior reference
- [ ] Add repair target references
- [ ] Preserve historical parse/provenance
- [ ] Update discourse commitments deterministically

Required validation:

- [ ] Repair target must resolve uniquely
- [ ] Correction does not silently mutate historical analyzer output
- [ ] Retraction semantics is explicit
- [ ] Nested/serial repairs remain deterministic

Completion result: interactive conversation can recover from mistakes formally.

---

## Phase 15 — Text/turn/section/document structure

Goal: move from sentence parsing to complete text/discourse parsing.

Implementation:

- [ ] Define normative utterance boundary
- [ ] Define turn boundary
- [ ] Define discourse block/section boundary
- [ ] Define document-level structure where needed
- [ ] Give semantically relevant written boundaries spoken equivalents
- [ ] Integrate boundaries with referent accessibility
- [ ] Integrate boundaries with alias scope
- [ ] Integrate boundaries with quotation and repair

Required validation:

- [ ] Boundary meaning is recoverable in speech and writing
- [ ] Reference lifetime changes only at declared boundaries
- [ ] Typography alone cannot alter semantic structure

Completion result: full multi-paragraph/multi-turn discourse is normative.

---

## Phase 16 — Whole-language compiler and ambiguity suite

Goal: make "unambiguity" a package compilation invariant rather than a design aspiration.

Implementation:

- [ ] Centralize package-level version/provenance manifest
- [ ] Compile every module into one immutable `LanguagePackage`
- [ ] Add cross-layer ownership validation
- [ ] Add reserved-token collision validation
- [ ] Add structured-literal/root collision validation
- [ ] Add deterministic grammar-overlap checks
- [ ] Add AST → surface → AST property suite
- [ ] Add semantic → surface → semantic suite for generatable domains
- [ ] Add spoken-form ambiguity analysis
- [ ] Add bounded exhaustive short-expression corpus
- [ ] Add adversarial ambiguity corpus
- [ ] Add revision compatibility diff checker

Required validation:

- [ ] Parser implementation order cannot silently choose between overlapping valid analyses
- [ ] Every generated canonical form reparses to the same canonical structure
- [ ] Every compatible version preserves parse/meaning of the compatibility corpus
- [ ] Package compilation fails on normative collisions

Completion result: the language package self-validates its central guarantee.

---

## Phase 17 — Analyzer/generator productization

Goal: expose the entire language pipeline as a practical language workbench.

CLI/WASM/API:

- [ ] Analyze whole discourse
- [ ] Generate canonical surface from semantic structures
- [ ] Show typed surface AST
- [ ] Show unresolved/resolved references
- [ ] Show ambiguity candidates
- [ ] Show semantic IR and canonicalization
- [ ] Show discourse-state transitions
- [ ] Show full provenance
- [ ] Expose package/version hash

Website:

- [ ] Unified word analyzer
- [ ] Unified discourse playground
- [ ] Dictionary browser with semantic frames
- [ ] Scope visualization
- [ ] Reference/discourse visualization
- [ ] Number/quantity/time inspectors
- [ ] Canonical generator UI
- [ ] Diagnostics grouped by failing layer

Completion result: language authoring and debugging no longer require reading raw engine internals.

---

## Phase 18 — Systean 1.0 language freeze

Goal: publish a stable first usable standard rather than an endlessly moving prototype.

Language content:

- [ ] Core conversational vocabulary is manually authored
- [ ] Every root has one stable definition and semantic binding
- [ ] Core names/reference/numeric/time/pragmatic surface forms are fixed
- [ ] Core unit inventory is fixed or explicitly versioned
- [ ] Standard examples cover ordinary conversation and technical expressions

Specification:

- [ ] All normative docs agree with executable package behavior
- [ ] No active rule depends on `language/legacy/`
- [ ] Package versioning policy is executable
- [ ] Compatibility corpus is frozen
- [ ] Canonical grammar/lexicon reference can be generated from the package

Validation:

- [ ] Full native suite passes
- [ ] WASM/browser suite passes
- [ ] Whole-language compiler passes
- [ ] Core discourse corpus round-trips
- [ ] Spoken/written collision audit passes
- [ ] Adversarial ambiguity suite passes

Release result: **Systean 1.0**.

---

## Milestones

### Architecture Complete

Reached now when `FINAL_ARCHITECTURE.md` and this roadmap are accepted. No claim that all layers are executable.

### Playable Systean

Reached after Phases 2–5. At this point real short conversations should be possible and should be used to drive later language authoring.

### Everyday Systean

Reached after Phases 6–15 with enough manually authored vocabulary to exercise them.

### Systean 1.0

Reached after Phases 16–18.

---

## Rule for all implementation phases

Do not add a new surface feature merely because a natural language has one. For each feature, first identify:

1. the exact semantic structure;
2. the ownership/source-of-truth file;
3. the deterministic parse rule;
4. the deterministic generation rule;
5. the spoken realization;
6. the ambiguity failure behavior;
7. the regression/property test that proves the intended invariant.

Architecture can be documented without tests. Implementation cannot be considered complete without them.
