# Phase 19 — Declarative surface grammar and discourse effects

Status: **in progress — Phase 19A implemented; Phase 19B remains**
Depends on: Phase 18
Architecture: [`SEMANTIC_DSL_ARCHITECTURE.md`](SEMANTIC_DSL_ARCHITECTURE.md)

## Goal

Finish the architectural migration by making ordinary Systean syntax and language-specific discourse/pragmatic behavior package-declared over the typed Phase 18 IR rather than encoded as growing Rust special cases.

## Phase split

### Phase 19A — typed surface ownership and reversible lexical forms

Implemented in the first half of Phase 19:

- typed `word { form ... }` declarations are compiled into one `CompiledSurfaceRule` owned by the typed package;
- source parameter names in forms resolve to numeric argument slots, so binder/role spelling is not canonical surface identity;
- ordinary arity-shaped words receive the same compiled surface-rule representation automatically;
- prefix, infix, class and predicate realizations are projected from those rules into the Phase 17 parser backend; outer-only speech acts already take their token order from typed `form` rules, while a temporary compatibility category preserves their outer-expression placement until 19B can enforce that constraint generically;
- precedence and associativity belong to the typed surface rule rather than `syntax.toml`;
- parser and canonical linearizer consume the same projected rule metadata;
- the ordinary-word surface overlays were removed from `dictionary.toml`; only `na`, `ra`, `mu`, `rov`, `mini`, `maks`, `ke`, `da`, and `me` remain as explicit Phase 19B compatibility overlays;
- `ne`, `va`, and `zo` now have abstract typed definitions through `logic.not`, `logic.and`, and `logic.or`, separating Systean realization from abstract semantic composition;
- context/reference/information ownership remains the structural Phase 18 model and is not moved back into surface TOML.

The existing `SurfaceExpr`/`LexemeConfig` parser is deliberately still a backend adapter in 19A. The important ownership cutover is already complete for representable ordinary lexical forms: it cannot invent or override a form independently of the typed package. Phase 19B removes the remaining quantifier/name compatibility branches and the Systean-specific discourse/effect machinery.

### Phase 19B — generic higher-order constructions, effects, and adapter removal

Still required:

- compile `na`, `ra`, `mu`, and counted quantifiers directly as generic typed constructions;
- make function-valued arguments/type-directed candidate elimination part of the production surface compiler rather than a quantifier-specific branch;
- introduce the generic discourse effect instruction set;
- package-declare assertion/question/request/command/focus/topic/repair effects;
- remove remaining Systean-specific communicative-act/source-name decisions from Rust;
- remove the obsolete `SurfaceFormConfig`/named-role compatibility adapter and fake policy alternatives.

## Surface-rule model

- [x] Implement typed `form` declarations that compile to both parsing and canonical linearization.
- [x] Define a generic representation for lexical root position, typed argument holes, grouping through precedence, precedence, associativity, and function-valued typed signatures.
- [x] Make default ordinary word frames compiler-derived from typed arity.
- [x] Migrate prefix/infix/class/predicate special cases to generic forms where semantics permits.
- [x] Express `ne`, `va`, `zo` through declarative forms and semantic definitions/bindings.
- [ ] Express `ra`/`mu` through typed higher-order predicate arguments rather than a quantifier-specific parser branch where possible.
- [ ] Finish migration of counted quantifiers and the remaining non-generic construction adapters. Focus/topic and aspect wrappers are already sourced from typed forms in 19A; outer-only speech acts already own their reversible token order in typed forms, but retain a 19B compatibility placement category because the old backend would otherwise broaden where that form may occur.
- [x] Keep `ki ... ku`, quotation boundaries, utterance boundaries, and other genuinely structural markers explicit where a generic lexical form is not the right abstraction.

## Type-directed elaboration

- [ ] Allow a parsed root to denote a first-class typed function in the production surface compiler where required by higher-order constructions.
- [ ] Use expected types to reject impossible candidates, never to probabilistically rank multiple valid ones.
- [ ] If two fully typed complete analyses remain, report ambiguity and fail package/runtime analysis.
- [ ] Preserve full candidate provenance for workbench diagnostics.

The typed IR already supports higher-order values and invocation from Phase 18; these items specifically refer to replacing the current quantifier-specific **surface parser path** in Phase 19B.

## Context/reference/omission

These invariants were structurally completed in Phase 18 and remain the production path during 19A:

- [x] Compile `mi`, `tu`, and future context words through declared `ContextSlotId` values.
- [x] Compile `ref<T>` to one typed generic resolution request.
- [x] Elaborate safe omitted arguments to the same underlying typed resolver request with different provenance.
- [x] Keep 0/1/many resolution semantics unchanged.
- [x] Keep exact alias resolution separate from generic shorthand search.

## Discourse/pragmatic effects

- [ ] Define a minimal stable generic effect instruction set.
- [ ] Express assertion/question/request/command/focus/topic/repair bindings as package declarations over typed semantics/effects.
- [ ] Remove Systean-specific communicative-act decisions keyed by source names from Rust.
- [ ] Preserve deterministic history, commitment, repair, frame, reference, and alias behavior through the effect cutover.
- [ ] Keep effects inspectable in the workbench so a user/developer can see why a word changed discourse state.

## Remove fake configurability

- [ ] Audit all syntax/policy enums whose values are parsed/serialized but rejected by validation.
- [ ] Remove unsupported alternatives from the public package schema unless they are implemented in this phase.
- [ ] Keep only capabilities that the compiler can actually compile and validate.
- [ ] Document extension points explicitly rather than advertising unimplemented enum variants.

Phase 19A has already removed the global precedence table as a source of truth. The remaining global `flatten_same_operator` compatibility setting is no longer consulted by parsing/linearization; deleting the obsolete public setting belongs to the 19B schema cleanup rather than silently changing the API halfway through the phase.

## Remove obsolete ownership

- [x] Eliminate ordinary lexical surface-form ownership from `dictionary.toml`; only nine Phase 19B compatibility overlays remain.
- [x] Eliminate precedence ownership from `syntax.toml`; precedence/associativity are compiled from typed forms.
- [ ] Eliminate the remaining quantifier/name form overlays and the generated named-role surface adapter.
- [ ] Finish eliminating `units.toml`/`literals.toml` compatibility type/name adapters where the Phase 19 typed checker makes them unnecessary.
- [ ] Move/delete any remaining old files only after no active runtime path consumes them.
- [ ] Ensure no active rule depends on `language/legacy/`.

## Required validation

- [x] Adding an ordinary primitive word with the default frame requires one DSL declaration and no Rust changes.
- [x] Adding a defined unary/binary operator with an existing generic surface shape requires no Rust changes.
- [ ] Adding a new Systean question/repair root using existing semantic/effect primitives requires no Rust changes.
- [x] Parser and generator are derived from the same typed surface rule metadata and round-trip canonically for the migrated construction set.
- [ ] `ra per viv` and the complete quantifier corpus preserve canonical meaning without the quantifier-specific compatibility branch.
- [x] Existing precedence/grouping behavior is driven by per-rule precedence/associativity rather than parser implementation order or a global TOML table.
- [ ] No parser implementation order can silently select one fully typed analysis over another after the type-directed production cutover.
- [ ] `cargo test --workspace`, whole-language check, compatibility corpus, adversarial ambiguity corpus, and WASM package compilation are green on the author toolchain for the 19A checkpoint.

## Completion result

After 19B, Systean-specific language growth is primarily package authoring. Rust owns a generic typed compiler/runtime, not an ever-growing list of Systean words and grammar constructions.
