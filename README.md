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
- executable tests for scope and activity distinctions discussed in the design documents.

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
type Activity;
type Proposition;

literal integer: Number;

const john: Entity;

operator smoke(agent: Entity, object: Entity) -> Activity;
operator cease(target: Activity) -> Proposition;
operator equal<T>(left: $T, right: $T) -> Proposition;
```

Expressions use named roles:

```text
cease(target = smoke(agent = john, object = cigarette_x))
```

Named-role order is not semantically significant. The parser canonicalizes call arguments by role name.

## Repository direction

Next major stages are:

1. validate/refine the semantic IR against a larger adversarial and everyday corpus;
2. extend the package/rule schema only where semantic tests require it;
3. design phonology and phonotactics;
4. design morphology;
5. build deterministic surface grammar parsing/generation over the semantic layer.

### Prototype DSL limitations

The current semantic DSL intentionally stays small. It does not yet provide comments, escaped string literals, modules/imports, source-level provenance spans in public IR, or recovery-oriented diagnostics. These are specification-tooling tasks, not Systean language semantics, and can be added without changing the semantic model.
