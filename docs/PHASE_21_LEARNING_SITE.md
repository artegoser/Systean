# Phase 21 — User-facing learning site and interactive analyzer

Status: **planned**
Depends on: Phase 20

## Goal

Turn the website from an engine/workbench inspector into a product for people who want to learn, search, read, and understand Systean.

The site remains a thin consumer of the Rust/WASM language package. It must not invent a parallel parser, semantic model, English translator, or dictionary schema.

The default UI must not expose raw JSON, internal enum names, serialized Rust DTOs, numeric IDs, or provenance blobs as the primary explanation of language behavior.

Developer diagnostics may remain available behind an explicitly technical view, but the public learning experience uses human explanations.

---

## 1. Information architecture

Primary user-facing sections:

```text
Learn / Guide
Dictionary
Analyzer
Examples / Corpus
About the language / specification links
```

Developer/package inspection is secondary and visually separated from learning content.

---

## 2. Dictionary search

Dictionary search becomes a first-class indexed interface rather than a flat list with substring matching.

### Search targets

- [ ] exact/prefix Systean root;
- [ ] English gloss;
- [ ] English explanation text;
- [ ] aliases/search keywords declared only for documentation discovery;
- [ ] semantic/domain tags;
- [ ] example text.

### Filters

At minimum:

- [ ] declaration kind: primitive / defined / intrinsic-backed;
- [ ] semantic result type;
- [ ] argument count/arity;
- [ ] accepted argument types where useful;
- [ ] semantic domain/documentation tags;
- [ ] words vs structural constructions/markers where that distinction exists in the compiled package;
- [ ] stable/public vs experimental vocabulary if package stability metadata is introduced.

Traditional noun/verb/adjective POS filters should not be invented unless Systean formally defines such a category.

### Sorting and navigation

- [ ] root alphabetical/order;
- [ ] English gloss alphabetical;
- [ ] relevance for free-text search;
- [ ] URL-addressable filters/query state;
- [ ] keyboard navigation;
- [ ] mobile-friendly result layout.

### Result cards/rows

Each compact result shows:

- root;
- pronunciation/stress affordance;
- English gloss;
- concise type/signature summary in human language;
- selected semantic/domain tag(s) when useful.

No `JSON.stringify(entry.semantic)` or equivalent raw object dump appears in the normal dictionary UI.

---

## 3. Word detail view

Clicking a word opens a dedicated page or stable side panel containing:

- [ ] root and pronunciation;
- [ ] stress/syllabification;
- [ ] short English gloss;
- [ ] detailed English explanation;
- [ ] semantic declaration kind: primitive / defined / intrinsic-backed;
- [ ] human-readable typed signature;
- [ ] explanation of each argument and its position in canonical surface order;
- [ ] all declared surface realizations/construction patterns;
- [ ] relevant precedence/associativity/scope behavior;
- [ ] context/reference/discourse effects where applicable;
- [ ] canonical examples with English rendering;
- [ ] contrasts/related words when documentation declares them;
- [ ] links to the corresponding formal specification/declaration for advanced users.

Because Systean v1 morphology is bare-root, “all forms” means all declared **surface realizations/construction patterns and contextual uses**, not fake inflection tables that the language does not have.

---

## 4. Analyzer: primary presentation

After analysis, the top of the page should answer the user’s actual questions first:

1. What does this expression mean in English?
2. How is the Systean expression structured?
3. What does each word contribute here?
4. What scopes/references/context values matter?
5. If invalid, exactly where and why?

Recommended primary output order:

```text
Systean expression
English rendering
interactive annotated expression
human-readable structure/scope explanation
reference/context/discourse explanation when relevant
errors/warnings
```

Raw semantic IR/AST/provenance is moved to an optional advanced/developer section.

---

## 5. Token hover behavior

Every analyzable Systean token becomes interactive.

Hover/focus on a word shows a compact context-sensitive card with:

- [ ] root;
- [ ] English gloss;
- [ ] one/two-sentence explanation of what it means **in this expression**;
- [ ] its semantic contribution in human terms;
- [ ] its current argument/role when applicable;
- [ ] scope owned/modified by the token;
- [ ] context/reference resolution result where applicable;
- [ ] concise note about discourse effect where applicable.

The hover must be context-sensitive. It must not merely repeat the dictionary definition.

Examples of interaction behavior:

### Ordinary relation

For `vid` in a two-participant expression:

- highlight the observer participant;
- highlight the observed participant;
- show arrows/links from `vid` to both participants;
- explain which argument each participant fills.

### Quantifier

For `ra`:

- highlight the restriction expression;
- highlight the body/scope expression;
- visually show the scope boundary;
- explain the resulting controlled-English reading.

### Negation

For `ne`:

- highlight exactly the proposition it negates;
- ensure the English card reflects that scope.

### Reference/context

For `ref`, omitted arguments, `mi`, or `tu`:

- highlight the resolved semantic value/referent;
- show whether resolution came from explicit context, alias, exact binding, or unique typed shorthand;
- on ambiguity, show all compatible candidates instead of a guessed winner.

### Repair/focus/topic

- highlight both the propositional content and the discourse/history target where relevant;
- explain the effect separately from the base proposition.

Hover interaction must also work with keyboard focus and touch through an equivalent tap/focus affordance.

---

## 6. Token click behavior

Clicking a token pins the contextual explanation and opens the complete word detail panel.

The panel combines:

```text
contextual use in current expression
+ canonical dictionary documentation
+ declared surface forms
+ examples
```

The user can move from “what does this token do here?” to “teach me this word generally” without losing the analyzed expression.

---

## 7. Visual relation/scope model

The analyzer should render semantic relationships in a human-readable way rather than drawing a generic raw AST.

Required primitives:

- participant-to-relation links;
- scope brackets/ranges;
- quantifier restriction/body distinction;
- reference/alias resolution links;
- context-origin markers;
- discourse-history links for correction/retraction/clarification;
- structured-value breakdown for numbers/quantities/time when requested.

The visualization must derive from structured workbench annotations produced by Rust/WASM. Svelte does not reconstruct semantic dependencies by inspecting token strings.

---

## 8. English rendering in the analyzer

The deterministic Phase 20 English renderer is displayed prominently.

- [ ] English rendering is tied to the canonical semantic result, not surface word substitution.
- [ ] Hovering a Systean token may highlight the corresponding English phrase/span when a stable mapping exists.
- [ ] Hovering an English span may highlight the contributing Systean token/construction where the renderer can expose provenance.
- [ ] When one Systean construction maps to a discontinuous or reorganized English phrase, the UI may highlight multiple spans rather than pretending there is a one-to-one word alignment.
- [ ] English rendering provenance is exposed structurally by the engine/renderer where feasible.

No arbitrary multilingual translation UI is required for 1.0. English is the fixed bridge language.

---

## 9. Diagnostics

Errors are displayed as user explanations, not engine serialization.

Each diagnostic should provide:

- [ ] highlighted source span/token;
- [ ] short plain-English problem statement;
- [ ] expected structure/type where useful;
- [ ] actual encountered structure/type;
- [ ] ambiguity candidates when applicable;
- [ ] direct links to relevant dictionary/spec explanations;
- [ ] safe correction examples only when deterministic and not misleading.

Advanced details may expose failing compiler layer and provenance after the user expands them.

---

## 10. Learning guide

- [ ] Build a guided path from pronunciation and bare-root morphology through ordinary frames, scope, quantifiers, reference/context, structured values, speech acts, and repair.
- [ ] Every lesson uses executable examples analyzed by the same engine.
- [ ] Examples fail the site/documentation build if they stop parsing or their frozen intended canonical meaning changes unexpectedly.
- [ ] Lessons deep-link to dictionary entries and analyzer examples.

---

## 11. API/workbench requirements for the site

Before UI implementation, extend the Rust workbench so the website receives user-level annotations directly.

Required structured fields include:

- token-to-symbol mapping;
- token-to-English documentation mapping;
- contextual semantic contribution summary inputs;
- argument/dependency edges;
- scope ownership/ranges;
- reference/context/alias resolution edges;
- discourse effect/history targets;
- English-rendering text plus provenance/alignment spans where possible;
- all declared surface forms for a selected symbol;
- dictionary search/filter indexes.

The UI must not infer these by parsing formatted semantic text.

---

## 12. Raw technical views

The current Phase 17 workbench data remains valuable for development, but it moves behind an explicit advanced surface such as:

```text
Advanced details
Developer inspector
/debug/workbench
```

Rules:

- [ ] no raw JSON by default;
- [ ] no raw numeric IDs without resolved labels;
- [ ] no serialized Rust enum presented as a word definition;
- [ ] JSON export/copy may remain as a developer feature;
- [ ] developer views consume the same report, not a separate implementation.

---

## 13. Accessibility/responsiveness

- [ ] token interactions are keyboard accessible;
- [ ] hover-only information has focus/tap equivalents;
- [ ] dependency/scope meaning is not encoded only by color;
- [ ] mobile layout keeps analyzed text and pinned word details usable;
- [ ] pronunciation controls have textual labels/state;
- [ ] dictionary filters remain usable without precision pointer interaction.

---

## 14. Required validation

### Dictionary

- [ ] search finds by root, gloss, explanation, and tags;
- [ ] every filter is backed by compiled/indexed data;
- [ ] URL state round-trips;
- [ ] every public root opens a complete detail view;
- [ ] no public word page is missing required English documentation.

### Analyzer

- [ ] every token in representative corpus exposes context hover/focus data;
- [ ] participant/operator dependency highlighting matches canonical semantics;
- [ ] quantifier/negation/grouping scope highlights match canonical scope;
- [ ] reference resolution links match engine resolution;
- [ ] ambiguity displays candidates without ranking;
- [ ] English rendering equals the deterministic Phase 20 renderer output;
- [ ] click-through detail view preserves analyzed context;
- [ ] no raw JSON is required to understand a valid or invalid expression.

### Build/integration

- [ ] `pnpm check` passes;
- [ ] production build passes;
- [ ] WASM package fingerprint matches native package semantics/surface fingerprints;
- [ ] browser analyzer corpus matches native workbench results;
- [ ] accessibility tests cover keyboard/focus behavior for token interactions.

## Completion result

A learner can search a word, understand it, analyze a real Systean expression, inspect exactly what every token contributes in context, see its relationships/scope/references, and read a deterministic English interpretation without seeing internal JSON or understanding compiler implementation details.
