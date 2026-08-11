# Phase 17 — Analyzer/generator workbench

Phase 17 productizes the already normative Systean pipeline. It does not introduce a second parser, semantic model, or browser-only interpretation layer. Native CLI and WebAssembly call the same `systean_core::workbench` API over one compiled `LanguagePackage`.

## Core workbench contract

`crates/systean-core/src/workbench.rs` exposes structured, serializable reports for:

- package identity, compatibility epoch, validation counters, complete source provenance and deterministic fingerprint;
- lexical word analysis combining dictionary definition, semantic binding/origin, bare-root morphology, pronunciation, syllabification and stress;
- surface analysis with typed surface AST, typed semantic template, unresolved slots, resolved reference/context/alias bindings, scope operators, semantic IR, canonical IR and semantic explanation provenance;
- utterance analysis with explicit pragmatic act and typed utterance semantics;
- whole text streams with before/after discourse snapshots, frame/scope state, referents, aliases, utterance history, commitments and per-event canonical spoken/written forms;
- structured literal inspection across number, quantity and temporal families;
- semantic IR to canonical Systean surface generation.

Diagnostics are represented as `WorkbenchDiagnostic` and classified by the first failing language layer: package, phonology, morphology, lexical, literal, syntax, discourse, semantics, pragmatics, text structure, or generation. Ambiguous discourse references retain the complete compatible candidate set; no candidate ranking is introduced by the tooling.

## Canonical reverse generation

`generate_surface` is a real semantic-to-surface path rather than a reformatter for an existing parsed AST:

```text
semantic source
  → semantic parse/type check
  → canonical semantic term
  → package-backed surface planning
  → canonical syntax linearization
  → ordinary package analysis
  → canonical semantic equality check
```

The planner realizes package lexemes by their declared semantic operators and surface frames. It handles atoms, proper names, structured/integer literals, opaque text, classes/predicates, prefix/infix operators, speech acts, and ordinary/counted quantifier structures currently expressible by the canonical grammar. If a typed semantic structure has no unique declared surface realization, generation fails explicitly.

Every successful generated form is re-analyzed by the ordinary language pipeline and must recover the same canonical semantic term. The generator therefore cannot silently invent an alternative interpretation.

## Native and embedded package identity

The browser no longer builds a synthetic `0.0.0` package. `systean-wasm` embeds `package.toml`, every normative module, every semantic `.semsys` source, and both frozen Phase 16 corpora, then constructs the package through `LanguagePackage::from_versioned_sources_full`.

The embedded source list uses the same normalized relative paths as native package loading. Identical normative sources therefore produce identical manifest, validation report, source provenance and package fingerprint in native and WASM consumers.

## CLI

The structured workbench is available as JSON:

```bash
cargo run --bin systean -- workbench package
cargo run --bin systean -- workbench word vid
cargo run --bin systean -- workbench surface 'na artemi vid na mari'
cargo run --bin systean -- workbench utterance 'ke na artemi viv'
cargo run --bin systean -- workbench text written 'na artemi viv. ke na artemi viv.'
cargo run --bin systean -- workbench literal '2000-12-31'
cargo run --bin systean -- workbench generate 'see(observed = proper_name(payload = "mari"), observer = proper_name(payload = "artemi"))'
```

Failures use the same JSON diagnostic structure on stderr.

## Website

`/analyzer` is now the unified workbench rather than three unrelated demo boxes. It exposes:

- package version/revision, validation summary, fingerprint and full provenance;
- word analysis with semantic binding and surface frame;
- typed surface AST, canonical/resolved surface, semantic IR and semantic provenance;
- scope visualization and reference/context/alias bindings;
- spoken/written whole-discourse analysis with state transitions, history and commitments;
- canonical semantic-to-surface generation with round-trip status;
- number/quantity/time structured-literal inspection;
- diagnostics grouped by failing engine layer.

`/dictionary` exposes the semantic binding and canonical surface frame for every lexical entry and supports searching those fields.

The Svelte layer renders reports only. It does not parse TOML, infer semantic roles, resolve references, plan surface syntax, or duplicate literal logic.

## Regression coverage

`workbench_phase17.rs` covers package identity/provenance, unified word reports, typed surface/reference reporting, ambiguity candidates, semantic generation, quantifier generation, discourse transitions, literal inspection, and native/embedded fingerprint equality.

`workbench_cli.rs` covers the JSON package/word contract, reverse generation, and layer-grouped CLI diagnostics.

Phase 17 should be considered validated only after the author toolchain passes:

```bash
cargo test --workspace
cargo run --bin systean -- check
cd site && pnpm check && pnpm build
```
