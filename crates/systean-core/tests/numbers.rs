use std::path::{Path, PathBuf};

use systean_core::language::LanguagePackage;
use systean_core::semantics::{Checker, Literal, StructuredValue, Term, Type};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    LanguagePackage::load(repository().join("language")).unwrap()
}

#[test]
fn written_and_spoken_numbers_share_one_exact_value() {
    let language = language();
    let written = language.analyze_surface("1000005").unwrap();
    let spoken = language.analyze_surface("mega uno pent").unwrap();

    assert_eq!(written.inferred_type, "Number");
    assert_eq!(spoken.inferred_type, "Number");
    assert_eq!(written.canonical_semantics, spoken.canonical_semantics);
    assert_eq!(written.canonical_semantics, "number<\"1000005\">");
    assert_eq!(spoken.canonical_surface, "mega uno pent");
}

#[test]
fn magnitude_first_sparse_integer_speech_omits_zero_padding() {
    let language = language();
    let literals = language.literals().unwrap();
    let literal = literals.parse_complete("1000005").unwrap();
    assert_eq!(literal.canonical_spoken, "mega uno pent");
    assert!(literal.canonical_spoken.starts_with("mega "));
    assert!(!literal.canonical_spoken.contains("nul"));

    let dense = literals.parse_complete("123456789").unwrap();
    assert_eq!(
        dense.canonical_spoken,
        "mega hek uno dek dva tri kilo hek kvar dek pent siks hek sev dek okt nin"
    );
}

#[test]
fn large_magnitude_chains_are_greedy_and_canonical() {
    let language = language();
    let literals = language.literals().unwrap();
    let value = format!("1{}", "0".repeat(100));
    let literal = literals.parse_complete(&value).unwrap();
    assert_eq!(
        literal.canonical_spoken,
        "keta keta keta giga dek uno"
    );
    assert_eq!(
        literals
            .parse_complete("keta keta keta giga dek uno")
            .unwrap()
            .canonical_written,
        value
    );
    let error = literals.parse_complete("kilo kilo uno").unwrap_err().to_string();
    assert!(error.contains("non-canonical outer magnitude chain"), "{error}");

    let coefficient_first = literals.parse_complete("uno mega").unwrap_err().to_string();
    assert!(coefficient_first.contains("consumed") || coefficient_first.contains("not a structured literal"), "{coefficient_first}");
}

#[test]
fn decimals_rationals_signs_and_explicit_exponents_are_exact() {
    let language = language();
    let literals = language.literals().unwrap();

    assert_eq!(
        literals.parse_complete("minus dek dva pent").unwrap().canonical_written,
        "-25"
    );
    let explicit_plus = literals.parse_complete("plus pent").unwrap();
    assert_eq!(explicit_plus.canonical_written, "5");
    assert_eq!(explicit_plus.canonical_spoken, "pent");
    assert_eq!(
        literals.parse_complete("uno dot dva pent").unwrap().canonical_written,
        "1.25"
    );
    assert_eq!(
        literals.parse_complete("rat ki uno ku ki tri ku").unwrap().canonical_written,
        "1/3"
    );
    assert_eq!(
        literals
            .parse_complete("eks ki uno dot dva ku ki siks ku")
            .unwrap()
            .canonical_written,
        "1200000"
    );

    let small = literals.parse_complete("0.000000000000000000000000000001").unwrap();
    assert_eq!(small.canonical_written, "0.000000000000000000000000000001");
    assert_eq!(small.canonical_spoken, "eks ki uno ku ki minus dek tri ku");
    assert_eq!(
        literals.parse_complete(&small.canonical_spoken).unwrap().canonical_written,
        small.canonical_written
    );
}

#[test]
fn digit_and_digit_sequence_are_typed_without_coercing_to_number() {
    let language = language();
    let literals = language.literals().unwrap();
    let digit = literals.digit_literal(5).unwrap();
    assert_eq!(digit.ty, Type::named("Digit"));

    let written = literals.parse_complete("dig 00123").unwrap();
    let spoken = literals.parse_complete("dig nul nul uno dva tri").unwrap();
    assert_eq!(written.semantic.ty, Type::named("DigitSequence"));
    assert!(matches!(
        &written.semantic.value,
        StructuredValue::DigitSequence(values) if values == &vec![0, 0, 1, 2, 3]
    ));
    assert_eq!(written.canonical_spoken, "dig nul nul uno dva tri");
    assert_eq!(written.semantic, spoken.semantic);
    assert_eq!(literals.digit_sequence_elements("00123").unwrap().len(), 5);

    let number = literals.parse_complete("123").unwrap();
    let term = Term::Call {
        function: "equal".into(),
        arguments: [
            ("left".into(), Term::Literal(Literal::Structured(number.semantic))),
            ("right".into(), Term::Literal(Literal::Structured(spoken.semantic))),
        ]
        .into_iter()
        .collect(),
    };
    assert!(Checker::new(language.semantics()).infer(&term).is_err());
}

#[test]
fn canonical_spoken_numbers_parse_completely_and_regenerate_identically() {
    let language = language();
    let literals = language.literals().unwrap();
    for source in [
        "0", "5", "20", "405", "2025", "1000005", "1005007",
        "123456789", "-25", "1.25", "1/3", "0.001", "0.0001",
    ] {
        let literal = literals.parse_complete(source).unwrap();
        let reparsed = literals.parse_complete(&literal.canonical_spoken).unwrap();
        assert_eq!(reparsed.semantic, literal.semantic, "source {source}");
        assert_eq!(reparsed.canonical_spoken, literal.canonical_spoken, "source {source}");
    }
}

#[test]
fn ordinary_number_words_containing_t_are_not_misclassified_as_written_instants() {
    let language = language();
    let literals = language.literals().unwrap();

    for source in [
        "pent",
        "rat ki uno ku ki tri ku",
        "keta keta keta giga dek uno",
    ] {
        let literal = literals.parse_complete(source).unwrap();
        assert_eq!(literal.semantic.ty, Type::named("Number"), "source {source}");
    }
}

#[test]
fn approximation_remains_distinct_from_exact_numbers() {
    let language = language();
    let literals = language.literals().unwrap();
    let exact = literals.parse_complete("pent").unwrap();
    let approximate = literals.parse_complete("apro pent").unwrap();

    assert_eq!(exact.semantic.ty, Type::named("Number"));
    assert_eq!(exact.semantic.family(), "number");
    assert_eq!(exact.canonical_written, "5");

    assert_eq!(approximate.semantic.ty.to_string(), "Approximate<Number>");
    assert_eq!(approximate.semantic.family(), "approximate_number");
    assert_eq!(approximate.canonical_written, "~5");
    assert_eq!(approximate.canonical_spoken, "apro pent");
    assert_ne!(exact.semantic, approximate.semantic);
}
