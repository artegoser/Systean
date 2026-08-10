use std::sync::OnceLock;

use serde_json::json;
use systean_core::language::LanguagePackage;
use wasm_bindgen::prelude::*;

const ALPHABET: &str = include_str!("../../../language/alphabet.toml");
const PHONOLOGY: &str = include_str!("../../../language/phonology.toml");
const MORPHOLOGY: &str = include_str!("../../../language/morphology.toml");
const SYNTAX: &str = include_str!("../../../language/syntax.toml");
const DICTIONARY: &str = include_str!("../../../language/dictionary.toml");
const SEMANTICS_CORE: &str = include_str!("../../../language/semantics/core.semsys");

static LANGUAGE: OnceLock<Result<LanguagePackage, String>> = OnceLock::new();

fn language() -> Result<&'static LanguagePackage, JsValue> {
    match LANGUAGE.get_or_init(|| {
        LanguagePackage::from_sources(
            ALPHABET,
            PHONOLOGY,
            MORPHOLOGY,
            SYNTAX,
            DICTIONARY,
            &[("language/semantics/core.semsys", SEMANTICS_CORE)],
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
        "explicitScope": format!("{:?}", config.scope.explicit),
        "quantifierScope": format!("{:?}", config.scope.quantifier_order),
        "precedence": &config.logic.precedence,
        "flattenSameOperator": config.logic.flatten_same_operator,
        "lexicalBindings": config.lexemes.len(),
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
