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

### Pre-implementation surface freeze

The following forms are **author-selected language forms**. They are frozen for implementation planning, but they are not treated as executable or collision-safe until the phonology/root audit and whole-stream ambiguity tests accept them. A failed validation changes the conflicting form, not the architecture.

#### Structural and speech-act forms

```text
ki    scope open                    reserved structural form
ku    scope close                   reserved structural form
du    utterance end                 reserved structural form
sit   opaque quotation open         reserved structural form
tis   opaque quotation close        reserved structural form

ke    truth question
ne    logical negation
va    conjunction
zo    disjunction
ra    universal quantifier
mu    existential quantifier
da    command
me    request
```

Written `.` realizes the same utterance boundary as spoken `du`. Pauses are never normative utterance boundaries. Written quotation marks may render `sit ... tis`, but the spoken delimiters carry the boundary. `?` and `!` do not create question/command semantics by themselves.

#### Reference, names, binding, and discourse

```text
ref   universal typed shorthand reference
ali   explicit local alias binding
na    proper-name marker
nom   lexical concept: name          not the proper-name marker
fra   explicit discourse frame/boundary
mi    current speaker context value
tu    current addressee context value
def   local formal definition
rel   relative/restrictive binding construction
```

`ref` and omitted arguments use the same 0/1/many typed resolver. `fra` creates a structural discourse frame. Ordinary referents live in deterministic structural frames rather than for a time/count heuristic. Inner scopes may reference accessible outer values; local values do not escape their scope. `ra` binders remain local. Referents introduced only inside negation, questions, counterfactuals, opaque quotation, and other non-assertive scopes are not exported by default. Aliases remain available only for their declared lexical scope.

#### Unknown information, information structure, and repair

```text
unk   unknown value
vak   deliberately unspecified value
hid   known-but-withheld value
fok   focus
top   topic
emo   explicit affect/emotion layer
kor   correction
ret   retraction
klar  clarification
```

These meanings remain semantically distinct; none is a pragmatic reinterpretation of another.

#### Quantification, collections, association, and logic

```text
rov   exactly N
mini  at least N
maks  at most N
set   unordered set
list  ordered sequence
grup  group treated as a collective entity
kol   collective interpretation
dis   distributive interpretation
aso   explicitly underspecified association
imp   logical implication
tip   typical/normal-case claim
stat  statistical claim
prob  probability claim
frek  frequency claim
hip   counterfactual causal frame
```

Generic/typical and statistical claims are separate explicit constructions. Counterfactual semantics is separate from material implication and is modeled as an explicit causal/counterfactual frame.

#### Aspect and time

```text
sta   start explicit occurrence target
dur   continue explicit occurrence target
fin   finish explicit occurrence target
stop  cease explicit occurrence target
rup   interrupt explicit occurrence target
reg   habitual/recurrent activity construction
rep   repeat explicit occurrence target

nau   current temporal anchor / now
ante  before
aft   after
pot   during
dat   calendar-date constructor
zon   timezone
```

Aspect always targets an explicit Event/Process/State/Activity structure. Temporal location remains separate from aspect.

#### Numeric surface system

Digits:

```text
0 nul
1 uno
2 dva
3 tri
4 kvar
5 pent
6 siks
7 sev
8 okt
9 nin
```

Numeric support forms:

```text
dig    DigitSequence marker
minus  negative sign/operator
plus   explicit positive sign/operator
dot    decimal separator
eks    explicit mathematical base-10 exponent/scientific-notation form
apro   approximation
ord    ordinal construction
rat    exact rational/fraction construction
```

Ordinary integers do **not** require a `num` wrapper. A one-digit number is exactly its digit root, e.g. `pent` = 5. `dig ...` instead constructs a `DigitSequence`, so numeric value and literal digit sequence remain type-distinct.

Direct decimal magnitude roots:

```text
dek   10^1
hek   10^2
kilo  10^3
mega  10^6
giga  10^9
tera  10^12
peta  10^15
eksa  10^18
zeta  10^21
yota  10^24
rona  10^27
keta  10^30
```

Canonical integer speech is a sparse sum of non-zero magnitude terms:

1. A magnitude precedes its coefficient, so the listener learns scale before coefficient content.
2. A magnitude coefficient is mandatory; there is no implicit `uno`.
3. `dek` and `hek` are used only inside a coefficient in the range 1..999.
4. `kilo` through `keta` are outer decimal magnitude roots.
5. Terms are emitted from larger effective magnitude to smaller effective magnitude.
6. Zero-valued terms are omitted completely; `nul` is not padding.
7. A final coefficient without an outer magnitude is the units term.
8. Adjacent outer magnitude roots form a magnitude chain whose exponents add. For magnitudes above `10^30`, the canonical chain is the greedy descending decomposition over `keta ... kilo`.
9. Outer magnitude chains are canonical; alternative decompositions of the same scale are not normative.
10. `eks` remains an explicit mathematical exponent/scientific form and is not required for ordinary integer naming.

Examples:

```text
pent
= 5

dek dva
= 20

hek kvar pent
= 405

kilo dva dek dva pent
= 2,025

mega uno pent
= 1,000,005

mega uno kilo pent sev
= 1,005,007

mega hek uno dek dva tri
kilo hek kvar dek pent siks
hek sev dek okt nin
= 123,456,789

keta mega uno
= 10^36

keta keta uno
= 10^60
```

The exact spoken grammar must prove that coefficient boundaries, magnitude chains, and term boundaries are uniquely recoverable from the token stream. Root similarity alone is advisory; a form is a release blocker only when it creates more than one valid complete normative analysis.

#### Initial manually selected lexical batch

These are lexical roots, not POS classes. Each keeps one exact lexical/semantic identity.

```text
per    human person
anim   animal
lok    location
temp   time
obj    physical object
vid    visual perception
aud    auditory perception
mov    physical movement
ven    arrive / come to a target
vad    go / move toward a destination
don    transfer / give
ten    physically hold
hab    possess
fak    intentional act / create
viv    be alive
mor    die
dur    continue an explicit occurrence target
fin    finish
nov    new
vet    old
bon    positive evaluation by explicit criterion
mal    negative evaluation by explicit criterion
ver    true
fal    false
sim    similar by explicit dimension
dif    different by explicit dimension
par    equal by explicit dimension
mag    greater by explicit dimension
min    lesser by explicit dimension
in     spatially inside
sur    spatially above
sub    spatially below
prok   spatially near
dist   spatially far
kaus   cause
kon    condition
pos    possible
nes    necessary
perm   permitted
kap    capable
vol    desire / want
int    intend
zna    know a proposition
bel    believe a proposition
mem    remember
dum    think / consider a proposition
nom    name as a lexical concept/value
tekst  text
ling   language
gov    speak / utter content
skrib  write / encode symbolic content
```

Where a lexical gloss above is still broad enough to hide multiple semantic frames, implementation must split or narrow the formal signature rather than add contextual polysemy.

#### Final author-selected replacements

The remaining pre-implementation spellings are now author-selected:

```text
rov   exactly N
fak   intentional act / create
pot   during
```

`rov` replaces the rejected `exa` candidate because `x` is intentionally absent from the fixed alphabet. `fak` replaces `fac` because `c` is intentionally absent from the fixed alphabet. `eksa` remains reserved for the numeric magnitude `10^18` and is unrelated to exact cardinality.

A dedicated `most`/majority primitive is **not** part of the core inventory. Strict majority is expressed compositionally through collection cardinality/proportion and comparison; `stat` and `tip` remain distinct statistical and typicality constructions rather than aliases for majority. Convenience surface sugar may be considered later only if it expands canonically to the same general semantic machinery.

### Pre-implementation gate

The pre-implementation architecture and core surface-form selection are complete. There is no remaining design or lexical-authoring gate before implementation can start. The selected forms above still require executable phonology, collision, and whole-stream ambiguity validation, but any failed form is a local language-authoring correction rather than an architecture change. Remaining choices such as the concrete unit inventory, individual emotion/interjection vocabulary, name-payload adaptation details, calendar literal formatting, and additional everyday roots belong to their implementation phases and do not block Phase 2.

---

## Phase 2 — Typed elaboration + discourse core

Goal: make surface syntax capable of carrying unresolved references safely into a deterministic resolver.

Implementation:

- [x] Add `TypedSurfaceAst` / equivalent elaboration representation
- [x] Infer expected semantic types and roles for reference slots
- [x] Represent selected explicit `ref` unresolved references without guessing
- [x] Represent omitted arguments as unresolved reference slots
- [x] Add runtime `DiscourseState`
- [x] Expose selected context values `mi` (speaker) and `tu` (addressee) through the deterministic context contract
- [x] Add stable internal referent IDs
- [x] Track introduction provenance and semantic type
- [x] Add deterministic accessibility scopes
- [x] Implement 0/1/many candidate resolution
- [x] Return candidate diagnostics on ambiguity
- [x] Ensure no recency/salience/world-knowledge fallback exists

Required validation:

- [x] Unique compatible referent resolves
- [x] Zero candidates fails as unresolved
- [x] Multiple candidates fail as ambiguous
- [x] Wrong-type candidates are excluded structurally
- [x] Resolution result is independent of candidate insertion order
- [x] Omitted argument uses the same resolver as an explicit shorthand reference

Status: complete. The dedicated Phase 2 discourse suite passed 11/11, and the full workspace regression suite, Svelte check, WASM build, and production site build also completed successfully on the author toolchain.

Completion result: multi-utterance state exists and no reference is resolved heuristically.

---

## Phase 3 — Aliases, boundaries, and safe omission

Goal: make long discourse manageable without ambiguous pronoun heuristics.

Implementation:

- [x] Add selected `ali` explicit local alias binding
- [x] Add selected local-definition/binding constructions `def` and `rel`
- [x] Add alias lexical scope/lifetime with deterministic inner-scope shadowing
- [x] Allow aliases for non-Entity semantic values
- [x] Add selected `fra` discourse-frame/section boundary
- [x] Retire ordinary shorthand candidates deterministically at boundaries
- [x] Permit alias lifetime to outlive ordinary shorthand frame boundaries until lexical scope exit
- [x] Enable required-argument omission only through unique reference resolution
- [x] Add canonical resolved-surface generation: successful omission materializes as the canonical explicit `ref`; exact aliases remain exact aliases
- [x] Add config-driven discourse markers to `syntax.toml` rather than hardcoding Systean forms in Rust
- [x] Add an interactive/scriptable `systean discourse` playground for contexts, introductions, reference resolution, aliases, definitions, relative bindings, scopes, frames, and surface analysis

Normative Phase 3 control forms:

```text
ali A VALUE                       bind A to one already-accessible discourse value
def A VALUE                       introduce VALUE as a new local definition bound to A
rel A ki TARGET ku ki BODY ku     evaluate BODY with A temporarily bound to TARGET in a nested lexical scope
fra                               start a new ordinary-reference frame
```

Alias spellings are local root-like spoken forms, not dictionary roots. The language package validates them against the fixed alphabet, lexical roots, structural markers, and exact pronunciation collisions before binding. `fra` changes ordinary shorthand accessibility only; an exact alias may continue to reference an older referent until that alias's lexical scope ends.

Required validation:

- [x] Alias resolves exactly regardless of other compatible referents
- [x] Alias cannot escape its lexical scope
- [x] Inner alias binding shadows an outer alias deterministically and the outer binding reappears after scope exit
- [x] Non-Entity aliases type-check in typed operator slots
- [x] Boundary retires ordinary shorthand while preserving an in-scope exact alias
- [x] Omission becomes invalid as soon as a second compatible candidate exists
- [x] `ali` fails when its target is not one existing accessible discourse value
- [x] `def` introduces one local value and exact alias
- [x] Private `def` bindings do not become ordinary shorthand candidates
- [x] `rel` never exports its temporary alias
- [x] Alias surfaces cannot collide with lexical or structural forms
- [x] Canonical resolved generation materializes safe shorthand deterministically
- [x] CLI playground integration tests cover successful and intentionally failing resolution paths

Status: complete. The dedicated Phase 3 core suite passed 12/12 and the CLI integration suite passed 3/3 on the author toolchain; the full workspace regression suite, Svelte check, WASM build, production site build, and scripted playground smoke test also completed successfully.

Completion result: safe cross-sentence reference, exact local naming, deterministic boundaries, and omission are usable.

---

## Phase 4 — Proper names + external quotation

Goal: allow real people, places, projects, titles, and foreign text in ordinary Systean discourse.

Implementation:

- [x] Implement selected audible/visible proper-name marker `na`
- [x] Define deterministic name payload representation
- [x] Define canonical pronunciation/adaptation rules
- [x] Keep native/foreign names in one grammatical class
- [x] Route same-name collisions through normal reference disambiguation
- [x] Implement selected explicit external quotation boundaries `sit ... tis`
- [x] Support nested quotation deterministically
- [x] Preserve quoted payload as opaque text
- [x] Prevent quoted foreign text from entering the ordinary root parser

Required validation:

- [x] Proper-name status round-trips in writing and speech
- [x] Two same-named referents remain distinguishable only through explicit context/aliasing
- [x] Foreign text never parses as Systean accidentally inside opaque quotation
- [x] Nested quotes preserve exact boundaries

Status: **validated**. The dedicated Phase 4 suite and the full workspace/site regression run passed on the author toolchain.

Completion result: names and arbitrary external text can appear in real conversations.

---

## Phase 5 — First playable vocabulary and conversation slice

Goal: stop testing only engine mechanics and make Systean usable for small real conversations.

Language authoring:

- [x] Implement and formally bind the manually selected initial 30–50 high-value content roots listed in the pre-implementation surface freeze
- [x] Cover people/entities, perception, possession, movement, communication, location, basic properties, and everyday actions
- [x] Give every predicate/relation an explicit semantic frame
- [x] Avoid contextual polysemy and vague derivations
- [x] Add only roots chosen manually by the language author

Tooling:

- [x] Add a discourse playground accepting several utterances
- [x] Show resolved references and ambiguity candidates
- [x] Show canonical semantic IR and regenerated/resolved surface form

Required validation:

- [x] Small greeting/introduction dialogue
- [x] Refer to two different people across several turns
- [x] Ask and answer a truth question
- [x] Give a command and request
- [x] State and negate propositions
- [x] Use existential and universal quantification
- [x] Use coordination with `va/zo` precedence
- [x] Demonstrate an intentional ambiguity error and explicit repair

Status: **validated**. The manually selected 40-root playable vocabulary, semantic bindings, dedicated core tests, CLI conversation tests, root audit, full workspace regression suite, WASM build, and site checks passed on the author toolchain. The exact implemented frames are summarized in [`PLAYABLE_VOCABULARY.md`](PLAYABLE_VOCABULARY.md).

Completion result: **Playable Systean milestone**.

---

## Phase 6 — Structured literals and numbers

Goal: support formal numeric content without allocating a root for every value.

Implementation:

- [x] Add generic structured-literal codec interface
- [x] Implement `Number`
- [x] Implement `Digit`
- [x] Implement `DigitSequence`
- [x] Implement canonical Arabic decimal numeral notation for written `Number` values
- [x] Implement selected spoken digit forms `nul/uno/dva/tri/kvar/pent/siks/sev/okt/nin`
- [x] Implement the selected sparse magnitude-first integer composition
- [x] Implement selected negative-number realization with `minus`
- [x] Implement exact decimal `dot` and exact rational `rat` realization
- [x] Implement direct `dek/hek/kilo/mega/giga/tera/peta/eksa/zeta/yota/rona/keta` magnitudes, canonical large-scale chaining, and explicit `eks` exponent notation
- [x] Keep approximation outside exact numeric parsing

Required validation:

- [x] Written number → semantic value → canonical written number round-trip
- [x] Spoken number → same semantic value
- [x] Number and digit sequence cannot silently coerce into each other
- [x] Very large/small supported forms remain deterministic
- [x] Magnitude is recoverable before its coefficient in canonical speech
- [x] Zero magnitude terms are omitted rather than spoken as padding
- [x] Every accepted spoken integer has exactly one complete numeric parse
- [x] Alternative accepted forms, if any, regenerate canonically

Status: **validated**. Dedicated numeric tests, CLI literal tests, the full workspace regression suite, root audit, WASM build, Svelte checks, and production site build passed on the author toolchain. See [`STRUCTURED_VALUES.md`](STRUCTURED_VALUES.md).

Completion result: exact numeric expressions are first-class language values.

---

## Phase 7 — Quantities and units

Goal: express measurements compositionally and without conflating values with units.

Implementation:

- [x] Define semantic `Unit`/`Quantity` representation
- [x] Declare unit dimensions
- [x] Add canonical unit identities
- [x] Add deterministic written/spoken unit forms
- [x] Add exact conversion metadata where applicable
- [x] Represent measurement uncertainty/precision explicitly
- [x] Keep conversion/evaluation separate from parsing

Required validation:

- [x] `Number` and `Quantity` remain type-distinct
- [x] Incompatible unit dimensions fail type checking where required
- [x] Exact conversions preserve exact values
- [x] Approximate measurement remains explicitly approximate

Status: **validated**. Dedicated quantity/unit tests and the complete Phase 6–8 regression/build sequence passed on the author toolchain. See [`STRUCTURED_VALUES.md`](STRUCTURED_VALUES.md).

Completion result: measurements are usable in ordinary and technical language.

---

## Phase 8 — Time, dates, duration, and temporal relations

Goal: express when events occur without obligatory tense morphology.

Implementation:

- [x] Finalize `Instant` / `Interval` / `Duration` / calendar type split
- [x] Add calendar/date literal codec if compact notation is used
- [x] Implement selected calendar/time support forms `dat` and `zon`
- [x] Add deterministic spoken date/time generation
- [x] Add selected temporal forms `ante` (before), `aft` (after), and `pot` (during)
- [x] Add duration relations
- [x] Add selected context-bound `nau` (`now`) and context-provider contract
- [x] Define context input contract for speaker/addressee/time/place when used
- [x] Keep temporal relation independent from aspectual target identity

Required validation:

- [x] Absolute date/time round-trip
- [x] Relative time uses an explicit/context-bound anchor
- [x] Same sentence under different explicit `now` contexts resolves deterministically to different intended values without changing parse rules
- [x] No hidden tense inference from word order

Status: **validated**. Dedicated temporal tests, explicit `now`/`during` discourse smoke tests, the full workspace suite, WASM build, Svelte checks, and production site build passed on the author toolchain. See [`STRUCTURED_VALUES.md`](STRUCTURED_VALUES.md).

Completion result: normal scheduling, history, duration, and temporal description are possible.

---

## Phase 9 — Event/aspect completion

Goal: make start/stop/continue/finish/repeat usable on explicit semantic targets.

Implementation:

- [x] Finalize concrete `Event` / `Process` / `State` / `Activity` construction patterns
- [x] Finalize habitual/repeated activity representation
- [x] Surface-realize `start` as `sta`
- [x] Surface-realize `cease` as `stop`
- [x] Surface-realize `continue` as `dur`
- [x] Surface-realize `finish` as `fin`
- [x] Surface-realize `interrupt` as `rup`
- [x] Surface-realize `repeat` as `rep`
- [x] Ensure target selection is structural, never guessed

Required validation:

- [ ] Stop one concrete occurrence differs from stop a habit
- [ ] Finish an object-targeted process differs from cease an activity
- [ ] Repetition count is explicit
- [ ] Parser does not check whether the event actually happened
- [ ] Parser does not require prior speaker knowledge

Status: implementation and dedicated Phase 9 core/CLI tests are present. Runtime validation remains pending on the author toolchain. See [`PHASES_9_11.md`](PHASES_9_11.md).

Completion result: aspect is expressive without lexical sense guessing.

---

## Phase 10 — Unknown/unspecified/withheld/approximate information

Goal: make incomplete knowledge explicit rather than pragmatically ambiguous.

Implementation:

- [x] Surface-realize explicit unspecified values as `vak`
- [x] Surface-realize unknown-to-agent claims with typed `unk` plus an explicit/context-bound knower
- [x] Surface-realize withheld values as `hid`
- [x] Keep existential quantification distinct
- [x] Add approximation construction `apro`
- [x] Add explicit tolerance/range where required
- [x] Add contextual-standard mechanism only with declared context parameters

Required validation:

- [ ] Unknown ≠ unspecified ≠ existential ≠ withheld
- [ ] Approximate 10 ≠ exact 10
- [ ] Contextual standards expose their context dependency
- [ ] No world-knowledge inference decides missing parameters

Status: implementation and dedicated Phase 10 core/CLI tests are present. Runtime validation remains pending on the author toolchain. See [`PHASES_9_11.md`](PHASES_9_11.md).

Completion result: uncertainty and deliberate underspecification are first-class.

---

## Phase 11 — Generic and statistical claims

Goal: express non-universal generalizations without relying on ordinary-language vagueness.

Implementation:

- [x] Implement the accepted separation between typical/normal (`tip`) and statistical (`stat`) claims
- [x] Implement selected frequency `frek` and typicality `tip` operators with explicit domain/measure parameters
- [x] Implement explicit probability claims as `prob`
- [x] Bind selected `tip/stat/prob/frek` forms to exact semantic signatures
- [x] Implement explicit cardinal constraints `rov` (exactly N), `mini` (at least N), and `maks` (at most N)
- [x] Implement `set`, `list`, and `grup` collection identities plus `kol` / `dis` interpretation operators
- [x] Implement explicitly underspecified association `aso`
- [x] Implement surface implication `imp` and counterfactual causal frame `hip` as distinct constructions

Required validation:

- [ ] Generic ≠ universal
- [ ] Strict majority, when expressed compositionally, is not conflated with existential, statistical, or typicality claims
- [ ] Typical/frequency claims expose their measure/domain
- [ ] No exception-tolerant pragmatic reinterpretation of universal claims

Status: implementation and dedicated Phase 11 core/CLI tests are present. Runtime validation remains pending on the author toolchain. See [`PHASES_9_11.md`](PHASES_9_11.md).

Completion result: statistical/general claims remain literal and typed.

---

## Phase 12 — Speech acts beyond the current core

Goal: complete ordinary conversational intent while keeping proposition and utterance distinct.

Implementation:

- [ ] Formalize unmarked/default assertion behavior
- [ ] Keep `ke` truth-question semantics
- [ ] Implement value questions as `ke` over an explicit typed `unk` slot; no interrogative word-order inversion
- [ ] Express choice questions compositionally with `ke` over explicit alternatives (for example `zo`) unless testing proves a dedicated construction is needed
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

- [ ] Define affect target model under selected explicit affect layer `emo`
- [ ] Define affect/intensity semantics
- [ ] Manually choose core expressive particles/interjections
- [ ] Define standalone affect utterances
- [ ] Define selected focus operator `fok`
- [ ] Define selected topic operator `top`
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

- [ ] Add selected explicit retract act `ret`
- [ ] Add selected explicit correction/replace act `kor`
- [ ] Add selected clarification act `klar`
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

- [ ] Implement spoken utterance boundary `du` and written `.` realization
- [ ] Define turn boundary as deterministic channel/discourse metadata; add a spoken marker only if a semantic distinction cannot otherwise be recovered
- [ ] Implement selected discourse block/frame boundary `fra`
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
