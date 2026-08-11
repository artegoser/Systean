use std::path::{Path, PathBuf};

use systean_core::discourse::{DiscourseState, IntroductionOrigin};
use systean_core::language::LanguagePackage;
use systean_core::semantics::{Checker, Term};
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
fn occurrence_reification_patterns_have_explicit_distinct_types() {
    let language = language();
    let checker = Checker::new(language.semantics());
    let proposition = "move(mover = sol)";
    let cases = [
        ("event_of", "Event"),
        ("process_of", "Process"),
        ("state_of", "State"),
        ("activity_of", "Activity"),
    ];

    for (constructor, expected) in cases {
        let term = semantic(&format!("{constructor}(content = {proposition})"));
        assert_eq!(checker.infer(&term).unwrap().to_string(), expected);
    }
}

#[test]
fn phase9_aspect_roots_apply_only_to_explicit_occurrence_targets() {
    let language = language();
    let mut discourse = DiscourseState::new();
    bind_semantic(
        &language,
        &mut discourse,
        "prun",
        "process_of(content = move(mover = sol))",
    );

    let cases = [
        ("sta prun", "start(target = process_of(content = move(mover = sol)))", "Proposition"),
        ("dur prun", "continue(target = process_of(content = move(mover = sol)))", "Proposition"),
        ("fin prun", "finish(target = process_of(content = move(mover = sol)))", "Proposition"),
        ("stop prun", "cease(target = process_of(content = move(mover = sol)))", "Proposition"),
        ("rup prun", "interrupt(target = process_of(content = move(mover = sol)))", "Proposition"),
        ("reg prun", "habitual(activity = process_of(content = move(mover = sol)))", "Activity"),
    ];

    for (surface, semantics, ty) in cases {
        let analysis = language
            .analyze_surface_with_discourse(surface, &discourse)
            .unwrap();
        assert_eq!(analysis.canonical_semantics, semantics);
        assert_eq!(analysis.inferred_type, ty);
    }
}

#[test]
fn stopping_one_process_is_not_stopping_its_habitual_activity() {
    let language = language();
    let mut discourse = DiscourseState::new();
    bind_semantic(
        &language,
        &mut discourse,
        "prun",
        "process_of(content = move(mover = sol))",
    );

    let concrete = language
        .analyze_surface_with_discourse("stop prun", &discourse)
        .unwrap();
    let habitual = language
        .analyze_surface_with_discourse("stop reg prun", &discourse)
        .unwrap();

    assert_ne!(concrete.canonical_semantics, habitual.canonical_semantics);
    assert_eq!(
        habitual.canonical_semantics,
        "cease(target = habitual(activity = process_of(content = move(mover = sol))))"
    );
}

#[test]
fn finish_and_cease_remain_different_even_on_the_same_process() {
    let language = language();
    let mut discourse = DiscourseState::new();
    bind_semantic(
        &language,
        &mut discourse,
        "prun",
        "process_of(content = create(creator = sol, product = sol))",
    );

    let finish = language
        .analyze_surface_with_discourse("fin prun", &discourse)
        .unwrap();
    let cease = language
        .analyze_surface_with_discourse("stop prun", &discourse)
        .unwrap();

    assert_ne!(finish.canonical_semantics, cease.canonical_semantics);
    assert!(finish.canonical_semantics.starts_with("finish(target = process_of("));
    assert!(cease.canonical_semantics.starts_with("cease(target = process_of("));
}

#[test]
fn repetition_count_is_explicit_and_changes_the_semantic_term() {
    let language = language();
    let mut discourse = DiscourseState::new();
    bind_semantic(
        &language,
        &mut discourse,
        "prun",
        "process_of(content = move(mover = sol))",
    );

    let three = language
        .analyze_surface_with_discourse("prun rep tri", &discourse)
        .unwrap();
    let four = language
        .analyze_surface_with_discourse("prun rep kvar", &discourse)
        .unwrap();

    assert_eq!(three.inferred_type, "Activity");
    assert!(three.canonical_semantics.contains("count = number<\"3\">"));
    assert!(four.canonical_semantics.contains("count = number<\"4\">"));
    assert_ne!(three.canonical_semantics, four.canonical_semantics);
}

#[test]
fn aspect_parsing_does_not_require_world_state_or_prior_speaker_knowledge() {
    let language = language();
    let mut first = DiscourseState::new();
    let mut second = DiscourseState::new();
    bind_semantic(
        &language,
        &mut first,
        "prun",
        "process_of(content = move(mover = sol))",
    );
    bind_semantic(
        &language,
        &mut second,
        "prun",
        "process_of(content = move(mover = sol))",
    );
    second
        .set_context_value("speaker", semantic("sol"), language.semantics())
        .unwrap();

    let without_context = language
        .analyze_surface_with_discourse("rup prun", &first)
        .unwrap();
    let with_unrelated_context = language
        .analyze_surface_with_discourse("rup prun", &second)
        .unwrap();
    assert_eq!(without_context.canonical_semantics, with_unrelated_context.canonical_semantics);
}
