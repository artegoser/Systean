use std::path::{Path, PathBuf};

use systean_core::language::LanguagePackage;
use systean_core::semantics::{Checker, Literal, Term};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    LanguagePackage::load(repository().join("language")).unwrap()
}

#[test]
fn units_numbers_and_quantities_are_distinct_typed_values() {
    let language = language();
    let literals = language.literals().unwrap();
    let number = literals.parse_complete("5").unwrap();
    let unit = literals.parse_complete("m").unwrap();
    let quantity = literals.parse_complete("5 m").unwrap();

    assert_eq!(number.semantic.ty.to_string(), "Number");
    assert_eq!(unit.semantic.ty.to_string(), "Unit<LengthDimension>");
    assert_eq!(quantity.semantic.ty.to_string(), "Quantity<LengthDimension>");
    assert_eq!(quantity.canonical_spoken, "pent metr");
    assert_eq!(
        literals.parse_complete("pent metr").unwrap().semantic,
        quantity.semantic
    );

    let mole_written = literals.parse_complete("5 mol").unwrap();
    let mole_spoken = literals.parse_complete("pent mol").unwrap();
    assert_eq!(mole_written.semantic.ty.to_string(), "Quantity<AmountDimension>");
    assert_eq!(mole_written.semantic, mole_spoken.semantic);
}

#[test]
fn exact_unit_conversion_preserves_exact_rational_values() {
    let language = language();
    let literals = language.literals().unwrap();

    let kilometer = literals.parse_complete("1 km").unwrap();
    let meter = literals
        .convert_quantity_literal(&kilometer.semantic, "meter")
        .unwrap();
    assert_eq!(meter.canonical_written, "1000 m");

    let centimeters = literals.parse_complete("100 cm").unwrap();
    let meter = literals
        .convert_quantity_literal(&centimeters.semantic, "meter")
        .unwrap();
    assert_eq!(meter.canonical_written, "1 m");

    let third_meter = literals.parse_complete("1/3 m").unwrap();
    let centimeters = literals
        .convert_quantity_literal(&third_meter.semantic, "centimeter")
        .unwrap();
    assert_eq!(centimeters.canonical_written, "100/3 cm");
}

#[test]
fn incompatible_dimensions_fail_conversion_and_semantic_type_matching() {
    let language = language();
    let literals = language.literals().unwrap();
    let length = literals.parse_complete("5 m").unwrap();
    let time = literals.parse_complete("5 s").unwrap();

    let conversion_error = literals
        .convert_quantity_literal(&length.semantic, "second")
        .unwrap_err()
        .to_string();
    assert!(conversion_error.contains("cannot convert"), "{conversion_error}");

    let term = Term::Call {
        function: "equal".into(),
        arguments: [
            ("left".into(), Term::Literal(Literal::Structured(length.semantic))),
            ("right".into(), Term::Literal(Literal::Structured(time.semantic))),
        ]
        .into_iter()
        .collect(),
    };
    assert!(Checker::new(language.semantics()).infer(&term).is_err());
}

#[test]
fn approximation_and_uncertainty_remain_explicit_across_conversion() {
    let language = language();
    let literals = language.literals().unwrap();

    let approximate = literals.parse_complete("~5 m").unwrap();
    assert_eq!(
        approximate.semantic.ty.to_string(),
        "Approximate<Quantity<LengthDimension>>"
    );
    assert_eq!(approximate.canonical_spoken, "apro pent metr");

    let uncertain = literals.parse_complete("1±0.1 km").unwrap();
    assert!(literals.parse_complete("1±-0.1 km").is_err());
    assert_eq!(uncertain.canonical_spoken, "apro ki uno ku ki nul dot uno ku kilometr");
    let converted = literals
        .convert_quantity_literal(&uncertain.semantic, "meter")
        .unwrap();
    assert_eq!(converted.canonical_written, "1000±100 m");
    assert_eq!(
        converted.semantic.ty.to_string(),
        "Approximate<Quantity<LengthDimension>>"
    );
}


#[test]
fn decimal_formatting_does_not_hide_measurement_precision() {
    let language = language();
    let literals = language.literals().unwrap();

    let exact = literals.parse_complete("1.00 m").unwrap();
    assert_eq!(exact.canonical_written, "1 m");
    assert_eq!(exact.semantic.ty.to_string(), "Quantity<LengthDimension>");

    let explicit = literals.parse_complete("1±0.01 m").unwrap();
    assert_eq!(explicit.canonical_written, "1±0.01 m");
    assert_eq!(
        explicit.semantic.ty.to_string(),
        "Approximate<Quantity<LengthDimension>>"
    );
}

#[test]
fn parsing_does_not_silently_convert_units() {
    let language = language();
    let literals = language.literals().unwrap();
    let kilometers = literals.parse_complete("1 km").unwrap();
    assert_eq!(kilometers.semantic.canonical, "1 km");
    assert_eq!(kilometers.canonical_written, "1 km");
}
