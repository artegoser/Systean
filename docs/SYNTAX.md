# Systean Surface Syntax and Scope

> **Pre-1.0 architecture note:** This document also describes the current Phase 17 implementation where applicable. The accepted target ownership/IR/DSL model is [`SEMANTIC_DSL_ARCHITECTURE.md`](SEMANTIC_DSL_ARCHITECTURE.md); Phases 18–19 migrate away from duplicated/stringly semantic ownership without changing already accepted language behavior unless explicitly revised.

Status: **structural v1 + typed discourse, proper names, opaque quotation, and playable core vocabulary implemented**

This document defines the current normative structural rules for Systean surface syntax. Structural policy lives in `language/syntax.toml`; lexical roots, semantic identities, and root-specific surface realizations live once in `language/dictionary.toml`. The generic parser/generator/lowering engine lives in `systean-core::syntax`.

The syntax engine does **not** invent grammatical roots. The following core forms were manually selected and are now normative:

| Form | Function | Semantic binding |
| --- | --- | --- |
| `ki` | open explicit scope | structural delimiter |
| `ku` | close explicit scope | structural delimiter |
| `sit` | open opaque quotation | structural delimiter |
| `tis` | close opaque quotation | structural delimiter |
| `ke` | truth-question marker | `ask_truth` |
| `ne` | negation | `not` |
| `va` | conjunction | `and` |
| `zo` | disjunction | `or` |
| `ra` | universal quantifier | `forall` |
| `mu` | existential quantifier | `exists` |
| `da` | command marker | `command` |
| `me` | request marker | `request` |
| `ref` | typed shorthand reference | runtime discourse resolver |
| `mi` | current speaker | runtime context `speaker` |
| `tu` | current addressee | runtime context `addressee` |
| `na` | proper-name marker | `proper_name` |

`ke`, `ne`, `va`, `zo`, `ra`, `mu`, `da`, `me`, `ref`, `mi`, `tu`, and `na` are declared once as lexical roots in `dictionary.toml`; the package compiler derives their surface lexicon entries from those same dictionary entries. `ref` is the universal typed shorthand-reference form; `mi` and `tu` are deterministic runtime-context values for the current speaker and addressee. `na` introduces one canonical Systean proper-name payload. `ki`/`ku` and `sit`/`tis` are reserved structural delimiters rather than lexical roots.

Test fixtures still provide temporary content vocabulary such as people, predicates, and classes. Those fixture-only forms must never be treated as normative Systean vocabulary.

## 1. Core rule

Surface syntax exists to expose one semantic structure without making ordinary speech sound like serialized IR.

The default strategy is therefore:

> **Use canonical order when it uniquely determines structure. Add an explicit marker only when structure would otherwise differ.**

The parser never chooses a structure from plausibility, world knowledge, or statistical preference.

## 2. Canonical frame order

The default clause order is:

```text
PRIMARY PARTICIPANT -> PREDICATE -> REMAINING FRAME ARGUMENTS
```

A predicate root's dictionary entry declares the exact semantic role of the primary participant and the exact ordered list of remaining roles.

Conceptually:

```text
John see Mary
```

may lower to:

```text
see(observer = John, observed = Mary)
```

The engine does not infer those roles from the nouns. The syntax binding declares them.

Free unmarked word-order permutation is disabled. A future focus/topic construction may mark information structure explicitly, but it must not silently reinterpret arbitrary word order.

## 3. No traditional POS grammar

The surface grammar does not depend on mandatory `noun`, `verb`, or `adjective` endings.

Lexical roots remain bare under morphology v1. Constant roots automatically become surface atoms. Operator roots attach one explicit structural realization in the same dictionary entry. No root is re-declared in `syntax.toml`.

## 4. Scope markers

Systean retains the existing universal spoken/written delimiters:

```text
ki ... ku
```

They are semantic grouping markers: the spoken equivalent of explicit structural grouping.

They are reserved and may not simultaneously be lexical roots. The language-package loader checks this.

Grouping is not emitted merely for decoration. The canonical generator removes redundant grouping and inserts `ki ... ku` when it is required to preserve the intended syntax tree.

## 5. Local scope from order

When order already yields exactly one structure, extra delimiters are unnecessary.

The chosen rule for negation and quantifiers is equivalent to:

```text
NOT EVERY person clause
```

having a different structural order from:

```text
EVERY person NOT clause
```

The first introduced scope-bearing construction is wider unless explicit grouping establishes another structure.

The parser therefore preserves distinctions equivalent to:

```text
not(forall(...))
```

and:

```text
forall(... not(...))
```

without requiring spoken brackets around every operator.

## 6. Quantifier scope

Default quantifier nesting follows order of appearance.

Conceptually:

```text
every person see some dog
```

has default nesting equivalent to:

```text
forall person:
    exists dog:
        see(person, dog)
```

The syntax engine implements this by introducing binders in surface order and lowering them in reverse construction order around the base predicate.

The exact surface mechanism for intentionally reversing quantifier scope is not frozen yet because it depends on the still-unresolved surface reference/binder system. `ki` / `ku` provide universal proposition grouping, but the engine must not invent an implicit way to refer back to a bound entity.

## 7. AND / OR precedence

Systean intentionally has one logical precedence rule:

> **AND binds more tightly than OR.**

The canonical policy assigns a higher precedence to semantic `and` than semantic `or`.

Therefore structures equivalent to:

```text
A and B or C
```

parse as:

```text
(A and B) or C
```

and:

```text
A or B and C
```

parse as:

```text
A or (B and C)
```

To request the opposite grouping, use `ki ... ku`.

Chains of the same surface operator are flattened in the surface AST:

```text
A and B and C
```

becomes one deterministic `and` chain rather than requiring arbitrary spoken binary brackets. The current binary semantic IR lowers such a chain deterministically; the surface structure remains n-ary.

## 8. Explicit speech acts

Questions, commands, and requests are explicit constructions. They are not encoded by inversion or predicate inflection.

The syntax engine supports configurable prefix speech-act bindings that lower to semantic operators over the proposition they contain.

`ke`, `da`, and `me` are the normative markers for truth questions, commands, and requests respectively.

## 9. Semantic roles

Role markers are not pronounced by default when canonical frame order already identifies each role uniquely.

This does not make roles implicit in semantics. The parser's construction rule maps each surface position to one named semantic role.

A future marked construction may expose roles explicitly if it enables a useful operation such as focus without ambiguity. There is no free-order fallback in v1.

## 10. Argument omission

The accepted design rule is:

> An argument may be omitted only if the formal discourse/reference state yields exactly one valid referent.

The parser now preserves structurally recoverable omissions as typed unresolved-reference slots rather than guessing a value during parsing. Elaboration obtains the exact semantic role and expected type from the predicate signature. Runtime `DiscourseState` resolution then applies the same 0/1/many rule used by explicit `ref`:

- zero accessible compatible referents -> unresolved-reference error;
- exactly one -> deterministic resolution;
- more than one -> ambiguity error with candidate diagnostics.

Only omissions whose slot is structurally identifiable are accepted. In particular, omission is permitted at a deterministic clause boundary (end of expression, scope close, or infix boundary); the parser does not skip an arbitrary middle argument and then guess which role was absent.

The discourse-free `analyze_surface` path deliberately rejects expressions that still contain unresolved references or runtime-context values. `analyze_surface_with_discourse` performs typed elaboration and deterministic resolution. No recency, salience, plausibility, or world-knowledge fallback exists.

## 11. Focus and modifiers

Unmarked word order cannot be rearranged for focus.

A future focus construction must be explicit and must preserve the canonical underlying frame-role mapping.

Likewise, relations/modifiers must structurally identify their target. Systean will not adopt natural-language-style ambiguous attachment such as an unmarked phrase that could modify either an entity or an event.

## 12. Typed-word surface lexicon

Phase 19A moves ordinary lexical surface ownership into the same typed package that already owns semantic identity. `dictionary.toml` is learning/documentation metadata and no longer declares class, predicate, prefix, or infix frames. Three outer-only speech-act overlays remain temporarily only as backend placement categories; their root/argument order is already owned and validated by typed `form` rules. The category carries a restriction that an unrestricted prefix projection would lose.

An ordinary typed word uses its compiler-derived default frame:

```text
word vid($observer: Entity, $observed: Entity) -> Proposition;
```

which compiles to the indexed reversible rule:

```text
$0 vid $1
```

Source parameter names (`$observer`, `$observed`) remain useful diagnostics/documentation labels, but the compiled surface rule stores numeric argument slots. Renaming them does not change the surface fingerprint.

A non-default realization is declared beside the word:

```text
word ne($value: Proposition) -> Proposition = logic.not($value) {
    form _ $value;
}

word va($left: Proposition, $right: Proposition) -> Proposition = logic.and($left, $right) {
    form $left _ $right;
    precedence 20;
    associative;
}
```

`_` denotes the lexical root. Every parameter must occur exactly once and the root exactly once. Invalid/missing/duplicate holes fail typed-package compilation. Infix precedence and associativity are properties of this rule; `syntax.toml` no longer owns an operator precedence table. Canonical parsing and linearization receive the same compiled rule metadata.

For Phase 19A the current Phase 17 `SurfaceExpr` parser remains a generated backend projection. It can project default frames, prefix forms, and binary infix forms from typed rules. Outer-only speech acts get their root/argument order from typed rules too, but still use a compatibility backend category until the generic Phase 19B parser can enforce the same placement constraint directly. This is a compatibility implementation detail, not a second authoring source.

Exactly nine dictionary overlays remain until Phase 19B: `na`, `ra`, `mu`, `rov`, `mini`, `maks`, `ke`, `da`, and `me`. The three speech-act overlays preserve only the current outer-expression placement rule; their `form _ $content` order already lives in typed DSL. Treating them as unrestricted Phase 17 prefixes would incorrectly accept forms such as `mi ke viv`. The name/quantifier overlays require generic construction parsing; the three speech-act overlays require generic typed placement/effect handling. Phase 19B removes both classes of adapter. No migrated ordinary root may reintroduce a `syntax = ...` overlay.

The Systean roots `ne`, `va`, and `zo` are additionally defined over abstract typed primitives (`logic.not`, `logic.and`, `logic.or`), so their concrete roots and their abstract compositional semantics are no longer the same accidental symbol.

At package load time the whole-language compiler still enforces exact dictionary↔typed-word coverage, root legality/phonology/reserved-token ownership, and surface-rule validity.

## 13. Parser and generator invariants

The surface subsystem targets:

```text
parse(linearize(ast)) = ast
```

for the supported canonical surface AST.

The regression suite covers:

- canonical primary/predicate/rest frame order;
- rejection of unmarked reordered arguments;
- `not every` versus `every ... not`;
- quantifier nesting by order of appearance;
- `AND > OR` precedence;
- `ki ... ku` precedence override;
- removal of redundant grouping;
- insertion of grouping when required;
- same-operator chain flattening;
- explicit question/command/request constructions;
- automatic dictionary-root visibility in surface syntax;
- dictionary-only addition of new constant roots without syntax-config edits;
- surface-to-semantic type checking, including wrong-type operator application;
- proper-name payload validation and writing/speech round-trip;
- opaque and nested external quotation without lexical parsing of its payload.

## 14. Planned higher syntax/discourse layers

The structural surface grammar in this document is implemented. The architecture of the remaining higher layers is now defined in [`FINAL_ARCHITECTURE.md`](FINAL_ARCHITECTURE.md) and scheduled in [`IMPLEMENTATION_ROADMAP.md`](IMPLEMENTATION_ROADMAP.md).

The typed discourse layer, scoped aliases/boundaries, proper-name construction, opaque quotation, and the first playable content vocabulary are now implemented. `ref`, omission, aliases, `mi`/`tu`, `na`, and `sit ... tis` all flow through the same validated language package.

Structured numbers/quantities, time/aspect, unknown-information constructions, generic/statistical claims, and the Phase 12–14 conversational pragmatics/affect/repair layer are now implemented. Remaining higher work is explicit utterance/document stream boundaries and the whole-language ambiguity compiler. No remaining layer may introduce heuristic parsing.

## Phase 3 discourse control layer

The surface parser remains responsible for ordinary expressions. Multi-utterance control forms are handled by the discourse layer above it, using markers declared in `language/syntax.toml`:

```toml
[discourse]
alias = "ali"
definition = "def"
relative = "rel"
frame = "fra"
```

They are structural forms, not global dictionary roots.

### Exact aliases

`ali` binds a local alias to one semantic value that is already represented by exactly one accessible discourse referent:

```text
ali A VALUE
```

The target is matched by exact canonical semantic value inside the current ordinary-reference frame. Zero matching referents fails. Multiple matching referents fails. After binding, `A` is exact and does not participate in 0/1/many shorthand search.

Alias spellings are validated as root-like spoken tokens against the fixed alphabet, existing lexical roots, reserved structural markers, and exact root-pronunciation collisions. An alias is not inserted into `dictionary.toml` or the global semantic environment.

Aliases are typed. They may refer to `Entity`, `Proposition`, `Event`, or any other declared semantic value. The current alias table is compiled into a temporary surface lexicon for each discourse-aware parse, so an alias can occupy any slot compatible with its declared semantic type.

Nested lexical scopes may shadow an outer alias with the same spelling. Leaving the inner scope deterministically restores the outer binding. A local binding never escapes the scope in which it was declared.

### Local definitions

`def` introduces a new local semantic value and binds an alias to it in the current lexical scope:

```text
def A VALUE
```

Unlike `ali`, `def` does not require `VALUE` to have been introduced previously as an ordinary shorthand candidate.

### Relative/local binding

`rel` creates a temporary nested lexical binding for one body:

```text
rel A ki TARGET ku ki BODY ku
```

`TARGET` is evaluated first. A fresh lexical scope is then entered, `A` is bound to that semantic value, and `BODY` is evaluated inside that scope. The scope is always left after the body completes or fails, so the temporary alias cannot leak into following discourse.

### Discourse-frame boundary

`fra` advances the ordinary-reference frame:

```text
fra
```

After the boundary, ordinary `ref` resolution and omitted arguments see only referents introduced in the new frame. Older referents remain stored because an exact alias may still point to them until the alias's lexical scope ends. This separates short-term shorthand lifetime from explicit alias lifetime without time, recency, sentence-count, or salience heuristics.

### Canonical resolved surface

Discourse-aware analysis now exposes two canonical surfaces:

- `canonical_surface`: normalization of the submitted surface structure;
- `canonical_resolved_surface`: a safe explicit realization after discourse resolution.

A successfully omitted argument is materialized as the canonical explicit `ref` root in the resolved surface. An explicit exact alias remains that alias. If a reference cannot be realized unambiguously from the current discourse state, generation fails rather than emitting a guessed shorthand.

### Discourse playground

The CLI exposes the same state machine through:

```text
systean discourse
```

It is both interactive and pipe/script friendly. Useful commands include `context`, `intro`, `intro-sem`, `analyze`, `resolve`, `bind`, `scope enter`, `scope leave`, `state`, plus the configured `ali`, `def`, `rel`, and `fra` forms. Analysis output includes resolved context values, aliases, shorthand-reference targets, canonical semantic IR, and canonical resolved surface.


## Phase 4 name and quotation layer

Proper names use the selected lexical marker `na` followed by exactly one canonical Systean spoken payload token:

```text
na artemi
```

The payload is not looked up as a dictionary root. The language package validates it directly against the fixed alphabet and root phonology, including the requirement that a root-like payload contain a vowel so lexical stress is defined. Source-language spelling or pronunciation is never guessed. Native and externally adapted names therefore use one grammatical mechanism: the author/speaker supplies the intended canonical Systean payload explicitly.

`na PAYLOAD` lowers through the package-defined semantic operator `proper_name(payload: Text) -> Entity`. Equal name payloads do not imply one discourse referent: two separately introduced people named `na alek` receive different referent IDs, and generic `ref` remains ambiguous until ordinary structural refinement or an exact alias distinguishes them.

External text uses the reserved structural boundaries:

```text
sit ... tis
```

The text between the matching boundaries is captured as an opaque `Text` literal before ordinary Systean lexical parsing. Foreign spelling, punctuation, digits, and otherwise invalid Systean tokens inside that payload are preserved rather than interpreted as roots. Nested `sit ... tis` pairs are balanced deterministically and remain literal boundary text inside the outer payload. A missing or stray boundary is a structural parse error.

Quotation boundaries are configured in `syntax.toml`; they are reserved against dictionary-root and local-alias collisions. The WASM syntax-policy API exposes the same boundaries used by the Rust parser.


## Phases 12–14 communicative and repair layer

The executable surface package now includes the selected lexical forms `emo`, `fok`, `top`, `ret`, `kor`, and `klar` in `dictionary.toml`. Their semantic operators are declared in `.semsys`; global communicative operator ownership is mapped through `[pragmatics]` in `syntax.toml`.

Ordinary surface syntax still lowers to literal proposition/utterance semantics. `LanguagePackage::analyze_utterance*` adds the deterministic communicative classification layer: an unmarked proposition becomes a default assertion, `ke` is classified as truth/value/choice from explicit typed structure, and `da`/`me` remain distinct.

`TARGET fok P` and `TARGET top P` are explicit infix constructions. Communicative analysis requires `TARGET` to be structurally present in `P`; neither construction may reassign semantic roles or silently alter logical scope.

Phase 14 repair forms use exact positive conversation-history numbers: `ret N`, `N kor P`, and `N klar P`. The parser only constructs their explicit semantic terms. `ConversationState` owns history and commitment transitions, preserving immutable historical analyses while applying corrections/retractions deterministically.

The CLI `discourse` playground exposes the full layer through `say`, `history`, and `commitments`. See [`PHASES_12_14.md`](PHASES_12_14.md).

## Phase 15 text-stream boundaries

Raw discourse streams now have an executable boundary layer. Spoken utterances terminate with configured `du`; written utterances terminate with configured `.`. Channel turns are explicit metadata containers and do not terminate an unfinished utterance. `fra` is recognized between utterances and advances the ordinary-reference frame/section. `du`, `.`, and `fra` inside `sit ... tis` remain opaque quoted text and do not split the stream.

Readability punctuation `, : ; ? !` is semantically inert outside quotation. In particular, `?` does not create a question and `!` does not create a command. Decimal points between digits remain part of structured literals rather than becoming utterance boundaries.

Utterance and turn boundaries preserve the current shorthand frame and alias scope. `fra` retires ordinary shorthand candidates by advancing the frame while exact aliases retain their previously defined lexical lifetime. Conversation repair history remains available across turns and `fra` inside one document; a fresh document starts fresh history by default.

The executable details and tests are recorded in [`PHASE_15_TEXT_STRUCTURE.md`](PHASE_15_TEXT_STRUCTURE.md).
