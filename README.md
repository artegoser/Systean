# Systean

**Unambiguity in every aspect.**

Systean is a human-first artificial language designed around one structural parse and one compositional semantic structure for every normative expression. The repository contains the language package, the generic Rust engine that validates and analyzes it, native CLI tooling, WebAssembly bindings, and the SvelteKit website.

The design is specified in:

- [`docs/DESIGN.md`](docs/DESIGN.md)
- [`docs/SEMANTICS.md`](docs/SEMANTICS.md)
- [`docs/PHONOLOGY.md`](docs/PHONOLOGY.md)
- [`docs/MORPHOLOGY.md`](docs/MORPHOLOGY.md)
- [`docs/SYNTAX.md`](docs/SYNTAX.md)
- [`docs/ENGINE_ARCHITECTURE.md`](docs/ENGINE_ARCHITECTURE.md)
- [`docs/FINAL_ARCHITECTURE.md`](docs/FINAL_ARCHITECTURE.md)
- [`docs/SEMANTIC_DSL_ARCHITECTURE.md`](docs/SEMANTIC_DSL_ARCHITECTURE.md)
- [`docs/SUBJECTIVE_STATES.md`](docs/SUBJECTIVE_STATES.md)
- [`docs/IMPLEMENTATION_ROADMAP.md`](docs/IMPLEMENTATION_ROADMAP.md)

## Repository architecture

```text
language/                     canonical Systean language package
├── package.toml              package version, provenance and validation policy
├── alphabet.toml
├── phonology.toml
├── morphology.toml
├── syntax.toml
├── dictionary.toml
├── literals.toml
├── units.toml
├── semantics/
│   └── *.semsys
├── corpus/                   frozen compatibility and adversarial inputs
└── legacy/                   superseded prototype configs

crates/
├── systean-core/             generic language engine
├── systean-cli/              native CLI consumers
└── systean-wasm/             thin browser bindings over systean-core

site/                         SvelteKit UI; no language logic or TOML parsing
tests/fixtures/               non-normative semantic test vocabulary
```

`language/` is data. `systean-core` is the implementation of generic mechanisms. The engine must not hardcode Systean-specific concepts such as noun, past tense, `sol`, or a particular semantic operator.

The website is deliberately a consumer of the Rust engine, not a second implementation of the language:

```text
language/*
   ↓
systean-core
   ├── systean-cli → terminal
   └── systean-wasm → SvelteKit
```

The WASM crate embeds the same canonical language files at compile time and constructs the same `LanguagePackage` used by the native CLI.

## Rust workspace

The workspace currently contains:

- semantic IR, type checking, canonicalization, provenance and explanation;
- Chumsky `0.13.0` semantic specification parser;
- the `.semsys` semantic DSL;
- canonical alphabet/pronunciation mapping;
- deterministic syllabification and root-aware lexical stress;
- reversible bare-root morphology with automatic root boundaries;
- config-driven recursive surface syntax, scope, precedence and semantic lowering;
- manual root validation/auditing and spoken segmentation checks;
- `WholeLanguageCompiler` and immutable `LanguagePackage`, which validate module versions, provenance, cross-layer ownership, generated surface/semantic round trips, spoken ambiguity, short-expression exhaustiveness and frozen compatibility/adversarial corpora as one versioned package;
- a shared Phase 17 workbench API for typed AST inspection, discourse-state tracing, provenance-aware diagnostics, structured-literal inspection and semantic IR → canonical surface generation.

Run the complete native test suite:

```bash
cargo test
```

Validate the canonical package:

```bash
cargo run --bin systean -- check
```

Loading a versioned package compiles `language/package.toml` together with every normative module. Normative collisions or frozen-corpus drift are package-load failures rather than parser tie-breaks.

All CLI commands use `./language` by default. Another package can be selected globally:

```bash
cargo run --bin systean -- --language path/to/language check
```

### Phonology

```bash
cargo run --bin systean -- phonology check
cargo run --bin systean -- phonology pronounce Systean
cargo run --bin systean -- phonology spell sjstean
cargo run --bin systean -- phonology analyze sol
cargo run --bin systean -- roots check sal
cargo run --bin systean -- roots audit
```

Roots are authored manually. Tooling validates proposed roots and reports collisions/similarity; it does not invent vocabulary.

### Morphology

```bash
cargo run --bin systean -- morphology check
cargo run --bin systean -- morphology analyze sol
cargo run --bin systean -- morphology generate sol
```

Morphology v1 is intentionally minimal: a lexical word is exactly its declared root. No POS ending or grammatical prefix stack is inferred.


### Surface syntax

```bash
cargo run --bin systean -- syntax check
```

The structural syntax engine is executable and enforces canonical frame order, `ki ... ku` scope grouping, typed scoped binders, outer-only utterance constructions, and package-declared precedence/associativity. Every lexical root and reversible lexical form is owned by `language/typed/lexicon.semsys`; `dictionary.toml` must cover those roots exactly and contributes human definitions only. Systean-specific discourse behavior is declared in `language/typed/effects.semsys`. No semantic or lexical surface binding is duplicated in TOML.

### Semantic IR

The normative semantic declarations live in `language/typed/*.semsys`. Phase 17 semantic sources are archived under `language/legacy/semantics/` only for migration parity; old DSL corpus/demo fixtures remain test-only compatibility inputs.

```bash
cargo run --bin systean -- explain 'equal(left = 1, right = 1)'
```

The parser validates structure and types; it does not validate truth, plausibility, speaker knowledge or world state.

### English reference

Phase 20 compiles `language/docs/*.sydoc` against the public typed vocabulary and exposes deterministic English rendering without making English part of semantic identity:

```bash
cargo run --bin systean -- english 'na artemi vid na mari'
```

Documentation examples pin their expected canonical semantics and are revalidated through the real parser/discourse pipeline when the package loads.

### Workbench

The Phase 17 workbench exposes the same structured reports through native CLI and WASM:

```bash
cargo run --bin systean -- workbench package
cargo run --bin systean -- workbench word vid
cargo run --bin systean -- workbench surface 'na artemi vid na mari'
cargo run --bin systean -- workbench text written 'na artemi viv. ke na artemi viv.'
cargo run --bin systean -- workbench literal '2000-12-31'
```

Reports include the package fingerprint and provenance. Workbench failures are layer-classified diagnostics; ambiguous references preserve their candidate set.

## Website

The SvelteKit site does not import TOML or duplicate pronunciation logic. Its engine facade dynamically initializes the WebAssembly build from `crates/systean-wasm`.

The build helper installs pinned `wasm-pack 0.15.0` if it is not already available and builds the generated package into an ignored directory under `site/src/lib/wasm/pkg`.

```bash
cd site
pnpm check
pnpm build
pnpm dev
```

`pnpm check`, `pnpm build`, and `pnpm dev` build the WASM package first.

Current engine-backed pages:

- `/alphabet` — alphabet and arbitrary pronunciation;
- `/dictionary` — searchable dictionary backed by typed word reports and canonical surface information;
- `/analyzer` — unified package/word/surface/discourse/generator/literal workbench with scope, reference and provenance visualization.

## Design status

Phases 2–20 are implemented. Phase 17 workbench productization remains the public inspection layer. Phase 19 moved reversible lexical forms, higher-order binders, capture/outer placement, precedence and Systean-specific discourse effect programs into typed `.semsys`; `dictionary.toml` is metadata-only and `syntax.toml` contains only real structural engine policy. Phase 20 adds complete compiled English documentation for all public roots, an independent documentation fingerprint, and deterministic controlled-English rendering exposed through native, CLI, WASM and workbench APIs. Rust author-toolchain validation for the latest phases is still pending.

Before the 1.0 freeze, the accepted roadmap now includes a deliberate architecture cleanup:

- Phase 18 — typed semantic identity and DSL core migration;
- Phase 19 — declarative surface grammar and discourse effects;
- Phase 20 — complete English reference documentation and deterministic controlled-English rendering;
- Phase 21 — user-facing learning site and interactive contextual analyzer;
- Phase 22 — Systean 1.0 language freeze.

The target model is specified in `docs/SEMANTIC_DSL_ARCHITECTURE.md`. `dictionary.toml` is documentation metadata only; semantic identity, signatures, lexical surface rules and discourse effects are owned by the typed DSL. Vocabulary remains manually authored.
