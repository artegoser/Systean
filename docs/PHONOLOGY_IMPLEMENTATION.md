# Phonology implementation plan

This checklist tracks the first executable phonology foundation.

- [x] Preserve `lib/config/alphabet.toml` as the alphabet/pronunciation source of truth.
- [x] Document the existing one-context-independent-pronunciation contract.
- [x] Load and validate the alphabet from Rust without hardcoded Systean letters.
- [x] Validate grapheme and pronunciation prefix-freedom and bijection.
- [x] Add canonical spelling -> pronunciation and pronunciation -> spelling round trips.
- [x] Add deterministic vowel-driven syllabification.
- [x] Add root-span-aware lexical stress on the first root syllable.
- [x] Add manual root validation with non-fatal similarity warnings.
- [x] Add exact spoken segmentation ambiguity detection over supplied inventories.
- [x] Expose phonology analysis and root checks through the CLI.
- [x] Restore the alphabet website's access to the canonical config without duplicating it.
- [x] Add unit/integration/property-style tests for all invariants above.
- [x] Update repository documentation and run instructions.
