use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use systean_core::discourse::{DiscourseError, DiscourseResolutionError, DiscourseState, IntroductionOrigin};
use systean_core::language::{LanguageError, LanguagePackage};
use systean_core::semantics::{Term, Type};

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
    let core = fs::read_to_string(repo.join("language/semantics/core.semsys")).unwrap();
    let fixture = fs::read_to_string(repo.join("tests/fixtures/semantics/syntax.semsys")).unwrap();

    LanguagePackage::from_sources(
        &alphabet,
        &phonology,
        &morphology,
        &syntax,
        &dictionary,
        &[
            ("language/semantics/core.semsys", &core),
            ("tests/fixtures/semantics/syntax.semsys", &fixture),
        ],
    )
    .unwrap()
}

fn discourse_with_context(language: &LanguagePackage) -> DiscourseState {
    let mut discourse = DiscourseState::new();
    discourse
        .set_context_value("speaker", Term::Const("john".into()), language.semantics())
        .unwrap();
    discourse
        .set_context_value("addressee", Term::Const("mary".into()), language.semantics())
        .unwrap();
    discourse
}

fn sleep(entity: &str) -> Term {
    Term::Call {
        function: "sleep".into(),
        arguments: BTreeMap::from([("sleeper".into(), Term::Const(entity.into()))]),
    }
}

fn introduce_entity(
    discourse: &mut DiscourseState,
    language: &LanguagePackage,
    entity: &str,
) -> systean_core::discourse::ReferentId {
    discourse
        .introduce(
            Term::Const(entity.into()),
            IntroductionOrigin::External {
                label: entity.into(),
            },
            language.semantics(),
        )
        .unwrap()
}

#[test]
fn exact_alias_resolves_even_when_generic_reference_is_ambiguous() {
    let language = language();
    let mut discourse = discourse_with_context(&language);
    introduce_entity(&mut discourse, &language, "john");
    let mary = introduce_entity(&mut discourse, &language, "mary");
    language.bind_alias(&mut discourse, "zen", mary).unwrap();

    let generic = language
        .analyze_surface_with_discourse("mi vi ref", &discourse)
        .unwrap_err();
    assert!(matches!(
        generic,
        LanguageError::Discourse(DiscourseResolutionError::Discourse(
            DiscourseError::AmbiguousReference { .. }
        ))
    ));

    let exact = language
        .analyze_surface_with_discourse("mi vi zen", &discourse)
        .unwrap();
    assert_eq!(exact.resolved.aliases.len(), 1);
    assert_eq!(exact.resolved.aliases[0].referent.id, mary);
    assert_eq!(exact.canonical_surface, "mi vi zen");
    assert_eq!(exact.canonical_semantics, "see(observed = mary, observer = john)");
}

#[test]
fn alias_scope_is_lexical_and_inner_binding_shadows_outer_binding() {
    let language = language();
    let mut discourse = discourse_with_context(&language);
    let john = introduce_entity(&mut discourse, &language, "john");
    let mary = introduce_entity(&mut discourse, &language, "mary");
    language.bind_alias(&mut discourse, "zen", john).unwrap();

    assert_eq!(
        language
            .analyze_surface_with_discourse("zen si", &discourse)
            .unwrap()
            .canonical_semantics,
        "sleep(sleeper = john)"
    );

    discourse.enter_scope();
    language.bind_alias(&mut discourse, "zen", mary).unwrap();
    assert_eq!(
        language
            .analyze_surface_with_discourse("zen si", &discourse)
            .unwrap()
            .canonical_semantics,
        "sleep(sleeper = mary)"
    );
    discourse.leave_scope().unwrap();

    assert_eq!(
        language
            .analyze_surface_with_discourse("zen si", &discourse)
            .unwrap()
            .canonical_semantics,
        "sleep(sleeper = john)"
    );
}

#[test]
fn local_alias_cannot_escape_its_scope() {
    let language = language();
    let mut discourse = discourse_with_context(&language);
    let john = introduce_entity(&mut discourse, &language, "john");

    discourse.enter_scope();
    language.bind_alias(&mut discourse, "zen", john).unwrap();
    assert!(language
        .analyze_surface_with_discourse("zen si", &discourse)
        .is_ok());
    discourse.leave_scope().unwrap();

    let error = language
        .analyze_surface_with_discourse("zen si", &discourse)
        .unwrap_err();
    assert!(error.to_string().contains("unknown lexical root `zen`"));
}

#[test]
fn aliases_can_name_non_entity_values_and_are_typed_in_operator_slots() {
    let language = language();
    let mut discourse = discourse_with_context(&language);
    let proposition = discourse
        .introduce(
            sleep("john"),
            IntroductionOrigin::External {
                label: "john sleeps".into(),
            },
            language.semantics(),
        )
        .unwrap();
    language.bind_alias(&mut discourse, "prop", proposition).unwrap();

    let analysis = language
        .analyze_surface_with_discourse("ne prop", &discourse)
        .unwrap();
    assert_eq!(analysis.resolved.aliases[0].referent.ty, Type::named("Proposition"));
    assert_eq!(analysis.canonical_semantics, "not(value = sleep(sleeper = john))");
}

#[test]
fn frame_boundary_retires_ordinary_reference_but_preserves_exact_alias() {
    let language = language();
    let mut discourse = discourse_with_context(&language);
    let mary = introduce_entity(&mut discourse, &language, "mary");
    language.bind_alias(&mut discourse, "zen", mary).unwrap();

    assert!(language
        .analyze_surface_with_discourse("mi vi ref", &discourse)
        .is_ok());
    let old_frame = discourse.current_frame();
    let new_frame = discourse.advance_frame();
    assert_ne!(old_frame, new_frame);

    let generic = language
        .analyze_surface_with_discourse("mi vi ref", &discourse)
        .unwrap_err();
    assert!(matches!(
        generic,
        LanguageError::Discourse(DiscourseResolutionError::Discourse(
            DiscourseError::UnresolvedReference { .. }
        ))
    ));

    assert_eq!(
        language
            .analyze_surface_with_discourse("mi vi zen", &discourse)
            .unwrap()
            .canonical_semantics,
        "see(observed = mary, observer = john)"
    );
}

#[test]
fn omission_becomes_invalid_immediately_after_second_candidate_is_introduced() {
    let language = language();
    let mut discourse = discourse_with_context(&language);
    introduce_entity(&mut discourse, &language, "mary");
    let unique = language
        .analyze_surface_with_discourse("mi vi", &discourse)
        .unwrap();
    assert_eq!(unique.canonical_surface, "mi vi");
    assert_eq!(unique.canonical_resolved_surface, "mi vi ref");

    introduce_entity(&mut discourse, &language, "john");
    let error = language
        .analyze_surface_with_discourse("mi vi", &discourse)
        .unwrap_err();
    assert!(matches!(
        error,
        LanguageError::Discourse(DiscourseResolutionError::Discourse(
            DiscourseError::AmbiguousReference { .. }
        ))
    ));
}

#[test]
fn ali_binds_only_an_existing_accessible_surface_value() {
    let language = language();
    let mut discourse = discourse_with_context(&language);

    let missing = language
        .bind_alias_to_surface(&mut discourse, "zen", "mari")
        .unwrap_err();
    assert!(matches!(
        missing,
        LanguageError::Discourse(DiscourseResolutionError::Discourse(
            DiscourseError::UnresolvedReference { .. }
        ))
    ));

    let mary = introduce_entity(&mut discourse, &language, "mary");
    let bound = language
        .bind_alias_to_surface(&mut discourse, "zen", "mari")
        .unwrap();
    assert_eq!(bound, mary);
}

#[test]
fn def_introduces_and_binds_a_new_local_semantic_value() {
    let language = language();
    let mut discourse = discourse_with_context(&language);

    let id = language
        .define_alias_from_surface(&mut discourse, "zen", "mari")
        .unwrap();
    assert_eq!(discourse.active_alias("zen").unwrap().referent, id);
    assert_eq!(
        language
            .analyze_surface_with_discourse("zen si", &discourse)
            .unwrap()
            .canonical_semantics,
        "sleep(sleeper = mary)"
    );
}

#[test]
fn definitions_do_not_become_ordinary_shorthand_candidates() {
    let language = language();
    let mut discourse = discourse_with_context(&language);
    let mary = introduce_entity(&mut discourse, &language, "mary");

    language
        .define_alias_from_surface(&mut discourse, "zen", "jon")
        .unwrap();

    let generic = language
        .analyze_surface_with_discourse("mi vi ref", &discourse)
        .unwrap();
    assert_eq!(generic.resolved.references[0].referent.id, mary);
    assert_eq!(generic.canonical_semantics, "see(observed = mary, observer = john)");

    let exact = language
        .analyze_surface_with_discourse("mi vi zen", &discourse)
        .unwrap();
    assert_eq!(exact.canonical_semantics, "see(observed = john, observer = john)");
}

#[test]
fn rel_creates_a_temporary_nested_binding_and_never_exports_it() {
    let language = language();
    let mut discourse = discourse_with_context(&language);

    let analysis = language
        .analyze_with_relative_binding(&mut discourse, "zen", "mari", "zen si")
        .unwrap();
    assert_eq!(analysis.canonical_semantics, "sleep(sleeper = mary)");
    assert!(discourse.active_alias("zen").is_none());
    assert!(language
        .analyze_surface_with_discourse("zen si", &discourse)
        .is_err());
}

#[test]
fn alias_surface_cannot_collide_with_lexicon_or_structural_markers() {
    let language = language();
    let mut discourse = discourse_with_context(&language);
    let mary = introduce_entity(&mut discourse, &language, "mary");

    assert!(language.bind_alias(&mut discourse, "sol", mary).is_err());
    assert!(language.bind_alias(&mut discourse, "fra", mary).is_err());
    assert!(language.bind_alias(&mut discourse, "ki", mary).is_err());
}

#[test]
fn ordinary_frame_changes_do_not_change_alias_identity_or_type() {
    let language = language();
    let mut discourse = discourse_with_context(&language);
    let proposition = discourse
        .introduce(
            sleep("john"),
            IntroductionOrigin::External {
                label: "proposition".into(),
            },
            language.semantics(),
        )
        .unwrap();
    language.bind_alias(&mut discourse, "prop", proposition).unwrap();
    let before = discourse.active_alias("prop").unwrap().clone();

    discourse.advance_frame();
    discourse.advance_frame();
    let after = discourse.active_alias("prop").unwrap().clone();
    assert_eq!(before.referent, after.referent);
    assert_eq!(before.ty, after.ty);
}
