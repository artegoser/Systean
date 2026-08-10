# Systean Semantic Architecture

Status: **normative semantic design baseline**  
Purpose: define what Systean means by "meaning", what the parser/semantic elaborator is responsible for, and the generic semantic machinery on which the future surface language should be built.

This document specializes `DESIGN.md`. If an older semantic example in `DESIGN.md` conflicts with this document, this document takes precedence for semantic architecture.

The examples below use a deliberately code-like notation only to expose structure. They are **not Systean surface syntax**. The implemented structural surface baseline is documented separately in [`SYNTAX.md`](SYNTAX.md).

---

## Implementation status

The executable implementation of this document lives in `crates/systean-core`.

Current modules:

```text
crates/systean-core/src/semantics/
  term.rs
  types.rs
  signature.rs
  environment.rs
  checker.rs
  normalize.rs
  origin.rs
  explain.rs

crates/systean-core/src/spec/
  ast.rs
  parser.rs
  compile.rs
  package.rs

language/semantics/
  core.semsys

tests/fixtures/semantics/
  corpus.semsys
  demo.semsys
```

The semantic specification/expression parser is implemented with `chumsky = 0.13.0` (pinned exactly in `Cargo.toml`). This parser is for the **specification DSL**, not the future spoken/written Systean surface grammar.

The prototype makes one additional baseline choice that should be preserved unless tests show a reason to change it: semantic operator calls accept named roles only. Argument order therefore carries no meaning, and calls are lowered into a role-keyed canonical map.

For example, these are the same semantic call:

```text
smoke(agent = john, object = cigarette_x)
smoke(object = cigarette_x, agent = john)
```

The implementation now includes representation-level canonicalization: bound variables are alpha-renamed deterministically, while role maps and record fields use deterministic key ordering. Core quantification is package-defined (`forall`, `exists`, `exactly`, `at_least`, and `at_most`) rather than represented by Rust enums. This is intentionally not a theorem prover: algebraic or logical rewrites such as commutativity are not assumed by the generic engine. Generic type constructors declare their arity explicitly, are invariant unless a future specification mechanism declares variance, and subtype cycles are rejected during specification compilation. Multiple `.semsys` files can be compiled as one semantic package, and definitions retain file/declaration provenance for the semantic explainer. The package-level core distinguishes configurable `Occurrence`, `Event`, `Process`, `State`, and `Activity` types without hardcoding them in Rust. Temporal interval semantics, discourse state, affect structures, logical inference, and the final richer event/activity model remain design work.

---

## 1. Semantic goal

A valid Systean expression must determine one compositional semantic structure without requiring the listener or parser to guess:

- lexical sense;
- syntactic attachment;
- scope;
- argument role;
- referent;
- whether an expression denotes a concrete event, a repeated activity, a state, a class, a quantity, a digit sequence, or another semantic object;
- whether missing information is unknown, unspecified, existentially quantified, context-bound, or intentionally withheld.

The semantic system exists to answer:

> **What exactly did this utterance state, ask, request, define, or express?**

It does **not** answer:

> Is it true? Is it plausible? Does the speaker know it? Does the real world satisfy it?

This distinction is fundamental.

---

## 2. The parser never validates the world

The normative Systean parser and semantic elaborator validate **form and semantic structure**, not reality.

They must not attempt to determine:

- whether an assertion is true;
- whether the described event actually happened;
- whether an entity exists in the real world;
- whether the speaker has evidence for the assertion;
- whether the speaker knows what they claim;
- whether an action is physically possible;
- whether a statement is socially reasonable;
- whether a semantic combination is common or strange in the real world.

For example, all of the following may be structurally valid even if false, bizarre, or impossible:

```text
SUN ceased_to_exist five_minutes_before_now
STONE loves DEMOCRACY
PERSON crossed OCEAN in one_second
```

A separate optional theorem prover, knowledge base, fact checker, or world model may reason about such claims later. That subsystem must not be required to determine the normative parse.

### 2.1 Required invariant

> **Parser validity is independent of world-state validity.**

A sentence cannot become syntactically or semantically invalid merely because external knowledge contradicts it.

---

## 3. No hidden "speaker knowledge" requirements

A lexical item or construction must never be legal only if the parser can somehow know that the speaker possesses prerequisite knowledge.

For example, the parser must never apply a rule such as:

```text
if speaker previously knows that John smoked:
    allow STOP_SMOKING
else:
    reject
```

This is impossible to verify from the language itself and violates the architecture.

If Systean needs to communicate epistemic state, that state is expressed as ordinary semantic content:

```text
KNOW(speaker, P)
NOT(KNOW(speaker, P))
UNCERTAIN(speaker, P)
WITHHOLD(speaker, X)
```

The parser determines the structure of these claims. It does not prove them.

---

## 4. Semantic representation layers

The semantic pipeline should conceptually distinguish at least:

```text
surface text / speech
        ↓
syntactic structure
        ↓
semantic elaboration
        ↓
canonical semantic term
        ↓
optional normalization / logical derivation
```

The first four stages belong to the language implementation.

The last stage may derive consequences, normalize arithmetic, expand definitions, or perform other reasoning, but those derived results must not be confused with the literal canonical structure that was expressed.

### 4.1 Literal structure vs. consequences

For example, if a formally defined `cease` relation logically entails that a targeted process no longer continues after a boundary, a reasoner may derive that consequence.

The parser does not need to insert every theorem that follows from the expression into the surface parse.

This distinction prevents the semantic tree from becoming an infinite closure of all logically implied facts.

---

## 5. Generic engine calculus

The engine should contain a small generic typed term calculus rather than hardcoded Systean semantic categories.

The exact implementation syntax remains open, but the minimum conceptual machinery should be close to:

```text
Const(id)
Var(id)
Literal(value)
Call(function, arguments)
Bind(variable, body)
Record(fields)
Field(record, field)
```

Potential implementation equivalents are acceptable if they preserve the same capabilities.

### 5.1 `Const`

References a declared semantic symbol.

```text
Const(PERSON)
Const(LOVE)
Const(FORALL)
```

The engine does not know what those symbols mean. Their identity, signatures, and language-level semantics come from the language package.

### 5.2 `Var`

References a bound semantic variable.

```text
Var(x)
```

### 5.3 `Literal`

Represents values that have a direct formal representation rather than a dictionary root.

Examples may include:

```text
Literal(Number(10))
Literal(String("abc"))
Literal(Date(...))
```

The exact literal families are implementation-level design questions. Numeric structure is discussed separately below.

### 5.4 `Call`

Applies a typed symbol/function to arguments.

```text
Call(RED, [x])
Call(AGENT, [event, john])
```

### 5.5 `Bind`

Introduces scope-bound variables.

This is required for quantification, local abstraction, and other scoped constructions.

Conceptually:

```text
Bind(x, body)
```

The engine must prevent accidental capture and preserve binder identity independently of surface variable names.

### 5.6 `Record`

Represents structured semantic values with named fields where this improves clarity and composability.

This is particularly useful for:

- utterances;
- named semantic roles;
- quantities;
- contextual objects;
- structured literals;
- metadata that is itself semantic content.

The implementation may compile records into another internal representation, but configuration should not require humans to remember positional argument numbers for complex structures.

---

## 6. Systean-specific semantics are not engine enums

The generic engine must not hardcode a closed enum such as:

```text
Tense
Emotion
Question
Negation
Comparison
Person
Verb
Plural
Future
```

Instead, the Systean language package declares symbols and types that can represent these ideas.

For example, `NOT` may have a signature equivalent to:

```text
NOT : Proposition -> Proposition
```

but `NOT` itself is a language-package symbol.

The same principle applies to:

- temporal relations;
- quantifiers;
- aspectual operations;
- speech acts;
- affect/emotion;
- comparison;
- causation;
- evidentiality, if later added.

A normal addition to Systean semantics should not require editing a Rust enum.

---

## 7. Minimal type system

The semantic system needs typing to prevent category mistakes, but typing must stay much smaller than an ontology of the world.

The engine should support mechanisms roughly equivalent to:

```text
NamedType
FunctionType
RecordType
GenericType
TypeVariable
Subtype relation
```

The Systean package may declare broad semantic types such as:

```text
Entity
Proposition
Event
Process
State
Time
Interval
Number
Digit
DigitSequence
Quantity
Utterance
```

This list is not frozen. Some items may later be represented through more generic structures.

### 7.1 Type system purpose

Typing exists to catch structural mistakes such as applying a proposition-only operator to a digit sequence.

Typing does not exist to determine that stones cannot love, that stars are hot, or that humans cannot run at the speed of light.

### 7.2 No implicit semantic coercions

Systean should avoid implicit conversions that introduce meaning.

For example, the engine must not silently convert:

```text
Person -> Group<Person>
Number -> Quantity
Digit -> Number
Entity -> Proposition
```

when the conversion changes semantic content.

A safe subtype relation may be implicit where it does not add information:

```text
SpecificType <: GeneralType
```

Everything else should be represented by an explicit construction/operator.

---

## 8. Lexical concept is not semantic type

Systean must keep lexical identity separate from type classification.

A word meaning `person`, `star`, or `book` does not require the engine to create a distinct runtime type for every lexical category.

A useful default representation is to treat many entity concepts as predicates over a broad entity domain:

```text
PERSON : Entity -> Proposition
STAR   : Entity -> Proposition
BOOK   : Entity -> Proposition
```

Then:

```text
PERSON(john)
```

states that the referent `john` satisfies the lexical concept `PERSON`.

This prevents the type hierarchy from becoming:

```text
SiberianCat <: Cat <: Felid <: Mammal <: ...
```

unless a grammatical mechanism genuinely requires such distinctions.

### 8.1 Properties can use the same compositional principle

A property may similarly be represented as a predicate or value-producing function according to its semantics.

For example:

```text
RED    : Entity -> Proposition
MASS   : Entity -> Quantity<Mass>
HEIGHT : Entity -> Quantity<Length>
```

The correct signature belongs to the concept definition, not to a hardcoded "adjective" category.

---

## 9. Predicates, relations, and roles

Predicates and relations need explicit signatures because argument roles are part of meaning.

Configuration should prefer named roles over opaque positional argument lists.

Instead of only:

```text
GIVE(Entity, Entity, Entity)
```

specification should be able to declare something conceptually like:

```text
GIVE_EVENT
roles:
    agent
    theme
    recipient
```

The exact frame system remains open, but the following invariant is settled:

> **Argument role must never be inferred from world plausibility.**

If a surface construction places `recipient` before `theme`, the grammar knows that from its formal rule. It does not guess from the nouns involved.

---

## 10. Events, processes, states, and activities are first-class semantic targets

Natural-language verbs frequently hide distinctions between:

- a concrete bounded event;
- an ongoing process;
- a state;
- a repeated activity;
- a habit/disposition;
- a generic event class.

Systean cannot rely on context to choose between them.

The semantic architecture must therefore make these targets explicit enough that aspectual operations can apply to the intended object.

The precise type hierarchy remains open, but the system must support at least the distinction between:

```text
concrete occurrence
ongoing process/state
repeated/habitual activity pattern
```

without requiring a new lexeme for every combination.

---

## 11. Event semantics

A strong default model is to represent concrete happenings using event variables plus explicit relations.

For example, the semantic equivalent of:

> John gave Mary a book.

may elaborate conceptually to:

```text
exists e:Event {
    GIVE(e)
    AND AGENT(e, john)
    AND THEME(e, book)
    AND RECIPIENT(e, mary)
}
```

Additional information composes onto the same event:

```text
TIME(e, yesterday)
LOCATION(e, house)
MANNER(e, ...)
```

This is preferable to creating a different lexical predicate for every possible combination of arguments and modifiers.

### 11.1 Surface brevity remains mandatory

The user should not need to speak event-variable boilerplate.

The lexicon and grammar may define compact frames that elaborate into event semantics automatically.

The semantic representation is an implementation/debugging layer, not the desired spoken style.

---

## 12. Aspectual operations: start, stop, continue, finish, interrupt, repeat

Aspect must be compositional.

Systean should not create opaque lexemes such as:

```text
STOP_SMOKING_FOREVER
STOP_SMOKING_CURRENT_CIGARETTE
STOP_READING_BOOKS_HABITUALLY
```

Instead, general aspectual operators apply to an **explicitly specified semantic target**.

Conceptually, operations may include meanings equivalent to:

```text
START(target, boundary/time)
CEASE(target, boundary/time)
CONTINUE(target, interval)
FINISH(target, boundary/time)
INTERRUPT(target, boundary/time)
REPEAT(target, count/pattern)
```

The exact symbols and signatures are language-package decisions.

### 12.1 Critical rule

> **The aspect operator does not decide what kind of activity the speaker meant. The surrounding expression identifies the target.**

This is one of the central semantic rules of Systean.

---

## 13. Example: stopping one concrete smoking process

The intended meaning:

> John stopped smoking this particular cigarette now.

must identify a concrete ongoing smoking process and the concrete cigarette.

A possible canonical structure is conceptually:

```text
exists e:Process {
    SMOKE(e)
    AND AGENT(e, john)
    AND THEME(e, cigarette_x)
    AND CEASE(e, now)
}
```

The exact representation of `CEASE` may later use a separate transition event or interval boundary. What matters is that:

- `e` is a concrete process;
- `cigarette_x` is the specified object;
- the cessation applies to `e`;
- `now` identifies the boundary/context time.

The parser does not need to know whether John really smoked the cigarette.

---

## 14. Example: stopping smoking as a habitual/repeated activity

The intended meaning:

> John stopped smoking altogether / stopped being a smoker.

is not the same semantic target as one cigarette-smoking process.

The language must explicitly construct something like a habitual/repeated activity pattern:

```text
habitual_activity h {
    kind = SMOKE
    agent = john
    object_class = CIGARETTE
}

CEASE(h, now)
```

or an equivalent formal structure.

The important point is not this pseudocode. The important point is that **habituality/repetition is expressed by the language**, not inferred by `CEASE` from the absence of a cigarette object.

### 14.1 It does not mean "John can never smoke again"

Cessation of a habit at time `t` does not automatically assert:

```text
for every future time t2:
    John never smokes at t2
```

If that stronger future claim is intended, Systean must express it separately.

This prevents ordinary aspect from silently acquiring stronger modal/future semantics.

---

## 15. Example: reading

These meanings must remain structurally different:

### 15.1 Stop the current reading of a particular book

```text
concrete reading process
agent = John
object = book_x
CEASE(process, now)
```

### 15.2 Stop habitually reading books

```text
habitual/repeated activity
kind = READ
agent = John
object_class = BOOK
CEASE(activity, now)
```

### 15.3 Finish the book

This is also different from merely stopping the current reading process.

`FINISH(reading_of_book_x)` may encode completion of the relevant bounded activity, whereas `CEASE(reading_of_book_x)` only marks cessation.

Therefore `finish` and `stop` must not be interchangeable through contextual inference.

---

## 16. No special presupposition checker

Systean does not need a parser subsystem that checks whether presuppositions are true.

The older idea "you may use stop only if the speaker knows the activity occurred before" is explicitly rejected.

Instead:

1. a lexical/operator definition has one formal meaning;
2. that meaning may have logical consequences;
3. the parser constructs the meaning;
4. an optional reasoner may derive consequences;
5. no parser step verifies that the world satisfies those consequences.

### 16.1 Presupposition vs. entailment should not be a source of hidden parser behavior

If Systean chooses to preserve a formal distinction between assertion, presupposition, background condition, or other discourse status later, that distinction must itself be explicit in the semantic representation.

There must never be an invisible human-pragmatic presupposition that the parser "just knows".

---

## 17. Quantification is explicit scope

Quantification must be represented through binders/operators with exact scope.

For example:

> Every person read some book.

and:

> There is one book that every person read.

must produce different semantic structures.

Conceptually:

```text
FORALL x {
    PERSON(x) ->
    EXISTS y {
        BOOK(y) AND READ_RELATION(x, y)
    }
}
```

versus:

```text
EXISTS y {
    BOOK(y) AND
    FORALL x {
        PERSON(x) -> READ_RELATION(x, y)
    }
}
```

The surface language must preserve this distinction without requiring contextual disambiguation.

---

## 18. Negation and other scope-bearing operators

Negation, modality, probability, temporal operators, logical coordination, and similar operators must all have explicit scope.

These are different:

```text
NOT(FORALL x: P(x))
FORALL x: NOT(P(x))
```

The generator must not produce the same normative surface form for them.

If two different canonical semantic structures linearize to the same normative form, the language revision is invalid.

---

## 19. Named roles and surface word order

Surface word order may be optimized for human use, but semantic roles must come from grammar rules rather than semantic guessing.

For an event with roles such as:

```text
agent
theme
recipient
source
destination
```

the grammar may realize them through:

- fixed positions;
- particles;
- morphology;
- a combination of these.

The implementation choice is open.

The semantic result is not open: each realized phrase must map to one role.

---

## 20. Collective and distributive meaning

Natural-language plural statements often hide whether an action is collective or distributive.

Systean must distinguish them explicitly.

For example:

> Three people lifted a table.

may denote:

```text
COLLECTIVE(group_of_three, lift_one_table)
```

or:

```text
FOR_EACH(member_of_group_of_three, lift_a_table)
```

The language must encode the intended structure rather than use plausibility to infer it.

---

## 21. Numbers, digits, and quantities are semantically distinct

The semantic layer must not collapse all numeric-looking forms into one undifferentiated `number` concept.

At minimum, Systean should be able to distinguish:

```text
Number(7)
Digit(7)
DigitSequence([7, 7, 7])
Quantity(Number(7), unit)
Ratio(...)
```

This allows precise constructions such as:

```text
10 × Number(7) = Number(70)
```

versus:

```text
repeat(Digit(7), 10)
= DigitSequence([7,7,7,7,7,7,7,7,7,7])
```

Thus the natural-language ambiguity of "ten sevens" is eliminated by semantic type and construction.

The exact numeral syntax and spoken realization belong to the numeral specification, not this document.

---

## 22. Proposition and utterance are different

A proposition describes content.

An utterance describes what a speaker does with content.

The semantic architecture should support a structure equivalent to:

```text
Utterance {
    act: ASSERT
    content: P
}
```

versus:

```text
Utterance {
    act: ASK
    content: P
}
```

or:

```text
Utterance {
    act: REQUEST
    content: desired_action
}
```

`ASSERT`, `ASK`, `REQUEST`, and other acts should be package-defined semantic symbols/constructions, not necessarily engine enums.

This prevents pragmatic reinterpretation such as an ability question automatically becoming a request.

---

## 23. Emotion belongs to explicit utterance semantics

Systean should preserve emotional communication without allowing emotion or prosody to secretly change proposition meaning.

A semantically expressed affect may conceptually attach to an utterance/content:

```text
Utterance {
    act: ASSERT
    content: P
    affect: JOY(target=P, intensity=high)
}
```

The surface form should be short and human, likely using regular expressive particles/interjections.

### 23.1 Prosody is not canonical semantic rewriting

A speaker may shout, whisper, laugh, stretch vowels, or vary pitch.

These may carry natural human expression, but the normative parser does not infer a different proposition solely from prosody.

If an emotion or attitude is semantically important, Systean provides an explicit construction for it.

---

## 24. Metaphor, idiom, and sarcasm

Normative Systean literal semantics rejects:

- implicit metaphor;
- opaque idiomatic meaning;
- sarcasm as a mechanism that tells the listener to replace the literal proposition with another inferred proposition.

If a speaker intends a concrete property, relation, comparison dimension, or emotion, they express that content.

This does not prohibit humor, expressive performance, surprising literal statements, or explicit comparisons.

---

## 25. References and discourse state

Semantic interpretation extends across sentences.

The discourse model must be able to track referents of different semantic kinds, including:

```text
entity
group
event/process
proposition
utterance
locally defined concept
```

A shorthand reference is permitted only when its resolution under the formal discourse rules is unique.

### 25.1 Unique-resolution rule

> **Implicit/shorthand reference is legal only when exactly one valid referent remains.**

If multiple candidates remain, the parser rejects that shorthand and requires a more explicit expression.

### 25.2 The parser still does not use world plausibility

If two people are candidates for a pronoun-like reference, the parser may not select one because "that person is more likely to perform the action".

---

## 26. Context-bound values

Context dependence is allowed when the dependency itself is explicit and deterministic.

Examples may include:

```text
context.speaker
context.addressee
context.time
context.location
```

Thus equivalents of `I`, `you`, `now`, and `here` do not inherently violate unambiguity.

The semantic term contains a context lookup with one defined interpretation.

---

## 27. Unknown, unspecified, existential, and withheld are different

Systean must not introduce one universal `UNKNOWN` operator for all incomplete information.

These are semantically different:

```text
there exists an x, identity unspecified
speaker does not know identity(x)
speaker knows identity(x) but withholds it
x may not exist
any x satisfying condition is acceptable
```

The language should construct each meaning from explicit semantic mechanisms.

This preserves the principle:

> Missing information is allowed; hidden reason for missing information is not.

---

## 28. Approximation and contextual standards

Vague and approximate statements are useful and must be expressible, but the source of vagueness should remain visible in semantics.

For example, a precise approximation may encode:

```text
APPROX(value=10, tolerance=2)
```

A deliberately context-dependent approximation may instead encode:

```text
APPROX(value=10, tolerance=context.standard_for_this_use)
```

These are not the same claim.

Likewise, a gradable property such as height may reference an explicit or context-bound comparison standard.

The parser must not invent a standard; the language either supplies one directly or explicitly selects a contextual standard mechanism.

---

## 29. Generic and statistical claims

A sentence equivalent to:

> Birds fly.

must not automatically become universal quantification if exceptions are intended.

Systean should distinguish constructions such as:

```text
FORALL
MOST
TYPICALLY
frequency/probability statement
```

according to the exact semantic model eventually chosen.

The `TYPICALLY`/generic model remains unresolved, but "generic because that sounds natural" is not acceptable.

---

## 30. No semantic overload by contextual sense selection

One symbol/root must not denote unrelated operations selected by argument context merely because a natural language reuses the same word.

For example, if natural-language `open` means materially different relations in:

```text
open a door
open a file
open a discussion
```

Systean should use distinct lexical concepts unless there is a genuine single semantic abstraction that applies without loss.

Parametric polymorphism is acceptable where the semantic operation is genuinely one operation:

```text
EQUAL<T>(T, T)
```

This is not lexical polysemy.

---

## 31. Definitions, aliases, and local concepts

A speaker must be able to define a complex semantic concept and bind a compact local reference to it.

Conceptually:

```text
define local X := complex_semantic_expression
```

This allows discussion of new concepts before the official lexicon assigns them a root.

A local alias must have explicit scope/lifetime and must not collide ambiguously with ordinary lexical roots or proper names.

---

## 32. Config-driven symbol declarations

The language package should be able to declare semantic symbols without engine changes.

Conceptually:

```text
symbol NOT
signature: Proposition -> Proposition

symbol AGENT
signature: Event × Entity -> Proposition

symbol CEASE
signature: ProcessLike × TimeBoundary -> Proposition
```

The exact specification DSL is unresolved.

### 32.1 Prefer a semantic DSL over deeply nested TOML trees

TOML is suitable for metadata and simple declarations, but complex compositional semantics will become unreadable if represented only as deeply nested tables.

A small declarative specification DSL is likely appropriate for:

- signatures;
- semantic templates;
- binders;
- frame elaboration;
- grammar-to-semantics mapping;
- constraints.

This DSL is an implementation/specification language, not Systean surface syntax.

---

## 33. Semantic elaboration provenance

Every semantic node produced from surface Systean should retain enough provenance to explain why it exists.

The analyzer should be able to answer:

- which token/morpheme/construction produced this node;
- which grammar rule assigned this argument role;
- which lexical declaration supplied this symbol;
- where this binder was introduced;
- why this reference resolved to this discourse object;
- which normalization is literal and which result is derived.

For example:

```text
surface construction
    ↓ rule R17
GIVE event frame
    ↓ role mapping
AGENT(e, x)
THEME(e, y)
RECIPIENT(e, z)
```

This makes the executable specification auditable rather than opaque.

---

## 34. Canonicalization and reasoning boundary

The implementation should distinguish several operations that are easy to conflate:

### Parsing

```text
surface -> syntax
```

### Semantic elaboration

```text
syntax -> canonical semantic term
```

### Canonicalization

Safe representation normalization that does not change meaning, for example binder renaming or canonical record field ordering.

### Evaluation/normalization

For explicitly mathematical structures, this may calculate:

```text
10 × 10 -> 100
```

while preserving source representation provenance.

### Logical inference

Derives consequences from semantic definitions and axioms.

This is optional and downstream.

### World validation

Compares claims with external facts/models.

This is completely outside normative parsing.

---

## 35. Semantic compiler invariants

A Systean revision should fail semantic compilation if any of the following can occur for normative forms:

1. one syntax tree elaborates into multiple competing canonical semantic terms;
2. argument roles can be assigned in multiple ways;
3. a shorthand reference has several valid formal resolutions but remains accepted;
4. scope is selected by heuristics rather than grammar;
5. a semantic coercion silently inserts nontrivial meaning;
6. two distinct canonical meanings are forced into one normative surface form without an explicit underspecification mechanism;
7. lexical sense is selected from context among unrelated alternatives;
8. an aspectual operator must guess whether its target is a concrete process, habit, state, or event class;
9. a parser rule relies on external world knowledge or speaker knowledge to choose a meaning.

---

## 36. Semantic regression and property-based tests

The executable implementation includes everyday, adversarial ambiguity, and canonical-equivalence corpora under `tests/corpus/`.

Property-based generation remains a later layer. It should generate semantic structures and verify round trips such as:

```text
parse(generate(term)) == term
```

up to defined canonical equivalences such as alpha-renaming of bound variables.

Test generation should intentionally stress:

- nested quantifiers;
- negation scope;
- collective vs. distributive readings;
- repeated/habitual vs. concrete events;
- start/stop/continue/finish distinctions;
- discourse reference collisions;
- number vs. digit vs. digit-sequence constructions;
- context-bound vs. explicit values;
- emotion attached to different targets;
- nested utterances/quotations;
- local definitions.

---

## 37. Everyday semantic-density constraint

The semantic representation may be verbose internally. The surface language must not expose that verbosity unnecessarily.

The grammar should provide compact deterministic constructions for common structures such as:

```text
I am hungry.
Where are you?
Come home.
I will arrive in ten minutes.
I do not know.
I like this very much.
We did it!
John stopped smoking this cigarette.
John stopped smoking habitually.
```

The purpose of the semantic calculus is to make these expressions precise, not to force speakers to verbalize a logic AST.

This should be measured using an everyday-language corpus as described in `DESIGN.md`.

---

## 38. Decisions settled by this document

The following semantic decisions should be treated as baseline unless a later dedicated design revision explicitly replaces them:

1. The parser validates language structure, not truth or world state.
2. The parser does not validate what the speaker knows.
3. Systean-specific semantic concepts are package-defined, not hardcoded engine enums.
4. Canonical semantics uses a small generic typed compositional calculus.
5. Lexical identity is separate from semantic type.
6. Most ordinary entity concepts should not require dedicated engine/runtime types.
7. Nontrivial implicit semantic coercions are rejected.
8. Context-selected lexical polysemy is rejected.
9. Events/processes/states/activities must be representable as explicit semantic targets.
10. Aspectual operations such as start/cease/continue/finish apply to an explicitly constructed target.
11. `stop`/`cease` never guesses whether the target is a current event, a particular-object process, a habit, or a generic activity.
12. Habituality/repetition is represented by the surrounding semantic construction, not encoded as a guessed special sense of `stop`.
13. Stopping a habit does not automatically assert that the activity can never occur again in the future.
14. No parser-side presupposition/world-knowledge checker is required.
15. Logical consequences of formal semantics belong to downstream reasoning, not parse validation.
16. Quantifier and negation scope are explicit.
17. Semantic roles are explicit and never inferred from plausibility.
18. Collective and distributive readings are distinct.
19. Number, digit, digit sequence, and quantity are semantically distinct.
20. Proposition and utterance are distinct.
21. Explicit emotion may attach to utterance/content; prosody does not silently rewrite literal semantics.
22. Metaphor, opaque idiom, and sarcasm-as-semantic-inversion are rejected in normative semantics.
23. Discourse shorthand resolves uniquely or is rejected.
24. Unknown, unspecified, existential, and withheld information are not collapsed into one operator.
25. Context dependence is allowed only through explicit deterministic context mechanisms.
26. Parser/elaborator provenance is a first-class implementation requirement.

---

## 39. Remaining semantic questions

The architecture is now constrained, but several concrete choices remain open:

- refinement of the current Rust term model after corpus testing;
- whether `Record`/`Field` are primitive IR nodes or compilation conveniences;
- exact type-system strength and subtyping rules;
- exact representation of events vs. processes vs. states;
- whether activity patterns/habits are a dedicated semantic type or a compositional operator over event predicates;
- exact aspectual signatures for `start`, `cease`, `continue`, `finish`, `interrupt`, and `repeat`;
- exact temporal interval model;
- exact generic/statistical semantics (`most`, `typically`, etc.);
- exact approximation and contextual-standard model;
- exact speech-act inventory;
- exact emotion/affect model and targeting;
- exact discourse-state and local-reference representation;
- refinement of the current semantic specification DSL (syntax, diagnostics, modules, provenance);
- which normalization rules are guaranteed by the core engine versus declared by packages.

These should be solved with semantic test cases before concrete surface grammar is frozen.

---

## 40. Semantic design rule for future proposals

For every proposed lexeme, operator, grammatical construction, or shorthand, ask:

1. What exact semantic object does it construct?
2. What are its argument roles and types?
3. What is its scope?
4. Does it require the parser to infer a missing relation?
5. Does it require world knowledge to choose a meaning?
6. Does it require knowledge of the speaker's beliefs?
7. Does it silently switch between concrete event, habit, state, class, or generic meaning?
8. Can its meaning be obtained compositionally from declared symbols/rules?
9. Can the analyzer explain every semantic contribution from provenance?
10. Can a different semantic structure produce the same normative surface form?

If questions 4, 5, 6, 7, or 10 expose hidden guessing, the proposal is incompatible with the current Systean goal.
