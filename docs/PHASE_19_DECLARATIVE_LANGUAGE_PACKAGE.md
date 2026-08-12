# Phase 19 — Declarative surface grammar and discourse effects

Status: **implementation complete; author-toolchain validation pending**
Depends on: Phase 18
Architecture: [`SEMANTIC_DSL_ARCHITECTURE.md`](SEMANTIC_DSL_ARCHITECTURE.md)

## Goal

Finish the pre-1.0 package architecture so Systean-specific surface realization and discourse behavior are declarations over the typed Phase 18 package rather than independent TOML tables or source-name switches in Rust.

Phase 19 deliberately keeps the existing legacy `Term`/`Environment` representation only as a checker/public-output compatibility boundary. Surface ownership no longer depends on it: compiled surface bindings use typed `SymbolId` values and numeric parameter slots, and only translate to legacy names at the final compatibility boundary.

## Result

The normative language package now owns four distinct layers:

```text
typed semantic declarations
        ↓
typed reversible surface rules
        ↓
typed discourse effect programs
        ↓
generic parser / linearizer / effect runtime
```

Human dictionary metadata, API aliases, and structured-literal codec spellings do not own semantic or grammatical identity.

## Phase 19A — reversible lexical surface rules

Completed in the first half:

- `word { form ... }` compiles to one `CompiledSurfaceRule` used by parsing and canonical linearization;
- source parameter names resolve to numeric parameter slots;
- ordinary unary/binary/n-ary frames are derived from typed arity;
- precedence and associativity moved out of `syntax.toml` and into each typed rule;
- `dictionary.toml` stopped owning ordinary class/predicate/prefix/infix forms;
- `ne`, `va`, and `zo` became Systean realizations over abstract `logic.not`, `logic.and`, and `logic.or` symbols;
- structural markers such as `ki ... ku`, quotation boundaries, and text boundaries remained explicit engine-level grammar mechanisms.

## Phase 19B — generic constructions and effects

Completed in the second half:

- `@restriction` plus `bind $parameter using symbol;` expresses scoped higher-order constructions;
- `ra`, `mu`, `rov`, `mini`, and `maks` use the same generic binder representation;
- binder restrictions are accepted only when the compiled surface candidate is a unary predicate of the declared bound value type;
- `capture $payload bare;` expresses `na` without a dedicated name AST/category;
- `outer;` expresses outer-only utterance constructions such as `ke`, `da`, `me`, `emo`, focus/topic, and repairs;
- all dictionary syntax overlays were removed;
- the old author-facing `LexemeConfig`/`SurfaceFormConfig` model was removed; the internal compiled backend stores typed IDs and numeric slots;
- unsupported/fake syntax policy alternatives were removed from `syntax.toml` and `SyntaxConfig`;
- unit dimension/type ownership is entirely typed; `units.toml` retains only user/API aliases and spellings;
- typed `effect` declarations now own Systean-specific communicative behavior;
- `ConversationState` consumes generic discourse effects instead of branching on Systean roots.

## Surface DSL

### Ordinary frame

```text
word vid($observer: Entity, $observed: Entity) -> Proposition;
```

The compiler derives the reversible form:

```text
$0 vid $1
```

Parameter names are retained for source diagnostics only. The compiled surface binding uses slots `0` and `1`.

### Infix

```text
word va($left: Proposition, $right: Proposition) -> Proposition =
    logic.and($left, $right)
{
    form $left _ $right;
    precedence 20;
    associative;
}
```

The same compiled rule controls both parse grouping and canonical linearization.

### Scoped higher-order binder

```text
word ra($predicate: fn(value: Entity) -> Proposition) -> Proposition
{
    form _ @restriction;
    bind $predicate using imp;
}
```

`ra per viv` introduces one bound `Entity`, realizes `per` as the unary restriction over that value, and applies the same generic scoped argument mechanism to the body. `mu` and counted quantifiers differ only in their typed signatures/direct arguments and declared combiner.

There is no `ra`-specific parser branch and no source-name check for quantifier behavior.

### Bare capture

```text
word na($payload: Text) -> Entity
{
    form _ $payload;
    capture $payload bare;
}
```

The generic capture backend receives the declared argument slot and capture kind. `na` is not a Rust `Name` construction.

### Outer-only construction

```text
word ke($content: Proposition) -> Utterance
{
    form _ $content;
    outer;
}
```

`outer` is a generic placement property. It preserves the rule that `ke P` is an utterance-level construction while forms such as `mi ke viv` are rejected.

## Type-directed construction selection

The production parser does not rank candidates probabilistically.

For higher-order restrictions the compiled binder declares:

- the function-valued parameter slot;
- its bound value type;
- the combiner `SymbolId`;
- the direct surface argument slots.

A restriction candidate must have the required unary-predicate shape. Impossible candidates are rejected. If a future grammar extension leaves more than one complete fully typed analysis, whole-language ambiguity validation must reject the package/analysis rather than select by parser branch order.

The existing Phase 16 ambiguity corpus remains the global invariant for this rule.

## Discourse effect DSL

Language-specific communicative behavior lives in `language/typed/effects.semsys`.

Example:

```text
default effect assert;

effect assert {
    act assertion($content);
    commit $content;
}

effect ke {
    act question($content);
    choice zo;
}

effect kor {
    act correction($target, $replacement);
    repair replace $target $replacement;
}
```

The compiled generic instruction set is deliberately small:

- `Act` — classify/report the communicative act and its typed argument slots;
- `Commit` — add propositional content to the commitment state;
- `RequireContains` — require one declared target to occur structurally inside another argument;
- `Choice` — identify the declared top-level choice constructor for question classification;
- `Repair::Retract`;
- `Repair::Replace`;
- `Repair::Clarify`.

Rust implements what these generic instructions do. Rust does **not** know that `ke` means question or that `kor` means correction. Adding a new root with an existing surface/effect shape requires package declarations only.

`CommunicativeAct` remains an output/reporting structure for API compatibility and workbench presentation. It is no longer the switch that decides conversation-state behavior.

## Conversation-state mutation

`ConversationState::apply` now consumes `DiscourseEffect` values only:

- `Commit` inserts a commitment;
- `Retract` deactivates the resolved target commitment;
- `Replace` supersedes the target and inserts the replacement;
- `Clarify` records the clarification content.

Target existence/active-commitment validation remains deterministic and unchanged. The Phase 14 repair behavior is therefore preserved without Systean-root-specific branches in the conversation state machine.

## Schema cleanup

`language/syntax.toml` now contains only implemented engine-level structural policy:

- frame order;
- scope markers;
- omission policy;
- discourse structural markers;
- quote markers;
- text/utterance boundaries.

Removed public fake/obsolete policy tables include global lexical precedence, lexical roles, question/command realization switches, focus-reordering policy, traditional-POS policy, and Systean-specific pragmatics root mappings.

`language/dictionary.toml` contains metadata only. It has zero `syntax = ...` or semantic binding fields.

`language/units.toml` contains only external/user-facing aliases and surface spellings. Dimension identity, dimension value type, base relations, and exact scales belong to `typed/units.semsys`.

`language/literals.toml` remains a codec configuration because it declares the concrete written/spoken notation understood by the generic structured-literal engine. Its remaining type bindings are codec contracts, not an independent lexical/semantic registry.

## Compatibility boundary that intentionally remains

The old semantic checker/public `Term` representation still uses source/debug names and named argument maps. Phase 18 already made it a generated projection of the typed package. Phase 19 removes it from **surface ownership**: `CompiledSurfaceBinding` stores `SymbolId` and numeric slots and resolves a debug source name only when entering that legacy checker/output representation.

Deleting the legacy `Term`/`Environment` API itself is not required to make language syntax/effects declarative and would be a separate semantic-IR/API migration. No language rule is read from `language/legacy/`.

## Implementation checklist

Surface rules:

- [x] Compile typed `form` declarations to parsing and canonical linearization.
- [x] Derive ordinary frames from typed arity.
- [x] Compile precedence and associativity per rule.
- [x] Compile `na` through generic bare capture.
- [x] Compile `ra`/`mu` through generic higher-order scoped binders.
- [x] Compile counted quantifiers through the same binder path plus direct arguments.
- [x] Compile utterance-level placement through generic `outer` metadata.
- [x] Keep genuinely structural scope/quotation/text markers explicit.

Type-directed behavior:

- [x] Represent binder parameters as typed function-valued slots.
- [x] Reject impossible restriction candidates by compiled predicate shape/type.
- [x] Preserve the no-ranking ambiguity invariant.
- [x] Preserve surface/declaration provenance in the typed package and workbench package reports.

Effects:

- [x] Define the generic effect instruction set.
- [x] Package-declare assertion/question/request/command/expressive/focus/topic/repair behavior.
- [x] Remove Systean-root-specific conversation-state mutation.
- [x] Expose executed effects in workbench pragmatic output.

Ownership/schema:

- [x] Remove all dictionary surface overlays.
- [x] Remove the author-facing `LexemeConfig`/`SurfaceFormConfig` ownership model.
- [x] Store compiled surface semantic references as `SymbolId` and arguments as numeric slots.
- [x] Remove the global precedence table and unsupported/fake syntax policy alternatives.
- [x] Move unit dimension/value-type ownership into typed DSL.
- [x] Ensure no active runtime rule consumes `language/legacy/`.

## Required validation

Implemented regressions cover:

- [x] ordinary words and explicit forms sharing one compiled rule model;
- [x] alpha-renamed surface parameter names producing identical compiled slots/fingerprints;
- [x] malformed/non-invertible forms failing structurally;
- [x] parser/linearizer round trips for ordinary, infix, outer, binder, counted-binder, and capture constructions;
- [x] `ra per viv` using the generic binder path;
- [x] invalid binder restriction `ra sol viv` being rejected instead of accepted by parser order;
- [x] package-only addition of a new question root with no Rust change;
- [x] package-only addition of a new repair root with no Rust change;
- [x] generic effect runtime preserving assertion/question/focus and existing repair semantics;
- [x] native and WASM packages loading the same typed effects source and package fingerprint path.

Author-toolchain gates still required before Phase 19 is marked validated:

- [ ] `cargo test --workspace`
- [ ] `cargo run --bin systean -- check`
- [ ] `cd site && pnpm check`
- [ ] `cd site && pnpm build`

## Completion result

After Phase 19, adding an ordinary word, reversible construction, question marker, or repair marker within the existing generic primitives is package work. Rust owns typed compilation, generic surface execution, generic resolution, and generic discourse effects rather than an expanding catalog of Systean roots.
