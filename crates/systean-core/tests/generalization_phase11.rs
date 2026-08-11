use std::path::{Path, PathBuf};

use systean_core::discourse::{DiscourseState, IntroductionOrigin};
use systean_core::language::LanguagePackage;
use systean_core::semantics::Term;
use systean_core::spec::{lower_term, parse_term};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    LanguagePackage::load(repository().join("language")).unwrap()
}

fn semantic(source: &str) -> Term {
    lower_term(parse_term(source).unwrap())
}

fn bind_semantic(
    language: &LanguagePackage,
    discourse: &mut DiscourseState,
    alias: &str,
    source: &str,
) {
    let term = semantic(source);
    let id = discourse
        .introduce(
            term,
            IntroductionOrigin::External {
                label: source.into(),
            },
            language.semantics(),
        )
        .unwrap();
    language.bind_alias(discourse, alias, id).unwrap();
}

#[test]
fn counted_quantifiers_are_explicit_and_semantically_distinct() {
    let language = language();
    let exactly = language.analyze_surface("rov tri per viv").unwrap();
    let at_least = language.analyze_surface("mini tri per viv").unwrap();
    let at_most = language.analyze_surface("maks tri per viv").unwrap();

    assert!(exactly.canonical_semantics.starts_with("exactly(count = number<\"3\">, predicate = bind"));
    assert!(at_least.canonical_semantics.starts_with("at_least(count = number<\"3\">, predicate = bind"));
    assert!(at_most.canonical_semantics.starts_with("at_most(count = number<\"3\">, predicate = bind"));
    assert_ne!(exactly.canonical_semantics, at_least.canonical_semantics);
    assert_ne!(at_least.canonical_semantics, at_most.canonical_semantics);
    assert_eq!(exactly.canonical_surface, "rov tri per viv");
    assert!(language.analyze_surface("rov apro tri per viv").is_err());
}

#[test]
fn collections_keep_set_list_and_group_identity_distinct() {
    let language = language();
    let set = language.analyze_surface("na alfa set na beta").unwrap();
    let list = language.analyze_surface("na alfa list na beta").unwrap();
    let group = language.analyze_surface("na alfa grup na beta").unwrap();

    assert!(set.canonical_semantics.starts_with("set_pair("));
    assert!(list.canonical_semantics.starts_with("list_pair("));
    assert!(group.canonical_semantics.starts_with("group_pair("));
    assert_ne!(set.canonical_semantics, list.canonical_semantics);
    assert_ne!(set.canonical_semantics, group.canonical_semantics);
}

#[test]
fn collective_and_distributive_interpretations_are_explicitly_different() {
    let language = language();
    let collective = language.analyze_surface("kol na alfa viv").unwrap();
    let distributive = language.analyze_surface("dis na alfa viv").unwrap();

    assert!(collective.canonical_semantics.starts_with("collective(content = alive("));
    assert!(distributive.canonical_semantics.starts_with("distributive(content = alive("));
    assert_ne!(collective.canonical_semantics, distributive.canonical_semantics);
}

#[test]
fn association_does_not_guess_a_more_specific_relation() {
    let language = language();
    let association = language.analyze_surface("na alfa aso na beta").unwrap();
    assert_eq!(
        association.canonical_semantics,
        "association(left = proper_name(payload = \"alfa\"), right = proper_name(payload = \"beta\"))"
    );
}

#[test]
fn material_implication_and_counterfactual_are_distinct_constructions() {
    let language = language();
    let implication = language
        .analyze_surface("na alfa viv imp na beta viv")
        .unwrap();
    let counterfactual = language
        .analyze_surface("na alfa viv hip na beta viv")
        .unwrap();

    assert!(implication.canonical_semantics.starts_with("implies("));
    assert!(counterfactual.canonical_semantics.starts_with("counterfactual("));
    assert_ne!(implication.canonical_semantics, counterfactual.canonical_semantics);
}

#[test]
fn typical_claim_is_not_an_exception_tolerant_universal() {
    let language = language();
    let universal = language.analyze_surface("ra per viv").unwrap();
    let typical = language
        .analyze_surface("na alfa tip na beta 0.8")
        .unwrap();

    assert!(universal.canonical_semantics.starts_with("forall(predicate = bind"));
    assert!(!universal.canonical_semantics.contains("typical("));
    assert!(!universal.canonical_semantics.contains("statistical("));
    assert!(typical.canonical_semantics.starts_with("typical("));
    assert_ne!(universal.canonical_semantics, typical.canonical_semantics);
}

#[test]
fn typical_and_statistical_claims_expose_domain_measure_and_value() {
    let language = language();
    let typical = language
        .analyze_surface("na alfa tip na beta 0.8")
        .unwrap();
    let statistical = language
        .analyze_surface("na alfa stat na beta 0.8")
        .unwrap();

    assert_eq!(
        typical.canonical_semantics,
        "typical(domain = proper_name(payload = \"alfa\"), measure = proper_name(payload = \"beta\"), standard = number<\"0.8\">)"
    );
    assert_eq!(
        statistical.canonical_semantics,
        "statistical(domain = proper_name(payload = \"alfa\"), measure = proper_name(payload = \"beta\"), value = number<\"0.8\">)"
    );
    assert_ne!(typical.canonical_semantics, statistical.canonical_semantics);
}

#[test]
fn probability_and_frequency_require_explicit_typed_targets_and_values() {
    let language = language();
    let mut discourse = DiscourseState::new();
    bind_semantic(&language, &mut discourse, "prop", "alive(entity = sol)");
    bind_semantic(
        &language,
        &mut discourse,
        "habit",
        "activity_of(content = move(mover = sol))",
    );

    let probability = language
        .analyze_surface_with_discourse("prop prob 0.7", &discourse)
        .unwrap();
    let frequency = language
        .analyze_surface_with_discourse("habit frek na kal 3", &discourse)
        .unwrap();

    assert_eq!(
        probability.canonical_semantics,
        "probability(claim = alive(entity = sol), value = number<\"0.7\">)"
    );
    assert_eq!(
        frequency.canonical_semantics,
        "frequency(activity = activity_of(content = move(mover = sol)), measure = proper_name(payload = \"kal\"), value = number<\"3\">)"
    );
}

#[test]
fn majority_is_compositional_and_not_a_primitive_typical_or_statistical_claim() {
    let language = language();
    assert!(!language.roots().roots().iter().any(|root| root == "most"));
    assert!(language.semantics().operator("most").is_none());

    // In an explicitly finite domain of exactly five persons, at least three
    // persons satisfying a predicate is a strict majority. The language states
    // both cardinal facts literally instead of reinterpreting `tip` or `stat`.
    let composition = language
        .analyze_surface("rov pent per per va mini tri per viv")
        .unwrap();
    let typical = language
        .analyze_surface("na alfa tip na beta 0.6")
        .unwrap();
    let statistical = language
        .analyze_surface("na alfa stat na beta 0.6")
        .unwrap();

    assert!(composition.canonical_semantics.contains("exactly(count = number<\"5\">"));
    assert!(composition.canonical_semantics.contains("at_least(count = number<\"3\">"));
    assert!(!composition.canonical_semantics.contains("typical("));
    assert!(!composition.canonical_semantics.contains("statistical("));
    assert_ne!(composition.canonical_semantics, typical.canonical_semantics);
    assert_ne!(composition.canonical_semantics, statistical.canonical_semantics);
}
