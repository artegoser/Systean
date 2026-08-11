use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use systean_core::discourse::DiscourseState;
use systean_core::language::{LanguageError, LanguagePackage};
use systean_core::pragmatics::{CommunicativeAct, PragmaticError};
use systean_core::semantics::{Literal, Term, Type};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    LanguagePackage::load(repository().join("language")).unwrap()
}

fn named(name: &str) -> Term {
    Term::Call {
        function: "proper_name".into(),
        arguments: BTreeMap::from([(
            "payload".into(),
            Term::Literal(Literal::String(name.into())),
        )]),
    }
}

fn discourse(language: &LanguagePackage) -> DiscourseState {
    let mut discourse = DiscourseState::new();
    discourse
        .set_context_value("speaker", named("artemi"), language.semantics())
        .unwrap();
    discourse
}

#[test]
fn all_frozen_subjective_roots_are_executable_dictionary_entries() {
    let language = language();
    let roots = [
        "felis", "satis", "seren", "amuz", "entuz", "inter", "kuri", "esper", "relif",
        "grati", "admir", "majes", "tenda", "afek", "amori", "simpat", "kompa", "piti",
        "empati", "prid", "trium", "fidu", "unita", "sekur", "trist", "grif", "timor",
        "anks", "dred", "panik", "furor", "irit", "frus", "resent", "odio", "avers",
        "kontem", "vergon", "kulpa", "embar", "humil", "regri", "remor", "disap", "desol",
        "lonel", "envi", "jelos", "boren", "nemog", "stres", "opres", "surpri", "nosta",
        "langu", "konfi", "dubia", "antik", "avida", "konfus", "romat", "eros", "eruz",
        "libid", "orgaz", "hedon", "komfor", "dolor", "nelag", "famen", "tirst", "satur",
        "nause", "prur", "verti", "dispne", "fati", "somni", "vigir", "frigi", "kalor",
        "superb", "avari", "lusta", "gula", "leni",
    ];

    assert_eq!(roots.len(), 86);
    assert_eq!(language.roots().roots().len(), 175);
    for root in roots {
        assert!(
            language.roots().roots().iter().any(|known| known == root),
            "missing frozen subjective root {root}"
        );
    }
}

#[test]
fn experienced_affect_is_an_assertable_state_separate_from_emo_expression() {
    let language = language();
    let discourse = discourse(&language);

    let experienced = language
        .analyze_utterance_with_discourse("mi felis 0.8", &discourse)
        .unwrap();
    let expressed = language
        .analyze_utterance_with_discourse("emo mi felis 0.8", &discourse)
        .unwrap();

    assert_eq!(experienced.surface.resolved.inferred_type, Type::named("Affect"));
    assert!(matches!(experienced.pragmatics.act, CommunicativeAct::Assertion { .. }));
    assert!(matches!(expressed.pragmatics.act, CommunicativeAct::Expressive { .. }));
    assert_ne!(experienced.pragmatics.utterance, expressed.pragmatics.utterance);
    assert!(expressed.pragmatics.utterance.to_string().contains("affect_felis"));
}

#[test]
fn explicit_affect_can_change_while_its_propositional_target_stays_identical() {
    let language = language();
    let mut discourse = discourse(&language);
    let proposition = language
        .analyze_surface_with_discourse("na alfa viv", &DiscourseState::new())
        .unwrap()
        .resolved
        .term;
    discourse
        .define_alias("zeni", proposition.clone(), language.semantics())
        .unwrap();

    let satisfied = language
        .analyze_utterance_with_discourse("emo mi satis zeni 0.8", &discourse)
        .unwrap();
    let disappointed = language
        .analyze_utterance_with_discourse("emo mi disap zeni 0.8", &discourse)
        .unwrap();

    let CommunicativeAct::Expressive { state: left } = &satisfied.pragmatics.act else {
        panic!("expected expressive affect")
    };
    let CommunicativeAct::Expressive { state: right } = &disappointed.pragmatics.act else {
        panic!("expected expressive affect")
    };
    assert_ne!(left, right);
    assert!(left.to_string().contains(&proposition.to_string()));
    assert!(right.to_string().contains(&proposition.to_string()));
}

#[test]
fn romantic_sexual_and_motivational_states_remain_distinct() {
    let language = language();
    let discourse = discourse(&language);

    let romantic = language.analyze_surface_with_discourse("mi romat na mari 0.7", &discourse).unwrap();
    let attraction = language.analyze_surface_with_discourse("mi eros na mari 0.7", &discourse).unwrap();
    let arousal = language.analyze_surface_with_discourse("mi eruz 0.7", &discourse).unwrap();
    let libido = language.analyze_surface_with_discourse("mi libid 0.7", &discourse).unwrap();
    let lust = language.analyze_surface_with_discourse("mi lusta na mari 0.7", &discourse).unwrap();

    let terms = [romantic, attraction, arousal, libido, lust]
        .map(|analysis| analysis.canonical_semantics);
    for left in 0..terms.len() {
        for right in left + 1..terms.len() {
            assert_ne!(terms[left], terms[right]);
        }
    }
}

#[test]
fn orgasm_keeps_event_identity_while_remaining_an_explicit_subjective_experience() {
    let language = language();
    let discourse = discourse(&language);
    let orgasm = language
        .analyze_surface_with_discourse("mi orgaz 0.8", &discourse)
        .unwrap();

    assert_eq!(orgasm.resolved.inferred_type, Type::named("SubjectiveEvent"));
    assert!(language
        .semantics()
        .is_assignable(&orgasm.resolved.inferred_type, &Type::named("Event")));
    assert!(language
        .semantics()
        .is_assignable(&orgasm.resolved.inferred_type, &Type::named("Proposition")));
}

#[test]
fn bodily_subjective_state_is_not_typed_as_affect() {
    let language = language();
    let discourse = discourse(&language);
    let pain = language
        .analyze_surface_with_discourse("mi dolor 0.5", &discourse)
        .unwrap();

    assert_eq!(pain.resolved.inferred_type, Type::named("SubjectiveState"));
    assert!(language
        .semantics()
        .is_assignable(&pain.resolved.inferred_type, &Type::named("Proposition")));
    assert!(!language
        .semantics()
        .is_assignable(&pain.resolved.inferred_type, &Type::named("Affect")));
}

#[test]
fn emo_can_explicitly_express_a_bodily_subjective_state_without_retyping_it_as_affect() {
    let language = language();
    let discourse = discourse(&language);
    let analysis = language
        .analyze_utterance_with_discourse("emo mi dolor 0.5", &discourse)
        .unwrap();

    let CommunicativeAct::Expressive { state } = &analysis.pragmatics.act else {
        panic!("expected expressive bodily state")
    };
    assert!(state.to_string().starts_with("state_dolor("));
}

#[test]
fn focus_marks_an_existing_target_without_changing_content_roles() {
    let language = language();
    let plain = language.analyze_utterance("na alfa vid na beta").unwrap();
    let focused = language
        .analyze_utterance("na beta fok na alfa vid na beta")
        .unwrap();

    let CommunicativeAct::Assertion { content: plain_content } = &plain.pragmatics.act else {
        panic!("expected assertion")
    };
    let CommunicativeAct::Focus { target, content } = &focused.pragmatics.act else {
        panic!("expected focus")
    };
    assert_eq!(plain_content, content);
    assert_eq!(target.to_string(), "proper_name(payload = \"beta\")");
    assert!(content.to_string().contains("observer = proper_name(payload = \"alfa\")"));
    assert!(content.to_string().contains("observed = proper_name(payload = \"beta\")"));
}

#[test]
fn focus_and_topic_reject_targets_not_structurally_present_in_content() {
    let language = language();
    for source in [
        "na beta fok na alfa viv",
        "na beta top na alfa viv",
    ] {
        let error = language.analyze_utterance(source).unwrap_err();
        assert!(matches!(
            error,
            LanguageError::Pragmatics(PragmaticError::FocusTargetAbsent { .. })
        ));
    }
}

#[test]
fn topic_preserves_explicit_logical_scope() {
    let language = language();
    let plain = language.analyze_utterance("ne na alfa viv").unwrap();
    let topical = language
        .analyze_utterance("na alfa top ne na alfa viv")
        .unwrap();

    let CommunicativeAct::Assertion { content: plain_content } = &plain.pragmatics.act else {
        panic!("expected assertion")
    };
    let CommunicativeAct::Topic { content, .. } = &topical.pragmatics.act else {
        panic!("expected topic")
    };
    assert_eq!(plain_content, content);
    assert!(content.to_string().starts_with("not("));
}

#[test]
fn expressive_affect_never_invents_sarcastic_negation() {
    let language = language();
    let mut discourse = discourse(&language);
    let proposition = language
        .analyze_surface_with_discourse("na alfa viv", &DiscourseState::new())
        .unwrap()
        .resolved
        .term;
    discourse
        .define_alias("zeni", proposition, language.semantics())
        .unwrap();

    let analysis = language
        .analyze_utterance_with_discourse("emo mi disap zeni 0.9", &discourse)
        .unwrap();
    let rendered = analysis.pragmatics.utterance.to_string();
    assert!(!rendered.contains("not("), "expressive affect must not create negation: {rendered}");
}
