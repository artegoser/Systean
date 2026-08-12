# Phase 18 — Typed semantic identity and DSL core migration

Status: **in progress — 18A and 18B1 implemented; final lexical/package cutover remains 18B2**
Architecture: [`SEMANTIC_DSL_ARCHITECTURE.md`](SEMANTIC_DSL_ARCHITECTURE.md)

## Goal

Replace the stringly semantic core and duplicated lexical/operator ownership with a compiled, typed, ID-based model while preserving the accepted Systean language behavior of the Phase 17 baseline.

This phase intentionally does **not** attempt the complete declarative surface/pragmatics migration. It establishes the new semantic foundation first so later migrations do not mix architecture changes with grammar changes.


## Phase 18A / 18B1 / 18B2 implementation boundary

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

**Phase 18B1 — implemented in this delivery:**

- production structured literals now store typed `StructuredValue` variants instead of a `family/canonical` string envelope;
- numbers, approximate numbers, digits, digit sequences, quantities, dates, times, time zones, instants, durations, and intervals keep their semantic payload structurally;
- information values now carry a typed status, constructor identity, and resolved `ContextSlotId`/value knower instead of the `unknown:context:...` mini-language;
- runtime unit/dimension identity is `UnitId`/`DimensionId`; legacy English unit IDs are compatibility lookup aliases only;
- duration semantics resolve their base unit from package configuration and derive the dimension from that unit, removing engine checks against `"time"`/`"second"`;
- `language/typed/units.semsys` is a typed migration bridge whose IDs/dimensions/exact scales are regression-checked against the current runtime registry;
- renderers may still produce canonical human strings, but later semantic layers never reparse those strings to recover meaning.

**Phase 18B2 — deliberately left for the final quarter:**

- cut production `LanguagePackage` lexical/semantic ownership over to the typed package as the single source of truth;
- migrate every lexical root out of `dictionary.toml` and duplicated legacy `.semsys` signatures;
- make typed unit declarations, rather than the transitional `units.toml` adapter, the production ownership source;
- replace remaining runtime named-role/source-name maps at the lexical boundary;
- delete legacy `LexicalSemantic` status/context/operator string adapters and the old semantic environment path after parity;
- run full compatibility-corpus/native/WASM/site parity and author-toolchain validation before Phase 18 is marked complete.

18B1 intentionally cuts over the **value/runtime boundary** without cutting over lexical ownership. That preserves the Phase 17 grammar and dictionary as the behavior oracle while ensuring the last quarter no longer has to migrate structured values and lexical identity simultaneously.

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

- [x] Replace `StructuredLiteral { family: String, canonical: String, ... }` with typed algebraic/scalar runtime values.
- [x] Migrate runtime information status away from `unknown:...`, `withheld`, and similar string encodings; the legacy lexical declaration remains an 18B2 source adapter.
- [x] Represent unknown/unspecified/withheld as typed structures.
- [x] Make context-backed unknown values contain resolved `ContextSlotId` references rather than encoded strings.
- [x] Migrate number/quantity/date/time/duration/interval runtime semantic representations to typed values without changing accepted Systean surface forms.

### Dimensions and units

- [x] Compile dimensions and units to IDs in the new package model; production conversion cutover remains 18B.
- [x] Remove runtime checks against source names such as `"time"` and `"second"`.
- [x] Preserve exact rational conversion and dimension compatibility through resolved IDs.
- [x] Remove engine base-unit string conventions; typed base-unit declarations exist and are parity-checked, while production declaration ownership moves from TOML to `.semsys` in 18B2.

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

- [ ] Source symbol names resolve to stable IDs and no runtime semantic branch needs their strings. Value/unit runtime is ID-based in 18B1; lexical/operator branches remain for 18B2.
- [x] Renaming a local parameter/binder leaves canonical terms and semantic fingerprint unchanged.
- [x] Changing an English/comment label leaves semantic/surface fingerprints unchanged.
- [x] Changing a word signature changes semantic compatibility.
- [x] Ordinary new lexical word requires one declaration and no Rust code in the typed compiler.
- [x] `hid`/`unk`/`vak` produce typed structures with no colon-delimited mini-language in the typed compiler.
- [x] Dates, durations, quantities, and intervals round-trip as typed values without parsing a canonical string in later semantic layers.
- [x] Unit conversion contains no source-name special cases after the package boundary resolves IDs.
- [ ] Existing Phase 2–17 behavioral tests remain green or are migrated to equivalent structural assertions.
- [ ] Full compatibility corpus preserves Phase 17 parse/meaning unless an explicitly approved semantic migration says otherwise.

## Completion result

The semantic core is no longer dependent on English identifiers, duplicated dictionary/operator declarations, or opaque structured-value strings. The package has one typed semantic identity model ready for declarative grammar/effect migration.
