use std::path::{Path, PathBuf};

use systean_core::discourse::DiscourseState;
use systean_core::language::LanguagePackage;
use systean_core::semantics::Term;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    LanguagePackage::load(repository().join("language")).unwrap()
}

fn named(language: &LanguagePackage, payload: &str) -> Term {
    language
        .analyze_surface_with_discourse(
            &format!("na {payload}"),
            &DiscourseState::new(),
        )
        .unwrap()
        .resolved
        .term
}

fn conversation(language: &LanguagePackage) -> DiscourseState {
    let mut discourse = DiscourseState::new();
    discourse
        .set_context_value("speaker", named(language, "artemi"), language.semantics())
        .unwrap();
    discourse
        .set_context_value("addressee", named(language, "mari"), language.semantics())
        .unwrap();
    discourse
}

#[test]
fn unknown_unspecified_and_withheld_are_three_distinct_typed_values() {
    let language = language();
    let discourse = conversation(&language);

    let unknown = language
        .analyze_surface_with_discourse("mi vid unk tu", &discourse)
        .unwrap();
    let unspecified = language
        .analyze_surface_with_discourse("mi vid vak", &discourse)
        .unwrap();
    let withheld = language
        .analyze_surface_with_discourse("mi vid hid", &discourse)
        .unwrap();

    assert!(unknown.canonical_semantics.contains("information(status = unknown, knower = context(@"));
    assert!(unspecified.canonical_semantics.contains("information(status = unspecified)"));
    assert!(withheld.canonical_semantics.contains("information(status = withheld)"));
    assert_ne!(unknown.canonical_semantics, unspecified.canonical_semantics);
    assert_ne!(unspecified.canonical_semantics, withheld.canonical_semantics);
    assert_ne!(unknown.canonical_semantics, withheld.canonical_semantics);
}

#[test]
fn unknown_requires_an_explicit_or_context_bound_knower() {
    let language = language();
    let discourse = conversation(&language);

    assert!(language
        .analyze_surface_with_discourse("mi vid unk", &discourse)
        .is_err());
    assert!(language
        .analyze_surface_with_discourse("mi vid unk nau", &discourse)
        .is_err());

    let named_knower = language
        .analyze_surface_with_discourse("mi vid unk na mari", &discourse)
        .unwrap();
    assert!(named_knower
        .canonical_semantics
        .contains("information(status = unknown, knower = na(payload = \"mari\"))"));
}

#[test]
fn information_markers_require_a_typed_argument_slot() {
    let language = language();
    for surface in ["unk tu", "vak", "hid"] {
        let error = language.analyze_surface(surface).unwrap_err().to_string();
        assert!(
            error.contains("typed argument slot") || error.contains("cannot start") || error.contains("expected"),
            "unexpected error for `{surface}`: {error}"
        );
    }
}

#[test]
fn existential_quantification_is_not_an_information_status() {
    let language = language();
    let exists = language.analyze_surface("mu per viv").unwrap();
    let unspecified = language.analyze_surface("na artemi vid vak").unwrap();

    assert!(exists.canonical_semantics.starts_with("mu(predicate = bind"));
    assert!(unspecified.canonical_semantics.contains("information(status = unspecified)"));
    assert_ne!(exists.canonical_semantics, unspecified.canonical_semantics);
}

#[test]
fn approximate_number_is_typed_separately_from_exact_number() {
    let language = language();
    let literals = language.literals().unwrap();
    let exact = literals.parse_complete("10").unwrap();
    let approximate = literals.parse_complete("~10").unwrap();
    let spoken = literals.parse_complete("apro dek uno").unwrap();

    assert_eq!(exact.semantic.ty.to_string(), "Number");
    assert_eq!(approximate.semantic.ty.to_string(), "Approximate<Number>");
    assert_eq!(approximate.semantic.family(), "approximate_number");
    assert_eq!(approximate.canonical_written, "~10");
    assert_eq!(spoken.semantic, approximate.semantic);
    assert_eq!(spoken.canonical_spoken, "apro dek uno");
}

#[test]
fn explicit_numeric_tolerance_round_trips_and_negative_tolerance_is_rejected() {
    let language = language();
    let literals = language.literals().unwrap();
    let written = literals.parse_complete("10±0.5").unwrap();
    let spoken = literals
        .parse_complete("apro ki dek uno ku ki nul dot pent ku")
        .unwrap();

    assert_eq!(written.semantic, spoken.semantic);
    assert_eq!(written.canonical_written, "10±0.5");
    assert_eq!(spoken.canonical_spoken, "apro ki dek uno ku ki nul dot pent ku");
    assert!(literals.parse_complete("10±-0.5").is_err());
}

#[test]
fn contextual_standards_remain_explicit_instead_of_using_world_knowledge() {
    let language = language();
    let error = language.analyze_surface("na alfa nov").unwrap_err().to_string();
    assert!(error.contains("reference") || error.contains("argument"), "{error}");

    let explicit = language.analyze_surface("na alfa nov na beta").unwrap();
    assert_eq!(
        explicit.canonical_semantics,
        "nov(standard = na(payload = \"beta\"), value = na(payload = \"alfa\"))"
    );
}
