# Phases 12–14 — Conversational Pragmatics, Subjective Experience, and Repair

Status: **validated on the author toolchain**

This document records the executable design added in Phases 12–14. The normative lexical choices were frozen before implementation; this phase does not invent additional roots.

## 1. Communicative layer

`Proposition` and `Utterance` remain distinct.

The ordinary surface analyzer continues to return the literal semantic value of the expression. A separate communicative analysis classifies that value as a conversational act and produces the canonical utterance-level term.

An unmarked top-level `Proposition` is therefore interpreted communicatively as:

```text
assert(content = P)
```

The assertion wrapper is a communicative-layer operation. It does not rewrite `P` in the ordinary surface/semantic analyzer.

The configured communicative operators and roles live in `[pragmatics]` in `language/syntax.toml`; Rust does not hardcode the Systean spellings.

## 2. Phase 12 — speech acts

### 2.1 Truth questions

`ke P` remains an explicit truth question:

```text
ke na alfa viv
→ ask_truth(content = alive(entity = proper_name(payload = "alfa")))
→ truth_question
```

Intonation is not part of the classification rule.

### 2.2 Value questions

A value question is the existing `ke` construction whose content contains at least one explicit typed `unk` value. The requested semantic type is recovered from that typed slot after elaboration/resolution.

```text
ke na artemi vid unk na mari
```

The unknown value occupies the typed `observed: Entity` role. No interrogative inversion or implicit question word is required.

### 2.3 Choice questions

A choice question is `ke` over an explicit disjunction. When grouping is required by the current surface grammar, `ki/ku` makes the question scope explicit:

```text
ke ki na alfa viv zo na beta viv ku
```

The alternatives are recovered structurally from the canonical `or(...)` term. The same disjunction without `ke` is an assertion, not a question.

A single baseline question may not simultaneously contain an explicit requested `unk` slot and use a top-level choice disjunction. That mixed structure is rejected rather than assigning a priority between two question classes.

### 2.4 Answers

The Phase 12 baseline does not create separate yes/no answer roots. An answer carrying propositional content is an ordinary assertion of that content (or its explicit negation). A value answer is likewise an assertion/construction containing the explicit value required by the surrounding discourse.

This avoids a second answer vocabulary whose interpretation would depend on the previous question type.

### 2.5 Commands and requests

`da P` and `me P` remain distinct semantic operators:

```text
da P → command(content = P)
me P → request(content = P)
```

Neither is derived from prosody or word order.

## 3. Phase 13 — subjective experience and expressivity

All 86 roots frozen in [`SUBJECTIVE_STATES.md`](SUBJECTIVE_STATES.md) are now executable dictionary entries.

The semantic hierarchy is:

```text
SubjectiveExperience <: Proposition
SubjectiveState      <: SubjectiveExperience
SubjectiveState      <: State
SubjectiveEvent      <: SubjectiveExperience
SubjectiveEvent      <: Event
Affect               <: SubjectiveState
```

This lets an experienced subjective state/event be asserted as explicit content while retaining occurrence typing for state/event-sensitive constructions.

`orgaz` is deliberately a `SubjectiveEvent`; ordinary affective roots return `Affect`; bodily and motivational state roots return `SubjectiveState`.

### 3.1 Experienced state versus expression

The experienced state and its explicit expression are separate structures:

```text
mi felis 0.8
→ affect_felis(experiencer = speaker, intensity = 0.8)
→ default assertion at the communicative layer
```

versus:

```text
emo mi felis 0.8
→ express_affect(state = affect_felis(...))
→ expressive
```

`emo` accepts an explicit `SubjectiveExperience`. It never infers an emotion from prosody and never negates or otherwise rewrites the nested literal content.

### 3.2 Explicit arguments

Every frozen subjective root has an explicit `experiencer` and `intensity: Number` role. Roots whose frozen meaning is inherently directed additionally require the declared target (`Entity` or `Proposition`, according to the lexical meaning).

A missing semantically relevant target is not guessed. Existing explicit unknown/unspecified mechanisms remain available in typed slots when the speaker intentionally does not provide a concrete value.

No weak/strong lexical duplicate is introduced merely to encode intensity.

### 3.3 Important preserved distinctions

The executable semantic operators preserve the frozen distinctions, including:

```text
amori != romat != eros != eruz != libid != lusta
prid  != superb
```

The seven-deadly-sins coverage remains compositional and does not add duplicate senses for `envi` or `furor`.

### 3.4 Focus and topic

The selected explicit forms are:

```text
TARGET fok PROPOSITION
TARGET top PROPOSITION
```

Both return `Utterance` structures. Their target must occur structurally in the declared proposition; otherwise communicative analysis rejects the construction.

The proposition itself is retained unchanged. `fok` therefore cannot swap semantic roles, and `top` cannot silently change quantifier/logical scope.

## 4. Phase 14 — repair protocol

The repair roots are executable as:

```text
ret N        retract utterance/commitment N
N kor P      correct commitment N with proposition P
N klar P     clarify history entry N with proposition P
```

`N` is a positive whole conversation-history identifier represented by the ordinary exact `Number` system. History identifiers are 1-based (`u1`, `u2`, ... in tooling output).

### 4.1 Immutable history

`ConversationState` stores every accepted communicative analysis as a `HistoryEntry` containing:

- stable `UtteranceId`;
- original source;
- canonical resolved surface;
- complete `PragmaticAnalysis` and canonical utterance term.

Repair never edits or replaces an existing history entry.

### 4.2 Commitments

Assertions, focus/topic utterances, corrections, and clarifications can create explicit commitments. Questions, commands, requests, and expressives do not automatically create propositional commitments.

A correction deactivates the current active commitment in the referenced correction chain and creates a replacement commitment. A retraction explicitly deactivates that current commitment. A clarification preserves the target history and adds its own explicit content.

### 4.3 Deterministic serial repair

Correction chains use stable IDs and explicit `superseded_by` links. Resolving a correction target follows only this formal chain:

```text
u1 -> u2 -> u3
```

There is no recency guess. Referencing `u1` after serial corrections deterministically reaches the one active descendant, if one exists. Referencing a nonexistent target, a target with no commitment, or a fully retracted chain is an error.

## 5. Tooling

The CLI discourse playground now provides:

```text
say <surface-expression>
history
commitments
```

`say` runs full surface/discourse/semantic/pragmatic analysis and then applies the deterministic conversation-state transition.

The WASM package embeds the same semantic sources and exposes `analyze_utterance_json` in addition to ordinary surface analysis. The browser layer therefore does not reimplement speech-act classification.

## 6. Validation status

Dedicated tests exist for:

- Phase 12 assertion/truth/value/choice question classification, answers, command/request separation;
- Phase 13 all 86 roots, affect/expression separation, sexual-state distinctions, bodily/event typing, focus/topic invariants, no implicit sarcastic negation;
- Phase 14 retraction, correction, clarification, immutable history, missing targets, serial/nested correction chains;
- CLI integration across all three phases.

The dedicated Phase 12–14 suites, full workspace regression suite, root audit, WASM build, Svelte checks, production site build, and scripted conversation smoke test passed on the author toolchain on 2026-08-11.
