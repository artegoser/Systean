# Legacy Phase 17 semantic snapshot

This directory is archival migration evidence only.

- `semantics/*.semsys` preserves the Phase 17 English/source semantic aliases so Phase 18 migration tests can prove signature parity.
- `lexical-map.tsv` records the one-way mapping from each Systean root to its archived Phase 17 semantic label.

`LanguagePackage` does not compile these files. `WholeLanguageCompiler` excludes `legacy/` from normative package provenance. New language declarations must never be added here; use `language/typed/*.semsys`.
