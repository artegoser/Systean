# Systean Design Baseline

Status: **design baseline for the repository**  
Purpose: preserve the goals, architectural decisions, invariants, and unresolved questions that should govern future Systean work.

This document supersedes historical prototype assumptions where they conflict with the current executable language package. Active normative files live under `language/`; superseded grammar and morphology prototypes live under `language/legacy/`.

Detailed semantic, phonological, morphological, and surface-syntax architecture is specified in [`SEMANTICS.md`](SEMANTICS.md), [`PHONOLOGY.md`](PHONOLOGY.md), [`MORPHOLOGY.md`](MORPHOLOGY.md), and [`SYNTAX.md`](SYNTAX.md). Those focused specifications take precedence over older examples in this baseline when they conflict.

---

## 1. Mission

Systean is a human-first artificial language designed around **unambiguous literal communication**.

The central goal remains:

> **Unambiguity in every aspect.**

For engineering purposes, this means that a valid Systean utterance should have one recoverable structural interpretation from its normative written or spoken form. The listener or parser must not have to guess which syntax, scope, referent, relation, lexical sense, or pragmatic reinterpretation the speaker intended.

Systean is not intended to become a programming language disguised as speech. It must remain practical to pronounce, learn, write, read, and use in ordinary conversation. Formal precision is the constraint under which a usable human language is designed, not a reason to abandon human usability.

The language must support both:

- **strict machine analysis**, with deterministic parsing and explicit semantics;
- **normal human conversation**, including emotions, emphasis, questions, commands, names, quantities, uncertainty, and discourse across multiple sentences.

The language must not rely on an LLM or probabilistic guesser to determine the normative meaning of valid Systean.

---

## 2. What Systean guarantees — and what it does not

Systean should remove **linguistic ambiguity**, not pretend to remove uncertainty from reality.

These are different:

### Forbidden ambiguity

A form has several possible interpretations and the listener has to choose one.

Examples of mechanisms that should not exist in normative Systean:

- one adjective form meaning `caused by`, `made of`, `located at`, or `similar to` depending on the noun;
- a pronoun that could refer to two active entities;
- an unmarked phrase whose quantifier scope can be parsed in two ways;
- a sarcastic statement whose intended proposition is inferred to be the opposite of its literal proposition;
- an ordinary root with several unrelated lexical senses selected from context.

### Allowed explicit underspecification

The speaker communicates exactly what is unknown, omitted, variable, approximate, or intentionally unspecified.

For example, Systean must be able to express the semantic equivalent of:

- there exists some person, but I do not specify which person;
- the value is approximately 10;
- I know that two objects are similar with respect to some property, but I am not specifying the property;
- I do not know whether a proposition is true;
- some but not necessarily all members of a set satisfy a condition.

The missing information is then **part of the explicit meaning** rather than something the listener has to infer.

A useful rule is:

> **Systean may represent unknown information. It may not hide which information is unknown.**

---

## 3. Core invariants

The language design and language compiler should target the following invariants.

### 3.1 Written and spoken form

1. Normative written Systean must determine normative pronunciation.
2. Normative pronunciation must determine the corresponding phonemic written Systean form.
3. Semantically significant distinctions must be available in both writing and speech.
4. Capitalization alone must never carry grammatical or semantic information, because capitalization does not exist in speech.
5. Semantically significant punctuation must have an equivalent spoken realization.
6. Word boundaries and morpheme boundaries must be recoverable without guessing.

### 3.2 Morphology

1. Every valid surface word has one morphological decomposition.
2. Morphological composition is deterministic and reversible.
3. Grammar must not depend on accumulating unreadable consonant prefixes.
4. Grammatical markers should be pronounceable units, normally containing vowels.
5. The language must define phonotactic constraints that prevent pathological consonant clusters.
6. A grammatical feature and its surface phonological realization are separate layers.

### 3.3 Lexicon

1. One ordinary lexical root has one lexical identity / normative concept.
2. Context must not select between unrelated senses of the same ordinary root.
3. A root must not automatically gain vague noun/adjective/adverb meanings such as `related to ROOT`.
4. Synonym proliferation should be avoided in the canonical lexicon: one canonical concept should normally have one canonical root.
5. New common concepts may receive new roots in later language revisions.
6. Existing canonical roots must not silently change meaning across compatible revisions.

### 3.4 Syntax and semantics

1. Every valid sentence has one syntactic structure.
2. Every syntactic structure elaborates to one semantic structure.
3. Scope must never be selected by contextual guessing.
4. A reference must either resolve uniquely or require a more explicit form.
5. Ellipsis/elision is allowed only when the omitted structure is uniquely recoverable.
6. Literal meaning is the normative meaning.
7. Metaphorical reinterpretation is not part of normative Systean.
8. Idiomatic meaning that cannot be compositionally recovered is not part of normative Systean.
9. Tone/prosody does not silently rewrite propositional meaning.

### 3.5 Round-trip properties

Where applicable, the implementation should enforce properties equivalent to:

```text
analyze(generate(x)) = x
parse(linearize(ast)) = ast
```

A language revision that creates collisions should fail to compile.

---

## 4. Language architecture: engine vs. language package

Systean should be **config-driven**, but "config-driven" does not mean that no logic may exist in code.

The engine implements universal mechanisms. The Systean package defines the actual language.

### 4.1 The engine may know about

- symbols and token streams;
- feature systems;
- generic types and type checking;
- terms, variables, binding, application, records, and composition;
- declarative parsing/generation rules;
- finite-state or equivalent morphology mechanisms;
- deterministic grammar compilation;
- phonological transformations;
- constraint solving/unification where required;
- provenance tracking;
- diagnostics;
- round-trip and collision testing;
- configuration loading and versioning.

### 4.2 The engine must not hardcode Systean-specific facts such as

- `noun` or `verb` as fixed engine enums unless the generic grammar formalism genuinely requires a category system and Systean declares those categories through config;
- SVO word order;
- a particular question marker;
- `past`, `future`, `plural`, `definite`, etc.;
- `ki` / `ku`;
- current suffixes such as `-o`, `-i`, `-a`, `-e`;
- particular pronouns;
- particular semantic operators such as `before`, `possible`, `give`, or `similar`;
- the inventory or semantics of Systean roots.

A practical architecture test is:

> If a normal change to the Systean language requires editing engine source code instead of the language package, the abstraction boundary is probably wrong.

A source-code change is justified when Systean needs a genuinely new **meta-mechanism**, not merely a new grammatical construction or lexical concept.

---

## 5. Three principal representation layers

Systean should be designed as three tightly linked layers.

```text
SEMANTIC LANGUAGE
        ↕
SURFACE SYSTEAN
        ↕
PHONOLOGICAL / ORTHOGRAPHIC SYSTEAN
```

These are not separate languages. They are separate representations of the same language.

### 5.1 Semantic layer

Represents what an utterance literally says:

- entities and properties;
- predicates and relations;
- quantities;
- variables and binders;
- scope;
- speech acts;
- references;
- time relations;
- uncertainty and explicit underspecification;
- discourse references;
- emotional expression when explicitly stated.

### 5.2 Surface layer

Defines the human sentence structure:

- constructions;
- word order;
- particles;
- morphology;
- grouping/scope markers;
- compact conversational forms;
- question/command/assertion realization;
- proper-name realization;
- numeral realization.

### 5.3 Phonological/orthographic layer

Defines:

- alphabet and phoneme mapping;
- syllable structures;
- phonotactics;
- stress;
- morpheme realization;
- permissible word forms;
- spoken boundaries;
- canonical spelling;
- phonological distinctness requirements.

The analyzer should be able to traverse these layers in both directions.

---

## 6. Semantic core

The semantic core should be small, generic, compositional, and extensible.

The engine should not contain a giant fixed enum such as:

```text
Temporal
Modal
Comparison
Quantifier
Causality
...
```

Instead, most language concepts should be declared as typed symbols/operators in the language package and composed using a smaller generic term system.

Conceptually, the generic engine may need mechanisms equivalent to:

```text
Symbol
Literal
Variable
Apply
Bind
Record / named arguments
```

The semantic architecture is now specified in more detail in [`SEMANTICS.md`](SEMANTICS.md). The baseline direction is a small typed compositional calculus with generic mechanisms equivalent to constants, variables, literals, application, binding, and structured/named values. Systean-specific operators remain package-defined rather than engine enums.

### 6.1 Composition creates expressiveness

Systean should not attempt to list every kind of thought as a separate grammar rule.

The language should gain open-ended expressiveness from recursively composable semantic structures. A clause may participate inside a larger clause; operators may scope over propositions; variables may be introduced and referenced; relations may take structured arguments.

The important target is:

> **Expanding the space of concepts should not require expanding the fundamental mechanics of the language.**

---

## 7. Semantic typing without a world ontology

Typing is useful, but Systean must not become Wikidata.

### 7.1 What typing is for

Typing should prevent **structural category errors**, not decide whether an assertion is sensible, physically possible, socially normal, or true.

For example, it can distinguish:

```text
Number
Digit
DigitSequence
Entity
Property
Proposition
Time
Quantity
```

and prevent applying an operator to the wrong semantic category.

### 7.2 What typing is not for

The lexicon should not contain giant fact tables such as:

```text
emits_light = true
self_luminous = true
has_mass = true
...
```

Those are encyclopedic facts, not grammar.

Likewise, roots should not contain exhaustive lists such as:

```text
allowed_with = [...]
forbidden_with = [...]
valid_adjectives = [...]
```

### 7.3 Predicate/relation signatures

Predicates and relations do require argument structure because that is part of their meaning.

Conceptually:

```text
give:
    giver
    transferred_thing
    recipient

inside:
    contained_entity
    container
```

These are semantic signatures, not lists of compatible dictionary words.

Reusable frames may reduce repetition, but they must not force unrelated concepts into inaccurate shared semantics.

### 7.4 Do not overtype lexical concepts

A lexical difference does not automatically require a distinct grammar type.

`STAR` and `PLANET` may be different lexical concepts while sharing whatever broad semantic category is sufficient for grammar. A type should exist because some language mechanism genuinely needs the distinction, not because a detailed taxonomic hierarchy can be invented.

---

## 8. Lexicon and lexical meaning

### 8.1 One root, one concept

An ordinary Systean root should identify one normative lexical concept.

The dictionary must not use the current pattern:

```text
[root]
noun = ...
adj = ...
adv = ...
```

when those entries represent different context-dependent meanings.

The current `sol` prototype illustrates the problem: noun/adjective/adverb forms introduce different relations and semantic assumptions. Future Systean must not generate meaning by silently choosing an appropriate English-like sense.

### 8.2 No generic `related_to(ROOT)` adjective

A construction equivalent to:

```text
ROOT + adjective => related_to(ROOT)
```

is too ambiguous.

Russian/English expressions like:

- solar person;
- solar weather;
- solar stroke;

hide different relations. Systean must express the actual relation or property.

If the intended meaning is cheerful, say cheerful. If something is caused by solar exposure, express causation. If a technical compound is frequent and has one fixed meaning, it may eventually receive its own lexical concept/root.

### 8.3 Explicit relation dimensions

Vague relations such as unrestricted `similar(A, B)` or `related_to(A, B)` should not serve as shortcuts for an intended specific relation.

Where similarity matters, the comparison dimension should normally be explicit:

```text
similar(A, B, dimension)
```

If the speaker genuinely does not know or intentionally does not specify the dimension, that can be expressed through explicit existential/unspecified semantics rather than leaving the listener to guess.

### 8.4 Atomic concepts are allowed

Systean does not need to reduce every concept into a finite set of more primitive concepts.

Some roots may be semantic primitives in a given revision. Human-readable definitions/glosses explain them to learners. Where practical, later concepts can be defined compositionally in Systean itself, but the language should not attempt to encode a complete ontology of reality.

A long-term possibility is a bootstrapped lexicon:

- a small primitive/core layer with normative external explanations;
- increasingly many concepts defined using existing Systean semantics;
- human glosses in other languages as documentation, not as the parser's source of truth.

---

## 9. Morphology

The current morphology is a prototype and should be redesigned.

### 9.1 Reject consonant-prefix stacking

The current architecture can produce sequences conceptually like:

```text
n + t + s + p + ROOT
```

This is machine-convenient but poor for speech, readability, and learnability.

Systean should not encode many independent grammatical features as single consonants concatenated before a root.

### 9.2 Abstract feature vs. realization

A grammatical feature such as `plural`, `past`, or `definite` is an abstract semantic/grammatical object.

Its spoken/written realization is a separate specification problem.

This allows the language compiler to enforce phonotactics rather than treating morphology as raw string concatenation.

### 9.3 Prefer pronounceable grammatical units

Grammatical markers should normally be syllabic or otherwise comfortably pronounceable. Exact forms are intentionally **not chosen in this document**.

The initial language should prefer regularity over natural-language-style irregular allomorphy. If allomorphy ever exists, it must remain deterministic and reversible.

### 9.4 Wide-scope operators should not be buried inside words

Many features currently represented as prefixes have different semantic scope.

Examples:

- definiteness concerns reference;
- number concerns cardinality;
- tense/time concerns an event or proposition;
- negation may scope over different semantic constituents.

Trying to encode all of them as a uniform prefix chain hides scope.

A likely direction is to use short, pronounceable particles/constructions for wide-scope operators and reserve bound morphology for genuinely word-local information.

### 9.5 Mandatory word-class marking is rejected

The old final-vowel noun/verb/adjective/etc. markers are not part of normative Systean.

Morphology v1 is `WORD = ROOT`. Surface syntax obtains the required structure from semantic bindings and deterministic constructions; it does not require every speaker to pronounce redundant POS metadata.

A future marker may be introduced only for a concrete semantic/structural need that cannot be recovered more cleanly. No suffix may mean "turn any concept into an adjective by choosing whichever relation sounds natural".

---

## 10. Phonology and orthography

The alphabet and its pronunciation mapping already exist and are normative. `language/alphabet.toml` remains the source of truth; the Rust engine consumes it rather than redefining Systean letters. The executable baseline is specified in [`PHONOLOGY.md`](PHONOLOGY.md).

### 10.1 Current baseline

The current phonological layer defines:

- the fixed existing phoneme/grapheme inventory;
- context-independent grapheme -> pronunciation mapping;
- reverse canonical pronunciation -> grapheme mapping;
- deterministic vowel-nucleus syllabification;
- lexical stress on the first syllable of the root;
- explicit root-span-aware analysis so future prefixes do not redefine lexical stress;
- a permissive cluster baseline because the previous language did not specify narrower onset/coda bans;
- exact spoken-stream segmentation checking over any supplied finite inventory.

Morpheme-boundary behaviour beyond root-aware stress, pauses that carry syntax, proper-name adaptation, and compound stress remain surface/morphology design questions rather than reasons to alter the existing alphabet.

### 10.2 Practical distinctness

Formal uniqueness is not enough, but root creation remains a human lexical-design task.

Tooling may report suspicious similarity between a proposed root and existing roots. Similarity is advisory rather than an automatic rejection threshold; exact spelling or pronunciation collisions remain errors. The engine must not randomly generate vocabulary.

### 10.3 Spoken and written equivalence

Any distinction represented by:

- parentheses;
- quotation marks;
- question marks;
- scope delimiters;
- separators;

must have a spoken equivalent if that distinction changes semantics.

Systean must not become unambiguous only on paper.

---

## 11. Syntax, grouping, and scope

The current `grammar.toml` state machine (`allowed_next`) is not an adequate long-term grammar model.

Systean needs recursive declarative grammar that can express nested structures without enumerating every part-of-speech transition.

### 11.1 Deterministic grammar

The language compiler should reject a grammar revision if the same normative token sequence can produce multiple syntax trees.

No "choose the first parse" behavior is acceptable.

### 11.2 Universal grouping/scope mechanism

The executable structural baseline uses the existing spoken/written pair:

```text
ki ... ku
```

as universal explicit scope/grouping delimiters. They are emitted only when required to preserve a non-default structure and are reserved from lexical-root use.

Local scope is otherwise recovered from deterministic construction order. In particular, scope-bearing operators and quantifiers nest in order of appearance where the grammar already yields one structure.

### 11.3 Logical precedence

Systean adopts one intentional logical precedence rule:

> `AND` binds more tightly than `OR`.

Both remain deterministic and are overridable with `ki ... ku`. Chains of the same operator may be flattened in the surface AST rather than requiring arbitrary binary spoken grouping.

---

## 12. Quantifiers, collections, and distributivity

Quantification is part of the semantic core and must not be added as an afterthought.

Systean must distinguish meanings equivalent to:

```text
for every person, there exists a book they read
```

and:

```text
there exists one book that every person read
```

Likewise, collective and distributive readings must be distinct.

For example, the equivalent of:

> three people lifted a table

must distinguish at least:

- the three people collectively lifted one table;
- each of the three independently lifted a table.

The language must not infer this distinction from plausibility.

Coordination has the same requirement. A construction equivalent to "old men and women" must structurally specify whether `old` scopes only over `men` or over both coordinated groups.

---

## 13. Reference and discourse

### 13.1 No inherently ambiguous generic pronoun

The current `su = He/She/It` model conflicts with the language goal.

A shorthand reference is acceptable only if its referent is uniquely recoverable under explicit discourse rules.

If two candidates remain, the shorthand is invalid and the speaker must disambiguate.

### 13.2 Discourse references are broader than people

Systean must support references to:

- entities;
- groups;
- events;
- propositions;
- previous utterances;
- locally defined concepts.

For example, the equivalent of:

> John bought a house. This surprised Mary.

requires `this` to refer to an event/proposition, not merely the most recent noun.

### 13.3 Context-dependent references

Expressions equivalent to `I`, `you`, `here`, and `now` may legitimately depend on context, provided their semantic rule is unique, e.g. conceptually:

```text
context.speaker
context.addressee
context.location
context.time
```

Context dependence is not the same as ambiguity when the dependency itself is explicit and deterministic.

---

## 14. Proper names

Systean should have **one productive proper-name class**, regardless of whether a name historically originated inside or outside Systean.

There is no grammatical distinction between "native proper name" and "foreign proper name".

### 14.1 Proper-name marker

A proper name must have a short marker or other overt morphological mechanism that is:

- visible in writing;
- audible in speech;
- unambiguous without capitalization;
- compatible with a previously unseen name;
- independently segmentable from the name body.

The exact marker is unresolved.

Conceptually:

```text
<NAME-MARKER> + <Systean-compatible phonological name form>
```

### 14.2 Names are labels, not global IDs

A name does not guarantee uniqueness.

Two entities may have the same name. If a reference to a named entity is not unique in the current discourse/context, additional identifying information is required.

### 14.3 Foreign spelling is not parsed as Systean

The proper-name body used in normative Systean must already be represented using Systean phonology/orthography.

The parser must not see `Schwarzenegger`, guess the source language, guess a pronunciation, and silently adapt it.

An editor may offer heuristic import assistance, but the deterministic parser never guesses.

Original foreign spelling may be stored by applications as metadata or explicitly quoted when the spelling itself matters. It is not required in ordinary Systean speech.

---

## 15. Borrowing and new vocabulary

Productive loanword marking should **not** be part of normative Systean.

If a new concept becomes useful enough to deserve a short lexical root, it should be added to a later revision of the Systean lexicon as an ordinary Systean root with:

- a single normative concept;
- a valid Systean phonological form;
- whatever minimal semantic signature/category is necessary.

Its etymological inspiration is not grammatically relevant.

The guiding principle is:

> **Grammar provides expressiveness. Lexicon provides compression.**

A missing root must never mean that a concept cannot be expressed. Before a concise root exists, the concept can be described compositionally and locally bound/referenced if necessary.

---

## 16. External text and quotation

Systean must be able to discuss external material without pretending that it is Systean.

This includes:

- foreign words;
- original name spellings;
- source code;
- arbitrary strings;
- quotations from other languages.

An external/quoted region is opaque to the normal Systean lexical parser unless a more specific quoted representation is explicitly requested.

The exact surface syntax must be human-friendly and available in speech. It must **not** require JSON-like inline syntax, language tags, or source-language metadata for ordinary use.

If source language or pronunciation matters, that information can be stated explicitly as normal Systean content rather than hidden in parsing heuristics.

Nested quotation and the distinction between:

- quoting exact characters;
- quoting exact sounds;
- quoting an utterance;
- referring to the proposition expressed by an utterance;

must eventually be specified.

---

## 17. Literal semantics: metaphor, idiom, sarcasm

### 17.1 Metaphor

Implicit metaphor is forbidden in normative Systean.

The equivalent of:

> John is a lion

must not conventionally mean brave, strong, aggressive, noble, etc.

If the speaker means brave, the speaker says brave. If a comparison is intended, its comparison dimension is expressed.

### 17.2 Idiom

A phrase whose conventional meaning cannot be compositionally recovered from its words should not be normative Systean semantics.

Frequent concepts can receive ordinary roots or transparent constructions instead of opaque idioms.

### 17.3 Sarcasm

Systean should not have an official mechanism where sarcasm transforms a literal proposition into an inferred different proposition.

Sarcasm fundamentally depends on conflict between literal content and intended interpretation and therefore conflicts with the core goal.

### 17.4 Pragmatic implicature

Normative lexical/grammatical meaning should not rely on conversational implicatures to add claims.

For example, a quantifier equivalent to `some` should have one explicit logical meaning (such as at least one, if that is what the language defines). If `some but not all` is intended, that must be stated by a distinct construction.

---

## 18. Emotions and human expressiveness

Removing metaphor and sarcasm must **not** make Systean emotionally sterile.

The language should have a rich but explicit expressive layer.

### 18.1 Emotion is legitimate semantic content

Systean should support concise expressions for states/attitudes such as:

- joy;
- excitement;
- surprise;
- admiration;
- affection;
- relief;
- frustration;
- anger;
- sadness;
- fear;
- disgust;
- disappointment;
- sympathy.

This list is illustrative, not final.

### 18.2 Expressive particles and interjections

Frequent emotions should be expressible through short, pronounceable particles/interjections rather than requiring long analytic sentences equivalent to "I experience emotion X regarding proposition Y".

Conceptually, an utterance may carry explicit affect such as:

```text
assert(P)
speaker_affect(joy, target=P)
```

but the surface language should realize this naturally and compactly.

An emotion particle may also function as a complete utterance when its target is uniquely established by discourse.

### 18.3 Intensity

Emotion intensity should be systematic rather than requiring unrelated lexical items for every degree.

Systean may support regular degree/intensity mechanisms, including an expressive intensifier, provided its semantic contribution is explicit.

### 18.4 Prosody remains free

Humans may still:

- speak loudly or softly;
- stretch sounds;
- laugh;
- whisper;
- pause;
- alter pitch/timbre;

without invalidating Systean.

However, normative parsing must not derive a different literal proposition solely from prosody.

If an emotional attitude is semantically important, the speaker expresses it using language.

This gives Systean three channels:

```text
literal semantic channel     -> deterministic meaning
explicit expressive channel  -> emotions/attitudes
free prosodic channel         -> human performance, non-semantic by default
```

---

## 19. Speech acts

Systean must distinguish the proposition from what the speaker is doing with it.

At minimum the design should account for acts such as:

- assertion;
- question;
- command;
- request;
- definition;
- promise/commitment;
- hypothesis;
- wish/desire.

A question such as "Can you close the door?" must not rely on pragmatic convention to mean a request if its literal form asks about ability. A request should have a request form.

The exact inventory and whether some acts are decomposable into more general mechanisms remains open.

---

## 20. Numbers and formal notation

Numbers should be a dedicated formal subsystem, not thousands of ordinary dictionary roots.

### 20.1 Distinct semantic objects

The implementation should distinguish at least where needed:

```text
Number(7)
Digit(7)
DigitSequence([7, 7, 7])
Quantity(value, unit)
```

This prevents natural-language ambiguities such as "ten sevens".

For example:

```text
ten groups of Number(10) -> numeric multiplication -> 100
ten groups of Number(7)  -> numeric multiplication -> 70
```

while:

```text
ten copies of Digit(7) -> DigitSequence([7,7,7,7,7,7,7,7,7,7])
```

are different constructions.

### 20.2 Numerals may be written with digits

Canonical written Systean should permit compact conventional numeral notation such as:

```text
0
15
1489
1/3
10^6
12%
```

where the notation is formally specified.

There is no benefit in forcing every numeric value to be spelled out alphabetically.

### 20.3 Every notation needs a normative spoken form

A written numeral must have a systematic Systean pronunciation generated from numeric structure.

At least two readings may be useful as distinct semantic/surface modes:

- cardinal numeric reading of the value;
- digit-sequence reading when the digits themselves matter, e.g. codes, identifiers, phone numbers.

These must not be silently conflated.

### 20.4 Positional composition

The numeric system should be regular and capable of expressing arbitrary magnitudes without a separate lexical item for every scale.

Decimal can be the normal default while the engine remains capable of explicitly represented alternative radices if the specification chooses to support them.

### 20.5 Exact representation vs. normalized value

The analyzer may preserve both:

- the written representation;
- the normalized numeric value.

For example, rational and decimal representations may denote equal values while remaining different surface representations.

Measurement precision must not be inferred from formatting unless the measurement construction explicitly defines that semantics.

---

## 21. Quantities and measurement

Systean should structurally distinguish:

- count of discrete entities;
- amount of substance;
- physical quantity with a unit;
- collection/group;
- portion.

Units should integrate with the numeric subsystem rather than requiring arbitrary natural-language agreement rules.

Conceptually:

```text
Quantity {
    magnitude: 10
    unit: meter
}
```

Equivalent physical quantities may be normalizable while their original representation is preserved by the analyzer.

---

## 22. Time, aspect, and dates

The language should not assume that a three-way `past/present/future` prefix system is sufficient.

Temporal semantics may need to represent:

- event time;
- reference time;
- intervals;
- ordering (`before`, `after`, `during`);
- completion;
- continuation;
- beginning/end;
- habitual/repeated occurrence;
- deadlines/durations.

The preferred architecture is likely events/intervals plus semantic relations, with convenient surface constructions, rather than a growing list of unrelated tense prefixes.

Dates, clock times, and time zones should have formal notation and systematic spoken forms.

Contextual forms equivalent to `today`, `tomorrow`, and `now` are acceptable when their mapping to context is explicit and deterministic.

---

## 23. Vagueness, approximation, uncertainty, and generic claims

Systean cannot remain useful if it only allows mathematically exact claims.

It must provide explicit mechanisms for:

- approximation;
- ranges/tolerances;
- uncertainty;
- probability/confidence where useful;
- unknown values;
- unspecified values;
- generic/statistical statements;
- frequency/habituality.

The design must distinguish, for example:

- `x exists but I do not specify which x`;
- `I do not know x`;
- `x may not exist`;
- `any x is acceptable`;
- `x is intentionally hidden`.

Likewise, an equivalent of "birds fly" should not silently mean universal quantification if exceptions are intended. Generic statements need explicit generic/statistical semantics or more precise quantification.

---

## 24. Aspect, entailment, and hidden meaning

Systean must not smuggle semantic claims through pragmatic convention, and the parser must never validate claims against the world or against presumed speaker knowledge.

Aspectual operations such as equivalents of `start`, `cease/stop`, `continue`, `finish`, `interrupt`, and `repeat` apply to an **explicit semantic target**. The surrounding expression determines whether that target is a concrete process involving a particular object, a state, a repeated/habitual activity, or another explicitly constructed target. The aspect operator must not guess that distinction from context.

For example, the equivalents of:

- John stopped smoking this particular cigarette now;
- John stopped smoking habitually / altogether;

must differ in the structure of the target to which cessation applies. They must not be two hidden senses of one `stop smoking` lexeme. Likewise, stopping a habitual activity does not automatically assert that the activity can never happen again in the future.

Formal semantic definitions may have logical consequences, but those consequences belong to downstream reasoning. The parser constructs the stated semantic structure; it does not check whether the prerequisite state really existed, whether the statement is true, or whether the speaker knows it.

Detailed rules and examples are specified in [`SEMANTICS.md`](SEMANTICS.md).

---

## 25. Local definitions and compositional naming

Because the lexicon is finite but human concepts are open-ended, Systean should permit a speaker to construct a complex concept and then refer to it compactly within a discourse.

Conceptually:

```text
define local concept X := <complex description>
```

The surface syntax must be human-friendly, not code-like.

This mechanism is important because new dictionary roots are conveniences/compression, not prerequisites for expressing new ideas.

---

## 26. Language revisions

Systean should be explicitly versioned.

### 26.1 Compatible revisions

Within a compatible revision line, the lexicon/grammar should evolve conservatively:

- new roots may be added;
- new non-conflicting constructions may be added;
- old canonical roots do not silently change meaning;
- a retired form is not reused for a different meaning.

### 26.2 Breaking revisions

A major revision may intentionally change deeper language behavior if necessary. Text whose exact machine interpretation matters should be associated with the language revision under which it was authored.

### 26.3 No special backward-compatibility grammar

A person who has not learned the current revision may simply fail to understand a new root or construction. This is not an ambiguity defect in the current language and does not require special compatibility syntax.

---

## 27. Analyzer, generator, and documentation

A central product goal is not merely parsing but **explainable parsing**.

### 27.1 Word analysis

Given one word, the system should be able to show:

```text
surface form
  -> phonemes
  -> syllables
  -> morphemes
  -> root
  -> grammatical features/class
  -> semantic contribution
```

For every component, documentation should be able to display:

- form;
- pronunciation;
- category;
- meaning;
- applicability/constraints;
- composition rule;
- source rule in the language specification.

Generated inflectional variants should not require separate hand-written dictionary entries when their meaning follows compositionally from the rules.

### 27.2 Sentence/text analysis

Given a sentence or text, the analyzer should expose both structural views:

```text
text
  -> sentences
  -> scopes/constructions
  -> phrases
  -> words
  -> morphemes
```

and:

```text
surface text
  -> syntax tree
  -> semantic term
  -> bindings/references
  -> speech act / explicit expressive content
```

### 27.3 Provenance

Every transformation should retain provenance.

The system should be able to explain why a phrase means what it means, e.g. conceptually:

```text
1. token X matched rule A
2. morpheme Y contributed feature B
3. construction C bound argument D to role E
4. scope rule F produced semantic operator G
5. reference H resolved uniquely to discourse entity I
```

Explanations should be generated from the same rules used by the parser, not fabricated independently.

### 27.4 Analyzer and generator share the specification

Parsing and generation must use the same language package so they cannot drift into separate informal implementations.

---

## 28. Language compiler and automated self-verification

The language package should be **compiled and validated**, not merely loaded as trusted config.

A Systean revision should fail compilation when it creates an ambiguity or violates an invariant detectable by tooling.

Potential checks include:

### 28.1 Orthography/phonology

- duplicate grapheme/phoneme mappings where uniqueness is required;
- illegal phonotactic outputs;
- unrecoverable word boundaries;
- root/marker collisions;
- dangerously small phonological distance where configured.

### 28.2 Morphology

Reject if two different underlying analyses generate the same normative surface word:

```text
generate(A) == generate(B) && A != B
```

### 28.3 Syntax

Reject grammar conflicts where one normative token sequence yields multiple syntax trees.

### 28.4 Semantics

Reject constructions that elaborate one syntax tree into multiple normative semantic interpretations.

### 28.5 Round-trip/property tests

Generate large numbers of valid structures and test:

```text
parse(linearize(ast)) == ast
analyze(generate(form)) == form
```

Fuzzing/property-based testing should become part of the language-development workflow.

---

## 29. Heuristic tooling is allowed — outside the normative parser

The deterministic language must not be made inconvenient merely because useful editor tooling can be probabilistic.

An editor may:

- suggest spelling corrections;
- suggest a proper-name adaptation from a foreign spelling;
- search dictionaries;
- detect likely source language;
- suggest a newer canonical root;
- suggest explicit disambiguation when the user's attempted form is invalid.

But these are suggestions.

The normative parser must never silently accept an ambiguous/unknown input by guessing what the user probably meant.

---

## 30. Implications for the current repository

The current repository is a useful prototype, but the following files encode assumptions that are no longer design targets.

### `language/alphabet.toml`

Normative. The alphabet and its pronunciation mapping are complete and are not a redesign target. Syllabification, lexical-root stress, round-trip transcription, and validation are implemented by `systean-core` using this file plus `language/phonology.toml`.

### `language/dictionary.toml`

Normative v2 lexical inventory. Each root has one canonical human-readable `definition`; POS-specific contextual meanings are rejected by `LanguagePackage`. Formal semantic typing remains separate from dictionary prose.

### `language/legacy/grammar.toml`

The current `allowed_next` finite-state SVO model should be replaced by a recursive declarative grammar/semantic composition system.

It cannot be the long-term representation for nested clauses, quantifier scope, discourse, coordination, or general composition.

### `language/legacy/morphology.toml`

The current model should be redesigned rather than extended.

Specific decisions that conflict with the design baseline:

- single-consonant prefix stacking;
- uniform prefix ordering for semantically different scopes;
- `su` as a generic he/she/it referent;
- automatic word-class semantics implied by current suffix descriptions.

The final-vowel class-marking idea is rejected for the normative baseline. Surface class/POS information is not pronounced when it is already recoverable from lexical semantics and syntax.

### `site/`

The site can eventually become the primary interactive specification browser/analyzer:

- alphabet/phonology exploration;
- dictionary concepts;
- generated word decomposition;
- sentence parsing;
- semantic trees;
- rule provenance;
- language revision documentation.

---

## 31. Decisions currently considered settled

These should not be casually reopened while implementing unrelated features.

1. Systean remains literal and unambiguous by design.
2. The architecture is config-driven with a generic engine and a Systean language package.
3. The normative parser is deterministic and never relies on LLM/probabilistic intent guessing.
4. The current `allowed_next` grammar model is not the future grammar architecture.
5. Single-consonant grammatical prefix stacking is rejected.
6. Wide-scope semantics should not be hidden inside a uniform word-prefix chain.
7. Generic `ROOT -> related adjective/adverb` derivation is rejected.
8. Ordinary roots have one lexical concept; polysemy is not selected from context.
9. Systean does not model arbitrary world knowledge in lexical entries.
10. Type checking validates semantic structure, not truth/plausibility.
11. Parser/elaborator validity is independent of world state and speaker knowledge; the parser never checks whether a claim is true, plausible, known, or physically possible.
12. Aspectual operations apply to explicitly constructed targets; they never guess whether the speaker means a concrete event/process, state, habitual activity, or generic event class.
13. Proper names form one productive marked class; native and foreign proper names are not grammatically distinguished.
14. Proper-name status must be audible and visible, not capitalization-only.
15. The parser never guesses the language/pronunciation of foreign spelling.
16. Productive marked loanwords are rejected; new common roots enter through language revisions.
17. External/foreign text is explicitly quoted/opaque rather than silently parsed as Systean.
18. Metaphorical normative semantics is rejected.
19. Opaque idiomatic normative semantics is rejected.
20. Sarcasm is not a normative semantic inversion mechanism.
21. Emotions and expressiveness are explicitly supported through semantic/expressive constructions and interjections.
22. Prosody may be emotionally free but does not silently change literal meaning.
23. Numbers are a dedicated formal subsystem with distinctions such as number vs. digit vs. digit sequence.
24. Numerals may use compact digit notation and must have systematic spoken forms.
25. References must resolve uniquely or be made explicit.
26. Unknown/unspecified information must be represented explicitly.
27. New concepts remain expressible compositionally even before receiving a concise dictionary root.
28. Language revisions may add vocabulary; failure to understand a newer revision is not a grammar problem.
29. Analyzer/generator/documentation should derive from the same executable specification and preserve provenance.
30. Morphology v1 is bare-root identity: `WORD = ROOT`.
31. Mandatory POS/class endings are rejected.
32. Negation, number, tense, aspect, agreement, case, and other wide/nonlocal grammar are not encoded in a default lexical affix stack.
33. The canonical dictionary stores one lexical definition per root and rejects POS-specific contextual meanings.

---

## 32. Major unresolved design questions

These are intentionally left open. They should be solved before prematurely freezing concrete grammar.

### Phonology

The alphabet/pronunciation inventory and first-root-syllable stress are settled and implemented. Remaining questions are limited to mechanisms that are not yet required by ordinary bare roots:

- adaptation procedure for proper names;
- whether future bound morphology requires additional morpheme-boundary phonotactic constraints;
- whether any deterministic allomorphy is ever desirable.

### Morphology

The v1 baseline is settled as bare-root identity and is implemented in `language/morphology.toml` / `systean-core`. Remaining morphology questions are intentionally demand-driven:

- exact form of proper-name marking;
- whether any future strictly local derivation is useful enough to justify bound morphology;
- phonological realization rules for such a derivation if one is introduced.

No general derivation DSL should be invented before a concrete semantic requirement exists.

### Syntax

The structural v1 baseline is now implemented in `language/syntax.toml` and `systean-core`: canonical primary-participant/predicate/rest frame order, no free unmarked reordering, `ki ... ku` grouping, quantifier scope by appearance, `AND > OR`, explicit speech-act constructions, and semantic-type-driven rather than POS-driven parsing.

Remaining syntax questions are narrower:

- manually chosen surface roots/particles for the implemented construction classes;
- clause/reference boundary rules and discourse references;
- explicit inverse-quantifier-scope syntax once reference binding has a surface form;
- focus/topic realization;
- quotation boundaries in speech and writing.

### Semantic calculus

- exact generic term representation;
- exact type system strength;
- binder representation;
- event/time model;
- exact representation of concrete processes, states, and habitual/repeated activity targets;
- exact aspectual signatures for start/cease/continue/finish/interrupt/repeat;
- quantifier inventory;
- generic/statistical statement model;
- uncertainty/approximation model;
- exact boundary between literal semantic structure, canonicalization, and downstream logical entailment;
- speech-act representation.

### Lexicon

The canonical dictionary format is settled as one human-readable `definition` per manually authored root. Remaining questions are:

- bootstrap strategy for primitive concepts;
- criteria for assigning new roots;
- deprecation rules and revision metadata.

Roots are authored manually; tooling validates legality and collisions but does not generate vocabulary.

### Conversation / discourse

- exact discourse-reference mechanism;
- shorthand reference rules;
- local definitions and lifetime/scope;
- emotion/interjection targeting;
- explicit repair/correction constructions in spoken conversation.

### Numerals and formal notation

- exact spoken names/forms for digits;
- cardinal composition algorithm;
- scale/exponent realization;
- supported numeric notations;
- exact treatment of precision/significant digits;
- unit system integration.

---


### Current engine phase

The Rust workspace implements executable semantic, phonological, and morphology-v1 foundations behind one `LanguagePackage`. Native CLI and browser WASM consumers both use `systean-core`; there is no independent TypeScript language implementation.

The current executable boundaries are:

```text
semantic spec DSL
    -> Chumsky parser
    -> configured Environment
    -> parsed semantic expression
    -> canonical named-role Term
    -> type checker

alphabet.toml + phonology.toml
    -> validated orthography/phoneme mapping
    -> pronunciation / reverse spelling
    -> syllabification
    -> root-aware stress
    -> manual-root validation / spoken segmentation checks

dictionary.toml + morphology.toml
    -> one lexical definition per root
    -> bare-root morphological analysis/generation
    -> root boundary
    -> phonological word analysis
```

See `SEMANTICS.md`, `PHONOLOGY.md`, `MORPHOLOGY.md`, `ENGINE_ARCHITECTURE.md`, and `../README.md` for the implemented subset and test commands.

## 33. Recommended next design order

Future work should not begin by inventing more vocabulary or individual grammar particles.

A safer order is:

1. keep extending the formal semantic model only where regression cases require it;
2. treat the existing alphabet and implemented phonology baseline as fixed input to surface design;
3. keep the implemented bare-root morphology invariant and extend it only for concrete local derivations;
4. keep the implemented recursive surface syntax/scope engine stable while manually assigning surface vocabulary;
5. define reference/discourse rules;
6. define numbers/quantities/time as structured subsystems;
7. define proper names and external quotation;
8. define speech acts and expressive/emotional constructions;
9. build the language compiler's whole-language ambiguity/collision checks;
10. begin constructing actual Systean surface grammar;
11. expose all layers through the analyzer/site.

The purpose of this order is to prevent the project from returning to the original failure mode: inventing surface grammar before the semantic and architectural constraints are stable.

---

## 34. Design test for every future feature

Every proposed Systean feature should be challenged with the following questions:

1. What exact semantic structure does it contribute?
2. Can the same surface form produce another structure?
3. Can another underlying structure produce the same normative surface form?
4. Is its scope explicit or uniquely recoverable?
5. Is it equally identifiable in speech and writing?
6. Does it require contextual guessing rather than explicit context dependence?
7. Does it introduce polysemy, metaphor, idiom, hidden implicature, or world knowledge?
8. Can it be implemented through the language package rather than Systean-specific engine code?
9. Can the analyzer explain exactly why the feature has this meaning?
10. Can the language compiler automatically test at least part of its uniqueness guarantees?
11. Is the resulting spoken language still comfortable enough for actual human conversation?

A feature that fails these questions should be redesigned before entering the language specification.

---

## 35. Short project definition

Systean is a versioned, config-defined, human-spoken artificial language whose normative written and spoken forms map deterministically to explicit compositional semantics. It rejects hidden lexical/syntactic ambiguity, metaphorical and idiomatic normative meaning, and probabilistic interpretation, while preserving human usability through pronounceable morphology, concise constructions, explicit emotion, normal prosody, names, discourse, quantities, and structured uncertainty.

The engine should make the specification executable: the same package drives parsing, generation, validation, explanation, documentation, and automated ambiguity testing.
