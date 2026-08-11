use std::path::{Path, PathBuf};

use systean_core::discourse::{
    ConversationState, DiscourseState, IntroductionOrigin, TextDocument, TextSessionState, TextTurn,
    UtteranceId,
};
use systean_core::language::{LanguagePackage, TextTurnEvent};
use systean_core::semantics::Type;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    LanguagePackage::load(repository().join("language")).unwrap()
}

fn utterances(
    analysis: &systean_core::language::TextDocumentAnalysis,
) -> Vec<&systean_core::language::TextUtteranceAnalysis> {
    analysis
        .turns
        .iter()
        .flat_map(|turn| turn.events.iter())
        .filter_map(|event| match event {
            TextTurnEvent::Utterance(utterance) => Some(utterance),
            TextTurnEvent::FrameBoundary { .. } => None,
        })
        .collect()
}

#[test]
fn spoken_du_and_written_period_recover_the_same_utterances() {
    let language = language();
    let spoken = language
        .analyze_text_document(&TextDocument::new(vec![TextTurn::spoken(
            "turn-a",
            "na alfa viv du ke na alfa viv du",
        )]))
        .unwrap();
    let written = language
        .analyze_text_document(&TextDocument::new(vec![TextTurn::written(
            "turn-a",
            "na alfa viv. ke na alfa viv.",
        )]))
        .unwrap();

    let spoken = utterances(&spoken);
    let written = utterances(&written);
    assert_eq!(spoken.len(), 2);
    assert_eq!(written.len(), 2);
    for (spoken, written) in spoken.iter().zip(written.iter()) {
        assert_eq!(spoken.pragmatics, written.pragmatics);
        assert_eq!(spoken.canonical_surface, written.canonical_surface);
        assert_eq!(spoken.canonical_spoken, format!("{} du", spoken.canonical_surface));
        assert_eq!(written.canonical_written, format!("{}.", written.canonical_surface));
    }
}

#[test]
fn turn_metadata_never_acts_as_an_implicit_utterance_boundary() {
    let language = language();
    let error = language
        .analyze_text_document(&TextDocument::new(vec![TextTurn::spoken(
            "turn-a",
            "na alfa viv",
        )]))
        .unwrap_err()
        .to_string();
    assert!(error.contains("explicit utterance boundary"), "{error}");

    let error = language
        .analyze_text_document(&TextDocument::new(vec![TextTurn::written(
            "turn-a",
            "ke na alfa viv?",
        )]))
        .unwrap_err()
        .to_string();
    assert!(error.contains("explicit utterance boundary"), "{error}");
}

#[test]
fn multiple_channel_turns_need_no_spoken_turn_marker() {
    let language = language();
    let analysis = language
        .analyze_text_document(&TextDocument::new(vec![
            TextTurn::spoken("speaker-a-1", "na alfa viv du"),
            TextTurn::spoken("speaker-b-1", "ke na alfa viv du"),
        ]))
        .unwrap();

    assert_eq!(analysis.turns.len(), 2);
    assert_eq!(analysis.turns[0].key, "speaker-a-1");
    assert_eq!(analysis.turns[1].key, "speaker-b-1");
    assert_eq!(utterances(&analysis).len(), 2);
}

#[test]
fn frame_boundary_retires_shorthand_but_not_exact_aliases() {
    let language = language();
    let named = language
        .analyze_surface_with_discourse("na mari", &DiscourseState::new())
        .unwrap()
        .resolved
        .term;

    let mut discourse = DiscourseState::new();
    let referent = discourse
        .introduce(
            named,
            IntroductionOrigin::External {
                label: "before section boundary".into(),
            },
            language.semantics(),
        )
        .unwrap();
    language.bind_alias(&mut discourse, "zen", referent).unwrap();
    let old_frame = discourse.current_frame();

    let mut session = TextSessionState::new();
    let mut conversation = ConversationState::new();
    let analysis = language
        .analyze_text_document_with_state(
            &TextDocument::new(vec![TextTurn::spoken("turn-a", "fra zen viv du")]),
            &mut session,
            &mut discourse,
            &mut conversation,
        )
        .unwrap();

    assert_ne!(old_frame, discourse.current_frame());
    assert_eq!(session.current_section().get(), 1);
    assert!(language.analyze_surface_with_discourse("zen", &discourse).is_ok());
    assert!(
        discourse
            .resolve_reference("after fra", &Type::named("Entity"), language.semantics())
            .is_err()
    );
    assert_eq!(utterances(&analysis)[0].canonical_surface, "zen viv");
}

#[test]
fn utterance_and_turn_boundaries_do_not_change_reference_or_alias_lifetime() {
    let language = language();
    let named = language
        .analyze_surface_with_discourse("na mari", &DiscourseState::new())
        .unwrap()
        .resolved
        .term;
    let mut discourse = DiscourseState::new();
    let referent = discourse
        .introduce(
            named,
            IntroductionOrigin::External {
                label: "stable referent".into(),
            },
            language.semantics(),
        )
        .unwrap();
    language.bind_alias(&mut discourse, "zen", referent).unwrap();

    let frame = discourse.current_frame();
    let mut session = TextSessionState::new();
    let mut conversation = ConversationState::new();
    language
        .analyze_text_document_with_state(
            &TextDocument::new(vec![
                TextTurn::spoken("turn-a", "zen viv du"),
                TextTurn::spoken("turn-b", "zen viv du"),
            ]),
            &mut session,
            &mut discourse,
            &mut conversation,
        )
        .unwrap();

    assert_eq!(frame, discourse.current_frame());
    assert_eq!(session.current_section().get(), 0);
    assert_eq!(
        discourse
            .resolve_reference(
                "still accessible",
                &Type::named("Entity"),
                language.semantics(),
            )
            .unwrap()
            .id,
        referent
    );
    assert_eq!(discourse.active_alias("zen").unwrap().referent, referent);
}

#[test]
fn quotation_keeps_boundary_forms_opaque() {
    let language = language();
    let spoken = language
        .analyze_text_document(&TextDocument::new(vec![TextTurn::spoken(
            "turn-a",
            "na artemi gov sit hello du world tis du",
        )]))
        .unwrap();
    assert_eq!(utterances(&spoken).len(), 1);
    assert!(utterances(&spoken)[0]
        .pragmatics
        .utterance
        .to_string()
        .contains("hello du world"));

    let written = language
        .analyze_text_document(&TextDocument::new(vec![TextTurn::written(
            "turn-a",
            "na artemi gov sit hello. world tis.",
        )]))
        .unwrap();
    assert_eq!(utterances(&written).len(), 1);
    assert!(utterances(&written)[0]
        .pragmatics
        .utterance
        .to_string()
        .contains("hello. world"));
}

#[test]
fn readability_punctuation_cannot_create_or_change_the_communicative_act() {
    let language = language();
    let analysis = language
        .analyze_text_document(&TextDocument::new(vec![TextTurn::written(
            "turn-a",
            "na alfa viv?. ke na beta viv!.",
        )]))
        .unwrap();
    let utterances = utterances(&analysis);
    assert_eq!(utterances[0].pragmatics.act.label(), "assertion");
    assert_eq!(utterances[1].pragmatics.act.label(), "truth_question");
}

#[test]
fn decimal_points_are_not_written_utterance_boundaries() {
    let language = language();
    let analysis = language
        .analyze_text_document(&TextDocument::new(vec![TextTurn::written(
            "turn-a",
            "na artemi felis 0.8.",
        )]))
        .unwrap();
    let utterances = utterances(&analysis);
    assert_eq!(utterances.len(), 1);
    assert!(utterances[0].canonical_surface.contains("0.8"));
}

#[test]
fn repairs_cross_turn_and_frame_boundaries_because_history_is_document_scoped() {
    let language = language();
    let mut session = TextSessionState::new();
    let mut discourse = DiscourseState::new();
    let mut conversation = ConversationState::new();
    language
        .analyze_text_document_with_state(
            &TextDocument::new(vec![
                TextTurn::spoken("turn-a", "na alfa viv du"),
                TextTurn::spoken("turn-b", "fra uno kor na alfa mor du"),
            ]),
            &mut session,
            &mut discourse,
            &mut conversation,
        )
        .unwrap();

    let active = conversation
        .current_commitment_for(UtteranceId::from_raw(1))
        .unwrap();
    assert_eq!(active.entry, UtteranceId::from_raw(2));
    assert!(active.content.to_string().starts_with("die("));
}

#[test]
fn document_application_is_atomic_on_late_structure_failure() {
    let language = language();
    let mut session = TextSessionState::new();
    let mut discourse = DiscourseState::new();
    let mut conversation = ConversationState::new();
    let error = language
        .analyze_text_document_with_state(
            &TextDocument::new(vec![
                TextTurn::spoken("turn-a", "na alfa viv du fra"),
                TextTurn::spoken("turn-b", "na beta viv"),
            ]),
            &mut session,
            &mut discourse,
            &mut conversation,
        )
        .unwrap_err()
        .to_string();

    assert!(error.contains("explicit utterance boundary"), "{error}");
    assert_eq!(session.current_section().get(), 0);
    assert_eq!(discourse.current_frame().get(), 0);
    assert_eq!(conversation.history().count(), 0);
}

#[test]
fn document_boundary_resets_document_scoped_repair_history() {
    let language = language();
    language
        .analyze_text_document(&TextDocument::new(vec![TextTurn::spoken(
            "first-document-turn",
            "na alfa viv du",
        )]))
        .unwrap();

    let error = language
        .analyze_text_document(&TextDocument::new(vec![TextTurn::spoken(
            "second-document-turn",
            "uno kor na alfa mor du",
        )]))
        .unwrap_err()
        .to_string();
    assert!(error.contains("repair target `u1` does not exist"), "{error}");
}
