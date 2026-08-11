# Systean Final Language Architecture

Status: **normative target architecture**
Scope: the intended end-state architecture of Systean from phonemic input to discourse-aware semantic output and back.
Implementation status: only some layers are executable today. This document freezes the architecture, not the completion state.

This document complements the focused specifications in `PHONOLOGY.md`, `MORPHOLOGY.md`, `SEMANTICS.md`, and `SYNTAX.md`. When implementation work reaches a layer described here, its focused specification may add detail but must preserve the invariants and ownership boundaries defined below unless the language design is deliberately revised.

The pre-1.0 semantic/package architecture has been deliberately revised after Phase 17. [`SEMANTIC_DSL_ARCHITECTURE.md`](SEMANTIC_DSL_ARCHITECTURE.md) is normative for semantic identity, the `.semsys` DSL, compiled IDs, declarative surface realization, typed structured values, English documentation separation, and compatibility fingerprinting. Where an older paragraph in this document would conflict with that focused revision, the focused revision wins.

---

## 1. End-state goal

Systean is a human-spoken, human-written, versioned artificial language whose normative forms map deterministically to compositional semantic structures.

A valid normative expression must not require guessing about:

- lexical identity;
- word or morpheme boundaries;
- syntactic attachment;
- argument role;
- operator scope;
- quantifier scope;
- referent identity;
- whether omitted material is recoverable;
- whether a phrase is literal Systean, a proper name, or quoted external text;
- whether a numeric form denotes a number, a digit sequence, a quantity, a date, or another structured literal;
- whether an utterance is an assertion, question, command, request, repair, or another declared speech act;
- whether affect/emphasis changes propositional content.

Systean does not attempt to make reality unambiguous. Unknown, approximate, withheld, context-bound, generic, or uncertain information remains legal when the missing information itself is represented explicitly.

---

## 2. Final processing pipeline

The end-state analyzer follows this conceptual pipeline:

```text
written form / phonemic speech
        ↓
orthography ↔ phonology
        ↓
tokenization + structured-literal recognition
        ↓
morphological analysis
        ↓
surface parse
        ↓
typed surface elaboration
        ↓
discourse/reference resolution
        ↓
semantic lowering
        ↓
semantic type checking
        ↓
canonical semantic term
        ↓
discourse-state update
        ↓
optional downstream reasoning / world validation
```

The normative language ends at the canonical semantic term plus the deterministic discourse-state transition it causes.

Logical inference, theorem proving, factual verification, plausibility checking, and world modeling are downstream consumers. They never decide the normative parse.

The canonical generator traverses the reverse architecture:

```text
canonical semantic term + explicit discourse context
        ↓
canonical surface planning
        ↓
reference realization
        ↓
syntax linearization
        ↓
morphological generation
        ↓
canonical spelling ↔ pronunciation
```

The generator must never use a shorthand reference whose interpretation would be ambiguous in the supplied discourse state.

---

## 3. Source-of-truth ownership

Each fact must have one authoritative home. Derived indexes may exist in memory but must never require a second manually synchronized declaration.

### 3.1 Lexical and semantic declarations

Ordinary Systean words are authored once in typed `.semsys` declarations. The final architecture does not use `dictionary.toml` plus a duplicated semantic operator signature as two authoritative declarations.

A lexical declaration owns:

- the Systean root/surface identity;
- one typed semantic declaration: primitive, defined, or intrinsic-backed;
- its parameter/result types;
- non-default surface behavior only when genuinely needed.

The Systean root is not mapped through an English semantic identifier. Human labels/explanations are separate documentation metadata.

### 3.2 Semantic model

`.semsys` modules own:

- semantic types and subtype/generic relationships;
- primitive symbols;
- algebraic data constructors;
- formal definitions;
- intrinsic capability declarations;
- contexts, dimensions, and units;
- declarative semantic/effect rules;
- lexical declarations that bind Systean roots to those structures.

Rust implements the generic typed calculus, compiler, intrinsic ABI, effect runtime, and validation machinery. It does not own the Systean word inventory or interpret English/source names as semantic behavior.

### 3.3 Surface realization

Typed `form` declarations and a small set of genuine structural policies own Systean surface realization. Parser and canonical linearizer are derived from the same compiled surface rule.

The architecture does not maintain a duplicate syntax-specific root inventory and does not require one Rust enum variant per Systean construction.

### 3.4 Human documentation and English rendering

English documentation is a separate package layer. Every public 1.0 word must have an English short gloss and detailed explanation, but those strings do not participate in semantic identity.

Controlled English rendering consumes canonical semantics. It is not word-for-word gloss substitution.

### 3.5 Phonology and morphology

`alphabet.toml`, `phonology.toml`, and `morphology.toml` own only their respective mechanisms. The current phoneme/alphabet inventory is fixed. Morphology v1 remains `WORD = ROOT` unless a concrete future local derivation justifies an extension.

### 3.6 Package identity and versioning

The package has one package-level version source and separate semantic, surface, and documentation fingerprints. Module-local schema versions may exist for file-format compatibility, but they are not separate language versions.

---

## 4. Representation layers

The final engine distinguishes these representations explicitly:

```text
PhonemicForm
OrthographicForm
TokenStream
MorphologyAnalysis
SurfaceAst
TypedSurfaceAst
ResolvedSurfaceAst
SemanticTerm
CanonicalSemanticTerm
DiscourseState
```

These representations must not be collapsed merely for implementation convenience when doing so would hide provenance or make ambiguity impossible to diagnose.

Every transformation records enough provenance for the analyzer to explain where a semantic component came from.

---

## 5. Lexicon architecture

### 5.1 One root, one lexical identity

An ordinary root has one normative lexical identity. Context does not choose between unrelated senses.

An ordinary lexical declaration is a typed function/value declaration, for example conceptually:

```text
word sol : Entity;
word per($entity: Entity) -> Prop;
word vid($observer: Entity, $observed: Entity) -> Prop;
```

The compiled semantic identity is a package-owned symbol ID, not the English strings `person`, `see`, `observer`, or similar labels.

Traditional POS tags are not required to determine meaning.

### 5.2 Primitive, defined, intrinsic-backed

A lexical declaration is exactly one of:

- primitive semantic identity;
- formally defined composition over other typed terms;
- binding to a declared generic runtime intrinsic.

Adding an ordinary primitive or defined word must not require editing Rust.

### 5.3 Manual vocabulary authoring

Roots are authored manually by the language author. Tooling may:

- validate spelling and pronunciation legality;
- detect exact collisions;
- warn about near collisions;
- report reserved-token conflicts;
- show semantic binding/type conflicts;
- compare a proposed root against the existing inventory.

Tooling must not allocate or invent roots automatically.

### 5.4 Human explanations are not identity

English glosses, explanations, examples, and search metadata are documentation. They may change without changing semantic identity.

Every public 1.0 word must eventually have complete English documentation under the Phase 20 contract.

### 5.5 No productive vague derivation

There is no generic `ROOT -> related_to(ROOT)` derivation. If a relation means `made_of`, `caused_by`, `similar_in_color_to`, `located_at`, or another specific concept, that relation is expressed explicitly.

---

## 6. Surface syntax architecture

The current syntax decisions remain normative:

- primary participant → predicate → remaining frame arguments;
- no unmarked free word order;
- semantic roles come from the declared frame order;
- `ki ... ku` is the universal explicit grouping/scope mechanism;
- explicit grouping is required only when the default structure would differ;
- quantifiers nest by order of appearance by default;
- `AND` binds more tightly than `OR`;
- chains of the same associative operator flatten canonically;
- questions, commands, and requests use explicit constructions;
- focus does not reorder core arguments;
- traditional noun/verb/adjective classes are not the grammar's primary categories.

The default realization of an ordinary typed word is derived from its arity and the canonical frame policy. Non-default realization is declared with typed bidirectional `form` rules. Quantifiers and other higher-order constructions should be expressed through typed function-valued arguments when possible instead of receiving dedicated parser branches. Parser implementation order never resolves an ambiguity; complete typed overlap remains an error.

The canonical core lexical forms currently fixed are:

```text
ki  scope open         structural marker
ku  scope close        structural marker
ke  truth question
ne  logical negation
va  conjunction
zo  disjunction
ra  universal quantifier
mu  existential quantifier
da  command
me  request
```

`ki` and `ku` are structural reserved forms rather than ordinary lexical roots.

---

## 7. Typed surface elaboration

Reference resolution needs semantic type expectations before all references are known. Therefore the final pipeline has a typed surface-elaboration phase between parsing and discourse resolution.

Typed elaboration determines, without resolving referents:

- which semantic operator/frame a lexical item denotes;
- expected argument roles;
- expected semantic types for each explicit or omitted slot;
- local binder scopes;
- scope structure;
- which nodes are unresolved reference requests;
- which nodes introduce potential discourse referents.

This phase may reject structurally impossible type combinations but may not choose a discourse referent by plausibility.

---

## 8. Discourse state

`DiscourseState` is runtime language state, not world knowledge.

A discourse state contains formal entries equivalent to:

```text
ReferentId
SemanticValue / semantic identity
SemanticType
IntroductionOrigin
AccessibilityScope
ExplicitAliases
Optional declared constraints
```

It may also retain prior utterances/propositions/events when they are referenceable.

It does not contain an open-ended model of what is physically likely, socially expected, or factually true.

---

## 9. Reference resolution

The reference architecture is fixed by the following rules.

### 9.1 Unique-resolution rule

A shorthand reference is valid only when its declared constraints leave exactly one accessible compatible referent.

```text
0 candidates  -> unresolved-reference error
1 candidate   -> valid resolution
2+ candidates -> ambiguous-reference error
```

There is no recency winner, salience score, probabilistic ranking, gender guess, or world-plausibility tie breaker.

### 9.2 What introduces referents

Concrete/specific expressions and existential constructions may introduce accessible discourse referents.

Universal quantifiers such as `ra` introduce local bound variables inside their scope. They do not create one post-scope individual referent corresponding to "every person".

### 9.3 What can be referenced

Reference is not limited to people or entities. Any language-declared semantic value may be referenceable when useful, including:

- Entity;
- Event;
- Process;
- State;
- Activity;
- Proposition;
- Quantity;
- Time/Interval values;
- utterances or other declared semantic structures.

### 9.4 Explicit aliases

The language supports explicit local aliases. An alias binds a stable discourse identifier to a semantic value and has a lexical scope/lifetime.

Aliases are exact. They do not use unique-resolution search after binding.

An alias may survive ordinary short-term discourse boundaries until the alias's own declared scope ends.

### 9.5 Discourse boundaries

The language has explicit discourse/section boundaries that retire ordinary shorthand candidates according to deterministic accessibility rules.

Referents do not remain equally accessible forever merely because they appeared earlier in a text.

### 9.6 Omitted arguments

Argument omission is not a separate heuristic subsystem. An omitted argument is an implicit reference request with an expected semantic type and role.

It is legal only when normal reference resolution yields exactly one candidate.

Until that resolver is implemented, required frame arguments remain mandatory.

### 9.7 Pronoun-like convenience forms

Human-friendly short reference words may be added later, but they are surface realizations of this same typed reference mechanism. They never introduce a second heuristic pronoun-resolution system.

---

## 10. Local definitions and aliases

Systean supports scoped local naming for values, propositions, events, and constructed concepts.

A local definition has:

- an explicit binder/alias;
- one semantic target;
- a deterministic lexical scope;
- capture-safe internal identity independent of the surface alias spelling.

Local definitions are not new global dictionary roots. They disappear when their scope ends.

The semantic IR uses ordinary binding/alias mechanisms rather than dynamically mutating the global lexicon.

---

## 11. Proper names

Proper names form one productive marked class.

The architecture is:

```text
proper-name marker + name payload
        ↓
NameExpression
        ↓
label/reference semantics
```

Rules:

1. Proper-name status must be identifiable in both speech and writing.
2. Native and foreign names do not use different grammatical classes.
3. A proper name is a label, not a globally unique identifier.
4. Two entities may share the same name; when that creates ambiguity, ordinary reference-disambiguation rules apply.
5. The parser never guesses the source language or pronunciation of an unmarked foreign string.
6. The name payload has a deterministic canonical Systean spoken representation or is carried through an explicit external-text mechanism.
7. Proper names do not become productive common-word loanwords merely by appearing in Systean discourse.

The exact surface marker and adaptation algorithm are language-authoring decisions, not unresolved engine architecture.

---

## 12. Quotation and external text

Foreign/non-Systean text is represented as an explicitly delimited opaque payload.

The architecture distinguishes:

- Systean syntax being interpreted;
- quoted text being mentioned as data;
- a proposition describing an act of saying/writing that text.

Quotation has explicit boundaries in both writing and speech. Nesting must remain recoverable without punctuation-only information.

The parser does not tokenize or semantically interpret the inside of an opaque external quote as Systean unless an explicit nested Systean quotation mode is opened.

Quoted payloads lower to a structured text value, not to silently parsed language.

---

## 13. Structured literals

Not every formal value should consume a manually authored lexical root. The engine therefore supports configured structured-literal families.

A literal family has a deterministic:

```text
written recognition
spoken recognition
typed semantic value
canonical written generation
canonical spoken generation
```

The initial required families are:

- numbers;
- digit sequences;
- text/quotation payloads;
- proper-name payloads where applicable;
- temporal/calendar literals if the final surface design uses compact notation.

Literal codecs are generic engine mechanisms. Their concrete Systean realizations are package-defined.

---

## 14. Numbers

Systean distinguishes at least:

```text
Number
Digit
DigitSequence
```

`7` as the number seven and `7` as one digit in an identifier are not silently interchangeable semantic objects.

The numeric subsystem must provide:

- canonical integer notation;
- negative values;
- exact finite decimal/rational representation as required;
- scale/exponent notation;
- one systematic normative pronunciation for each supported written form;
- deterministic parse/generate round trips.

Alternative spellings that normalize to the same mathematical value may be accepted only if the standard deliberately declares them; the generator emits one canonical form.

Approximation is not encoded by silently rounding a `Number`; approximation is an explicit semantic construction.

---

## 15. Quantities and units

A quantity is structurally distinct from a bare number.

Conceptually:

```text
Quantity {
    value: Number,
    unit: Unit
}
```

The language package declares units and exact formal conversion relationships where applicable.

The unit subsystem must distinguish:

- dimensions;
- units;
- scalar numeric value;
- exact versus approximate measurement;
- precision/uncertainty when semantically stated.

A unit symbol/name has one normative identity and deterministic spoken form. Unit conversion belongs to formal evaluation, not syntactic parsing.

---

## 16. Time and calendar values

Temporal semantics uses explicit values and relations rather than obligatory tense inflection.

The final semantic vocabulary may include broad types equivalent to:

```text
Instant
Interval
Duration
CalendarDate
TimeOfDay
```

The exact internal type split may be refined, but the architecture requires the distinction between:

- an event/process/state;
- the interval or instant associated with it;
- a duration;
- a calendar representation;
- a context-bound value such as `now`.

Relative temporal expressions use explicit relations such as before/after/during and explicit or context-bound anchors.

`now`, `today`, speaker, addressee, and similar deictic values are deterministic context inputs. They are not guessed from textual plausibility.

Dates/times may use compact notation only if that notation has one canonical interpretation and one normative spoken rendering.

---

## 17. Events, processes, states, activities, and aspect

The existing first-class `Occurrence` family remains the basis for aspect.

Aspectual operators act on an explicitly constructed target:

```text
start(target)
cease(target)
continue(target)
finish(target)
interrupt(target)
repeat(target, count)
```

The surface language must not make the parser guess whether the target means:

- one concrete event;
- a process involving a particular object;
- a state;
- a repeated activity;
- a habit.

Habituality/repetition is represented compositionally around an explicit target.

Temporal relations and aspect are orthogonal: tense-like temporal location does not change which occurrence is being targeted.

---

## 18. Unknown, unspecified, existential, and withheld values

These meanings remain distinct.

The language must be able to distinguish structures equivalent to:

- a value exists but is not identified (`exists`);
- the speaker explicitly leaves a field unspecified;
- the value is unknown to a stated knower;
- the value is intentionally withheld;
- the value is a local variable/binder.

No generic "missing thing" marker may collapse these into one interpretation.

---

## 19. Approximation, vagueness, and contextual standards

Approximation is explicit semantic content.

A statement such as "about 10" must represent that approximation rather than parse as exactly `10` and rely on pragmatic forgiveness.

Contextual standards such as "tall" may exist only when their contextual parameter is formally represented or deterministically supplied by an explicit context mechanism.

The parser never invents a comparison class from world knowledge.

---

## 20. Generic and statistical statements

Universal, existential, generic, typical, majority, and probabilistic claims are not conflated.

If the language later provides concepts equivalent to `most`, `usually`, `typically`, or probability claims, each receives a declared semantic operator/signature.

A generic statement is not merely an unmarked universal assertion whose exceptions are tolerated pragmatically.

---

## 21. Speech acts

`Proposition` and `Utterance` remain distinct semantic objects.

The surface architecture supports at least:

- unmarked/default assertion of a top-level proposition;
- truth question (`ke`);
- command (`da`);
- request (`me`).

Additional speech acts may be added compositionally when their semantics are defined.

Question types that request a value, choice, reason, location, quantity, etc. are explicit operators/constructions rather than word-order inversion or an overloaded generic question that requires guessing what slot is requested.

---

## 22. Affect, emotion, and expressivity

Emotion is permitted as explicit semantic/pragmatic content. The author-selected subjective-state inventory is frozen in [`SUBJECTIVE_STATES.md`](SUBJECTIVE_STATES.md).

The architecture distinguishes an experienced affect/state from the explicit expressive `emo` layer. Affect roots denote experienced state concepts; `emo` attaches an explicitly declared expressive attitude to an utterance/content/target. These structures must not be silently collapsed.

The architecture supports affect attached to a declared target, such as:

- the utterance as a whole;
- a proposition;
- a referenced event/value;
- a standalone expressive utterance.

Intensity, target, and cause are explicit parameters when semantically relevant. Intensity variants are not separate lexical senses merely because ordinary languages use distinct words for weak and strong degrees. Bodily subjective states remain typed separately from affective states. Romantic attraction, sexual attraction, sexual arousal, baseline sexual drive, lust, general love, and action desire remain distinct semantic concepts.

The 1.0 baseline does not require a parallel lexicon of dedicated interjection roots. Standalone expressives compose through `emo`; future convenience sugar is acceptable only with one canonical expansion.

Prosody remains naturally expressive but does not reverse or replace literal propositional content. Sarcasm is not a hidden semantic-negation mechanism.

---

## 23. Focus and topic

Focus/topic does not alter canonical core argument order.

It is represented by explicit pragmatic constructions that point to a target already present in the utterance/discourse structure.

Focus/topic may affect information structure or discourse salience declared by the language, but it must not silently change semantic roles or quantifier scope.

---

## 24. Repair, correction, and retraction

Conversation needs deterministic self-repair.

The final discourse layer supports explicit acts equivalent to:

- retract a prior utterance/proposition;
- replace a referenced expression/value with another;
- correct a previous assertion;
- clarify a reference or alias.

Repairs reference the material they modify. A listener never has to infer from a hesitation which earlier constituent was replaced.

Repair semantics modifies discourse commitments/state; it does not rewrite the historical parse invisibly.

---

## 25. Text, turn, and section structure

A complete Systean implementation operates on discourse, not only isolated sentences.

The structural layer therefore distinguishes boundaries equivalent to:

```text
Expression
Utterance
Turn
DiscourseBlock / Section
Document
```

Semantically relevant boundaries must have spoken equivalents. Typography alone cannot create a normative scope distinction.

Boundary rules determine:

- reference accessibility;
- local-alias lifetime;
- repair targets;
- quotation nesting;
- where default assertion/speech-act interpretation applies.

The Phase 15 executable baseline now realizes this hierarchy as `TextDocument` → channel-metadata `TextTurn` → utterances/`fra` blocks. Raw spoken utterances end in `du`; raw written utterances end in `.`. Turn boundaries are supplied by channel metadata rather than a new lexical marker because they currently introduce no otherwise-unrecoverable semantic distinction. `fra` advances ordinary-reference accessibility while exact aliases keep their declared lexical scope. Boundary markers inside `sit ... tis` remain opaque text.

---

## 26. Canonicalization

Canonicalization is representation-level normalization, not theorem proving.

It may normalize structures such as:

- alpha-equivalent binders;
- deterministic named-role ordering;
- record field ordering;
- same-operator associative flattening where declared;
- redundant `ki ... ku` grouping;
- canonical numeric/literal spelling.

It must not silently apply semantic theorems such as distributivity, commutativity, implication equivalences, or real-world facts unless the language standard explicitly elevates such a transformation into representational identity.

---

## 27. Canonical generation

The standard generator emits one preferred normative surface form for a canonical semantic structure and explicit discourse context.

Generation must respect:

- canonical frame order;
- precedence;
- minimum required grouping;
- reference uniqueness;
- canonical literal notation;
- canonical spelling/pronunciation;
- explicit speech-act and affect constructions;
- declared text/discourse boundaries.

The generator may choose a shorthand reference only if parsing that shorthand in the same discourse state resolves to the intended referent uniquely.

---

## 28. Whole-language compiler

The final `LanguagePackage` compiler validates cross-layer invariants before a package is accepted.

### 28.1 Lexical/phonological checks

- every reserved or lexical surface form is phonologically valid;
- written and phonemic forms round-trip;
- no exact lexical/pronunciation collisions exist unless structurally disambiguated by an explicit declared mechanism;
- reserved structural forms cannot collide with ordinary roots;
- near-collision warnings remain advisory unless a future standard makes a threshold normative.

### 28.2 Morphological checks

- every lexical word has one decomposition;
- generation/analyze round-trip holds for declared roots and any future derivations;
- structured literals cannot be mistaken for ordinary roots without an explicit delimiter/class marker.

### 28.3 Syntax checks

- precedence and grouping rules are internally consistent;
- parser/generator share one compiled grammar model;
- declared lexical construction kinds have valid semantic signatures;
- frame roles can be linearized deterministically;
- structural tokens are not duplicated across ownership layers.

### 28.4 Semantic checks

- operator bindings exist;
- role names match signatures;
- generic arity is valid;
- subtype graph is valid;
- surface constructions lower to type-correct terms for their declared domains.

### 28.5 Discourse checks

- reference forms declare deterministic candidate constraints;
- omission routes through the same resolver;
- alias/boundary policies cannot create hidden fallback heuristics.

### 28.6 Ambiguity verification

Some language-wide ambiguity properties cannot be proved by naively enumerating an infinite language. The compiler therefore combines:

- statically decidable grammar/config checks;
- deterministic parser construction constraints;
- generated AST → surface → AST property tests;
- generated semantic term → surface → semantic term tests where generation is defined;
- bounded exhaustive corpora for short constructions;
- adversarial ambiguity corpora;
- spoken-form collision analysis;
- revision-diff analysis.

A deterministic parser that merely picks the first branch is not sufficient evidence of unambiguity. The grammar compiler must reject or expose overlapping alternatives rather than silently resolve them by implementation order.

---

## 29. Versioning and compatibility

Systean language versions are independent of engine crate versions.

The final package exposes at least three compatibility domains:

```text
semantic_fingerprint
surface_fingerprint
documentation_fingerprint
```

Semantic fingerprints exclude comments, English glosses/explanations, source formatting, and alpha-equivalent local binder names. They include signatures, definitions, type/data structure, intrinsic bindings, and semantic/discourse behavior.

Surface fingerprints include roots, forms, precedence/grouping, structured notation, and canonical spoken/written realization.

Documentation fingerprints include English glosses, explanations, examples, and other learning/reference metadata.

### Compatible revision

A compatible revision may:

- add new roots without changing existing root meanings or old parses;
- add new operators/constructions that do not change previously valid analyses;
- add structured-value domains without reinterpreting old forms;
- improve diagnostics/tooling;
- improve English documentation without changing semantic/surface identity.

### Breaking revision

A breaking semantic/surface revision is required to:

- change an existing root's normative meaning;
- remove a previously valid root/construction;
- change the parse or scope of an existing valid surface expression;
- change canonical pronunciation/spelling of existing forms;
- change reference resolution or discourse effect of an existing valid discourse under the same explicit context.

Deprecation metadata may discourage generation of an old form, but compatible revisions do not silently change its meaning.

---

## 30. Analyzer, English reference, and learning tooling

The analyzer exposes the same engine pipeline used by CLI/WASM, not a parallel implementation.

For a word, user-level tooling can show:

- lexical root;
- pronunciation, syllables, and stress;
- English short gloss;
- detailed English explanation;
- primitive/defined/intrinsic-backed status;
- human-readable typed signature and argument explanations;
- declared surface realizations/construction patterns;
- examples and controlled English rendering;
- advanced provenance/spec links when requested.

For an utterance/discourse, user-level tooling can show:

- deterministic controlled English rendering;
- interactive token-level semantic contribution;
- argument/dependency relationships;
- grouping/scope;
- expected and resolved reference/context/alias relationships;
- candidate referents on ambiguity errors;
- discourse effects/history links;
- canonical regenerated form;
- optional advanced AST/semantic IR/provenance.

Raw JSON, numeric IDs, serialized Rust enums, and provenance blobs are developer diagnostics, not the default learning interface.

The workbench/API must expose enough structured dependency/scope/reference/rendering-provenance data that the Svelte site never reconstructs semantics from token strings or formatted IR.

Diagnostics should distinguish the layer that failed while presenting a human explanation first.

---

## 31. Heuristic assistants remain non-normative

LLMs and heuristic tools may help a human:

- suggest how to express an intended meaning;
- propose a more explicit reference;
- explain an ambiguity error;
- search the dictionary;
- warn that two candidate roots sound similar;
- propose paraphrases.

They never silently become the normative parser, reference resolver, or semantic interpreter.

---

## 32. Final engine module boundaries

The target Rust architecture is conceptually:

```text
systean-core/
  package/       load, module graph, IDs, version, fingerprints, provenance
  dsl/           `.semsys` parser + source AST
  symbols/       interning/resolution for symbols/types/constructors/units/etc.
  phonology/     spelling/pronunciation/syllables/stress
  morphology/    word analysis/generation
  semantics/     typed term/data IR, elaboration, checking, canonicalization
  surface/       typed bidirectional rules, parser, linearizer, scope structures
  codecs/        structured written/spoken notation <-> typed values
  discourse/     referents, aliases, boundaries, typed resolution, generic effects
  compiler/      cross-layer validation, ambiguity checks, compatibility/fingerprints
  english/       deterministic controlled-English renderer + alignment provenance
  documentation/ compiled English reference/search metadata
  workbench/     explainable end-to-end user/developer reports
```

Exact Rust file names may differ. The architectural ownership must not.

`cli` and `wasm` remain thin consumers of `systean-core`. The Svelte site remains a UI consumer and never reimplements language semantics or English rendering.

---

## 33. Target language-package boundaries

The exact file split may evolve, but the final package must preserve single ownership and the Phase 18–21 separation.

A plausible end state is:

```text
language/
  package.toml
  alphabet.toml
  phonology.toml
  morphology.toml

  semantics/
    core.semsys
    logic.semsys
    discourse.semsys
    values.semsys
    quantities.semsys
    time.semsys
    pragmatics.semsys

  lexicon/
    core.semsys
    subjective.semsys
    ...

  surface/
    core.semsys
    literals.semsys
    ...

  docs/
    en.sydoc

  corpus/
    compatibility.tsv
    adversarial.tsv
```

Data-only alphabet/phonology/morphology configuration may remain TOML. Semantic declarations, lexical function signatures, algebraic structured values, dimensions/units, and nontrivial surface/effect behavior must not be encoded as nested TOML objects that merely serialize engine enums.

The Phase 17 files `dictionary.toml`, semantic parts of `syntax.toml`, `literals.toml`, and string-identified `units.toml` are migration sources, not the desired permanent ownership model.

---

## 34. What remains language authoring rather than architecture

After this architecture is frozen, the following still require deliberate human language design:

- actual content roots;
- exact proper-name marker and name adaptation surface form;
- exact reference/alias particles;
- numeral pronunciations and compact notation choices;
- unit vocabulary;
- temporal vocabulary;
- value-question constructions;
- implementation of the already frozen affect/subjective-state vocabulary in `SUBJECTIVE_STATES.md`;
- focus/topic/repair particles;
- discourse boundary markers;
- everyday predicate frames and definitions.

These are not reasons to redesign the engine architecture. They are language-package content to be authored and tested within it.

---

## 35. Definition of architectural completion

The architecture is considered complete when every normative language feature belongs to exactly one of the mechanisms above and no ordinary future Systean feature requires a new Systean-specific interpretation path in Rust.

The language implementation is considered 1.0-ready only when the implementation roadmap is complete, the canonical package compiles without ambiguity violations, the core conversational corpus round-trips, and the analyzer/generator expose the complete pipeline.
