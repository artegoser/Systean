use std::path::{Path, PathBuf};

use systean_core::language::LanguagePackage;
use systean_core::pragmatics::{CommunicativeAct, QuestionKind};
use systean_core::semantics::Type;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    LanguagePackage::load(repository().join("language")).unwrap()
}

#[test]
fn unmarked_top_level_proposition_is_a_default_assertion() {
    let language = language();
    let analysis = language.analyze_utterance("na alfa viv").unwrap();

    assert_eq!(analysis.surface.resolved.inferred_type, Type::named("Proposition"));
    assert_eq!(analysis.pragmatics.inferred_type, Type::named("Utterance"));
    assert!(matches!(analysis.pragmatics.act, CommunicativeAct::Assertion { .. }));
    assert_eq!(
        analysis.pragmatics.utterance.to_string(),
        "assert(content = alive(entity = proper_name(payload = \"alfa\")))"
    );
}

#[test]
fn ke_without_requested_slot_is_a_truth_question() {
    let language = language();
    let analysis = language.analyze_utterance("ke na alfa viv").unwrap();

    assert!(matches!(
        analysis.pragmatics.act,
        CommunicativeAct::Question {
            kind: QuestionKind::Truth,
            ..
        }
    ));
    assert_eq!(analysis.pragmatics.act.label(), "truth_question");
}

#[test]
fn value_question_is_recovered_from_explicit_typed_unknown_slot() {
    let language = language();
    let analysis = language
        .analyze_utterance("ke na artemi vid unk na mari")
        .unwrap();

    let CommunicativeAct::Question {
        kind: QuestionKind::Value { requested },
        ..
    } = &analysis.pragmatics.act
    else {
        panic!("expected an explicit value question")
    };
    assert_eq!(requested.len(), 1);
    assert_eq!(requested[0].ty, Type::named("Entity"));
    assert!(requested[0].status.starts_with("unknown:"));
}

#[test]
fn choice_question_is_ke_over_explicit_disjunction() {
    let language = language();
    let analysis = language
        .analyze_utterance("ke ki na alfa viv zo na beta viv ku")
        .unwrap();

    let CommunicativeAct::Question {
        kind: QuestionKind::Choice { alternatives },
        ..
    } = &analysis.pragmatics.act
    else {
        panic!("expected an explicit choice question")
    };
    assert_eq!(alternatives.len(), 2);
    assert!(alternatives[0].to_string().contains("alfa"));
    assert!(alternatives[1].to_string().contains("beta"));
}

#[test]
fn identical_disjunction_without_ke_is_an_assertion_not_a_question() {
    let language = language();
    let analysis = language
        .analyze_utterance("na alfa viv zo na beta viv")
        .unwrap();

    assert!(matches!(analysis.pragmatics.act, CommunicativeAct::Assertion { .. }));
    assert_eq!(analysis.pragmatics.act.label(), "assertion");
}

#[test]
fn command_and_request_remain_distinct_explicit_acts() {
    let language = language();
    let command = language.analyze_utterance("da na alfa viv").unwrap();
    let request = language.analyze_utterance("me na alfa viv").unwrap();

    assert!(matches!(command.pragmatics.act, CommunicativeAct::Command { .. }));
    assert!(matches!(request.pragmatics.act, CommunicativeAct::Request { .. }));
    assert_ne!(command.pragmatics.utterance, request.pragmatics.utterance);
}

#[test]
fn truth_answers_are_ordinary_explicit_assertions() {
    let language = language();
    let positive = language.analyze_utterance("na alfa viv").unwrap();
    let negative = language.analyze_utterance("ne na alfa viv").unwrap();

    assert!(matches!(positive.pragmatics.act, CommunicativeAct::Assertion { .. }));
    assert!(matches!(negative.pragmatics.act, CommunicativeAct::Assertion { .. }));
    assert!(negative.pragmatics.utterance.to_string().contains("not("));
}

#[test]
fn mixed_value_and_choice_question_structure_is_rejected_instead_of_ranked() {
    let language = language();
    let error = language
        .analyze_utterance("ke ki na artemi vid unk na mari zo na beta viv ku")
        .unwrap_err();
    assert!(error.to_string().contains("cannot simultaneously request an unknown value"));
}
