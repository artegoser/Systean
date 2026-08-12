use std::path::{Path, PathBuf};

use systean_core::discourse::{DiscourseState, IntroductionOrigin};
use systean_core::language::LanguagePackage;
use systean_core::semantics::Term;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    LanguagePackage::load(repository().join("language")).unwrap()
}

fn named(language: &LanguagePackage, payload: &str) -> Term {
    language
        .analyze_surface_with_discourse(
            &format!("na {payload}"),
            &DiscourseState::new(),
        )
        .unwrap()
        .resolved
        .term
}

fn conversation(language: &LanguagePackage) -> DiscourseState {
    let mut discourse = DiscourseState::new();
    discourse
        .set_context_value("speaker", named(language, "artemi"), language.semantics())
        .unwrap();
    discourse
        .set_context_value("addressee", named(language, "mari"), language.semantics())
        .unwrap();
    discourse
}

#[test]
fn selected_playable_vocabulary_is_loaded_from_the_dictionary() {
    let language = language();
    let roots = [
        "per", "anim", "lok", "obj", "vid", "aud", "mov", "ven", "vad", "don",
        "ten", "hab", "fak", "viv", "mor", "nov", "vet", "bon", "mal", "ver",
        "fal", "sim", "dif", "par", "mag", "min", "in", "sur", "sub", "prok",
        "dist", "zna", "bel", "mem", "dum", "nom", "tekst", "ling", "gov", "skrib",
    ];
    assert_eq!(roots.len(), 40);
    for root in roots {
        assert!(
            language.syntax().lexicon().contains_key(root),
            "selected playable root `{root}` is missing"
        );
    }
}

#[test]
fn small_greeting_and_introduction_dialogue_is_expressible() {
    let language = language();
    let discourse = conversation(&language);

    let greeting = language
        .analyze_surface_with_discourse("mi gov sit sal tis", &discourse)
        .unwrap();
    assert_eq!(greeting.inferred_type, "Proposition");
    assert_eq!(
        greeting.canonical_semantics,
        "gov(content = \"sal\", speaker = na(payload = \"artemi\"))"
    );

    let introduction = language.analyze_surface("na artemi per").unwrap();
    assert_eq!(introduction.inferred_type, "Proposition");
    assert_eq!(
        introduction.canonical_semantics,
        "per(entity = na(payload = \"artemi\"))"
    );
}

#[test]
fn two_people_can_be_referred_to_exactly_across_multiple_turns() {
    let language = language();
    let mut discourse = conversation(&language);
    let artemi = discourse
        .introduce(
            named(&language, "artemi"),
            IntroductionOrigin::External {
                label: "Artemi".into(),
            },
            language.semantics(),
        )
        .unwrap();
    let mari = discourse
        .introduce(
            named(&language, "mari"),
            IntroductionOrigin::External {
                label: "Mari".into(),
            },
            language.semantics(),
        )
        .unwrap();
    language.bind_alias(&mut discourse, "arta", artemi).unwrap();
    language.bind_alias(&mut discourse, "mara", mari).unwrap();

    assert_eq!(
        language
            .analyze_surface_with_discourse("arta vid mara", &discourse)
            .unwrap()
            .canonical_semantics,
        "vid(observed = na(payload = \"mari\"), observer = na(payload = \"artemi\"))"
    );
    discourse.advance_frame();
    assert_eq!(
        language
            .analyze_surface_with_discourse("mara vid arta", &discourse)
            .unwrap()
            .canonical_semantics,
        "vid(observed = na(payload = \"artemi\"), observer = na(payload = \"mari\"))"
    );
}

#[test]
fn truth_question_and_answer_use_explicit_speech_act_structure() {
    let language = language();
    let discourse = conversation(&language);
    let question = language
        .analyze_surface_with_discourse("ke mi viv", &discourse)
        .unwrap();
    let answer = language
        .analyze_surface_with_discourse("mi viv", &discourse)
        .unwrap();
    assert_eq!(question.inferred_type, "Utterance");
    assert_eq!(answer.inferred_type, "Proposition");
    assert!(question.canonical_semantics.starts_with("ke(content = viv("));
    assert!(answer.canonical_semantics.starts_with("viv(entity = "));
}

#[test]
fn command_and_request_use_explicit_operators_with_playable_roots() {
    let language = language();
    let discourse = conversation(&language);
    let command = language
        .analyze_surface_with_discourse("da tu mov", &discourse)
        .unwrap();
    let request = language
        .analyze_surface_with_discourse("me tu gov sit sal tis", &discourse)
        .unwrap();
    assert_eq!(command.inferred_type, "Utterance");
    assert_eq!(request.inferred_type, "Utterance");
    assert!(command.canonical_semantics.starts_with("da(content = mov("));
    assert!(request.canonical_semantics.starts_with("me(content = gov("));
}

#[test]
fn propositions_negation_quantification_and_coordination_are_playable() {
    let language = language();
    let discourse = conversation(&language);

    let positive = language
        .analyze_surface_with_discourse("mi viv", &discourse)
        .unwrap();
    let negative = language
        .analyze_surface_with_discourse("ne mi viv", &discourse)
        .unwrap();
    assert_ne!(positive.canonical_semantics, negative.canonical_semantics);

    let exists = language.analyze_surface("mu per viv").unwrap();
    let every = language.analyze_surface("ra per viv").unwrap();
    assert!(exists.canonical_semantics.starts_with("mu(predicate = bind"));
    assert!(every.canonical_semantics.starts_with("ra(predicate = bind"));

    let mixed = language
        .analyze_surface_with_discourse("mi viv zo tu viv va mi vid tu", &discourse)
        .unwrap();
    let explicit = language
        .analyze_surface_with_discourse("mi viv zo ki tu viv va mi vid tu ku", &discourse)
        .unwrap();
    assert_eq!(mixed.canonical_semantics, explicit.canonical_semantics);
}

#[test]
fn ambiguity_fails_and_exact_alias_repairs_it_without_ranking() {
    let language = language();
    let mut discourse = conversation(&language);
    let first = discourse
        .introduce(
            named(&language, "alek"),
            IntroductionOrigin::External {
                label: "first candidate".into(),
            },
            language.semantics(),
        )
        .unwrap();
    discourse
        .introduce(
            named(&language, "boris"),
            IntroductionOrigin::External {
                label: "second candidate".into(),
            },
            language.semantics(),
        )
        .unwrap();

    let error = language
        .analyze_surface_with_discourse("mi vid ref", &discourse)
        .unwrap_err()
        .to_string();
    assert!(error.contains("ambiguous"), "{error}");

    language.bind_alias(&mut discourse, "zen", first).unwrap();
    let repaired = language
        .analyze_surface_with_discourse("mi vid zen", &discourse)
        .unwrap();
    assert!(repaired.canonical_semantics.contains("payload = \"alek\""));
}

#[test]
fn context_dependent_properties_require_explicit_standards() {
    let language = language();
    assert_eq!(
        language
            .analyze_surface("na alfa nov na beta")
            .unwrap()
            .canonical_semantics,
        "nov(standard = na(payload = \"beta\"), value = na(payload = \"alfa\"))"
    );
    assert_eq!(
        language
            .analyze_surface("na alfa bon na beta")
            .unwrap()
            .canonical_semantics,
        "bon(criterion = na(payload = \"beta\"), value = na(payload = \"alfa\"))"
    );
    assert_eq!(
        language
            .analyze_surface("na alfa prok na beta na gama")
            .unwrap()
            .canonical_semantics,
        "prok(reference = na(payload = \"beta\"), standard = na(payload = \"gama\"), value = na(payload = \"alfa\"))"
    );
}

#[test]
fn text_and_communication_roots_accept_opaque_text_values() {
    let language = language();
    assert_eq!(
        language
            .analyze_surface("sit hello world tis tekst")
            .unwrap()
            .canonical_semantics,
        "tekst(value = \"hello world\")"
    );
    assert_eq!(
        language
            .analyze_surface("na artemi skrib sit hello world tis")
            .unwrap()
            .canonical_semantics,
        "skrib(content = \"hello world\", writer = na(payload = \"artemi\"))"
    );
}
