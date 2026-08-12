# Systean Implementation Roadmap to 1.0

Status: **authoritative implementation roadmap**
Architecture sources: [`FINAL_ARCHITECTURE.md`](FINAL_ARCHITECTURE.md) and the accepted pre-1.0 revision [`SEMANTIC_DSL_ARCHITECTURE.md`](SEMANTIC_DSL_ARCHITECTURE.md)

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

The pre-implementation architecture and core surface-form selection are complete. There is no remaining design or lexical-authoring gate before implementation can start. The selected forms above still require executable phonology, collision, and whole-stream ambiguity validation, but any failed form is a local language-authoring correction rather than an architecture change. The subjective-state/affect inventory is now author-frozen in [`SUBJECTIVE_STATES.md`](SUBJECTIVE_STATES.md), including the decision not to create a parallel 1.0 interjection lexicon. Remaining language-content work is primarily additional everyday roots, final unit-inventory decisions, and corpus-driven signature refinement inside the existing architecture.

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

- [x] Stop one concrete occurrence differs from stop a habit
- [x] Finish an object-targeted process differs from cease an activity
- [x] Repetition count is explicit
- [x] Parser does not check whether the event actually happened
- [x] Parser does not require prior speaker knowledge

Status: **validated**. Dedicated Phase 9 tests, the full workspace regression suite, the root audit, WASM/Svelte production checks, and the scripted discourse smoke test passed on the author toolchain. See [`PHASES_9_11.md`](PHASES_9_11.md).

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

- [x] Unknown ≠ unspecified ≠ existential ≠ withheld
- [x] Approximate 10 ≠ exact 10
- [x] Contextual standards expose their context dependency
- [x] No world-knowledge inference decides missing parameters

Status: **validated**. Dedicated Phase 10 tests, the full workspace regression suite, the root audit, WASM/Svelte production checks, and the scripted discourse smoke test passed on the author toolchain. See [`PHASES_9_11.md`](PHASES_9_11.md).

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

- [x] Generic ≠ universal
- [x] Strict majority, when expressed compositionally, is not conflated with existential, statistical, or typicality claims
- [x] Typical/frequency claims expose their measure/domain
- [x] No exception-tolerant pragmatic reinterpretation of universal claims

Status: **validated**. Dedicated Phase 11 tests, the full workspace regression suite, the root audit, WASM/Svelte production checks, and the scripted discourse smoke test passed on the author toolchain. See [`PHASES_9_11.md`](PHASES_9_11.md).

Completion result: statistical/general claims remain literal and typed.

---

## Phase 12 — Speech acts beyond the current core

Goal: complete ordinary conversational intent while keeping proposition and utterance distinct.

Language-authoring freeze:

- [x] Baseline value questions remain compositional through `ke` + an explicit typed `unk` slot
- [x] Baseline choice questions remain compositional through `ke` + explicit alternatives such as `zo`
- [x] No additional question-word or yes/no-answer roots are required for the Phase 12 baseline unless executable testing demonstrates a missing semantic distinction

Implementation:

- [x] Formalize unmarked/default assertion behavior
- [x] Keep `ke` truth-question semantics
- [x] Implement value questions as `ke` over an explicit typed `unk` slot; no interrogative word-order inversion
- [x] Express choice questions compositionally with `ke` over explicit alternatives (for example `zo`) unless testing proves a dedicated construction is needed
- [x] Define answer structures where they carry semantic content
- [x] Keep `da` command semantics
- [x] Keep `me` request semantics
- [x] Add other speech acts only when exact semantics are justified

Required validation:

- [x] Question type is recoverable without intonation
- [x] Requested value slot is explicit
- [x] Command/request distinction remains semantic
- [x] Prosody cannot turn assertion into question normatively

Status: **validated**. Dedicated Phase 12 tests, CLI integration, full workspace regressions, root audit, WASM build, Svelte checks, and production site build passed on the author toolchain. See [`PHASES_12_14.md`](PHASES_12_14.md).

Completion result: normal conversational acts are deterministic.

---

## Phase 13 — Affect, emotion, focus, and topic

Goal: preserve human expressivity without allowing prosody or word order to rewrite literal meaning.

Language-authoring freeze:

- [x] Freeze the 86 newly selected subjective-state roots in [`SUBJECTIVE_STATES.md`](SUBJECTIVE_STATES.md)
- [x] Keep experienced affect/state semantics separate from the expressive `emo` layer
- [x] Keep intensity, target, and cause explicit rather than creating hidden/intensity-only lexical synonyms
- [x] Keep romantic attraction, sexual attraction, sexual arousal, baseline sexual drive, lust, general love, and action desire semantically distinct
- [x] Keep bodily subjective states separate from affect semantics
- [x] Do not create a parallel dedicated interjection-root inventory for the 1.0 baseline; standalone expressives compose through `emo`
- [x] Freeze the selected seven-deadly-sins coverage (`superb/avari/lusta/envi/gula/furor/leni`) without duplicating `envi` or `furor` senses

Implementation:

- [x] Define affect target model under selected explicit affect layer `emo`
- [x] Define affect/intensity semantics and typed state signatures
- [x] Add the author-selected affective/social roots required by Phase 13
- [x] Define standalone affect utterances compositionally through `emo`
- [x] Define selected focus operator `fok`
- [x] Define selected topic operator `top`
- [x] Keep canonical core argument order unchanged
- [x] Keep prosody semantically non-rewriting

Required validation:

- [x] Same proposition with different explicit affect remains the same proposition but different utterance structure
- [x] Focus does not change semantic roles
- [x] Topic does not change scope unless explicitly defined to do so
- [x] Sarcasm never maps a proposition to its negation implicitly

Status: **validated**. Dedicated Phase 13 tests, CLI integration, full workspace regressions, root audit, WASM build, Svelte checks, and production site build passed on the author toolchain. See [`PHASES_12_14.md`](PHASES_12_14.md).

Completion result: Systean can sound human without sacrificing literal semantics.

---

## Phase 14 — Repair, correction, and discourse editing

Goal: support natural conversation when speakers make mistakes or refine previous statements.

Implementation:

- [x] Add selected explicit retract act `ret`
- [x] Add selected explicit correction/replace act `kor`
- [x] Add selected clarification act `klar`
- [x] Add repair target references
- [x] Preserve historical parse/provenance
- [x] Update discourse commitments deterministically

Required validation:

- [x] Repair target must resolve uniquely
- [x] Correction does not silently mutate historical analyzer output
- [x] Retraction semantics is explicit
- [x] Nested/serial repairs remain deterministic

Status: **validated**. Dedicated Phase 14 tests, CLI integration, full workspace regressions, root audit, WASM build, Svelte checks, and production site build passed on the author toolchain. See [`PHASES_12_14.md`](PHASES_12_14.md).

Completion result: interactive conversation can recover from mistakes formally.

---

## Phase 15 — Text/turn/section/document structure

Goal: move from sentence parsing to complete text/discourse parsing.

Implementation:

- [x] Implement spoken utterance boundary `du` and written `.` realization
- [x] Define turn boundary as deterministic channel/discourse metadata; add a spoken marker only if a semantic distinction cannot otherwise be recovered
- [x] Implement selected discourse block/frame boundary `fra`
- [x] Define document-level structure where needed
- [x] Give semantically relevant written boundaries spoken equivalents
- [x] Integrate boundaries with referent accessibility
- [x] Integrate boundaries with alias scope
- [x] Integrate boundaries with quotation and repair

Required validation:

- [x] Boundary meaning is recoverable in speech and writing
- [x] Reference lifetime changes only at declared boundaries
- [x] Typography alone cannot alter semantic structure

Status: **validated**. Dedicated Phase 15 core tests and CLI integration coverage are present and the phase is green on the author toolchain. See [`PHASE_15_TEXT_STRUCTURE.md`](PHASE_15_TEXT_STRUCTURE.md).

Completion result: full multi-paragraph/multi-turn discourse is normative.

---

## Phase 16 — Whole-language compiler and ambiguity suite

Goal: make "unambiguity" a package compilation invariant rather than a design aspiration.

Implementation:

- [x] Centralize package-level version/provenance manifest
- [x] Compile every module into one immutable `LanguagePackage`
- [x] Add cross-layer ownership validation
- [x] Add reserved-token collision validation
- [x] Add structured-literal/root collision validation
- [x] Add deterministic grammar-overlap checks
- [x] Add AST → surface → AST property suite
- [x] Add semantic → surface → semantic suite for generatable domains
- [x] Add spoken-form ambiguity analysis
- [x] Add bounded exhaustive short-expression corpus
- [x] Add adversarial ambiguity corpus
- [x] Add revision compatibility diff checker

Required validation:

- [x] Parser implementation order cannot silently choose between overlapping valid analyses
- [x] Every generated canonical form reparses to the same canonical structure
- [x] Every compatible version preserves parse/meaning of the compatibility corpus
- [x] Package compilation fails on normative collisions

Status: **validated**. The whole-language compiler, frozen compatibility/adversarial corpora, generated round-trip suite, spoken ambiguity pass, bounded exhaustive corpus, and dedicated Phase 16 regressions are green on the author toolchain. See [`PHASE_16_COMPILER_AMBIGUITY.md`](PHASE_16_COMPILER_AMBIGUITY.md).

Completion result: the language package self-validates its central guarantee.

---

## Phase 17 — Analyzer/generator productization

Goal: expose the entire language pipeline as a practical language workbench.

CLI/WASM/API:

- [x] Analyze whole discourse
- [x] Generate canonical surface from semantic structures
- [x] Show typed surface AST
- [x] Show unresolved/resolved references
- [x] Show ambiguity candidates
- [x] Show semantic IR and canonicalization
- [x] Show discourse-state transitions
- [x] Show full provenance
- [x] Expose package/version hash

Website:

- [x] Unified word analyzer
- [x] Unified discourse playground
- [x] Dictionary browser with semantic frames
- [x] Scope visualization
- [x] Reference/discourse visualization
- [x] Number/quantity/time inspectors
- [x] Canonical generator UI
- [x] Diagnostics grouped by failing layer

Status: **implemented; awaiting author-toolchain validation**. Core, CLI, WASM and website consume one structured workbench API; the embedded browser package now has the same versioned provenance/fingerprint contract as native loading. See [`PHASE_17_WORKBENCH.md`](PHASE_17_WORKBENCH.md).

Completion result: language authoring and debugging no longer require reading raw engine internals.

---

## Phase 18 — Typed semantic identity and DSL core migration

Goal: replace stringly semantic identity and duplicate lexical/operator ownership before freezing 1.0.

Detailed plan: [`PHASE_18_SEMANTIC_DSL_CORE.md`](PHASE_18_SEMANTIC_DSL_CORE.md)
Architecture: [`SEMANTIC_DSL_ARCHITECTURE.md`](SEMANTIC_DSL_ARCHITECTURE.md)

Semantic identity and DSL:

- [x] Extend/rework `.semsys` around typed `word`, `primitive`, `def`, `intrinsic`, `data`, `context`, `dimension`, and `unit` declarations
- [x] Compile source names to stable package IDs (`SymbolId`, `TypeId`, `ConstructorId`, `ContextSlotId`, `DimensionId`, `UnitId`, ...)
- [ ] Remove runtime semantic dependence on English/source identifiers
- [x] Make binder/parameter source names non-semantic and alpha-normalized in the Phase 18 typed IR
- [x] Preserve source spans/names outside canonical terms for diagnostics, documentation, workbench display, and migration diffs

Lexical ownership:

- [ ] Migrate ordinary roots from `dictionary.toml` + duplicated `.semsys` operator signature to one typed DSL declaration
- [ ] Make the Systean declaration own semantic identity directly instead of `root -> English identifier -> opaque symbol`
- [x] Derive ordinary default surface frames from typed arity where possible
- [x] Ensure adding an ordinary primitive word requires one typed declaration and no Rust change

Typed values:

- [x] Replace `StructuredLiteral { family: String, canonical: String }` with typed algebraic/scalar runtime values
- [x] Remove `unknown:context:speaker` and other string-encoded runtime semantic mini-languages
- [x] Represent unknown/unspecified/withheld information structurally
- [x] Migrate numbers, quantities, dates, times, durations, and intervals to typed runtime semantic values
- [x] Replace runtime string-identified unit/dimension semantics with `UnitId`/`DimensionId`
- [x] Remove engine special cases keyed by names such as `"time"` or `"second"`

Compatibility:

- [x] Introduce separate semantic and surface fingerprints in the Phase 18 typed package
- [x] Ensure source formatting, comments, and binder renames do not change semantic fingerprint
- [x] Ensure semantic signature/definition changes do change semantic fingerprint
- [x] Ensure surface spelling/form changes do change surface fingerprint
- [ ] Preserve Phase 17 behavior through the frozen compatibility corpus unless a change is deliberately approved

Required validation:

- [ ] Canonical semantic IR contains no human semantic identity strings
- [ ] Ordinary lexical growth needs no Rust code
- [x] Structured values never require a later layer to parse their canonical string representation
- [x] Unit/dimension runtime behavior contains no English-name special cases after package-boundary resolution
- [ ] Alpha-equivalent DSL declarations produce equal canonical semantics/fingerprint
- [ ] Existing behavioral, compatibility, ambiguity, native, and WASM suites remain green

Status: **in progress — Phase 18A typed compiler and Phase 18B1 structured-value/unit runtime cutover implemented; final lexical ownership/package cutover remains Phase 18B2.**

Completion result: the core has one typed semantic identity model and no longer uses English/source strings as hidden semantics.

---

## Phase 19 — Declarative surface grammar and discourse effects

Goal: complete the package architecture so ordinary syntax and Systean-specific discourse/pragmatic behavior are declarative over the Phase 18 typed IR.

Detailed plan: [`PHASE_19_DECLARATIVE_LANGUAGE_PACKAGE.md`](PHASE_19_DECLARATIVE_LANGUAGE_PACKAGE.md)

Surface grammar:

- [ ] Implement typed `form` declarations compiled to both parser and canonical linearizer
- [ ] Replace growing `Class/Predicate/Prefix/Infix/Quantifier/SpeechAct/...` special-case inventories with generic typed surface rules where possible
- [ ] Express ordinary unary/binary/n-ary frames through the default word frame
- [ ] Express `ne`, `va`, `zo` through declarative forms and semantic composition
- [ ] Express `ra`/`mu` through higher-order typed predicate arguments where possible
- [ ] Migrate counted quantifiers, aspect, focus/topic, speech acts, and related constructions to generic forms when representable
- [ ] Keep genuinely structural boundaries/markers explicit rather than forcing them into a lexical abstraction

Elaboration/reference:

- [ ] Allow predicate/function values as first-class typed arguments for higher-order constructions
- [ ] Use expected type to reject impossible analyses without ranking multiple valid analyses
- [ ] Compile context roots through `ContextSlotId`
- [ ] Compile `ref<T>` and safe omission to one generic typed resolver request with distinct provenance
- [ ] Preserve exact alias semantics and 0/1/many reference resolution

Discourse effects:

- [ ] Define a small stable generic effect instruction set
- [ ] Declare question/assertion/request/command/focus/topic/repair behavior through typed package effects
- [ ] Remove Systean-specific communicative-act behavior keyed by source strings/enums from Rust
- [ ] Keep every effect visible in workbench provenance

Schema cleanup:

- [ ] Remove unsupported/fake configuration alternatives that validators currently reject
- [ ] Remove old semantic root/form inventories from TOML after migration
- [ ] Ensure no active language rule depends on `language/legacy/`

Required validation:

- [ ] Parser and generator derive from the same rule and round-trip canonically
- [ ] `ra per viv` and the full quantifier corpus preserve intended meaning without a quantifier-specific parse shortcut
- [ ] Existing precedence/grouping/reference/repair behavior remains deterministic
- [ ] New language-specific constructions using existing semantic/effect primitives require no Rust changes
- [ ] Whole-language compiler still rejects every complete typed ambiguity rather than ranking candidates
- [ ] Native/WASM/site engine contract remains one implementation

Completion result: Rust is a generic typed language engine; Systean-specific language growth is predominantly `.semsys` package authoring.

---

## Phase 20 — English reference documentation and controlled rendering

Goal: make every public word understandable in English and make every supported canonical Systean expression renderable into one deterministic bridge language.

Detailed plan: [`PHASE_20_ENGLISH_REFERENCE.md`](PHASE_20_ENGLISH_REFERENCE.md)

English is fixed as the Systean 1.0 bridge language. English documentation/rendering is **not** semantic identity and does not participate in normative parsing.

Per-word documentation:

- [ ] Every public root has a short English `gloss`
- [ ] Every public root has a detailed English `explain` entry
- [ ] Documentation displays the compiled semantic signature rather than duplicating it manually
- [ ] Every public root has at least one canonical executable example
- [ ] Operators/constructions have examples that make scope/argument behavior visible where relevant
- [ ] Explanations state important non-implications/contrasts when a short English gloss would otherwise be misleading
- [ ] Documentation indicates primitive / defined / intrinsic-backed status

Documentation compiler:

- [ ] Store human documentation separately from normative semantic declarations, provisionally under `language/docs/en.sydoc`
- [ ] Resolve documentation entries to compiled symbols
- [ ] Fail release validation when any public 1.0 root lacks required English documentation
- [ ] Parse/test documentation examples through the real engine
- [ ] Give documentation an independent fingerprint
- [ ] Ensure documentation-only edits do not change semantic/surface fingerprints

Controlled English rendering:

- [ ] Render from canonical typed semantics, never by concatenating word glosses
- [ ] Cover ordinary predicates/relations, logical operators, quantifiers, generic/statistical claims, structured values, information status, context/reference, speech acts, and repair
- [ ] Preserve scope explicitly even when the most idiomatic English wording would hide it
- [ ] Preserve unknown/unspecified/withheld information without inventing content
- [ ] Expose rendering provenance/alignment data where feasible for the analyzer
- [ ] Keep optional future idiomatic/natural paraphrasing non-normative and outside 1.0 acceptance

Required validation:

- [ ] 100% public vocabulary documentation coverage
- [ ] All documentation examples parse and preserve intended canonical semantics
- [ ] Controlled English golden corpus covers every major semantic subsystem
- [ ] Rendering is deterministic and cannot alter Systean parse/meaning
- [ ] English wording changes affect only documentation/rendering compatibility, not semantic identity

Completion result: English is a complete learning/reference bridge for Systean 1.0 without becoming part of Systean semantics.

---

## Phase 21 — User-facing learning site and interactive analyzer

Goal: turn the Phase 17 developer-oriented workbench UI into a site for actual Systean users and learners.

Detailed plan: [`PHASE_21_LEARNING_SITE.md`](PHASE_21_LEARNING_SITE.md)

Dictionary:

- [ ] Replace flat/raw semantic-object presentation with structured user-facing word pages
- [ ] Search by Systean root, English gloss, English explanation, tags, and examples
- [ ] Add filters for primitive/defined/intrinsic-backed declaration, result type, arity, argument types, semantic/documentation domain, structural category, and stability where available
- [ ] Keep filters URL-addressable and mobile/keyboard usable
- [ ] Show root, pronunciation/stress, gloss, human-readable signature, explanation, examples, and all declared surface realizations
- [ ] Treat “all forms” as all real surface/construction realizations; do not invent inflection tables for bare-root morphology

Analyzer primary UX:

- [ ] Put deterministic English rendering near the top of every successful analysis
- [ ] Replace raw JSON/serialized AST as the default view with an interactive annotated Systean expression
- [ ] On hover/focus, show each token's short meaning, contextual meaning, semantic contribution, argument/role, scope, resolution, and discourse effect where applicable
- [ ] Highlight the other words/phrases influenced by the hovered token
- [ ] Highlight participant-to-relation links for predicates
- [ ] Highlight restriction/body and scope for quantifiers
- [ ] Highlight the exact proposition affected by negation/scope operators
- [ ] Highlight resolved referent/context/alias targets for reference words and omissions
- [ ] Highlight repair/history targets for correction/retraction/clarification
- [ ] Make all hover information available through keyboard focus/tap

Click-through learning:

- [ ] Clicking a token pins its contextual explanation
- [ ] Open the full dictionary/detail view without losing the analyzed expression
- [ ] Show all declared forms, examples, pronunciation, detailed English explanation, typed signature, and formal-spec links
- [ ] Preserve the distinction between “what this word means generally” and “what it contributes here”

English alignment:

- [ ] Use Phase 20 rendering provenance to connect Systean tokens/constructions with English spans where possible
- [ ] Support many-to-one/one-to-many/discontinuous alignment rather than pretending every Systean word maps to one English word

Diagnostics and raw technical views:

- [ ] Explain errors in plain English with highlighted source spans and candidates/expectations
- [ ] Remove raw JSON from normal user flows
- [ ] Keep raw IR/AST/provenance/JSON only in an explicit advanced/developer inspector
- [ ] Do not reconstruct semantics in Svelte; all relationships/scopes/resolution data come from Rust/WASM workbench annotations

Learning content:

- [ ] Add a guided learning path with executable examples
- [ ] Cross-link lessons, dictionary entries, and analyzer examples
- [ ] Fail documentation/site checks when teaching examples stop parsing or silently change intended meaning

Required validation:

- [ ] Representative corpus exposes context-sensitive hover/focus data for every token
- [ ] Visual dependency/scope/reference links match canonical engine structures
- [ ] English rendering exactly matches Phase 20 deterministic renderer output
- [ ] Every public root has a complete word page
- [ ] Dictionary search/filter behavior is backed by compiled indexes
- [ ] No raw JSON is necessary to understand a valid or invalid expression
- [ ] `pnpm check`, production build, browser/native corpus parity, and interaction accessibility tests pass

Completion result: a learner can search words, understand complete expressions, inspect every token's contextual contribution, follow scope/reference relationships, and read deterministic English without understanding compiler internals.

---

## Phase 22 — Systean 1.0 language freeze

Goal: publish a stable first usable standard after the semantic/DSL, documentation, and learning-surface architecture is no longer provisional.

Language content:

- [ ] Core conversational vocabulary is manually authored, including the frozen subjective-state inventory in [`SUBJECTIVE_STATES.md`](SUBJECTIVE_STATES.md)
- [ ] Every root has one stable formal declaration
- [ ] Every public root has complete Phase 20 English documentation
- [ ] Core names/reference/numeric/time/pragmatic surface forms are fixed
- [ ] Core unit inventory is fixed or explicitly versioned
- [ ] Standard examples cover ordinary conversation and technical expressions

Architecture/specification:

- [ ] No ordinary lexical/operator addition requires duplicate semantic declarations
- [ ] Canonical IR and runtime behavior contain no human semantic identity strings
- [ ] No active rule depends on obsolete `dictionary.toml`/semantic-form TOML ownership or `language/legacy/`
- [ ] All normative docs agree with executable package behavior
- [ ] Semantic/surface/documentation compatibility policy is executable
- [ ] Compatibility corpus is frozen for 1.0
- [ ] Canonical grammar/lexicon/reference documentation can be generated from the package

Documentation/site:

- [ ] English reference coverage is complete
- [ ] Controlled English rendering corpus is frozen
- [ ] User-facing dictionary/analyzer is complete and does not require raw engine JSON
- [ ] Learning guide examples are executable/frozen

Validation:

- [ ] Full native suite passes
- [ ] WASM/browser suite passes
- [ ] Whole-language compiler passes
- [ ] Core discourse corpus round-trips
- [ ] Controlled English renderer golden corpus passes
- [ ] Spoken/written collision audit passes
- [ ] Adversarial ambiguity suite passes
- [ ] Documentation coverage/build checks pass
- [ ] Learning-site accessibility/integration checks pass

Release result: **Systean 1.0**.

---

## Milestones

### Architecture Complete (original baseline)

Reached by Phase 1 for the original implementation line. Phase 17 exposed architectural debt at the lexicon/semantics/surface boundary, so the accepted pre-1.0 revision in `SEMANTIC_DSL_ARCHITECTURE.md` deliberately supersedes those ownership assumptions before 1.0.

### Playable Systean

Reached after Phases 2–5. At this point real short conversations should be possible and should be used to drive later language authoring.

### Everyday Systean

Reached after Phases 6–15 with enough manually authored vocabulary to exercise them.

### Clean semantic architecture

Reached after Phases 18–19. At this point ordinary lexical and construction growth no longer depends on duplicate declarations, semantic strings, or Systean-specific Rust branches.

### Documented Systean

Reached after Phase 20. Every public word has complete English reference material and canonical semantics can be rendered deterministically into controlled English.

### Learnable Systean

Reached after Phase 21. The public site explains words and complete expressions contextually without exposing engine JSON as the learning interface.

### Systean 1.0

Reached after Phase 22.

---

## Rule for all implementation phases

Do not add a new surface feature merely because a natural language has one. For each feature, first identify:

1. the exact semantic structure and whether it is primitive, defined, or intrinsic-backed;
2. the single ownership/source-of-truth declaration;
3. the deterministic parse rule;
4. the deterministic generation rule;
5. the spoken realization;
6. the ambiguity failure behavior;
7. the controlled-English rendering behavior for public language content;
8. the user-facing explanation/provenance needed by documentation/analyzer tooling;
9. the regression/property test that proves the intended invariant.

Architecture can be documented without tests. Implementation cannot be considered complete without them.
