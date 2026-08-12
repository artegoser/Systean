use std::sync::OnceLock;

use serde_json::json;
use systean_core::discourse::{DiscourseState, TextRealization};
use systean_core::language::LanguagePackage;
use wasm_bindgen::prelude::*;

const PACKAGE: &str = include_str!("../../../language/package.toml");
const ALPHABET: &str = include_str!("../../../language/alphabet.toml");
const PHONOLOGY: &str = include_str!("../../../language/phonology.toml");
const MORPHOLOGY: &str = include_str!("../../../language/morphology.toml");
const SYNTAX: &str = include_str!("../../../language/syntax.toml");
const DICTIONARY: &str = include_str!("../../../language/dictionary.toml");
const LITERALS: &str = include_str!("../../../language/literals.toml");
const UNITS: &str = include_str!("../../../language/units.toml");
const COMPATIBILITY_CORPUS: &str = include_str!("../../../language/corpus/compatibility.tsv");
const ADVERSARIAL_CORPUS: &str = include_str!("../../../language/corpus/adversarial.tsv");
const TYPED_CORE: &str = include_str!("../../../language/typed/core.semsys");
const TYPED_LEXICON: &str = include_str!("../../../language/typed/lexicon.semsys");
const TYPED_UNITS: &str = include_str!("../../../language/typed/units.semsys");

static LANGUAGE: OnceLock<Result<LanguagePackage, String>> = OnceLock::new();

fn language() -> Result<&'static LanguagePackage, JsValue> {
    match LANGUAGE.get_or_init(|| {
        LanguagePackage::from_versioned_sources_full(
            PACKAGE,
            ALPHABET,
            PHONOLOGY,
            MORPHOLOGY,
            SYNTAX,
            DICTIONARY,
            LITERALS,
            UNITS,
            &[
                ("typed/core.semsys", TYPED_CORE),
                ("typed/lexicon.semsys", TYPED_LEXICON),
                ("typed/units.semsys", TYPED_UNITS),
            ],
            COMPATIBILITY_CORPUS,
            ADVERSARIAL_CORPUS,
        )
        .map_err(|error| error.to_string())
    }) {
        Ok(language) => Ok(language),
        Err(error) => Err(JsValue::from_str(error)),
    }
}

fn json_string(value: serde_json::Value) -> Result<String, JsValue> {
    serde_json::to_string(&value).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn alphabet_json() -> Result<String, JsValue> {
    let language = language()?;
    let letters = language
        .phonology()
        .alphabet
        .letters()
        .iter()
        .map(|letter| {
            json!({
                "symbol": letter.symbol,
                "pronunciation": letter.pronunciation,
                "_type": letter.kind.to_string(),
            })
        })
        .collect::<Vec<_>>();
    json_string(json!({ "letters": letters }))
}

#[wasm_bindgen]
pub fn dictionary_json() -> Result<String, JsValue> {
    let language = language()?;
    serde_json::to_string(language.dictionary())
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn pronounce(text: &str) -> Result<String, JsValue> {
    language()?
        .phonology()
        .alphabet
        .pronounce(text)
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn spell(pronunciation: &str) -> Result<String, JsValue> {
    language()?
        .phonology()
        .alphabet
        .spell(pronunciation)
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn analyze_word_json(word: &str) -> Result<String, JsValue> {
    let analysis = language()?
        .analyze_word(word)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    let syllables = analysis
        .phonology
        .syllables
        .iter()
        .map(|syllable| {
            json!({
                "spelling": syllable.spelling,
                "pronunciation": syllable.pronunciation,
                "stressed": syllable.stressed,
                "graphemeStart": syllable.grapheme_range.start,
                "graphemeEnd": syllable.grapheme_range.end,
            })
        })
        .collect::<Vec<_>>();
    let morphemes = analysis
        .morphology
        .morphemes
        .iter()
        .map(|morpheme| {
            json!({
                "kind": morpheme.kind,
                "spelling": morpheme.spelling,
                "graphemeStart": morpheme.grapheme_range.start,
                "graphemeEnd": morpheme.grapheme_range.end,
            })
        })
        .collect::<Vec<_>>();
    json_string(json!({
        "spelling": analysis.phonology.canonical_spelling,
        "root": analysis.morphology.root,
        "morphemes": morphemes,
        "pronunciation": analysis.phonology.pronunciation,
        "stressedPronunciation": analysis.phonology.stressed_pronunciation,
        "rootStart": analysis.phonology.root_grapheme_range.start,
        "rootEnd": analysis.phonology.root_grapheme_range.end,
        "stressedSyllable": analysis.phonology.stressed_syllable,
        "syllables": syllables,
    }))
}

#[wasm_bindgen]
pub fn generate_word(root: &str) -> Result<String, JsValue> {
    language()?
        .generate_word(root)
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn explain_json(expression: &str) -> Result<String, JsValue> {
    let analysis = language()?
        .explain(expression)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    serde_json::to_string(&analysis).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn syntax_policy_json() -> Result<String, JsValue> {
    let language = language()?;
    let config = language.syntax().config();
    json_string(json!({
        "frameOrder": format!("{:?}", config.order.frame),
        "freeOrder": config.order.free_order,
        "scopeOpen": &config.scope.open,
        "scopeClose": &config.scope.close,
        "quoteOpen": &config.quotation.open,
        "quoteClose": &config.quotation.close,
        "explicitScope": format!("{:?}", config.scope.explicit),
        "quantifierScope": format!("{:?}", config.scope.quantifier_order),
        "precedence": &config.logic.precedence,
        "flattenSameOperator": config.logic.flatten_same_operator,
        "lexicalRoots": language.syntax().lexicon().len(),
    }))
}

#[wasm_bindgen]
pub fn analyze_surface_json(expression: &str) -> Result<String, JsValue> {
    let analysis = language()?
        .analyze_surface(expression)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    json_string(json!({
        "canonicalSurface": analysis.canonical_surface,
        "inferredType": analysis.inferred_type,
        "canonicalSemantics": analysis.canonical_semantics,
        "syntax": analysis.syntax.to_string(),
    }))
}

#[wasm_bindgen]
pub fn analyze_utterance_json(expression: &str) -> Result<String, JsValue> {
    let analysis = language()?
        .analyze_utterance(expression)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    json_string(json!({
        "canonicalSurface": analysis.surface.canonical_surface,
        "canonicalResolvedSurface": analysis.surface.canonical_resolved_surface,
        "surfaceType": analysis.surface.resolved.inferred_type.to_string(),
        "canonicalSemantics": analysis.surface.canonical_semantics,
        "act": analysis.pragmatics.act.label(),
        "canonicalUtterance": analysis.pragmatics.utterance.to_string(),
        "utteranceType": analysis.pragmatics.inferred_type.to_string(),
    }))
}


fn workbench_json<T: serde::Serialize>(
    result: Result<T, systean_core::workbench::WorkbenchDiagnostic>,
) -> Result<String, JsValue> {
    match result {
        Ok(value) => serde_json::to_string(&value)
            .map_err(|error| JsValue::from_str(&error.to_string())),
        Err(error) => {
            let payload = serde_json::to_string(&error).unwrap_or_else(|_| error.message.clone());
            Err(JsValue::from_str(&payload))
        }
    }
}

#[wasm_bindgen]
pub fn package_info_json() -> Result<String, JsValue> {
    serde_json::to_string(&systean_core::workbench::package_info(language()?))
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn workbench_word_json(word: &str) -> Result<String, JsValue> {
    workbench_json(systean_core::workbench::analyze_word(language()?, word))
}

#[wasm_bindgen]
pub fn workbench_surface_json(expression: &str) -> Result<String, JsValue> {
    workbench_json(systean_core::workbench::analyze_surface(
        language()?,
        expression,
        &DiscourseState::new(),
    ))
}

#[wasm_bindgen]
pub fn workbench_utterance_json(expression: &str) -> Result<String, JsValue> {
    workbench_json(systean_core::workbench::analyze_utterance(
        language()?,
        expression,
        &DiscourseState::new(),
    ))
}

#[wasm_bindgen]
pub fn workbench_text_json(source: &str, realization: &str) -> Result<String, JsValue> {
    let realization = match realization {
        "spoken" => TextRealization::Spoken,
        "written" => TextRealization::Written,
        other => return Err(JsValue::from_str(&format!("unknown text realization `{other}`"))),
    };
    workbench_json(systean_core::workbench::analyze_text_stream(
        language()?,
        source,
        realization,
    ))
}

#[wasm_bindgen]
pub fn generate_surface_json(semantics: &str) -> Result<String, JsValue> {
    workbench_json(systean_core::workbench::generate_surface(language()?, semantics))
}

#[wasm_bindgen]
pub fn inspect_literal_json(source: &str) -> Result<String, JsValue> {
    workbench_json(systean_core::workbench::analyze_literal(language()?, source))
}
