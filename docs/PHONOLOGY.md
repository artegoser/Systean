# Systean phonology

Status: normative baseline plus implementation contract.

## 1. Existing alphabet is authoritative

Systean already has a complete alphabet and pronunciation mapping in
`lib/config/alphabet.toml`. This project does not redesign, extend, or replace that
inventory.

The existing website behaviour is normative: every supported written letter maps
to one fixed pronunciation, independent of context. Letter case is not phonemic.
Whitespace separates written words and is preserved as a boundary in phonological
analysis.

The Rust engine must consume the same configuration instead of carrying a second
hardcoded alphabet.

## 2. Canonical orthography / pronunciation invariant

For the configured alphabet:

- every grapheme must have exactly one pronunciation;
- every pronunciation must map back to exactly one grapheme;
- grapheme tokens must be uniquely segmentable;
- pronunciation tokens must be uniquely segmentable;
- canonical spelling is lower-case;
- unsupported graphemes are errors, never guessed or silently normalized.

The current alphabet consists of one-character graphemes, but the engine validates
prefix-freedom rather than depending on that implementation detail.

## 3. Syllables

Syllables are determined mechanically from the configured vowel class.

Current rule:

1. every vowel is a syllable nucleus;
2. consonants before the first nucleus belong to the first syllable;
3. for a consonant run between two nuclei, the final consonant begins the next
   syllable and preceding consonants close the previous syllable;
4. adjacent vowels therefore form adjacent syllable nuclei;
5. consonants after the final nucleus close the final syllable.

This rule is deterministic and does not introduce a new consonant-cluster ban.
The current phonotactic baseline deliberately remains permissive because the old
language specification did not define narrower cluster constraints.

A lexical root must contain at least one vowel so that its lexical stress is
defined.

## 4. Stress

Lexical stress falls on the **first syllable of the root**.

This is not equivalent to "first syllable of the surface word". Prefixes or other
future morphology must preserve the root span and ask the phonology layer to place
stress from that span.

Suffixes and prefixes do not move lexical stress.

For compounds or other structures with more than one lexical root, primary and
secondary stress are intentionally deferred until compounding itself is designed.
The phonology engine therefore accepts one explicit lexical-root span for a simple
word today rather than inventing compound semantics.

## 5. Root creation

Roots are human-created. Systean tooling must not generate random lexical roots.

The engine may validate a proposed root for:

- alphabet legality;
- presence of a syllable nucleus;
- exact collision with existing roots;
- pronunciation collision;
- suspicious similarity to existing roots as an advisory warning.

Similarity warnings are not validity failures. Human lexical design remains the
source of new roots.

## 6. Spoken segmentation

The phonology engine provides a generic exact segmentation checker for a supplied
inventory of spoken forms. It reports zero, one, or multiple segmentations of a
phoneme stream.

A language-wide guarantee cannot yet be asserted because complete surface word
forms depend on morphology. The checker is the mechanism that morphology and the
future language-package compiler will use to reject spoken collisions once those
forms exist.

Stress may provide redundancy for human listeners but is not treated as the only
word-boundary signal.

## 7. Non-goals of the phonology layer

The phonology layer does not:

- infer intended words from invalid input;
- autocorrect spelling;
- invent roots;
- infer morphology;
- change the existing alphabet;
- make world/semantic decisions.

It maps and validates explicit linguistic structure.

## 8. Required round-trip properties

The implementation must test at minimum:

```text
spell(pronounce(root)) == canonical(root)
pronounce(spell(phonemes)) == phonemes
analyze(root).stress == first root syllable
```

For any supplied finite spoken inventory:

```text
segment(stream) = exactly one path | ambiguous | impossible
```

Ambiguity is reported rather than resolved heuristically.
