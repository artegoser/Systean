# Phonology implementation plan

This checklist tracks the first executable phonology foundation.

- [x] Preserve `lib/config/alphabet.toml` as the alphabet/pronunciation source of truth.
- [x] Document the existing one-context-independent-pronunciation contract.
- [ ] Load and validate the alphabet from Rust without hardcoded Systean letters.
- [ ] Validate grapheme and pronunciation prefix-freedom and bijection.
- [ ] Add canonical spelling -> pronunciation and pronunciation -> spelling round trips.
- [ ] Add deterministic vowel-driven syllabification.
- [ ] Add root-span-aware lexical stress on the first root syllable.
- [ ] Add manual root validation with non-fatal similarity warnings.
- [ ] Add exact spoken segmentation ambiguity detection over supplied inventories.
- [ ] Expose phonology analysis and root checks through the CLI.
- [ ] Restore the alphabet website's access to the canonical config without duplicating it.
- [ ] Add unit/integration/property-style tests for all invariants above.
- [ ] Update repository documentation and run instructions.
