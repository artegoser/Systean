# Systean

Unambiguous artificial language.

The repository now contains executable semantic and phonological foundations described in `docs/DESIGN.md`, `docs/SEMANTICS.md`, and `docs/PHONOLOGY.md`.

## Current implementation

The root Rust crate is the new language engine foundation. The old TypeScript implementation under `lib/` is retained as a legacy prototype and has not been migrated yet.

Implemented now:

- generic semantic IR (`Const`, `Var`, `Literal`, `Call`, `Bind`, `Record`, `Field`);
- configurable named and generic type constructors with checked arity;
- acyclic named-type subtyping;
- function, generic, variable, and record types;
- configured literal typing;
- configured constants and operators;
- generic operator signatures such as `equal<T>`;
- named semantic roles only (no positional call arguments in the semantic DSL);
- stricter type checking, invariant generic constructors, record field checks, and function variance;
- representation-level canonicalization with deterministic alpha-renaming of binders;
- multi-file `.semsys` package compilation with definition provenance;
- a typed semantic explainer that reports roles, types, and definition origins;
- lexical/bound-name lowering;
- a small semantic specification DSL;
- a Chumsky `0.13.0` parser for the specification DSL and semantic expressions;
- data-driven everyday, ambiguity, and canonical-equivalence semantic regression corpora.

The phonology foundation now also implements:

- the existing `lib/config/alphabet.toml` as the single alphabet/pronunciation source of truth;
- config validation for a bijective, prefix-free spelling/pronunciation mapping;
- exact spelling -> pronunciation and pronunciation -> canonical spelling round trips;
- deterministic vowel-driven syllabification configured by `lib/config/phonology.toml`;
- lexical stress on the first syllable of an explicitly identified root;
- manual root validation with collisions as errors and similarity as advisory warnings;
- exact zero/one/multiple spoken-segmentation detection over supplied form inventories;
- CLI phonology analysis and root-checking commands;
- regression tests for orthographic round trips, stress, roots, and spoken segmentation.

This DSL is **not Systean surface syntax**. It is an implementation/specification language used to build and test the canonical semantic layer before morphology and human-facing grammar are frozen.

## Requirements

- A current stable Rust toolchain with Rust 2024 edition support
- Cargo

`chumsky` is pinned exactly to `0.13.0` in `Cargo.toml`.

## Run tests

```bash
cargo test
```

## Check and explain semantic packages

```bash
cargo run --bin systean -- check spec/semantics

cargo run --bin systean -- explain spec/semantics \
  'cease(target = smoke(agent = john, object = cigarette_x))'

cargo run --bin systean -- explain spec/semantics \
  'cease(target = habitual(activity = smoke_activity(agent = john, object_kind = cigarette_kind)))'
```

The explainer prints the inferred type, canonical IR, role tree, and `.semsys` definition provenance. `systean-sem` remains as a compatibility CLI and accepts either one `.semsys` file or a directory.


## Check and analyze phonology

```bash
cargo run --bin systean -- phonology check \
  lib/config/alphabet.toml lib/config/phonology.toml

cargo run --bin systean -- phonology pronounce \
  lib/config/alphabet.toml Systean

cargo run --bin systean -- phonology spell \
  lib/config/alphabet.toml sjstean

cargo run --bin systean -- phonology analyze \
  lib/config/alphabet.toml lib/config/phonology.toml nasol --root sol

cargo run --bin systean -- roots check \
  lib/config/alphabet.toml lib/config/phonology.toml lib/config/dictionary.toml sal

cargo run --bin systean -- roots audit \
  lib/config/alphabet.toml lib/config/phonology.toml lib/config/dictionary.toml
```

Roots remain manually authored. The checker validates a proposed root but never generates one.

## Semantic specification DSL (prototype)

```text
type Entity;
type Occurrence;
type Process;
subtype Process: Occurrence;
type Proposition;
type Number;

literal integer: Number;

const john: Entity;

operator smoke(agent: Entity, object: Entity) -> Process;
operator cease(target: Occurrence) -> Proposition;
operator equal<T>(left: $T, right: $T) -> Proposition;
```

Expressions use named roles:

```text
cease(target = smoke(agent = john, object = cigarette_x))
```

Named-role order is not semantically significant. The parser canonicalizes call arguments by role name.

## Semantic regression corpus

```text
tests/corpus/everyday.tsv
tests/corpus/ambiguity.tsv
tests/corpus/equivalence.tsv
```

The everyday corpus checks types, the ambiguity corpus checks that intended distinctions do not collapse, and the equivalence corpus checks representation-only differences such as binder names and named-role order. `spec/semantics/corpus.semsys` is testing vocabulary, not frozen surface-language vocabulary.

## Repository direction

Next major stages are:

1. grow semantic and phonological regression corpora as new edge cases are discovered;
2. extend the package/rule schema only where those tests require it;
3. design morphology as a reversible abstract-feature -> surface-realization layer over the existing semantic and phonological foundations;
4. prove unique morphological decomposition and use the spoken-segmentation checker on generated surface inventories;
5. design recursive deterministic surface grammar parsing/generation over the semantic layer;
6. finish discourse/reference, numeral/time, proper-name, and expressive subsystems as their surface realization is designed.

### Prototype DSL limitations

The current semantic DSL intentionally stays small. It does not yet provide comments, escaped string literals, modules/imports, source-level provenance spans in public IR, or recovery-oriented diagnostics. These are specification-tooling tasks, not Systean language semantics, and can be added without changing the semantic model.
