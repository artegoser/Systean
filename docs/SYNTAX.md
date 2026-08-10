# Systean Surface Syntax and Scope

Status: **structural v1 implemented; core grammatical/operator vocabulary fixed**

This document defines the current normative structural rules for Systean surface syntax. The executable policy lives in `language/syntax.toml`; the generic parser/generator/lowering engine lives in `systean-core::syntax`.

The syntax engine does **not** invent grammatical roots. The following core forms were manually selected and are now normative:

| Form | Function | Semantic binding |
| --- | --- | --- |
| `ki` | open explicit scope | structural delimiter |
| `ku` | close explicit scope | structural delimiter |
| `ke` | truth-question marker | `ask_truth` |
| `ne` | negation | `not` |
| `va` | conjunction | `and` |
| `zo` | disjunction | `or` |
| `ra` | universal quantifier | `forall` |
| `mu` | existential quantifier | `exists` |
| `da` | command marker | `command` |
| `me` | request marker | `request` |

`ke`, `ne`, `va`, `zo`, `ra`, `mu`, `da`, and `me` are declared lexical roots as well as syntax bindings, so the ordinary word analyzer and root inventory recognize them. `ki` and `ku` are reserved structural delimiters rather than lexical roots.

Test fixtures still provide temporary content vocabulary such as people, predicates, and classes. Those fixture-only forms must never be treated as normative Systean vocabulary.

## 1. Core rule

Surface syntax exists to expose one semantic structure without making ordinary speech sound like serialized IR.

The default strategy is therefore:

> **Use canonical order when it uniquely determines structure. Add an explicit marker only when structure would otherwise differ.**

The parser never chooses a structure from plausibility, world knowledge, or statistical preference.

## 2. Canonical frame order

The default clause order is:

```text
PRIMARY PARTICIPANT -> PREDICATE -> REMAINING FRAME ARGUMENTS
```

A predicate's surface binding declares the exact semantic role of the primary participant and the exact ordered list of remaining roles.

Conceptually:

```text
John see Mary
```

may lower to:

```text
see(observer = John, observed = Mary)
```

The engine does not infer those roles from the nouns. The syntax binding declares them.

Free unmarked word-order permutation is disabled. A future focus/topic construction may mark information structure explicitly, but it must not silently reinterpret arbitrary word order.

## 3. No traditional POS grammar

The surface grammar does not depend on mandatory `noun`, `verb`, or `adjective` endings.

Lexical roots remain bare under morphology v1. Surface bindings are selected by semantic function/type and construction, not by a final-vowel POS code.

## 4. Scope markers

Systean retains the existing universal spoken/written delimiters:

```text
ki ... ku
```

They are semantic grouping markers: the spoken equivalent of explicit structural grouping.

They are reserved and may not simultaneously be lexical roots. The language-package loader checks this.

Grouping is not emitted merely for decoration. The canonical generator removes redundant grouping and inserts `ki ... ku` when it is required to preserve the intended syntax tree.

## 5. Local scope from order

When order already yields exactly one structure, extra delimiters are unnecessary.

The chosen rule for negation and quantifiers is equivalent to:

```text
NOT EVERY person clause
```

having a different structural order from:

```text
EVERY person NOT clause
```

The first introduced scope-bearing construction is wider unless explicit grouping establishes another structure.

The parser therefore preserves distinctions equivalent to:

```text
not(forall(...))
```

and:

```text
forall(... not(...))
```

without requiring spoken brackets around every operator.

## 6. Quantifier scope

Default quantifier nesting follows order of appearance.

Conceptually:

```text
every person see some dog
```

has default nesting equivalent to:

```text
forall person:
    exists dog:
        see(person, dog)
```

The syntax engine implements this by introducing binders in surface order and lowering them in reverse construction order around the base predicate.

The exact surface mechanism for intentionally reversing quantifier scope is not frozen yet because it depends on the still-unresolved surface reference/binder system. `ki` / `ku` provide universal proposition grouping, but the engine must not invent an implicit way to refer back to a bound entity.

## 7. AND / OR precedence

Systean intentionally has one logical precedence rule:

> **AND binds more tightly than OR.**

The canonical policy assigns a higher precedence to semantic `and` than semantic `or`.

Therefore structures equivalent to:

```text
A and B or C
```

parse as:

```text
(A and B) or C
```

and:

```text
A or B and C
```

parse as:

```text
A or (B and C)
```

To request the opposite grouping, use `ki ... ku`.

Chains of the same surface operator are flattened in the surface AST:

```text
A and B and C
```

becomes one deterministic `and` chain rather than requiring arbitrary spoken binary brackets. The current binary semantic IR lowers such a chain deterministically; the surface structure remains n-ary.

## 8. Explicit speech acts

Questions, commands, and requests are explicit constructions. They are not encoded by inversion or predicate inflection.

The syntax engine supports configurable prefix speech-act bindings that lower to semantic operators over the proposition they contain.

`ke`, `da`, and `me` are the normative markers for truth questions, commands, and requests respectively.

## 9. Semantic roles

Role markers are not pronounced by default when canonical frame order already identifies each role uniquely.

This does not make roles implicit in semantics. The parser's construction rule maps each surface position to one named semantic role.

A future marked construction may expose roles explicitly if it enables a useful operation such as focus without ambiguity. There is no free-order fallback in v1.

## 10. Argument omission

The accepted design rule is:

> An argument may be omitted only if the formal discourse/reference state yields exactly one valid referent.

The current parser does not yet have the discourse state necessary to prove that condition, so v1 takes the conservative implementation: required frame arguments must be present.

It must never weaken this to ordinary pragmatic guessing.

## 11. Focus and modifiers

Unmarked word order cannot be rearranged for focus.

A future focus construction must be explicit and must preserve the canonical underlying frame-role mapping.

Likewise, relations/modifiers must structurally identify their target. Systean will not adopt natural-language-style ambiguous attachment such as an unmarked phrase that could modify either an entity or an event.

## 12. Config-driven surface bindings

The generic engine supports declarative surface lexeme kinds:

- `atom` -> semantic constant;
- `class` -> unary restriction predicate used by quantifiers;
- `predicate` -> semantic operator plus primary/rest frame roles;
- `prefix` -> scope-bearing unary semantic operator;
- `infix` -> binary semantic operator with configured precedence;
- `quantifier` -> binder operator plus explicit restriction composition;
- `speech_act` -> explicit operator over content.

These are generic surface mechanisms. Concrete Systean words belong in the language package when manually chosen.

A predicate binding conceptually declares information equivalent to:

```toml
kind = "predicate"
semantic = "see"
primary_role = "observer"
rest_roles = ["observed"]
```

The engine then builds named-role semantic IR. No world-knowledge role inference is involved.

## 13. Parser and generator invariants

The surface subsystem targets:

```text
parse(linearize(ast)) = ast
```

for the supported canonical surface AST.

The regression suite covers:

- canonical primary/predicate/rest frame order;
- rejection of unmarked reordered arguments;
- `not every` versus `every ... not`;
- quantifier nesting by order of appearance;
- `AND > OR` precedence;
- `ki ... ku` precedence override;
- removal of redundant grouping;
- insertion of grouping when required;
- same-operator chain flattening;
- explicit question/command/request constructions;
- surface-to-semantic type checking.

## 14. Current intentionally unresolved items

Structural syntax is now executable and its core logic/quantifier/speech-act particles are normative. These items remain unresolved:

1. manually chosen content predicates/classes and any future focus particle;
2. surface reference/discourse syntax;
3. explicit inverse-quantifier-scope syntax that uses those references without hidden binding;
4. proper-name syntax;
5. quotation and nested quotation boundaries;
6. actual focus/topic operator semantics;
7. discourse-proven safe argument omission.

None of these gaps permit heuristic parsing in the meantime.
