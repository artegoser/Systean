# Phase 18 — Typed semantic identity and DSL core migration

Status: **in progress — first half (18A typed core) implemented; runtime cutover and full value migration remain**
Architecture: [`SEMANTIC_DSL_ARCHITECTURE.md`](SEMANTIC_DSL_ARCHITECTURE.md)

## Goal

Replace the stringly semantic core and duplicated lexical/operator ownership with a compiled, typed, ID-based model while preserving the accepted Systean language behavior of the Phase 17 baseline.

This phase intentionally does **not** attempt the complete declarative surface/pragmatics migration. It establishes the new semantic foundation first so later migrations do not mix architecture changes with grammar changes.


## Phase 18A / 18B implementation boundary

Phase 18 is intentionally split at a compiler boundary rather than by raw checkbox count. This keeps the repository usable after the first half and avoids a half-migrated production semantic IR.

**Phase 18A — implemented in the first-half delivery:**

- a new typed `.semsys` frontend for `type`, `word`, `primitive`, `def`, `intrinsic`, `data`, `context`, `dimension`, and `unit`;
- deterministic compiled identities (`SymbolId`, `TypeId`, `ConstructorId`, `ContextSlotId`, `DimensionId`, `UnitId`, `IntrinsicId`, `FieldId`);
- canonical signatures that contain parameter types/order but not authoring parameter names;
- de Bruijn lowering for lexical parameters and local lambdas, including higher-order invocation;
- structural algebraic constructors, context references, scalar values, and exact unit-base relations;
- separate semantic and surface fingerprints;
- declaration provenance/source spans and debug-only source names;
- representative dual-path migration coverage for `vid`, `per`, `viv`, `mi`, `tu`, `unk`, `vak`, `hid`, an exact `Time` unit family, and one calendar date value;
- Phase 17 `LanguagePackage` remains the behavior oracle while the new compiler is exercised in parallel.

**Phase 18B — deliberately not performed yet:**

- production `LanguagePackage` cutover to the typed package as the single source of truth;
- migration of every lexical root out of `dictionary.toml` and duplicated legacy `.semsys` signatures;
- replacement of all production `StructuredLiteral` paths for numbers, quantities, dates, times, durations, and intervals;
- conversion runtime cutover from string unit/dimension identifiers to `UnitId`/`DimensionId`;
- deletion of old `unknown:...` parsing and other string semantic branches after corpus parity;
- full compatibility-corpus parity and author-toolchain validation before Phase 18 is marked complete.

The first-half compiler is therefore not allowed to silently become a second production semantic implementation. It is a migration compiler whose output must replace the legacy path only after 18B proves full parity.

## Scope

### DSL frontend

- [x] Extend/rework `.semsys` syntax to support `word`, `primitive`, `def`, `intrinsic`, `data`, `context`, `dimension`, and `unit` declarations.
- [x] Define one unambiguous grammar for generic parameters, typed function parameters, return types, algebraic constructors, definitions, and local lambdas.
- [x] Make source parameter names authoring-only; compile them to declaration-local IDs/de Bruijn binders.
- [x] Preserve useful source spans/provenance for every compiled declaration.
- [x] Reject duplicate declarations and cyclic definitions where cycles are not explicitly legal.

### Compiled symbol/type model

- [x] Introduce interned/resolved `SymbolId`, `TypeId`, `ConstructorId`, `ContextSlotId`, `DimensionId`, `UnitId`, and other IDs required by the accepted architecture.
- [x] Replace semantic operator lookups by source `String` with resolved IDs in the new canonical IR. Production cutover remains 18B.
- [ ] Replace runtime named-role maps with resolved parameter order/IDs while preserving names for diagnostics.
- [x] Make alpha-equivalent source binders produce equal canonical terms.
- [x] Make semantic equality independent of comments, glosses, formatting, and local parameter names.

### Lexical ownership

- [ ] Migrate ordinary lexical roots from `dictionary.toml` + duplicated semantic operator declaration to one `.semsys` declaration.
- [ ] Treat the Systean root itself as the lexical/surface identity of the declared word rather than routing it through an English semantic identifier.
- [ ] Remove mandatory `kind = operator` / `kind = information` / `kind = context` JSON-shaped semantic bindings.
- [x] Derive ordinary default surface frames from word arity/types where the current grammar permits it.
- [ ] Preserve root legality, pronunciation, reserved-token, and collision validation.
- [ ] Keep manual root authoring; no allocator is introduced.

### Structured semantic values

- [ ] Replace `StructuredLiteral { family: String, canonical: String, ... }` with typed algebraic/scalar terms.
- [ ] Migrate information status away from `unknown:...`, `withheld`, and similar string encodings.
- [ ] Represent unknown/unspecified/withheld as typed structures.
- [ ] Make context-backed unknown values contain resolved context references rather than encoded strings.
- [ ] Migrate number/quantity/date/time/duration/interval canonical representations to typed terms without changing accepted Systean surface forms.

### Dimensions and units

- [x] Compile dimensions and units to IDs in the new package model; production conversion cutover remains 18B.
- [ ] Remove runtime checks against source names such as `"time"` and `"second"`.
- [ ] Preserve exact rational conversion and dimension compatibility.
- [ ] Make base-unit relationships declarations rather than engine string conventions.

### Compatibility/fingerprints

- [x] Split package identity into at least semantic and surface fingerprints in the new package model.
- [x] Alpha-renaming and source formatting changes must not change the semantic fingerprint.
- [x] Documentation/comment changes must not change semantic/surface fingerprints.
- [x] Semantic signature/definition changes must change the semantic fingerprint.
- [x] Surface spelling/form changes must change the surface fingerprint.
- [ ] Add migration diagnostics that map old lexical/operator declarations to new symbols during the transition.

## Migration strategy

1. [x] Introduce the new compiled IDs and IR behind adapters while old package loading still works.
2. [x] Compile a representative subset (`vid`, `per`, `viv`, `mi`, `tu`, `unk/vak/hid`, one unit family, one date/time value) through both paths.
3. [ ] Assert canonical behavior parity on the frozen compatibility corpus. This is the 18B cutover gate; 18A only asserts representative parity while Phase 17 remains the oracle.
4. [ ] Migrate all lexical/structured-value declarations.
5. [ ] Delete old semantic string parsing only after the new path covers the complete corpus.
6. [x] Keep the Phase 16 compiler as the release gate throughout; do not disable collision checks to simplify migration.

## Required tests

- [ ] Source symbol names resolve to stable IDs and no runtime semantic branch needs their strings.
- [x] Renaming a local parameter/binder leaves canonical terms and semantic fingerprint unchanged.
- [x] Changing an English/comment label leaves semantic/surface fingerprints unchanged.
- [x] Changing a word signature changes semantic compatibility.
- [x] Ordinary new lexical word requires one declaration and no Rust code in the typed compiler.
- [x] `hid`/`unk`/`vak` produce typed structures with no colon-delimited mini-language in the typed compiler.
- [ ] Dates, durations, quantities, and intervals round-trip as typed values without parsing a canonical string in later semantic layers.
- [ ] Unit conversion contains no source-name special cases.
- [ ] Existing Phase 2–17 behavioral tests remain green or are migrated to equivalent structural assertions.
- [ ] Full compatibility corpus preserves Phase 17 parse/meaning unless an explicitly approved semantic migration says otherwise.

## Completion result

The semantic core is no longer dependent on English identifiers, duplicated dictionary/operator declarations, or opaque structured-value strings. The package has one typed semantic identity model ready for declarative grammar/effect migration.
