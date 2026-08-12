use std::path::{Path, PathBuf};

use systean_core::discourse::{ConversationError, ConversationState, DiscourseState, UtteranceId};
use systean_core::language::LanguagePackage;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    LanguagePackage::load(repository().join("language")).unwrap()
}

fn say(
    language: &LanguagePackage,
    conversation: &mut ConversationState,
    source: &str,
) -> Result<UtteranceId, ConversationError> {
    let analysis = language
        .analyze_utterance_with_discourse(source, &DiscourseState::new())
        .unwrap();
    conversation.apply(
        source,
        analysis.surface.canonical_resolved_surface.clone(),
        analysis.pragmatics,
    )
}

#[test]
fn retraction_is_explicit_and_preserves_historical_analysis() {
    let language = language();
    let mut conversation = ConversationState::new();
    let first = say(&language, &mut conversation, "na alfa viv").unwrap();
    let before = conversation.entry(first).unwrap().clone();

    let retraction = say(&language, &mut conversation, "ret uno").unwrap();

    assert_eq!(retraction.get(), 2);
    assert_eq!(conversation.entry(first).unwrap(), &before);
    let commitment = conversation.commitment(first).unwrap();
    assert!(!commitment.active);
    assert_eq!(commitment.retracted_by, Some(retraction));
    assert!(conversation.active_commitments().is_empty());
}

#[test]
fn correction_supersedes_commitment_without_mutating_history() {
    let language = language();
    let mut conversation = ConversationState::new();
    let first = say(&language, &mut conversation, "na alfa viv").unwrap();
    let historical = conversation.entry(first).unwrap().clone();

    let correction = say(&language, &mut conversation, "uno kor na alfa mor").unwrap();

    assert_eq!(conversation.entry(first).unwrap(), &historical);
    let original = conversation.commitment(first).unwrap();
    assert!(!original.active);
    assert_eq!(original.superseded_by, Some(correction));
    let active = conversation.active_commitments();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].entry, correction);
    assert!(active[0].content.to_string().starts_with("mor("));
}

#[test]
fn serial_corrections_follow_one_deterministic_active_chain() {
    let language = language();
    let mut conversation = ConversationState::new();
    let first = say(&language, &mut conversation, "na alfa viv").unwrap();
    let second = say(&language, &mut conversation, "uno kor na alfa mor").unwrap();
    let third = say(&language, &mut conversation, "uno kor na alfa per").unwrap();

    assert_eq!(conversation.current_commitment_for(first).unwrap().entry, third);
    assert_eq!(conversation.commitment(first).unwrap().superseded_by, Some(second));
    assert_eq!(conversation.commitment(second).unwrap().superseded_by, Some(third));
    assert_eq!(conversation.active_commitments().len(), 1);
}

#[test]
fn nested_correction_can_target_the_previous_correction_directly() {
    let language = language();
    let mut conversation = ConversationState::new();
    say(&language, &mut conversation, "na alfa viv").unwrap();
    let second = say(&language, &mut conversation, "uno kor na alfa mor").unwrap();
    let third = say(&language, &mut conversation, "dva kor na alfa per").unwrap();

    assert_eq!(conversation.current_commitment_for(second).unwrap().entry, third);
    assert_eq!(conversation.active_commitments().len(), 1);
}

#[test]
fn clarification_preserves_target_and_adds_explicit_content() {
    let language = language();
    let mut conversation = ConversationState::new();
    let first = say(&language, &mut conversation, "na alfa viv").unwrap();
    let historical = conversation.entry(first).unwrap().clone();
    let clarification = say(&language, &mut conversation, "uno klar na alfa per").unwrap();

    assert_eq!(conversation.entry(first).unwrap(), &historical);
    assert!(conversation.commitment(first).unwrap().active);
    assert!(conversation.commitment(clarification).unwrap().active);
    assert_eq!(conversation.active_commitments().len(), 2);
}

#[test]
fn nonexistent_repair_target_is_rejected_before_history_changes() {
    let language = language();
    let mut conversation = ConversationState::new();
    say(&language, &mut conversation, "na alfa viv").unwrap();

    let error = say(&language, &mut conversation, "ret dva").unwrap_err();
    assert_eq!(error, ConversationError::MissingRepairTarget(UtteranceId::from_raw(2)));
    assert_eq!(conversation.history().count(), 1);
}

#[test]
fn assertion_repair_cannot_silently_target_a_question() {
    let language = language();
    let mut conversation = ConversationState::new();
    let question = say(&language, &mut conversation, "ke na alfa viv").unwrap();

    let error = say(&language, &mut conversation, "ret uno").unwrap_err();
    assert_eq!(error, ConversationError::TargetHasNoCommitment(question));
    assert_eq!(conversation.history().count(), 1);
}
