# Systean Semantic Model and DSL — Pre-1.0 Target Architecture

Status: **accepted target architecture for the pre-1.0 migration**
Scope: semantic identity, lexical declarations, typed IR, surface realization, structured values, discourse effects, documentation metadata, package compilation, and compatibility.

This document supersedes the earlier assumption that `dictionary.toml` plus duplicated `.semsys` operator declarations are the final ownership model. The existing Phase 17 package remains the migration baseline until Phases 18–19 replace it.

The design is influenced by several established ideas rather than copied from one system:

- Grammatical Framework: keep abstract meaning structures separate from concrete surface realization, and make parsing/generation operate on the same abstract structures;
- Lean/Agda-style elaboration: source names are resolved to stable internal identities, and binder names are not semantic identity;
- K/Redex-style executable semantics: language-specific behavior is declared as rules/effects over a small generic runtime rather than as string-name branches in the engine;
- SKOS-style concept identity: human-language labels and explanations are metadata attached to a concept, not the identity of the concept;
- typed controlled-language systems such as ACE/MRS: discourse/scope structure remains explicit and is not collapsed into opaque strings.

The primary architectural objective is:

> Adding an ordinary Systean word must not require a Rust change, a second semantic declaration, or an English semantic identifier.

The secondary objective is:

> After package compilation, normative semantic evaluation must not depend on human-readable strings.

---

## 1. Core separation

Systean 1.0 distinguishes five independent concerns:

```text
semantic identity        what formal symbols/constructions exist
semantic composition     how defined symbols reduce to other semantic terms
surface realization      how Systean forms parse/linearize those terms
runtime intrinsics       generic computations the declarative language invokes
human documentation      English glosses/explanations/examples and UI copy
```

A change in one concern must not implicitly change another.

### 1.1 DSL source surface

The `.semsys` language should remain small and declaration-oriented. The accepted declaration families are conceptually:

```text
module systean.lexicon.core;
use core.*;
use logic.*;

type Entity;

data InfoMode { unknown($knower: Entity); unspecified; withheld; }

primitive logic.not($p: Prop) -> Prop;
intrinsic meta.context<T>($slot: ContextSlot<T>) -> T;
def xor($p: Prop, $q: Prop) -> Prop = ...;

context speaker : Entity;
dimension Time;
unit sek : Time;
unit milisek : Time = 1/1000 * sek;

word vid($observer: Entity, $observed: Entity) -> Prop;

word ne($p: Prop) -> Prop = logic.not($p) {
    form _ $p;
    precedence 80;
}

effect speech.assert($p) {
    commit $p;
}
```

Source conventions:

- `word` declares a Systean lexical root and its typed semantic value/function; a body after `=` formally defines/lower it through other terms; without a body it is primitive unless explicitly bound to an intrinsic declaration;
- `primitive` declares an abstract semantic primitive that need not itself be a Systean surface word;
- `def` declares a non-lexical formal definition;
- `intrinsic` declares a typed engine capability boundary;
- `data` declares algebraic semantic data;
- `context`, `dimension`, and `unit` declare typed package entities;
- `form` declares exceptional/non-default bidirectional Systean surface realization;
- `effect` declares generic discourse-state behavior attached to a typed semantic construction;
- local variables begin with `$` only for readability and are resolved away during compilation;
- `_` inside a `word` form denotes that word's lexical root;
- comments/documentation are not semantic identity.

The DSL should not grow a generic annotation/object syntax that recreates nested TOML/JSON. New declaration forms require a clear semantic/compiler concept, not arbitrary key-value bags.

Examples:

- changing an English explanation does not change semantic or surface compatibility;
- renaming a local DSL parameter does not change semantic compatibility;
- changing a Systean root changes surface compatibility but need not change semantic identity if done through an explicit migration/alias policy;
- changing `Entity` to a human-friendlier source name does not change compiled type identity if the declaration identity is preserved by the package migration;
- changing a semantic signature or definition changes semantic compatibility.

---

## 2. Three kinds of semantic declaration

Every semantic symbol is exactly one of the following.

### 2.1 Primitive

A primitive is a formally atomic concept inside the current Systean semantic ontology.

```text
word vid($observer: Entity, $observed: Entity) -> Prop;
```

`vid` does **not** mean the English string `see`. It denotes one package-owned primitive symbol with a typed signature.

A primitive may have English documentation such as `see` or `visual perception`, but those strings are documentation only.

Primitive does not mean philosophically irreducible. A later language revision may replace a primitive with a formal definition if that is an intentional semantic compatibility change.

### 2.2 Defined

A defined symbol has an executable formal definition in terms of other semantic terms.

```text
def xor($p: Prop, $q: Prop) -> Prop =
    logic.or(
        logic.and($p, logic.not($q)),
        logic.and(logic.not($p), $q)
    );
```

Its semantic meaning is its typed definition after elaboration/canonicalization, not its source identifier.

### 2.3 Intrinsic

An intrinsic is a typed boundary to generic engine behavior that cannot usefully be expressed as ordinary pure term rewriting in the package.

Examples include:

- exact rational arithmetic;
- structured date/time decoding and validation;
- exact unit conversion;
- explicit runtime context lookup;
- typed discourse reference resolution;
- discourse history mutation primitives;
- opaque quotation/name payload handling where host integration is required.

Intrinsics are declared in the package and implemented by stable engine capability IDs. Systean-specific roots never become Rust `match "english_name"` branches.

```text
intrinsic meta.context<T>($slot: ContextSlot<T>) -> T;
intrinsic meta.resolve<T>() -> T;
intrinsic quantity.convert<T>($value: Quantity<T>, $unit: Unit<T>) -> Quantity<T>;
```

---

## 3. Compiled identities

Source identifiers exist for authoring and diagnostics. Compiled identity is numeric/interned and package-scoped.

Target compiled IDs include:

```text
SymbolId
TypeId
ConstructorId
ParameterId / SlotId
ContextSlotId
EffectId
IntrinsicId
DimensionId
UnitId
SurfaceRuleId
DocumentationEntryId
```

Normative runtime code must not branch on source strings after compilation.

Forbidden target-state patterns include:

```rust
if unit.dimension == "time" { ... }
match function.as_str() { "ask" => ... }
canonical.split(':')
StructuredLiteral { family: "duration", canonical: "PT60S" }
```

Source names may remain in a symbol table for diagnostics, workbench output, generated documentation, and package diffs.

---

## 4. Canonical semantic IR

The target IR is typed and structural. A representative model is:

```rust
enum Term {
    Symbol(SymbolId),
    Apply {
        function: SymbolId,
        arguments: Vec<Term>,
    },
    Bound(u32),
    Lambda {
        parameter_type: TypeId,
        body: Box<Term>,
    },
    Data {
        constructor: ConstructorId,
        fields: Vec<Term>,
    },
    Scalar(Scalar),
    Hole {
        ty: TypeId,
        annotation: Box<Term>,
    },
}
```

Exact Rust representation may differ, but the following invariants are mandatory:

- semantic/operator identity is not a `String`;
- role/parameter identity is not a runtime `String` lookup;
- structured values are algebraic data, not `family + canonical-string` envelopes;
- binders canonicalize independently of source variable names;
- semantic equality does not depend on documentation labels;
- source provenance remains available separately from semantic identity.

### 4.1 Binder identity

Source binder names are ergonomic only:

```text
fn ($person: Entity) => ...
fn ($x: Entity) => ...
```

must compile to the same alpha-equivalent term when their bodies are otherwise identical.

The target internal representation should use de Bruijn indices or an equivalent resolved binder identity.

### 4.2 Arguments

Named arguments are permitted in source syntax where they improve authoring clarity, but compiled function application uses declaration order/resolved parameter IDs.

Renaming `$observer` to `$x` must not change semantic fingerprints.

---

## 5. Types and algebraic data

The DSL supports:

```text
type Entity;
type Prop;

data Option<T> { none; some($value: T); }
```

Package-defined algebraic data replaces opaque semantic string encodings.

For information status, for example:

```text
data InfoMode {
    unknown($knower: Entity);
    unspecified;
    withheld;
}

intrinsic meta.hole<T>($mode: InfoMode) -> T;
```

Then a withheld Entity is represented structurally as a typed `Hole<Entity>` annotated by `InfoMode.withheld`, not by:

```text
family = "information"
canonical = "withheld"
```

Unknown information tied to the speaker becomes a tree containing a resolved context value, not a string such as `unknown:context:speaker`.

---

## 6. Words are typed functions, not POS-tagged special cases

Ordinary lexical roots declare a typed semantic object and use a default surface frame derived from their arity.

```text
word sol : Entity;
word per($entity: Entity) -> Prop;
word viv($entity: Entity) -> Prop;
word vid($observer: Entity, $observed: Entity) -> Prop;
```

Default surface realization:

```text
0 args:  sol
1 arg:   $0 per
2 args:  $0 vid $1
3 args:  $0 root $1 $2
```

The exact default-frame rule remains package policy and is compiled once. An ordinary word should not repeat its signature in a second file or specify a `kind = "predicate"` tag merely to obtain this behavior.

A word may override the default only when its surface construction genuinely differs.

---

## 7. Declarative surface realization

Special surface behavior is declared against typed terms rather than represented by a growing Rust enum such as `Class`, `Predicate`, `Prefix`, `Infix`, `Quantifier`, or `SpeechAct`.

Representative syntax:

```text
word ne($p: Prop) -> Prop = logic.not($p) {
    form _ $p;
    precedence 80;
}

word va($a: Prop, $b: Prop) -> Prop = logic.and($a, $b) {
    form $a _ $b;
    precedence 50;
    associative;
}

word zo($a: Prop, $b: Prop) -> Prop = logic.or($a, $b) {
    form $a _ $b;
    precedence 40;
    associative;
}
```

`_` denotes the lexical root being declared.

The compiler derives both parser and canonical linearizer from the same `form` declaration. A surface form that cannot be inverted uniquely is a package compilation error.

### 7.1 Quantification as higher-order composition

Classes/properties become first-class predicate values:

```text
per : Entity -> Prop
viv : Entity -> Prop
```

Universal quantification can therefore be declared without a Systean-specific quantifier parser branch:

```text
word ra(
    $restriction: Entity -> Prop,
    $body: Entity -> Prop
) -> Prop =
    logic.all(
        fn ($x: Entity) =>
            logic.implies(
                $restriction($x),
                $body($x)
            )
    )
{
    form _ $restriction $body;
}
```

`ra per viv` then elaborates compositionally from typed function values.

The same principle should be used for counted quantifiers, aspect wrappers, focus/topic, and other constructions whenever they can be represented as ordinary typed composition.

---

## 8. Context, reference, omission, and aliases

Context slots are declarations with compiled IDs:

```text
context speaker : Entity;
context addressee : Entity;

word mi : Entity = meta.context(speaker);
word tu : Entity = meta.context(addressee);
```

Runtime context storage uses `ContextSlotId`, not human labels.

Generic typed reference is one intrinsic semantic request:

```text
word ref<T> : T = meta.resolve<T>();
```

Safe omitted arguments elaborate to the same typed reference request as explicit `ref`, while provenance records whether the request was explicit or omitted.

Aliases and lexical definitions bind resolved semantic values; their source names are scope-local presentation names, not global semantic identity.

---

## 9. Structured values

Numbers, dates, time, quantities, intervals, information states, and related values become typed terms/data.

Target examples:

```text
Date.date(2026, 8, 11)
Time.time(23, 15, 0)
Duration.seconds(60)
Exact.rational(1, 3)
```

A codec may recognize/print a compact written or spoken notation, but codecs exchange typed values with semantic IR.

They must not exchange opaque `canonical: String` payloads that later layers parse again.

A codec declaration owns:

- accepted surface notation;
- typed output/input;
- canonical rendering;
- collision namespace;
- validation rules that are intrinsic to the notation.

---

## 10. Dimensions and units

Dimensions and units have package identities:

```text
dimension Length;
dimension Time;

unit metr : Length;
unit kilometr : Length = 1000 * metr;

unit sek : Time;
unit milisek : Time = 1/1000 * sek;
unit hor : Time = 3600 * sek;
```

Codecs or semantic definitions refer to declared IDs:

```text
codec duration : Duration {
    dimension Time;
    base sek;
}
```

The engine must not know source strings such as `"time"` or `"second"`.

Exact conversions remain rational and dimension-checked.

---

## 11. Discourse and pragmatic effects

Language-specific communicative roots should not be encoded as a Rust enum keyed by Systean/English names.

The package declares typed semantic constructions and attaches generic discourse effects.

Representative model:

```text
primitive speech.assert($p: Prop) -> Utterance;
primitive speech.ask($p: Prop) -> Utterance;
primitive speech.retract($target: UtteranceRef) -> Utterance;

effect speech.assert($p) {
    commit $p;
}

effect speech.retract($target) {
    deactivate $target;
}
```

The runtime implements a small stable effect instruction set such as:

```text
Commit
Deactivate
Supersede
IntroduceReferent
RetireFrame
BindAlias
OpenScope
CloseScope
```

Exact instruction inventory is part of the Phase 19 design validation. The engine knows effect semantics; it does not know which Systean word expresses a question, correction, topic, or repair.

---

## 12. Human documentation is a separate layer

English documentation is mandatory for Systean 1.0 but is not semantic identity.

Documentation lives outside the normative semantic declarations, for example:

```text
language/docs/en.sydoc
```

Representative syntax:

```text
word vid {
    gloss "see";
    explain """
    Expresses visual perception by the first participant of the second participant.
    It does not imply recognition, attention, belief, or continued observation unless
    another construction states those meanings explicitly.
    """;

    example "mi vid tu" => "I see you.";
}
```

Every public Systean word must have at minimum:

- one short English gloss;
- one detailed English explanation;
- semantic signature shown by reference to the compiled declaration rather than duplicated manually;
- at least one canonical Systean example once the word is part of the public 1.0 vocabulary.

Operators/constructions additionally require an example that demonstrates scope or argument behavior where relevant.

Documentation compilation cross-validates that all referenced words exist and that all public words have required English documentation.

Documentation has a separate fingerprint and compatibility policy.

---

## 13. English as the fixed bridge language for 1.0

Systean 1.0 fixes **English** as the single normative human-facing bridge language for explanations and deterministic semantic rendering.

This does not make English part of Systean semantics.

The system distinguishes:

### 13.1 Short lexical gloss

Used for search results, hover cards, compact dictionary rows, and simple labels.

Examples:

```text
vid -> see
ra  -> every / all
hid -> withheld
```

A gloss is intentionally short and may be lossy. It is never used as the translation algorithm.

### 13.2 Controlled English rendering

A deterministic renderer consumes canonical semantic/discourse structures and emits explicit English that preserves Systean structure as closely as practical.

Example conceptually:

```text
ra per viv
```

may render as:

```text
Every person is alive.
```

while a more structurally unusual Systean expression may intentionally produce less idiomatic but more faithful English.

The controlled renderer is composition-driven. It must not translate an expression by concatenating per-word glosses.

### 13.3 Idiomatic paraphrase

Natural/idiomatic English paraphrasing may be offered later as a non-normative tool. It is not part of the 1.0 acceptance criteria and must never replace controlled rendering in tests or compatibility checks.

The decision to fix English first avoids pretending that arbitrary natural languages can share one trivial word-level template system. Future language renderers are separate modules over the same canonical semantics.

---

## 14. Fingerprints and compatibility

The package exposes independent fingerprints:

```text
semantic_fingerprint
surface_fingerprint
documentation_fingerprint
```

Optionally a separate runtime-capability fingerprint may be added if intrinsic/effect ABI evolution requires it.

### Semantic fingerprint excludes

- comments;
- English glosses/explanations;
- source formatting;
- local binder/parameter names after alpha-normalization.

### Semantic fingerprint includes

- symbol/type/data identities;
- signatures;
- definitions;
- subtype/data structure;
- intrinsic capability bindings;
- semantic/effect behavior that changes canonical meaning or discourse transition.

### Surface fingerprint includes

- Systean roots;
- surface forms;
- precedence/associativity/grouping rules;
- structured notation codecs;
- canonical spoken/written realization.

### Documentation fingerprint includes

- English glosses;
- explanations;
- examples and documentation metadata.

Compatibility tooling must explain which domain changed rather than returning one opaque package hash only.

---

## 15. Package layout target

The exact split may evolve during migration, but ownership should converge toward:

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

A future consolidation of `semantics/`, `lexicon/`, and `surface/` into fewer `.semsys` modules is allowed if it improves authoring without reintroducing duplicate ownership.

The following current files are migration sources, not desired permanent semantic ownership:

```text
dictionary.toml
syntax.toml semantic-form inventory
literals.toml semantic family strings
units.toml string-identified semantic dimensions
```

Data-only phonology/alphabet configuration may remain TOML because it is genuinely tabular configuration rather than a hidden programming language.

---

## 16. Compilation pipeline

The target package compiler performs:

```text
parse modules
  -> build declaration graph
  -> resolve source identifiers
  -> allocate/intern stable compiled IDs
  -> elaborate generic types and binders
  -> type-check definitions
  -> compile algebraic data
  -> compile surface rules bidirectionally
  -> compile codecs
  -> compile discourse effects
  -> compile English documentation references
  -> build immutable indexes
  -> run cross-layer collision/ambiguity checks
  -> run generated round-trip/property corpus
  -> compute separate fingerprints
  -> emit immutable LanguagePackage
```

Parser implementation order must never decide between two otherwise valid analyses.

---

## 17. Workbench/provenance contract

The workbench should expose both compiled identity and human presentation without confusing them.

For a token such as `vid`, reports should eventually contain structured fields conceptually equivalent to:

```text
surface root: vid
symbol: SymbolId(...)
kind: primitive word
signature: (Entity, Entity) -> Prop
argument 0: resolved current semantic participant
argument 1: resolved current semantic participant
canonical semantic contribution: Apply(SymbolId(...), [...])
English gloss: see
English explanation: ...
source provenance: lexicon/core.semsys:...
```

Raw IDs are diagnostic/developer data. User-facing website views translate them to names and explanations.

---

## 18. Explicit anti-goals

The migration must not create:

- a giant TOML schema that reimplements a programming language through nested objects;
- English concept names as hidden runtime semantics;
- one Rust enum variant per Systean grammatical construction;
- a second parser/generator in the website;
- word-level gloss concatenation marketed as translation;
- automatic root generation;
- probabilistic ambiguity resolution;
- implicit world knowledge in normative parsing/reference resolution;
- a requirement to edit Rust for ordinary lexical growth.

---

## 19. Definition of success

The new semantic/DSL architecture is complete when all of the following are true:

1. an ordinary new primitive word is declared exactly once in `.semsys` and requires no Rust change;
2. a new defined operator can be expressed through typed DSL composition without a Rust change;
3. only genuinely new generic runtime capabilities require Rust intrinsics/effect instructions;
4. canonical semantic IR contains no human semantic identity strings;
5. structured values contain no re-parsed `family/canonical` string encoding;
6. unit/dimension behavior is ID-based rather than English-name-based;
7. ordinary surface patterns are declarative and parser/generator derive from the same rule;
8. Systean-specific pragmatic roots are package declarations over generic effects;
9. every public word can be documented without changing its semantic fingerprint;
10. Phase 16 ambiguity/compatibility guarantees remain enforced by the new compiler.
