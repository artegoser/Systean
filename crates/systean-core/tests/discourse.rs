use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use systean_core::discourse::{
    DiscourseError, DiscourseResolutionError, DiscourseState, IntroductionOrigin,
};
use systean_core::language::{LanguageError, LanguagePackage};
use systean_core::semantics::{Term, Type};
use systean_core::syntax::ReferenceSource;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    let repo = repository();
    let alphabet = fs::read_to_string(repo.join("language/alphabet.toml")).unwrap();
    let phonology = fs::read_to_string(repo.join("language/phonology.toml")).unwrap();
    let morphology = fs::read_to_string(repo.join("language/morphology.toml")).unwrap();
    let syntax = fs::read_to_string(repo.join("language/syntax.toml")).unwrap();
    let mut dictionary = fs::read_to_string(repo.join("language/dictionary.toml")).unwrap();
    dictionary.push('\n');
    dictionary.push_str(
        &fs::read_to_string(repo.join("tests/fixtures/syntax/lexicon.toml")).unwrap(),
    );
    let core = fs::read_to_string(repo.join("language/typed/core.semsys")).unwrap();
    let lexicon = fs::read_to_string(repo.join("language/typed/lexicon.semsys")).unwrap();
    let effects = fs::read_to_string(repo.join("language/typed/effects.semsys")).unwrap();
    let units = fs::read_to_string(repo.join("language/typed/units.semsys")).unwrap();
    let fixture = fs::read_to_string(repo.join("tests/fixtures/typed/syntax.semsys")).unwrap();

    LanguagePackage::from_sources(
        &alphabet,
        &phonology,
        &morphology,
        &syntax,
        &dictionary,
        &[
            ("language/typed/core.semsys", &core),
            ("language/typed/lexicon.semsys", &lexicon),
            ("language/typed/effects.semsys", &effects),
            ("language/typed/units.semsys", &units),
            ("tests/fixtures/typed/syntax.semsys", &fixture),
        ],
    )
    .unwrap()
}

fn discourse_with_context(language: &LanguagePackage) -> DiscourseState {
    let mut discourse = DiscourseState::new();
    discourse
        .set_context_value(
            "speaker",
            Term::Const("jon".into()),
            language.semantics(),
        )
        .unwrap();
    discourse
        .set_context_value(
            "addressee",
            Term::Const("mari".into()),
            language.semantics(),
        )
        .unwrap();
    discourse
}

fn sleep(term: &str) -> Term {
    Term::Call {
        function: "si".into(),
        arguments: BTreeMap::from([("sleeper".into(), Term::Const(term.into()))]),
    }
}

fn introduce(
    discourse: &mut DiscourseState,
    language: &LanguagePackage,
    semantic: &str,
    label: &str,
) {
    discourse
        .introduce(
            Term::Const(semantic.into()),
            IntroductionOrigin::External {
                label: label.into(),
            },
            language.semantics(),
        )
        .unwrap();
}

#[test]
fn context_forms_resolve_through_explicit_runtime_context() {
    let language = language();
    let discourse = discourse_with_context(&language);
    let analysis = language
        .analyze_surface_with_discourse("mi vi tu", &discourse)
        .unwrap();

    assert_eq!(analysis.typed.contexts.len(), 2);
    assert!(analysis.typed.references.is_empty());
    assert_eq!(analysis.canonical_semantics, "vi(observed = mari, observer = jon)");
}

#[test]
fn explicit_reference_resolves_only_unique_compatible_referent() {
    let language = language();
    let mut discourse = discourse_with_context(&language);
    let mary = discourse
        .introduce(
            Term::Const("mari".into()),
            IntroductionOrigin::Surface {
                source: "previous utterance".into(),
            },
            language.semantics(),
        )
        .unwrap();
    discourse
        .introduce(
            sleep("jon"),
            IntroductionOrigin::External {
                label: "proposition".into(),
            },
            language.semantics(),
        )
        .unwrap();

    let analysis = language
        .analyze_surface_with_discourse("mi vi ref", &discourse)
        .unwrap();

    assert_eq!(analysis.typed.references.len(), 1);
    assert_eq!(analysis.typed.references[0].role, "observed");
    assert_eq!(analysis.typed.references[0].expected_type, Type::named("Entity"));
    assert!(matches!(
        &analysis.typed.references[0].source,
        ReferenceSource::Explicit { surface } if surface == "ref"
    ));
    assert_eq!(analysis.resolved.references[0].referent.id, mary);
    assert_eq!(analysis.canonical_semantics, "vi(observed = mari, observer = jon)");
}

#[test]
fn zero_compatible_candidates_is_an_unresolved_reference_error() {
    let language = language();
    let mut discourse = discourse_with_context(&language);
    discourse
        .introduce(
            sleep("jon"),
            IntroductionOrigin::External {
                label: "proposition only".into(),
            },
            language.semantics(),
        )
        .unwrap();

    let error = language
        .analyze_surface_with_discourse("mi vi ref", &discourse)
        .unwrap_err();
    assert!(matches!(
        error,
        LanguageError::Discourse(DiscourseResolutionError::Discourse(
            DiscourseError::UnresolvedReference { ref role, ref expected }
        )) if role == "observed" && expected == &Type::named("Entity")
    ));
}

#[test]
fn multiple_compatible_candidates_are_reported_without_ranking() {
    let language = language();
    let mut discourse = discourse_with_context(&language);
    introduce(&mut discourse, &language, "jon", "first");
    introduce(&mut discourse, &language, "mari", "second");

    let error = language
        .analyze_surface_with_discourse("mi vi ref", &discourse)
        .unwrap_err();
    let LanguageError::Discourse(DiscourseResolutionError::Discourse(
        DiscourseError::AmbiguousReference {
            role,
            expected,
            candidates,
        },
    )) = error
    else {
        panic!("expected ambiguous reference error");
    };
    assert_eq!(role, "observed");
    assert_eq!(expected, Type::named("Entity"));
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].value, Term::Const("jon".into()));
    assert_eq!(candidates[1].value, Term::Const("mari".into()));
}

#[test]
fn wrong_type_candidates_are_excluded_before_resolution() {
    let language = language();
    let mut discourse = discourse_with_context(&language);
    discourse
        .introduce(
            sleep("jon"),
            IntroductionOrigin::External {
                label: "wrong type".into(),
            },
            language.semantics(),
        )
        .unwrap();
    introduce(&mut discourse, &language, "mari", "entity");

    let analysis = language
        .analyze_surface_with_discourse("mi vi ref", &discourse)
        .unwrap();
    assert_eq!(analysis.resolved.references.len(), 1);
    assert_eq!(
        analysis.resolved.references[0].referent.value,
        Term::Const("mari".into())
    );
}

#[test]
fn unique_resolution_is_independent_of_candidate_insertion_order() {
    let language = language();

    let mut first = discourse_with_context(&language);
    first
        .introduce(
            sleep("jon"),
            IntroductionOrigin::External {
                label: "proposition".into(),
            },
            language.semantics(),
        )
        .unwrap();
    introduce(&mut first, &language, "mari", "entity");

    let mut second = discourse_with_context(&language);
    introduce(&mut second, &language, "mari", "entity");
    second
        .introduce(
            sleep("jon"),
            IntroductionOrigin::External {
                label: "proposition".into(),
            },
            language.semantics(),
        )
        .unwrap();

    let first_semantics = language
        .analyze_surface_with_discourse("mi vi ref", &first)
        .unwrap()
        .canonical_semantics;
    let second_semantics = language
        .analyze_surface_with_discourse("mi vi ref", &second)
        .unwrap()
        .canonical_semantics;
    assert_eq!(first_semantics, second_semantics);
}

#[test]
fn omitted_argument_uses_exactly_the_same_reference_resolver() {
    let language = language();
    let mut discourse = discourse_with_context(&language);
    introduce(&mut discourse, &language, "mari", "entity");

    let explicit = language
        .analyze_surface_with_discourse("mi vi ref", &discourse)
        .unwrap();
    let omitted = language
        .analyze_surface_with_discourse("mi vi", &discourse)
        .unwrap();

    assert_eq!(explicit.canonical_semantics, omitted.canonical_semantics);
    assert_eq!(
        explicit.resolved.references[0].referent.id,
        omitted.resolved.references[0].referent.id
    );
    assert!(matches!(
        &omitted.typed.references[0].source,
        ReferenceSource::Omitted
    ));
}

#[test]
fn omitted_primary_argument_uses_the_same_typed_resolver() {
    let language = language();
    let mut discourse = discourse_with_context(&language);
    introduce(&mut discourse, &language, "jon", "sleeper");

    let explicit = language
        .analyze_surface_with_discourse("ref si", &discourse)
        .unwrap();
    let omitted = language
        .analyze_surface_with_discourse("si", &discourse)
        .unwrap();

    assert_eq!(explicit.canonical_semantics, omitted.canonical_semantics);
    assert_eq!(
        explicit.resolved.references[0].referent.id,
        omitted.resolved.references[0].referent.id
    );
    assert_eq!(omitted.typed.references[0].role, "sleeper");
    assert!(matches!(
        &omitted.typed.references[0].source,
        ReferenceSource::Omitted
    ));
}

#[test]
fn runtime_context_is_required_and_type_checked_deterministically() {
    let language = language();
    let missing = DiscourseState::new();
    let error = language
        .analyze_surface_with_discourse("mi si", &missing)
        .unwrap_err();
    assert!(matches!(
        error,
        LanguageError::Discourse(DiscourseResolutionError::Discourse(
            DiscourseError::ContextMissing { ref key }
        )) if key == "speaker"
    ));

    let mut wrong_type = DiscourseState::new();
    wrong_type
        .set_context_value("speaker", sleep("jon"), language.semantics())
        .unwrap();
    let error = language
        .analyze_surface_with_discourse("mi si", &wrong_type)
        .unwrap_err();
    assert!(matches!(
        error,
        LanguageError::Discourse(DiscourseResolutionError::Discourse(
            DiscourseError::ContextTypeMismatch { ref key, ref expected, ref actual }
        )) if key == "speaker"
            && expected == &Type::named("Entity")
            && actual == &Type::named("Proposition")
    ));
}

#[test]
fn nested_accessibility_scopes_are_deterministic() {
    let language = language();
    let mut discourse = discourse_with_context(&language);
    let outer = discourse
        .introduce(
            Term::Const("jon".into()),
            IntroductionOrigin::External {
                label: "outer".into(),
            },
            language.semantics(),
        )
        .unwrap();

    discourse.enter_scope();
    let inner = discourse
        .introduce(
            Term::Const("mari".into()),
            IntroductionOrigin::External {
                label: "inner".into(),
            },
            language.semantics(),
        )
        .unwrap();
    assert!(discourse.referent(inner).is_some());
    assert!(language
        .analyze_surface_with_discourse("ref si", &discourse)
        .is_err());

    discourse.leave_scope().unwrap();
    let analysis = language
        .analyze_surface_with_discourse("ref si", &discourse)
        .unwrap();
    assert_eq!(analysis.resolved.references[0].referent.id, outer);
}

#[test]
fn referent_ids_types_and_origins_are_stable_runtime_metadata() {
    let language = language();
    let mut discourse = DiscourseState::new();
    let first_origin = IntroductionOrigin::External {
        label: "first".into(),
    };
    let second_origin = IntroductionOrigin::Surface {
        source: "second utterance".into(),
    };
    let first = discourse
        .introduce(
            Term::Const("jon".into()),
            first_origin.clone(),
            language.semantics(),
        )
        .unwrap();
    let second = discourse
        .introduce(
            Term::Const("mari".into()),
            second_origin.clone(),
            language.semantics(),
        )
        .unwrap();

    assert_ne!(first, second);
    assert_eq!(first.get() + 1, second.get());
    assert_eq!(discourse.referent(first).unwrap().ty, Type::named("Entity"));
    assert_eq!(discourse.referent(first).unwrap().origin, first_origin);
    assert_eq!(discourse.referent(second).unwrap().origin, second_origin);
}
