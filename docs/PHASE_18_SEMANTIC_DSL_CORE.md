# Phase 18 — Typed semantic identity and DSL core migration

Status: **planned**
Architecture: [`SEMANTIC_DSL_ARCHITECTURE.md`](SEMANTIC_DSL_ARCHITECTURE.md)

## Goal

Replace the stringly semantic core and duplicated lexical/operator ownership with a compiled, typed, ID-based model while preserving the accepted Systean language behavior of the Phase 17 baseline.

This phase intentionally does **not** attempt the complete declarative surface/pragmatics migration. It establishes the new semantic foundation first so later migrations do not mix architecture changes with grammar changes.

## Scope

### DSL frontend

- [ ] Extend/rework `.semsys` syntax to support `word`, `primitive`, `def`, `intrinsic`, `data`, `context`, `dimension`, and `unit` declarations.
- [ ] Define one unambiguous grammar for generic parameters, typed function parameters, return types, algebraic constructors, definitions, and local lambdas.
- [ ] Make source parameter names authoring-only; compile them to declaration-local IDs/de Bruijn binders.
- [ ] Preserve useful source spans/provenance for every compiled declaration.
- [ ] Reject duplicate declarations and cyclic definitions where cycles are not explicitly legal.

### Compiled symbol/type model

- [ ] Introduce interned/resolved `SymbolId`, `TypeId`, `ConstructorId`, `ContextSlotId`, `DimensionId`, `UnitId`, and other IDs required by the accepted architecture.
- [ ] Replace semantic operator lookups by source `String` with resolved IDs in canonical IR.
- [ ] Replace runtime named-role maps with resolved parameter order/IDs while preserving names for diagnostics.
- [ ] Make alpha-equivalent source binders produce equal canonical terms.
- [ ] Make semantic equality independent of comments, glosses, formatting, and local parameter names.

### Lexical ownership

- [ ] Migrate ordinary lexical roots from `dictionary.toml` + duplicated semantic operator declaration to one `.semsys` declaration.
- [ ] Treat the Systean root itself as the lexical/surface identity of the declared word rather than routing it through an English semantic identifier.
- [ ] Remove mandatory `kind = operator` / `kind = information` / `kind = context` JSON-shaped semantic bindings.
- [ ] Derive ordinary default surface frames from word arity/types where the current grammar permits it.
- [ ] Preserve root legality, pronunciation, reserved-token, and collision validation.
- [ ] Keep manual root authoring; no allocator is introduced.

### Structured semantic values

- [ ] Replace `StructuredLiteral { family: String, canonical: String, ... }` with typed algebraic/scalar terms.
- [ ] Migrate information status away from `unknown:...`, `withheld`, and similar string encodings.
- [ ] Represent unknown/unspecified/withheld as typed structures.
- [ ] Make context-backed unknown values contain resolved context references rather than encoded strings.
- [ ] Migrate number/quantity/date/time/duration/interval canonical representations to typed terms without changing accepted Systean surface forms.

### Dimensions and units

- [ ] Compile dimensions and units to IDs.
- [ ] Remove runtime checks against source names such as `"time"` and `"second"`.
- [ ] Preserve exact rational conversion and dimension compatibility.
- [ ] Make base-unit relationships declarations rather than engine string conventions.

### Compatibility/fingerprints

- [ ] Split package identity into at least semantic and surface fingerprints.
- [ ] Alpha-renaming and source formatting changes must not change the semantic fingerprint.
- [ ] Documentation changes must not change semantic/surface fingerprints.
- [ ] Semantic signature/definition changes must change the semantic fingerprint.
- [ ] Surface spelling/form changes must change the surface fingerprint.
- [ ] Add migration diagnostics that map old lexical/operator declarations to new symbols during the transition.

## Migration strategy

1. Introduce the new compiled IDs and IR behind adapters while old package loading still works.
2. Compile a representative subset (`vid`, `per`, `viv`, `mi`, `tu`, `unk/vak/hid`, one unit family, one date/time value) through both paths.
3. Assert canonical behavior parity on the frozen compatibility corpus.
4. Migrate all lexical/structured-value declarations.
5. Delete old semantic string parsing only after the new path covers the complete corpus.
6. Keep the Phase 16 compiler as the release gate throughout; do not disable collision checks to simplify migration.

## Required tests

- [ ] Source symbol names resolve to stable IDs and no runtime semantic branch needs their strings.
- [ ] Renaming a local parameter/binder leaves canonical terms and semantic fingerprint unchanged.
- [ ] Changing an English/comment label leaves semantic/surface fingerprints unchanged.
- [ ] Changing a word signature changes semantic compatibility.
- [ ] Ordinary new lexical word requires one declaration and no Rust code.
- [ ] `hid`/`unk`/`vak` produce typed structures with no colon-delimited mini-language.
- [ ] Dates, durations, quantities, and intervals round-trip as typed values without parsing a canonical string in later semantic layers.
- [ ] Unit conversion contains no source-name special cases.
- [ ] Existing Phase 2–17 behavioral tests remain green or are migrated to equivalent structural assertions.
- [ ] Full compatibility corpus preserves Phase 17 parse/meaning unless an explicitly approved semantic migration says otherwise.

## Completion result

The semantic core is no longer dependent on English identifiers, duplicated dictionary/operator declarations, or opaque structured-value strings. The package has one typed semantic identity model ready for declarative grammar/effect migration.
