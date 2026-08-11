# Phase 5 Playable Vocabulary

Status: **implemented; runtime validation pending on the author toolchain**

Phase 5 installs the first manually selected content vocabulary into the canonical `language/dictionary.toml`. These forms are ordinary bare roots. Their semantic identity and surface frame are compiled from the dictionary; the full typed operator signature remains in `language/semantics/core.semsys`.

The phase intentionally implements 40 roots from the pre-implementation author-selected batch. Time, modality, causality, and aspect roots remain assigned to their dedicated later phases rather than receiving premature semantics here.

## Entity and class predicates

| Root | Exact Phase 5 meaning | Semantic operator |
| --- | --- | --- |
| `per` | value is a human person | `person(entity)` |
| `anim` | value is an animal | `animal(entity)` |
| `lok` | value functions as a spatial location | `location(entity)` |
| `obj` | value is a physical object | `physical_object(entity)` |
| `viv` | entity is alive | `alive(entity)` |
| `ling` | entity is a language | `language_entity(entity)` |
| `tekst` | value is text | `text_value(value)` |
| `nom` | textual value functions as a name | `name_text(value)` |

`nom` remains a lexical concept and is not the proper-name marker. Proper names use `na PAYLOAD`.

## Perception, movement, possession, and action

| Root | Frame |
| --- | --- |
| `vid` | `see(observer, observed)` |
| `aud` | `hear(observer, heard)` |
| `mov` | `move(mover)` |
| `ven` | `arrive(mover, target)` |
| `vad` | `go_toward(mover, destination)` |
| `don` | `give(giver, item, recipient)` |
| `ten` | `hold(holder, held)` |
| `hab` | `possess(possessor, possessed)` |
| `fak` | `create(creator, product)`; the broad early gloss is narrowed to intentional creation so the root has one formal identity |
| `mor` | `die(entity)` |

## Relative properties and comparison

| Root | Frame |
| --- | --- |
| `nov` | `new_relative(value, standard)` |
| `vet` | `old_relative(value, standard)` |
| `bon` | `positive_by(value, criterion)` |
| `mal` | `negative_by(value, criterion)` |
| `sim` | `similar_by(left, right, dimension)` |
| `dif` | `different_by(left, right, dimension)` |
| `par` | `equal_by(left, right, dimension)` |
| `mag` | `greater_by(left, right, dimension)` |
| `min` | `less_by(left, right, dimension)` |

No comparison dimension, evaluation criterion, or relative standard is guessed from context.

## Spatial relations

| Root | Frame |
| --- | --- |
| `in` | `inside(value, container)` |
| `sur` | `above(value, reference)` |
| `sub` | `below(value, reference)` |
| `prok` | `near_by(value, reference, standard)` |
| `dist` | `far_by(value, reference, standard)` |

`prok` and `dist` require an explicit standard; the engine does not infer a culturally or physically convenient distance threshold.

## Propositional and cognitive relations

| Root | Frame |
| --- | --- |
| `ver` | `truth(value: Proposition)` |
| `fal` | `falsity(value: Proposition)` |
| `zna` | `know(knower, content: Proposition)` |
| `bel` | `believe(believer, content: Proposition)` |
| `mem` | `remember(rememberer, content: Proposition)` |
| `dum` | `think(thinker, content: Proposition)` |

`fal` is not an alias for logical `ne`; both remain explicitly distinct semantic identities.

## Communication

| Root | Frame |
| --- | --- |
| `gov` | `speak(speaker, content: Text)` |
| `skrib` | `write(writer, content: Text)` |

Opaque quotation supplies `Text` values directly, so foreign content can fill these frames without entering lexical parsing.

## Minimal playable slice

Examples exercised by the Phase 5 suites include:

```text
na artemi per
mi gov sit sal tis
ke mi viv
da tu mov
me tu gov sit sal tis
mu per viv
ra per viv
mi viv zo tu viv va mi vid tu
```

The dedicated tests also create two named discourse referents, force an ambiguous `ref`, and repair it through an exact local alias. No candidate ranking or recency heuristic is used.
