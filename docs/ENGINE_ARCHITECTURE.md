# Unified Engine Architecture

## Status

Implemented baseline.

Systean has one normative implementation path:

```text
language package → systean-core → consumer
                              ├→ systean-cli
                              └→ systean-wasm → site
```

The website must not parse the language configuration, infer pronunciation, implement morphology, or reconstruct semantic meaning independently.

## Workspace boundaries

### `language/`

Canonical language data. A language change belongs here when it changes Systean itself rather than the generic engine.

Active files currently include:

- `alphabet.toml`
- `phonology.toml`
- `morphology.toml`
- `syntax.toml`
- `dictionary.toml`
- `semantics/*.semsys`

Superseded prototype grammar and morphology files are retained under `language/legacy/` for reference only. They are not loaded by `LanguagePackage`.

### `systean-core`

Generic mechanisms only:

- semantic parsing and compilation;
- typed IR and canonicalization;
- phonological configuration and analysis;
- reversible morphology and root-boundary analysis;
- dictionary-compiled surface lexicon plus config-driven parsing, grouping, precedence, generation and semantic lowering;
- root inventory validation;
- unified `LanguagePackage` loading.

It may load language data from files for native consumers or from source strings for embedded consumers.

### `systean-cli`

Native interface. It defaults to `./language` and accepts `--language <path>` for another package. Individual commands no longer receive separate alphabet/phonology/dictionary paths.

### `systean-wasm`

Thin `wasm-bindgen` boundary. It embeds the canonical language sources and creates the same `LanguagePackage` as the native CLI. Exports return JSON for structured UI data and plain strings for direct spelling/pronunciation transformations.

No Systean rule belongs in this crate.

### `site`

Presentation only. The site dynamically initializes `systean-wasm` and exposes a small TypeScript facade. It has no TOML dependency, Vite filesystem alias, or duplicated pronunciation algorithm.

## Build flow

`scripts/build-wasm.sh`:

1. ensures the `wasm32-unknown-unknown` target when `rustup` is present;
2. installs pinned `wasm-pack 0.15.0` if necessary;
3. builds `crates/systean-wasm` with the `web` target;
4. writes generated bindings to ignored `site/src/lib/wasm/pkg/`.

Svelte `dev`, `check`, and `build` invoke this step first.

## Invariants

1. A canonical Systean rule exists once, either as generic engine behavior or language-package data.
2. Browser and CLI results come from `systean-core`.
3. Test-only vocabulary never enters the canonical language package.
4. Legacy prototype grammar/morphology files are never loaded as active rules.
5. Word structure comes from the Rust morphology engine; the site never guesses root boundaries.
6. Surface structure comes from the Rust syntax engine; the site never implements precedence or scope itself.
7. Every dictionary root is compiled into the surface lexicon; `syntax.toml` never duplicates lexical roots.
8. Typed constant roots are installed into the semantic environment from the dictionary; operator signatures remain authoritative in `.semsys`.
9. A malformed canonical package prevents consumers from initializing instead of allowing partial interpretation.
