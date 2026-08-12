# Phase 18 — Typed semantic identity and DSL core migration

Status: **implementation complete; author-toolchain validation pending**
Architecture: [`SEMANTIC_DSL_ARCHITECTURE.md`](SEMANTIC_DSL_ARCHITECTURE.md)

## Goal

Replace stringly semantic ownership and duplicated lexical/operator declarations with one compiled, typed, ID-based semantic package while preserving the accepted Systean surface language of the Phase 17 baseline.

Phase 18 deliberately stops at the semantic/package boundary. The Phase 17 surface parser/checker still consumes a generated compatibility projection (`Environment`) until Phase 19 replaces the remaining `SurfaceFormConfig`/named-role machinery with declarative typed surface rules. That projection is derived from the typed package and is not a second normative semantic source.

## Final ownership after Phase 18

Normative semantic input:

```text
language/typed/core.semsys
language/typed/lexicon.semsys
language/typed/units.semsys
```

Human/surface metadata still awaiting later phases:

```text
language/dictionary.toml   # definition + temporary Phase 18 surface realization metadata
language/units.toml        # written/spoken aliases + temporary checker dimension-type adapter
language/syntax.toml       # Phase 17 surface/pragmatics policy; migrated in Phase 19
```

Archived migration evidence only:

```text
language/legacy/semantics/*.semsys
language/legacy/lexical-map.tsv
```

`language/legacy/` is excluded from normative package provenance and never participates in `LanguagePackage` compilation.

## Delivery boundary

### Phase 18A — typed compiler foundation

- new typed `.semsys` frontend for `type`, `subtype`, `literal`, `word`, `primitive`, `def`, `intrinsic`, `data`, `context`, `dimension`, and `unit`;
- deterministic compiled identities (`SymbolId`, `TypeId`, `ConstructorId`, `ContextSlotId`, `DimensionId`, `UnitId`, `IntrinsicId`, `FieldId`);
- parameter-order/type signatures independent of parameter source names;
- de Bruijn lowering for lexical parameters and local lambdas, including higher-order invocation;
- structural constructors, context references, scalar values, and exact unit relations;
- separate semantic and surface fingerprints;
- declaration provenance/source spans and debug-only source names.

### Phase 18B1 — structured-value and unit runtime cutover

- production structured literals store typed `StructuredValue` variants instead of a `family/canonical` semantic string envelope;
- numbers, approximate numbers, digits, digit sequences, quantities, dates, times, time zones, instants, durations, and intervals retain structured semantic payloads;
- information values carry `InformationStatus`, `ConstructorId`, and a structural knower (`ContextSlotId` or value term), not `unknown:context:...` strings;
- runtime unit/dimension identity is `UnitId`/`DimensionId`;
- exact conversion derives from the typed unit graph; engine checks against names such as `"time"` and `"second"` are gone.

### Phase 18B2 — final lexical/package ownership cutover

- all 175 lexical roots are normative `word` declarations in `language/typed/lexicon.semsys`;
- the Systean root itself is the semantic symbol identity (`vid`, `per`, `viv`, ...); the old English pivot (`see`, `person`, `alive`, ...) is not production semantic input;
- `dictionary.toml` no longer accepts `semantic = { ... }`; it is metadata/surface overlay only;
- old `LexicalSemantic::{Operator,Information,Context,...}` is removed;
- context/information/reference lexemes are derived structurally from typed word definitions;
- `language/typed/core.semsys` owns the full type/subtype/literal ontology and the small non-lexical primitive/intrinsic layer;
- `language/typed/units.semsys` owns dimensions and exact unit relations; `units.toml` no longer duplicates dimension/scale semantics per unit;
- the Phase 17 `Environment` used by the existing surface/checker runtime is generated from typed IDs/signatures and Systean root labels;
- migration diagnostics resolve the archived 175 root→Phase-17-alias mappings to typed `SymbolId`s and prove signature parity;
- canonical information rendering is structural and no longer emits the old colon-delimited mini-language;
- typed semantic and surface fingerprints are exposed directly by `LanguagePackage` and the workbench package report.

## Scope checklist

### DSL frontend

- [x] Support `type`, `subtype`, `literal`, `word`, `primitive`, `def`, `intrinsic`, `data`, `context`, `dimension`, and `unit` declarations.
- [x] Define one unambiguous grammar for generic parameters, typed function parameters, return types, constructors, definitions, and local lambdas.
- [x] Make source parameter names authoring-only; compile binders to declaration-local order/de Bruijn indices.
- [x] Preserve source spans/provenance for compiled declarations.
- [x] Reject duplicate declarations, invalid type arities, subtype cycles, definition cycles, and invalid unit graphs.

### Compiled symbol/type model

- [x] Resolve semantic identity to typed IDs rather than source `String`s in the normative IR.
- [x] Compile parameter roles to ordered typed slots; source role names survive only in debug/provenance metadata.
- [x] Make alpha-equivalent binders produce equal canonical terms.
- [x] Make semantic equality/fingerprints independent of comments, formatting, and local parameter names.
- [x] Compile subtype and literal-kind ownership into the typed package rather than loading it from the legacy semantic package.

### Lexical ownership

- [x] Migrate every ordinary lexical root from dictionary + duplicated operator declaration to one typed `word` declaration.
- [x] Use the Systean root itself as lexical semantic identity; English Phase 17 aliases are archived migration metadata only.
- [x] Remove mandatory JSON-shaped `kind = operator/information/context/...` semantic bindings from TOML.
- [x] Derive ordinary default frames from typed arity where the Phase 17 grammar permits it.
- [x] Preserve root legality, pronunciation, reserved-token, and collision validation through the whole-language compiler.
- [x] Keep manual root authoring; no allocator is introduced.

### Structured semantic values

- [x] Replace opaque structured semantic string envelopes with typed algebraic/scalar values.
- [x] Represent unknown/unspecified/withheld structurally.
- [x] Store context-backed unknown values using `ContextSlotId` rather than encoded strings.
- [x] Keep number/quantity/date/time/duration/interval semantics structured through later runtime layers.
- [x] Ensure canonical rendering is output only; semantic layers do not parse it back to recover meaning.

### Dimensions and units

- [x] Compile dimensions and units to IDs in the normative package.
- [x] Make exact rational unit relations owned by typed `.semsys`.
- [x] Remove source-name runtime checks such as `"time"`/`"second"`.
- [x] Preserve exact conversion and dimension compatibility through resolved IDs.
- [x] Reduce `units.toml` to surface/API aliases plus the temporary Phase 17 checker type adapter; Phase 19 can remove that adapter when the checker consumes typed types directly.

### Compatibility and fingerprints

- [x] Expose separate semantic and surface fingerprints.
- [x] Alpha-renaming, source formatting, comments, and documentation do not change semantic identity.
- [x] Semantic signature/definition changes change semantic fingerprint.
- [x] Surface root/form changes change surface fingerprint.
- [x] Provide explicit migration diagnostics from archived Phase 17 lexical aliases to Phase 18 typed symbols.
- [x] Include typed sources, and exclude archived legacy sources, in native and embedded/WASM package provenance.

## Compatibility strategy

1. [x] Introduce the typed compiler behind the Phase 17 behavior oracle.
2. [x] Prove representative parity for ordinary predicates, context, information, units, and structured values.
3. [x] Migrate all 175 lexical declarations and the complete type/subtype/literal ontology.
4. [x] Migrate unit/dimension ownership and structured semantic values.
5. [x] Generate the Phase 17 checker environment from the typed package instead of loading an independent semantic package.
6. [x] Freeze the previous semantic sources under `language/legacy/` for one-way migration diagnostics only.
7. [x] Keep the Phase 16 whole-language compiler/corpora as the release gate; no ambiguity check is disabled for the migration.

The compatibility corpus expected semantic strings were updated only for the intentional internal identity migration from Phase 17 English aliases to Systean root symbols. Surface forms, types, and denotational signatures remain unchanged. The compatibility epoch therefore remains `1`.

## Required tests

Implemented tests cover:

- [x] stable typed IDs and ID-only normative semantic identity;
- [x] alpha-equivalent binder/parameter renaming;
- [x] fingerprint independence from comments/source formatting;
- [x] fingerprint sensitivity to signature and surface changes;
- [x] one-declaration lexical growth without Rust changes;
- [x] structural information/context values with no colon mini-language;
- [x] typed structured-value runtime for numbers, quantities, dates, durations, and intervals;
- [x] exact ID-based unit conversion with no source-name special cases;
- [x] dictionary↔typed-word coverage is exactly 1:1 (175 roots);
- [x] archived Phase 17 lexical/operator signatures are alpha-equivalent to the generated Phase 18 compatibility projection;
- [x] native and embedded/WASM packages consume the same typed source set;
- [x] legacy semantic fields in `dictionary.toml` are rejected.

Author-toolchain validation status. The package check and both site gates passed on the author toolchain. A subsequent workspace rerun passed the repaired Phase 16 regressions and reached the Phase 18 suite, where it exposed two final Phase 18 regressions: source-span detection did not recognize semicolon-terminated declarations, and one interval test used a date/date form outside the frozen instant/instant interval grammar. Both are repaired; the complete workspace suite must now be rerun from the beginning:

- [ ] `cargo test --workspace`
- [x] `cargo run --bin systean -- check`
- [x] `cd site && pnpm check`
- [x] `cd site && pnpm build`

## Deliberate Phase 19 boundary

Phase 18 does **not** pretend the Phase 17 surface implementation has disappeared. `SurfaceFormConfig`, syntax named-role labels, and the compatibility `Environment` remain as generated adapters so the accepted grammar continues to run while semantic ownership changes underneath it.

Phase 19 removes those adapters by compiling declarative typed `form` rules and discourse effects directly against the Phase 18 IDs. No semantic declaration may move back into TOML or into Rust to make that migration easier.

## Completion result

The language package now has one normative typed semantic owner. Systean lexical roots no longer route through English semantic identifiers, TOML no longer carries semantic JSON shapes, structured values no longer hide meaning in canonical strings, and units/dimensions use resolved identities and exact typed relations. The remaining string/named-role surface compatibility layer is explicitly a Phase 19 concern rather than a second semantic source of truth.
