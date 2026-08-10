# Syntax / Lexicon Unification

Status: implemented in this phase.

## Goal

Remove the duplicate lexical registry that previously existed in both `dictionary.toml` and `syntax.toml`, while preserving deterministic surface parsing and typed semantic lowering.

## Implementation plan

1. Keep structural policy only in `language/syntax.toml`.
2. Make `language/dictionary.toml` the single source of lexical roots and lexical semantic identity.
3. Compile constant roots automatically into surface atoms.
4. Store operator-root surface realization beside that root's semantic binding in the dictionary.
5. Keep complete operator signatures authoritative in `.semsys`.
6. Install typed dictionary constants into the semantic environment when building `LanguagePackage`.
7. Build one `SurfaceLexicon` and pass it to parser, generator, lowering, CLI, WASM, and site consumers.
8. Allow bare constant expressions so semantic type errors are reported by the semantic checker rather than misreported as missing syntax bindings.
9. Replace unit-like syntax fixtures with package-level integration tests using canonical policy and dictionary compilation.

## Checklist

- [x] Remove `[lexemes.*]` from `language/syntax.toml`.
- [x] Add one formal semantic binding to every canonical dictionary root.
- [x] Add root-local syntax realization only for semantic operator roots.
- [x] Compile every dictionary root into `SurfaceLexicon`.
- [x] Install dictionary constants into `Environment` with validated semantic types.
- [x] Reject operator roots without a surface realization.
- [x] Reject redundant syntax realization on constant roots.
- [x] Keep `ki` / `ku` outside the lexical root inventory and collision-check them.
- [x] Recognize `sol` as a surface atom without a syntax-config entry.
- [x] Make `ne sol` reach semantic type checking.
- [x] Test that a newly added constant root works without editing `syntax.toml`.
- [x] Test lexical-root / compiled-surface-lexicon completeness.
- [x] Keep precedence, scope, quantifier, speech-act, frame-order, round-trip, and rejection regressions.
- [x] Update CLI/WASM/site terminology from duplicate "surface bindings" to lexical roots.
