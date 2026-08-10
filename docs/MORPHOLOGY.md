# Systean Morphology

Status: **implemented v1 baseline**
Purpose: define the normative structure of lexical words between the phonological and future syntactic layers.

## 1. Core decision

Systean does not mark information inside a word merely because natural languages commonly do so.

The normative v1 lexical word is:

```text
WORD = ROOT
```

No mandatory part-of-speech/class ending is added. No grammatical feature is inserted into the word by default.

The active configuration is `language/morphology.toml`:

```toml
[morphology]
strategy = "bare_roots"
```

The old morphology in `language/legacy/morphology.toml` is historical only and is never loaded by `LanguagePackage`.

## 2. No POS endings

The old prototype endings such as `-o`, `-i`, `-a`, `-e`, `-u`, and `-y` are not normative.

A root is not converted into a noun, verb, adjective, adverb, relation, or conjunction by changing its final vowel. Semantic typing and future syntax already have enough information to determine how an expression can compose.

Therefore, if `sol` is a declared root, the morphology engine currently accepts:

```text
sol
```

and does not implicitly accept:

```text
solo
soli
sola
sole
solu
soly
```

Those strings may only become valid in the future if they are independently declared roots or are produced by an explicitly specified future morphological rule.

## 3. No grammatical prefix stack

The old prototype stack for polarity, determination, number, and tense is rejected.

Morphology v1 does not encode any of the following inside a lexical word:

- negation;
- definiteness;
- number/quantity;
- tense;
- aspect;
- modality;
- person;
- agreement;
- case;
- semantic role;
- speech act;
- emotion.

These distinctions have scopes and semantic structures that are not generally word-local. They belong in future syntax/constructions/particles unless a later design proves that a particular local morphological encoding is both necessary and unambiguous.

## 4. One root, one lexical identity

`language/dictionary.toml` now requires exactly one `definition` per root.

POS-specific fields such as `noun`, `verb`, `adj`, or `adv` are rejected by the canonical package loader. A lexical root does not acquire context-selected meanings by being placed into different word classes.

Human-readable dictionary definitions document lexical identity; they are not a world-knowledge database and are not a substitute for formal semantic typing.

## 5. Analysis and generation

The generic morphology engine exposes both directions:

```text
analyze(surface) -> morphological structure
generate(structure/root) -> surface
```

For the current strategy:

```text
analyze("sol")
=> ROOT("sol")

generate(ROOT("sol"))
=> "sol"
```

Every declared root is validated by `LanguagePackage` with a morphology round trip when the package loads.

The required invariant is:

```text
analyze(generate(root)).root == root
```

For v1, generation is also identity:

```text
generate(root) == root
```

## 6. Root boundary and stress

Morphology owns the structural fact of which span is the lexical root. Phonology owns pronunciation, syllabification, and stress realization.

The combined analyzer therefore flows as:

```text
surface word
  -> morphology: identify morphemes/root span
  -> phonology: tokenize/syllabify/pronounce
  -> stress: first syllable belonging to the root
```

With bare-root morphology, the root span is the entire word. This separation is intentional so future local derivation can add material without moving lexical stress away from the first root syllable.

## 7. Unknown words are not guessed

If a written form is phonologically legal but has no declared morphological analysis, the normative analyzer rejects it.

It does not:

- strip a familiar-looking ending;
- infer an old POS suffix;
- guess a prefix;
- choose the nearest dictionary root;
- silently autocorrect the word.

Editor tooling may eventually offer suggestions, but suggestions are outside normative parsing.

## 8. Future derivation policy

No productive derivational morphology is defined yet.

A future derivation should only be added when all of the following are true:

1. it contributes one exact compositional semantic operation;
2. its scope is strictly local to the derived lexical expression;
3. analysis and generation are reversible;
4. it creates no surface collision with roots or other morphology;
5. its phonological realization remains pronounceable;
6. root boundaries and lexical stress remain recoverable;
7. the same meaning cannot be obtained more cleanly without obligatory morphology.

There is deliberately no speculative derivation DSL in v1. The generic engine should be extended when an actual language requirement exists.

## 9. Compiler invariants

The language package must reject detectable violations of:

```text
one valid word -> one morphological analysis
one morphological structure -> one canonical surface form
analyze(generate(x)) == x
```

The current bare-root strategy proves these properties over the declared root inventory by construction and regression tests.

## 10. CLI and site

Native tooling:

```bash
systean morphology check
systean morphology analyze sol
systean morphology generate sol
```

The website does not ask the user to identify the root manually. Its word analyzer calls the same Rust `LanguagePackage` path:

```text
word -> morphology -> phonology -> structured analysis
```

This keeps CLI, WASM, and browser behavior identical.
