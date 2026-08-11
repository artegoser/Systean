use std::path::{Path, PathBuf};

use systean_core::discourse::DiscourseState;
use systean_core::language::LanguagePackage;
use systean_core::semantics::{Checker, Literal, Term};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    LanguagePackage::load(repository().join("language")).unwrap()
}

fn literal_term(language: &LanguagePackage, source: &str) -> Term {
    let literal = language.literals().unwrap().parse_complete(source).unwrap();
    Term::Literal(Literal::Structured(literal.semantic))
}

#[test]
fn absolute_calendar_time_timezone_and_instant_round_trip() {
    let language = language();
    let literals = language.literals().unwrap();

    for source in [
        "2026-08-11",
        "12:34:56",
        "+03:00",
        "2026-08-11T12:34:56+03:00",
    ] {
        let literal = literals.parse_complete(source).unwrap();
        let spoken = literal.canonical_spoken.clone();
        let reparsed = literals.parse_complete(&spoken).unwrap();
        assert_eq!(reparsed.semantic, literal.semantic, "source {source}; spoken {spoken}");
        assert_eq!(reparsed.canonical_written, literal.canonical_written);
    }
}

#[test]
fn spoken_date_fields_are_structurally_bounded() {
    let language = language();
    let literals = language.literals().unwrap();
    let date = literals.parse_complete("2000-12-31").unwrap();
    assert_eq!(
        date.canonical_spoken,
        "dat ki kilo dva ku ki dek uno dva ku ki dek tri uno ku"
    );
    assert_eq!(
        literals.parse_complete(&date.canonical_spoken).unwrap().canonical_written,
        "2000-12-31"
    );
}

#[test]
fn intervals_and_durations_are_distinct_structured_temporal_values() {
    let language = language();
    let literals = language.literals().unwrap();
    let interval = literals
        .parse_complete("2026-08-11T12:00:00Z/2026-08-11T13:00:00Z")
        .unwrap();
    assert_eq!(interval.semantic.ty.to_string(), "Interval");
    let short_interval = literals
        .parse_complete("2026-08-11T12:00Z/2026-08-11T13:00Z")
        .unwrap();
    assert_eq!(short_interval.semantic, interval.semantic);
    assert_eq!(
        literals.parse_complete(&interval.canonical_spoken).unwrap().semantic,
        interval.semantic
    );

    let duration = literals.parse_complete("PT3600S").unwrap();
    assert_eq!(duration.semantic.ty.to_string(), "Duration");
    assert_eq!(literals.parse_complete("PT3600.0S").unwrap().semantic, duration.semantic);
    assert_eq!(duration.canonical_spoken, "dat ki uno ku hor");
    assert_eq!(
        literals.parse_complete("dat ki uno ku hor").unwrap().canonical_written,
        "PT3600S"
    );
    assert!(literals.parse_complete("PT-1S").is_err());

    let duration_relation = Term::Call {
        function: "duration_of".into(),
        arguments: [
            ("interval".into(), Term::Literal(Literal::Structured(interval.semantic))),
            ("duration".into(), Term::Literal(Literal::Structured(duration.semantic))),
        ]
        .into_iter()
        .collect(),
    };
    assert_eq!(
        Checker::new(language.semantics()).infer(&duration_relation).unwrap().to_string(),
        "Proposition"
    );
}

#[test]
fn temporal_relations_use_explicit_typed_values_without_tense_inference() {
    let language = language();
    let before = language
        .analyze_surface("2026-08-11 ante 2026-08-12")
        .unwrap();
    let after = language
        .analyze_surface("2026-08-12 aft 2026-08-11")
        .unwrap();
    assert_eq!(before.inferred_type, "Proposition");
    assert_eq!(after.inferred_type, "Proposition");
    assert!(before.canonical_semantics.starts_with("before("));
    assert!(after.canonical_semantics.starts_with("after("));

    let interval = "2026-08-11T12:00:00Z/2026-08-11T13:00:00Z";
    let during = language
        .analyze_surface(&format!("2026-08-11T12:30:00Z pot {interval}"))
        .unwrap();
    assert!(during.canonical_semantics.starts_with("during("));
}

#[test]
fn now_is_context_bound_and_different_contexts_change_only_resolution() {
    let language = language();
    let mut first = DiscourseState::new();
    first
        .set_context_value(
            "now",
            literal_term(&language, "2026-08-11T12:00:00Z"),
            language.semantics(),
        )
        .unwrap();
    let mut second = DiscourseState::new();
    second
        .set_context_value(
            "now",
            literal_term(&language, "2026-08-12T12:00:00Z"),
            language.semantics(),
        )
        .unwrap();

    let source = "nau ante 2026-08-13T00:00:00Z";
    let a = language.analyze_surface_with_discourse(source, &first).unwrap();
    let b = language.analyze_surface_with_discourse(source, &second).unwrap();
    assert_eq!(a.syntax, b.syntax);
    assert_eq!(a.canonical_surface, b.canonical_surface);
    assert_ne!(a.canonical_semantics, b.canonical_semantics);
    assert!(a.canonical_semantics.contains("2026-08-11T12:00:00Z"));
    assert!(b.canonical_semantics.contains("2026-08-12T12:00:00Z"));
}

#[test]
fn now_without_context_is_rejected_instead_of_reading_a_clock() {
    let language = language();
    let error = language
        .analyze_surface_with_discourse(
            "nau ante 2026-08-13T00:00:00Z",
            &DiscourseState::new(),
        )
        .unwrap_err()
        .to_string();
    assert!(error.contains("context") && error.contains("now"), "{error}");
}
