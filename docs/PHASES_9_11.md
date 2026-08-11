# Systean Phases 9–11 — Aspect, Information Status, and Generalization

Status: implemented; dedicated runtime validation is pending on the author toolchain.

This document records the concrete language-package behavior implemented for roadmap Phases 9, 10, and 11. The implementation preserves the frozen alphabet, bare-root morphology, dictionary-owned lexical source of truth, typed semantic IR, and deterministic discourse rules.

## 1. Phase 9 — explicit occurrences and aspect

Aspect operators apply to an explicit semantic occurrence. They never ask the parser to infer whether an event happened, whether a process is active, or what the speaker believes about it.

The semantic package provides explicit reification constructors:

```text
event_of(content: Proposition) -> Event
process_of(content: Proposition) -> Process
state_of(content: Proposition) -> State
activity_of(content: Proposition) -> Activity
```

These are semantic construction patterns, not newly invented lexical roots. A concrete language value can be introduced or locally defined and then referenced by the normal typed discourse machinery.

The selected surface roots are:

```text
sta   start one Occurrence
stop  cease one Occurrence
dur   continue one Occurrence
fin   finish one Occurrence
rup   interrupt one Occurrence
reg   construct habitual Activity from one Occurrence
rep   construct repeated Activity from one Occurrence plus explicit Number count
```

`stop proc` and `stop reg proc` are therefore different terms. Stopping a habitual activity does not imply that no future occurrence can happen. `fin proc` is also different from `stop proc`: completion and cessation remain separate operators.

`rep` always carries an explicit numeric count. There is no implicit "again", default count, world-state test, or previous-event lookup.

## 2. Phase 10 — explicit incomplete information

Three statuses are first-class typed slot values:

```text
unk <knower>  value unknown to the explicit/context-bound knower
vak           value explicitly left unspecified
hid           value intentionally withheld
```

They are intrinsic typed-slot forms in `dictionary.toml`, not overloaded ordinary operators. Their concrete semantic type is taken from the argument slot in which they occur. A standalone `vak`, `hid`, or `unk ...` therefore has no type to guess and is rejected.

`unk` declares its knower type as `Entity`. The knower must be either a compatible declared context value such as `tu`, or a marked proper name such as `na mari`. A context value of another type, for example `nau : Instant`, cannot silently function as a knower.

Existential quantification remains separate: `mu` introduces a quantified entity; it is not an alias for missing, unknown, unspecified, or withheld information.

### Approximate numbers

The existing selected form `apro` now also realizes `Approximate<Number>` directly:

```text
~10                  <-> apro dek uno
10±0.5               <-> apro ki dek uno ku ki nul dot pent ku
```

Exact `10 : Number` and approximate `~10 : Approximate<Number>` are type-distinct. A negative tolerance is rejected. Approximate quantities continue to use the same explicit approximation marker without being confused with approximate bare numbers.

Context-sensitive lexical relations continue to require their declared standards. No comparison class or missing parameter is supplied from world knowledge.

## 3. Phase 11 — cardinal, collection, logical, typical, and statistical claims

### Counted quantifiers

The accepted cardinal roots are implemented as counted quantifiers:

```text
rov N CLASS BODY   exactly N
mini N CLASS BODY  at least N
maks N CLASS BODY  at most N
```

The count is an explicit `Number` structured literal. The class restriction and quantified body are both structurally represented; no cardinal reading is inferred from pragmatics.

### Collections and interpretation

The selected collection roots preserve different identities:

```text
set   unordered set constructor
list  ordered sequence constructor
grup  collective group constructor
kol   collective interpretation of Proposition
dis   distributive interpretation of Proposition
aso   explicitly underspecified association
```

The current minimal collection constructors are binary and nest compositionally. They do not collapse set, sequence, and group identity into a single generic container.

`aso` means only that an association is asserted. The parser does not refine it to ownership, location, causation, kinship, or another relation from context.

### Implication and counterfactual

```text
imp  material/logical implication
hip  counterfactual causal frame
```

These lower to distinct semantic operators and are not alternative spellings of one relation.

### Typicality, statistics, probability, and frequency

```text
tip   typical(domain, measure, standard)
stat  statistical(domain, measure, value)
prob  probability(claim, value)
frek  frequency(activity, measure, value)
```

Every construction exposes the parameters declared by its signature. `tip` is not an exception-tolerant universal; `stat` is not an alias for `tip`; probability requires an explicit proposition and numeric value; frequency requires an explicit `Activity`, measure, and numeric value.

### Majority remains compositional

There is no `most` root and no `most` semantic primitive. For a finite explicitly cardinalized domain, strict majority is expressed by literal cardinal facts. For example, a domain asserted to contain exactly five persons together with a claim that at least three persons satisfy a predicate encodes the majority condition without turning `tip` or `stat` into majority operators.

This is intentionally literal. The language does not infer a hidden domain size, threshold, or statistical interpretation. Future convenience syntax, if author-approved, must expand canonically to the same general cardinal machinery rather than introduce a second meaning.

## 4. Configuration ownership

All new lexical surface bindings live in `language/dictionary.toml`. `language/syntax.toml` only adds precedence for the semantic infix operators `implies` and `counterfactual` and the generic counted-quantifier surface form schema lives in the Rust engine.

The engine changes are generic:

- typed information markers declare status identity and optional knower type;
- counted quantifiers declare their semantic count/binder roles, binder type, and restriction composition;
- structured approximation remains a literal codec concern;
- no Rust branch checks for the literal Systean roots `unk`, `rov`, `tip`, and so on.

## 5. Validation suites

Dedicated tests cover:

- explicit Event/Process/State/Activity reification;
- start/continue/finish/cease/interrupt/habitual/repeat distinctions;
- explicit repetition counts;
- absence of world-state or speaker-knowledge checks;
- unknown/unspecified/withheld distinctions and typed knower validation;
- exact versus approximate numbers and explicit tolerance;
- counted quantifiers and their canonical semantics;
- set/list/group and collective/distributive distinctions;
- association without semantic guessing;
- implication versus counterfactual;
- typical versus statistical claims;
- explicit probability and frequency targets;
- compositional finite-domain majority without a `most` primitive;
- scripted CLI coverage across all three phases.
