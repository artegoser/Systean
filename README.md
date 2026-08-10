# Systean

**Unambiguity in every aspect.**

Systean is a human-first artificial language designed around one structural parse and one compositional semantic structure for every normative expression. The repository contains the language package, the generic Rust engine that validates and analyzes it, native CLI tooling, WebAssembly bindings, and the SvelteKit website.

The design is specified in:

- [`docs/DESIGN.md`](docs/DESIGN.md)
- [`docs/SEMANTICS.md`](docs/SEMANTICS.md)
- [`docs/PHONOLOGY.md`](docs/PHONOLOGY.md)
- [`docs/ENGINE_ARCHITECTURE.md`](docs/ENGINE_ARCHITECTURE.md)

## Repository architecture

```text
language/                     canonical Systean language package
├── alphabet.toml
├── phonology.toml
├── dictionary.toml
├── semantics/
│   └── core.semsys
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
- manual root validation/auditing and spoken segmentation checks;
- `LanguagePackage`, which validates phonology, dictionary/root inventory and semantic specifications as one unit.

Run the complete native test suite:

```bash
cargo test
```

Validate the canonical package:

```bash
cargo run --bin systean -- check
```

All CLI commands use `./language` by default. Another package can be selected globally:

```bash
cargo run --bin systean -- --language path/to/language check
```

### Phonology

```bash
cargo run --bin systean -- phonology check
cargo run --bin systean -- phonology pronounce Systean
cargo run --bin systean -- phonology spell sjstean
cargo run --bin systean -- phonology analyze nasol --root sol
cargo run --bin systean -- roots check sal
cargo run --bin systean -- roots audit
```

Roots are authored manually. Tooling validates proposed roots and reports collisions/similarity; it does not invent vocabulary.

### Semantic IR

The normative semantic declarations live in `language/semantics/*.semsys`. Corpus/demo declarations used only for tests live under `tests/fixtures/semantics/`.

```bash
cargo run --bin systean -- explain 'equal(left = 1, right = 1)'
```

The parser validates structure and types; it does not validate truth, plausibility, speaker knowledge or world state.

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
- `/dictionary` — dictionary loaded by Rust;
- `/analyzer` — phonological word analysis and semantic IR explanation.

## Design status

Semantic and phonological foundations are executable. The next major language layer is morphology: abstract grammatical features must map bidirectionally to pronounceable surface forms without restoring the old consonant-prefix pileup or contextual semantic guessing.
