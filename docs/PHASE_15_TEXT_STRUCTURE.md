# Phase 15 — Text, turn, section, and document structure

Status: implemented; author-toolchain validation pending.

This phase makes the already-frozen discourse boundaries executable without adding any new lexical root. The language remains config-driven: the spoken utterance boundary, written realization, readability-only punctuation, quotation delimiters, and discourse-frame marker are read from `language/syntax.toml`.

## 1. Normative boundary model

The executable hierarchy is:

```text
Document
└── channel turn metadata
    ├── Utterance
    ├── Utterance
    ├── fra  -> next discourse block / ordinary-reference frame
    └── Utterance
```

The distinctions are intentionally not all lexical:

- spoken utterance end: `du`;
- written utterance end: `.`;
- discourse block / ordinary-reference frame: `fra` in both modalities;
- turn boundary: supplied by deterministic channel metadata (`TextTurn`), with no extra spoken root because no otherwise-unrecoverable semantic distinction currently depends on a turn marker;
- document boundary: the outer `TextDocument` container. A fresh document analysis starts fresh document-scoped discourse/conversation state unless the embedding API explicitly supplies continuation state.

A pause is never an utterance boundary. Ending a channel turn is also not an implicit utterance boundary: a raw spoken turn must still end each utterance with `du`, and a raw written turn must still end each utterance with `.`.

The existing single-utterance `say` CLI command remains valid as an explicitly framed API operation. Its command envelope supplies the boundary structurally, just as a message-oriented API may do; it is not a counterexample to the raw-stream rule.

## 2. Written punctuation

`language/syntax.toml` now owns:

```toml
[text]
utterance_spoken = "du"
utterance_written = "."
readability_punctuation = [",", ":", ";", "?", "!"]
```

Readability punctuation is removed only outside opaque quotation before ordinary surface parsing. It cannot create question, command, negation, scope, focus, or any other semantic operation.

Consequences:

```text
na alfa viv?.
```

is still an assertion. The `?` is visual only.

```text
ke na alfa viv!.
```

is a truth question because of `ke`, not because of `!`.

A trailing `?` or `!` without the written utterance boundary `.` does not terminate a normative raw written utterance.

The written scanner distinguishes decimal points from utterance boundaries. A `.` between ASCII digits remains part of the structured numeric/time literal; a non-decimal `.` outside quotation is the utterance boundary.

## 3. Quotation

Boundary recognition is suspended inside the configured opaque quotation delimiters `sit ... tis`.

Therefore:

```text
na artemi gov sit hello du world tis du
```

contains exactly one spoken utterance. The inner `du` belongs to opaque `Text`.

Likewise:

```text
na artemi gov sit hello. world tis.
```

contains exactly one written utterance. The inner period belongs to opaque `Text`.

Nested `sit/tis` balancing remains owned by the existing surface quotation parser. Phase 15 does not assign semantics to typography inside opaque quoted content.

## 4. `fra`, references, and aliases

`fra` is a structural stream item, not an ordinary communicative utterance. It must occur between terminated utterances.

Applying `fra`:

1. advances `DiscourseState` to a fresh ordinary-reference frame;
2. advances the text session to a fresh discourse block/section id;
3. leaves stored historical referents intact;
4. makes old shorthand candidates invisible to ordinary `ref` and omission because those use the current frame only;
5. preserves exact alias bindings until their already-defined lexical scope ends.

Utterance boundaries and turn boundaries do **not** advance the ordinary-reference frame and do **not** expire aliases. This preserves the earlier discourse invariant that shorthand lifetime changes at declared `fra` boundaries, not after N sentences, pauses, turns, or elapsed time.

A fresh document analysis starts with fresh state. Cross-document continuation is therefore explicit rather than guessed.

## 5. Repair/history integration

Conversation history remains document-scoped and utterance-numbered (`u1`, `u2`, ...).

Turn boundaries and `fra` do not silently erase history, so an explicit `kor`, `ret`, or `klar` may target an earlier utterance across turns or discourse blocks while it remains in the same document conversation state.

Starting a fresh document with the ordinary `analyze_text_document` entry point starts fresh repair history. A repair cannot silently target an utterance from an unrelated prior document.

## 6. Atomic application

`apply_text_turn` and `analyze_text_document_with_state` stage changes on cloned text/discourse/conversation state and commit them only after successful analysis.

A malformed later turn therefore cannot leave half-applied frame changes or history entries behind.

## 7. Public core API

Phase 15 adds:

```text
TextRealization::{Spoken, Written}
TextTurn
TextDocument
TextSessionState
SectionId
parse_text_turn(...)
LanguagePackage::apply_text_turn(...)
LanguagePackage::analyze_text_document(...)
LanguagePackage::analyze_text_document_with_state(...)
```

Each analyzed utterance exposes both normative boundary realizations:

```text
canonical_spoken = <canonical surface> du
canonical_written = <canonical surface>.
```

The CLI discourse playground additionally exposes:

```text
turn spoken <turn-key> <raw spoken stream>
turn written <turn-key> <raw written stream>
```

The turn key is opaque channel metadata and does not enter semantic IR.

## 8. Required Phase 15 validation

Dedicated tests cover:

- spoken `du` / written `.` equivalence;
- mandatory explicit utterance termination even at a turn boundary;
- multi-turn parsing without a lexical turn marker;
- `fra` shorthand retirement with exact-alias preservation;
- no reference/alias lifetime change at `du` or turn boundaries;
- boundary opacity inside `sit ... tis`;
- readability punctuation being semantically inert;
- decimal points not splitting utterances;
- repair across turns and `fra` within one document;
- fresh document repair history;
- atomic rollback after a late structure error;
- CLI spoken/written turn integration.

The roadmap validation boxes remain open until these tests, the full workspace suite, root audit, WASM build, Svelte checks, and production site build pass on the author toolchain.
