use std::fs;
use std::path::{Path, PathBuf};

use systean_core::discourse::DiscourseState;
use systean_core::language::LanguagePackage;
use systean_core::semantics::{Term, Type, canonicalize};
use systean_core::syntax::SurfaceExpr;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture_language(extra_dictionary: &str) -> LanguagePackage {
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
    dictionary.push('\n');
    dictionary.push_str(extra_dictionary);
    let core = fs::read_to_string(repo.join("language/semantics/core.semsys")).unwrap();
    let pragmatics = fs::read_to_string(repo.join("language/semantics/pragmatics.semsys")).unwrap();
    let subjective = fs::read_to_string(repo.join("language/semantics/subjective.semsys")).unwrap();
    let fixture = fs::read_to_string(repo.join("tests/fixtures/semantics/syntax.semsys")).unwrap();

    LanguagePackage::from_sources(
        &alphabet,
        &phonology,
        &morphology,
        &syntax,
        &dictionary,
        &[
            ("language/semantics/core.semsys", &core),
            ("language/semantics/pragmatics.semsys", &pragmatics),
            ("language/semantics/subjective.semsys", &subjective),
            ("tests/fixtures/semantics/syntax.semsys", &fixture),
        ],
    )
    .unwrap()
}

fn language() -> LanguagePackage {
    fixture_language("")
}

fn discourse(language: &LanguagePackage) -> DiscourseState {
    let mut discourse = DiscourseState::new();
    discourse
        .set_context_value("speaker", Term::Const("john".into()), language.semantics())
        .unwrap();
    discourse
        .set_context_value("addressee", Term::Const("mary".into()), language.semantics())
        .unwrap();
    discourse
}

fn semantics(source: &str) -> String {
    let language = language();
    language
        .analyze_surface_with_discourse(source, &discourse(&language))
        .unwrap()
        .canonical_semantics
}

#[test]
fn every_dictionary_root_is_compiled_into_the_surface_lexicon() {
    let language = language();
    assert_eq!(language.syntax().lexicon().len(), language.roots().roots().len());
    for root in language.roots().roots() {
        assert!(
            language.syntax().lexicon().contains_key(root),
            "dictionary root `{root}` was missing from the compiled surface lexicon"
        );
    }
}

#[test]
fn bare_constant_root_needs_no_duplicate_syntax_binding() {
    let language = language();
    let parsed = language.syntax().parse("sol").unwrap();
    assert_eq!(parsed, SurfaceExpr::Atom("sol".into()));

    let analysis = language.analyze_surface("sol").unwrap();
    assert_eq!(analysis.canonical_surface, "sol");
    assert_eq!(analysis.inferred_type, "Entity");
    assert_eq!(analysis.canonical_semantics, "sol");
}

#[test]
fn adding_a_constant_root_requires_no_syntax_config_change() {
    let language = fixture_language(
        r#"
[lu]
definition = "Fixture entity added only to the dictionary."
semantic = { kind = "constant", type = "Entity" }
"#,
    );

    assert!(language.syntax().lexicon().contains_key("lu"));
    assert_eq!(
        language.analyze_surface("lu si").unwrap().canonical_semantics,
        "sleep(sleeper = lu)"
    );
}

#[test]
fn invalid_operator_application_reaches_semantic_type_checking() {
    let language = language();
    let error = language.analyze_surface("ne sol").unwrap_err().to_string();
    assert!(error.contains("expects `Proposition` but received `Entity`"), "{error}");
    assert!(!error.contains("unknown lexical root"), "{error}");
}

#[test]
fn unknown_root_is_rejected_as_lexical_not_as_missing_config_binding() {
    let language = language();
    let error = language.syntax().parse("xax").unwrap_err().to_string();
    assert!(error.contains("unknown lexical root `xax`"), "{error}");
}

#[test]
fn canonical_frame_order_lowers_roles_without_role_markers() {
    assert_eq!(
        semantics("mi vi tu"),
        "see(observed = mary, observer = john)"
    );
}

#[test]
fn negation_scope_follows_surface_order() {
    let not_every = semantics("ne ra pe si");
    let every_not = semantics("ra pe ne si");
    assert_ne!(not_every, every_not);
    assert_eq!(
        not_every,
        "not(value = forall(predicate = bind v0: Entity => implies(condition = person(entity = v0), consequence = sleep(sleeper = v0))))"
    );
    assert_eq!(
        every_not,
        "forall(predicate = bind v0: Entity => implies(condition = person(entity = v0), consequence = not(value = sleep(sleeper = v0))))"
    );
}

#[test]
fn quantifiers_nest_in_order_of_appearance() {
    assert_eq!(
        semantics("ra pe vi mu kan"),
        "forall(predicate = bind v0: Entity => implies(condition = person(entity = v0), consequence = exists(predicate = bind v1: Entity => and(left = dog(entity = v1), right = see(observed = v1, observer = v0)))))"
    );
}

#[test]
fn and_binds_more_tightly_than_or() {
    let mixed = semantics("mi si zo tu si va mi vi tu");
    let explicit = semantics("mi si zo ki tu si va mi vi tu ku");
    assert_eq!(mixed, explicit);

    let opposite = semantics("ki mi si zo tu si ku va mi vi tu");
    assert_ne!(mixed, opposite);
}

#[test]
fn generator_inserts_scope_markers_only_when_precedence_requires_them() {
    let language = language();
    let parsed = language
        .syntax()
        .parse("ki mi si zo tu si ku va mi vi tu")
        .unwrap();
    assert_eq!(
        language.syntax().linearize(&parsed).unwrap(),
        "ki mi si zo tu si ku va mi vi tu"
    );

    let redundant = language.syntax().parse("ki mi si ku").unwrap();
    assert_eq!(language.syntax().linearize(&redundant).unwrap(), "mi si");
}

#[test]
fn same_logical_operator_chain_is_flattened() {
    let language = language();
    let parsed = language
        .syntax()
        .parse("mi si va tu si va mi vi tu")
        .unwrap();
    let SurfaceExpr::Infix { operator, operands } = parsed else {
        panic!("expected flattened infix expression");
    };
    assert_eq!(operator, "va");
    assert_eq!(operands.len(), 3);
}

#[test]
fn redundant_grouping_inside_associative_chains_is_normalized() {
    let language = language();
    let grouped = language
        .syntax()
        .parse("mi si va ki tu si va mi vi tu ku")
        .unwrap();
    let flat = language
        .syntax()
        .parse("mi si va tu si va mi vi tu")
        .unwrap();
    assert_eq!(grouped, flat);
    assert_eq!(
        language.syntax().linearize(&grouped).unwrap(),
        "mi si va tu si va mi vi tu"
    );
}

#[test]
fn explicit_speech_acts_do_not_use_word_order_tricks() {
    let question = semantics("ke mi si");
    let command = semantics("da mi si");
    let request = semantics("me mi si");
    assert_eq!(question, "ask_truth(content = sleep(sleeper = john))");
    assert_eq!(command, "command(content = sleep(sleeper = john))");
    assert_eq!(request, "request(content = sleep(sleeper = john))");
}

#[test]
fn omitted_frame_arguments_are_carried_to_discourse_resolution() {
    let language = language();
    let parsed = language.syntax().parse("mi vi").unwrap();
    let typed = language
        .syntax()
        .elaborate(&parsed, language.semantics())
        .unwrap();
    assert_eq!(typed.references.len(), 1);
    assert_eq!(typed.references[0].role, "observed");
    assert_eq!(typed.references[0].expected_type, Type::named("Entity"));
    assert!(language.analyze_surface("mi vi").is_err());
}

#[test]
fn noncanonical_argument_order_is_rejected() {
    let language = language();
    assert!(language.syntax().parse("vi mi tu").is_err());
}

#[test]
fn parse_linearize_round_trip_preserves_surface_ast() {
    let language = language();
    for source in [
        "sol",
        "mi vi tu",
        "mi vi",
        "ref si",
        "vi tu",
        "ne ra pe si",
        "ra pe ne si",
        "ra pe vi mu kan",
        "mi si zo tu si va mi vi tu",
        "ki mi si zo tu si ku va mi vi tu",
        "ke ne ki mi si va tu si ku",
    ] {
        let parsed = language.syntax().parse(source).unwrap();
        let canonical = language.syntax().linearize(&parsed).unwrap();
        let reparsed = language.syntax().parse(&canonical).unwrap();
        assert_eq!(parsed, reparsed, "source={source}, canonical={canonical}");
    }
}

#[test]
fn surface_lowering_is_type_checked() {
    let language = language();
    let analysis = language.analyze_surface("ra pe vi mu kan").unwrap();
    assert_eq!(analysis.inferred_type, "Proposition");
    let lowered = language
        .syntax()
        .lower(&analysis.syntax, language.semantics())
        .unwrap();
    assert_eq!(
        canonicalize(&lowered.term).to_string(),
        analysis.canonical_semantics
    );
}
