# Systean Subjective-State and Affect Lexical Freeze

Status: **author-selected language-content freeze for Phases 13 and 18**

This document freezes the currently approved Systean roots for affective, social, romantic/sexual, bodily, and selected motivational/dispositional subjective states. These forms are manually authored language content. They are not executable dictionary entries until their implementation phase adds semantic signatures and the whole-language compiler accepts them.

The fixed alphabet remains authoritative. No form in this document may expand or modify it.

### Supersession of earlier candidates

This inventory supersedes exploratory spellings proposed before the full freeze. Only the roots listed in this document are author-selected for these concepts. In particular, the final sexual-arousal spelling is `eruz`; earlier exploratory forms such as `aruz`/`sexar`/`sexat`, and earlier temporary emotion candidates such as `rad`, `dol`, or `frai`, are not normative selections.

---

## 1. Semantic separation rules

The following distinctions are normative design requirements for implementation.

1. An affect root denotes the experienced affect/state concept. The explicit `emo` construction is a separate expressive/pragmatic layer that attaches affect to a declared utterance/content/target.
2. Experiencing an affect and expressing that affect are not the same semantic structure.
3. Intensity is an explicit parameter. Systean does not create unrelated lexical roots merely for weak/strong degrees such as mild anger versus rage or mild fear versus terror.
4. Affect target and cause are explicit when semantically relevant. They are never guessed from world knowledge, prosody, or conversational convention.
5. Prosody remains free and human but does not create, invert, or replace normative affect semantics.
6. The 1.0 baseline does not introduce a parallel inventory of dedicated `wow`/`ouch`/`ugh`-style interjection roots. Standalone expressive utterances are built compositionally through the explicit affect layer. Convenience sugar may be considered later only if it has one canonical expansion.
7. Bodily subjective states are not silently treated as emotions. They have their own semantic state/event signatures even when they can be the target of an expressive utterance.
8. Sexual attraction, sexual arousal, baseline sexual drive, lust, romantic attraction, and general love are distinct concepts. None entails desire or intention to perform a sexual action; action desire remains expressible through the ordinary `vol` mechanism plus an explicit action.
9. Ordinary pride and excessive self-exaltation are distinct: `prid` is ordinary pride, while `superb` is excessive self-exaltation.
10. The traditional seven-deadly-sins inventory is representable without adding duplicate senses. `envi` and `furor` retain their single approved lexical identities and may participate compositionally in a dispositional/habitual construction when a persistent vice rather than a current state is intended.

A conceptual affect structure may expose fields equivalent to:

```text
affect-kind
experiencer
target
cause
intensity
```

The exact `.semsys` signatures and surface argument order are Phase 13 implementation work, but omitted fields must follow the normal explicit/context-bound/unspecified rules and must never be inferred heuristically.

---

## 2. Positive and social affective states

| Root | Frozen meaning |
| --- | --- |
| `felis` | joy; a general positive affective state |
| `satis` | satisfaction; positive response to an achieved or sufficient state |
| `seren` | calm; low affective tension |
| `amuz` | amusement; pleasure from something perceived as funny or entertaining |
| `entuz` | excitement/enthusiasm; high-activation positive affect |
| `inter` | interest; sustained affective engagement with an object/topic |
| `kuri` | curiosity; motivation to obtain currently unknown information |
| `esper` | hope; positive anticipation of a desired possible future outcome |
| `relif` | relief; positive response to removal or reduction of a threat/burden |
| `grati` | gratitude; positive affect toward the source of a received benefit |
| `admir` | admiration; strong positive evaluation of another's qualities/actions |
| `majes` | awe; affective response to exceptional scale, grandeur, or perceived magnitude |
| `tenda` | tenderness; gentle caring positive affect |
| `afek` | affection; stable positive emotional attachment |
| `amori` | love; strong stable love without mandatory romantic or sexual content |
| `simpat` | liking/sympathy; positive personal disposition toward another |
| `kompa` | compassion; affective concern for another's suffering |
| `piti` | pity; negative affect about another's adverse condition |
| `empati` | empathy; affective experiencing/modeling of another's emotional state |
| `prid` | ordinary pride; positive affect concerning an achievement/quality of self or an associated target |
| `trium` | triumph; positive response to victory or successful overcoming |
| `fidu` | trust; subjective readiness to rely on a specific agent/system |
| `unita` | belonging; subjective experience of inclusion in a group/relationship |
| `sekur` | felt safety; subjective experience of absence of significant threat |

---

## 3. Negative affective states

| Root | Frozen meaning |
| --- | --- |
| `trist` | sadness; general low-activation negative affect |
| `grif` | grief; affective response to a significant loss |
| `timor` | fear; response to a perceived threat |
| `anks` | anxiety; negative anticipation of an uncertain threat |
| `dred` | dread; fear directed toward an anticipated future event |
| `panik` | panic; acute high-intensity threat response with felt loss of control |
| `furor` | anger/wrath; negative response to perceived harm or violation |
| `irit` | irritation; negative response to a recurring or obstructive nuisance |
| `frus` | frustration; negative response to blockage of a goal |
| `resent` | resentment; persistent negative response to perceived unfairness/wrong |
| `odio` | hatred; stable strong negative orientation toward a target |
| `avers` | disgust/aversion; immediate avoidance-oriented response to an unpleasant target |
| `kontem` | contempt; negative social evaluation of a target as unworthy/inferior |
| `vergon` | shame; negative evaluation of the self |
| `kulpa` | guilt; negative evaluation of one's own specific action/responsibility |
| `embar` | embarrassment; social discomfort caused by unwanted attention/error |
| `humil` | humiliation; painful experience of imposed loss of status |
| `regri` | regret; negative counterfactual evaluation of a past choice |
| `remor` | remorse; regret with responsibility for harm caused |
| `disap` | disappointment; negative response to an unmet expectation |
| `desol` | despair; experience that no available route to a desired outcome remains |
| `lonel` | loneliness; experience of insufficient desired social connection |
| `envi` | envy; negative response to another possessing a desired good/state |
| `jelos` | jealousy; perceived threat of losing a valued relationship to a rival/third party |
| `boren` | boredom; negative state of insufficient meaningful stimulation |
| `nemog` | helplessness; experience of lacking effective control/action capacity |
| `stres` | stress; sustained tension under perceived demands |
| `opres` | overwhelm; experience of demands exceeding available capacity |

---

## 4. Neutral, mixed, and cognitive-affective states

| Root | Frozen meaning |
| --- | --- |
| `surpri` | surprise; violation of expectation without built-in positive/negative valence |
| `nosta` | nostalgia; affective orientation toward a personally significant past |
| `langu` | longing; affectively significant desire for an absent/inaccessible target |
| `konfi` | confidence; high subjective confidence regarding an evaluation/outcome |
| `dubia` | doubt; subjective uncertainty |
| `antik` | anticipation; affective response to an expected future event |
| `avida` | eagerness; high motivational readiness toward a desired action/outcome |
| `konfus` | confusion; inability to form a coherent understanding of the current situation |

---

## 5. Romantic and sexual states/events

| Root | Frozen meaning |
| --- | --- |
| `romat` | romantic attraction |
| `eros` | sexual attraction toward an explicit target |
| `eruz` | current sexual arousal state |
| `libid` | baseline/persistent level of sexual drive |
| `orgaz` | orgasm as a distinct physiological event |

These identities are deliberately separate. In particular:

```text
amori  != romat
romat  != eros
eros   != eruz
eruz   != libid
lusta  != eros / eruz / libid
```

`lusta` is defined below as lust/strong sexual desire as a motivational state. Attraction or arousal alone never entails an action desire or intention.

---

## 6. Bodily subjective states

| Root | Frozen meaning |
| --- | --- |
| `hedon` | subjective pleasure |
| `komfor` | bodily comfort |
| `dolor` | pain |
| `nelag` | physical discomfort |
| `famen` | hunger |
| `tirst` | thirst |
| `satur` | satiety/fullness |
| `nause` | nausea |
| `prur` | itch |
| `verti` | dizziness/vertigo sensation |
| `dispne` | subjective shortness of breath/air hunger |
| `fati` | fatigue |
| `somni` | sleepiness |
| `vigir` | alertness/wakeful vigor |
| `frigi` | subjective feeling of cold |
| `kalor` | subjective feeling of heat |

These roots belong to bodily/interoceptive semantics rather than the affect layer itself.

---

## 7. Motivational/dispositional roots and the seven deadly sins

The following five additional roots are frozen as motivational/dispositional concepts:

| Root | Frozen meaning |
| --- | --- |
| `superb` | excessive pride/self-exaltation; deliberately distinct from ordinary `prid` |
| `avari` | greed; drive to accumulate/appropriate goods |
| `lusta` | lust; strong sexual desire as a motivational state |
| `gula` | gluttony; excessive drive toward food consumption |
| `leni` | laziness/sloth; persistent unwillingness to expend effort |

The traditional seven-deadly-sins set is therefore expressible as:

```text
superb  excessive self-exaltation / pride-as-vice
avari   greed
lusta   lust
envi    envy
gula    gluttony
furor   wrath
leni    sloth
```

`envi` and `furor` reuse the already frozen envy and anger/wrath concepts instead of introducing duplicate lexical senses. When a persistent disposition rather than a current affect is intended, persistence/habit/disposition must be expressed compositionally rather than by changing the root's lexical identity.

---

## 8. Intensity and compositional coverage

The inventory intentionally does not create separate roots merely for intensity variants. Examples of concepts that should normally be expressed compositionally include:

```text
rage                 = furor + high explicit intensity
terror               = timor + high explicit intensity
mild sadness         = trist + low explicit intensity
strong love          = amori + high explicit intensity
homesickness/longing = langu + explicit target
sexual action desire = vol + explicit sexual/action content
```

Likewise, concepts such as schadenfreude or moral disgust should first be expressed from general affect plus explicit target/cause/criterion structures. A dedicated root should be added later only if it proves to be a genuinely distinct high-value lexical identity rather than hidden composition or an intensity synonym.

---

## 9. Interjections

No dedicated interjection-root inventory is frozen for the 1.0 baseline.

The intended baseline is a compact standalone expressive construction using `emo` plus the appropriate affect/state when that state is being expressed. This avoids a second lexicon that duplicates meanings already represented by affect roots. If later usability tests justify shortened interjection sugar, each such form must have one explicit canonical expansion and must pass the same whole-language ambiguity checks as every other normative form.

---

## 10. Static spelling/collision audit at freeze time

The approved inventory contains:

- 60 positive/negative/neutral/social/cognitive-affective roots;
- 5 romantic/sexual roots;
- 16 bodily subjective-state roots;
- 5 additional motivational/dispositional roots;
- 86 unique newly selected root spellings in total;
- 7 traditional deadly-sin concepts, with `envi` and `furor` reusing roots already counted above.

At the time of this freeze, all 86 unique new spellings were checked against the current repository data:

- every character belongs to the fixed Systean alphabet;
- every root contains a vowel;
- no exact spelling collides with the current 83 lexical roots;
- no exact spelling collides with structural/discourse/quotation forms;
- no exact spelling collides with numeric/calendar literal forms;
- no exact spelling collides with current spoken unit forms;
- no exact pronunciation collides with the current inventory under the fixed alphabet mapping;
- no pair of new forms has an exact pronunciation collision;
- no new form is at Levenshtein distance 1 or less from a current/reserved form or another newly selected form.

These static checks are not the final normative acceptance test. Phase 16 must still test complete spoken streams and grammar overlap. A future whole-stream ambiguity failure changes the conflicting surface form, not the frozen semantic distinction.

---

## 11. Implementation ownership

- Root spelling and one lexical identity belong in `language/dictionary.toml` when implemented.
- Full typed semantic signatures belong in `language/semantics/*.semsys`.
- Generic affect/state machinery belongs in the engine only as generic mechanisms; Rust must not hardcode individual Systean emotions or bodily states.
- `emo`, `fok`, and `top` remain explicit pragmatic/information-structure constructions selected elsewhere in the language package.
- The whole-language compiler remains responsible for cross-layer spelling, pronunciation, structured-literal, grammar, and complete spoken-stream ambiguity checks.
