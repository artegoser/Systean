# Phase 19 — Declarative surface grammar and discourse effects

Status: **planned**
Depends on: Phase 18
Architecture: [`SEMANTIC_DSL_ARCHITECTURE.md`](SEMANTIC_DSL_ARCHITECTURE.md)

## Goal

Finish the architectural migration by making ordinary Systean syntax and language-specific discourse/pragmatic behavior package-declared over the typed Phase 18 IR rather than encoded as growing Rust special cases.

## Surface-rule model

- [ ] Implement typed `form` declarations that compile to both parsing and canonical linearization.
- [ ] Define a generic representation for lexical root position, typed argument holes, grouping, precedence, associativity, and binder/function-valued arguments.
- [ ] Make default ordinary word frames compiler-derived from typed arity.
- [ ] Migrate prefix/infix/class/predicate special cases to generic forms where semantics permits.
- [ ] Express `ne`, `va`, `zo` through declarative forms and semantic definitions/bindings.
- [ ] Express `ra`/`mu` through typed higher-order predicate arguments rather than a quantifier-specific parser branch where possible.
- [ ] Migrate counted quantifiers, focus/topic, aspect wrappers, and speech-act surface forms to the same construction mechanism when representable.
- [ ] Keep `ki ... ku`, quotation boundaries, utterance boundaries, and other genuinely structural markers explicit where a generic lexical form is not the right abstraction.

## Type-directed elaboration

- [ ] Allow a parsed root to denote a first-class typed function where required by higher-order constructions.
- [ ] Use expected types to reject impossible candidates, never to probabilistically rank multiple valid ones.
- [ ] If two fully typed complete analyses remain, report ambiguity and fail package/runtime analysis.
- [ ] Preserve full candidate provenance for workbench diagnostics.

## Context/reference/omission

- [ ] Compile `mi`, `tu`, and future context words through declared `ContextSlotId` values.
- [ ] Compile `ref<T>` to one typed generic resolution request.
- [ ] Elaborate safe omitted arguments to the same underlying typed resolver request with different provenance.
- [ ] Keep 0/1/many resolution semantics unchanged.
- [ ] Keep exact alias resolution separate from generic shorthand search.

## Discourse/pragmatic effects

- [ ] Define a minimal stable generic effect instruction set.
- [ ] Express assertion/question/request/command/focus/topic/repair bindings as package declarations over typed semantics/effects.
- [ ] Remove Systean-specific communicative-act decisions keyed by source names from Rust.
- [ ] Preserve deterministic history, commitment, repair, frame, reference, and alias behavior.
- [ ] Keep effects inspectable in the workbench so a user/developer can see why a word changed discourse state.

## Remove fake configurability

- [ ] Audit all syntax/policy enums whose values are parsed/serialized but rejected by validation.
- [ ] Remove unsupported alternatives from the public package schema unless they are implemented in this phase.
- [ ] Keep only capabilities that the compiler can actually compile and validate.
- [ ] Document extension points explicitly rather than advertising unimplemented enum variants.

## Remove obsolete ownership

- [ ] Eliminate the semantic root inventory from `dictionary.toml` after migration.
- [ ] Eliminate semantic-form duplication from `syntax.toml` after migration.
- [ ] Eliminate `units.toml`/`literals.toml` semantic string conventions after typed declarations/codecs replace them.
- [ ] Move old files to `language/legacy/` only if they are required for migration tooling; otherwise delete them.
- [ ] Ensure no active rule depends on `language/legacy/`.

## Required validation

- [ ] Adding an ordinary primitive word requires one DSL declaration and no Rust changes.
- [ ] Adding a defined operator with an existing generic surface shape requires no Rust changes.
- [ ] Adding a new Systean question/repair root using existing semantic/effect primitives requires no Rust changes.
- [ ] Parser and generator are derived from the same surface rule and round-trip canonically.
- [ ] `ra per viv` and the complete quantifier corpus preserve canonical meaning.
- [ ] Existing precedence/grouping corpus remains deterministic.
- [ ] No parser implementation order can silently select one typed analysis over another.
- [ ] `cargo test --workspace`, whole-language check, compatibility corpus, adversarial ambiguity corpus, and WASM package compilation are green.

## Completion result

Systean-specific language growth is primarily package authoring. Rust owns a generic typed compiler/runtime, not an ever-growing list of Systean words and grammar constructions.
