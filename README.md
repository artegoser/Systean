# Systean

Unambiguous artificial language.

The repository now contains the first Rust implementation of the semantic foundation described in `docs/DESIGN.md` and `docs/SEMANTICS.md`.

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

This DSL is **not Systean surface syntax**. It is an implementation/specification language used to build and test the canonical semantic layer before phonology, morphology, and human-facing grammar are frozen.

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

1. grow the semantic regression corpus as new edge cases are discovered;
2. extend the package/rule schema only where those semantic tests require it;
3. finish still-open semantic areas in `docs/SEMANTICS.md`, especially temporal intervals, discourse/reference state, and richer affect/utterance structures;
4. design phonology and phonotactics;
5. design morphology and deterministic surface grammar parsing/generation over the semantic layer.

### Prototype DSL limitations

The current semantic DSL intentionally stays small. It does not yet provide comments, escaped string literals, modules/imports, source-level provenance spans in public IR, or recovery-oriented diagnostics. These are specification-tooling tasks, not Systean language semantics, and can be added without changing the semantic model.
