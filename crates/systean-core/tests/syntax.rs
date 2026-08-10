use std::fs;
use std::path::{Path, PathBuf};

use systean_core::semantics::canonicalize;
use systean_core::spec::compile_sources;
use systean_core::syntax::{SurfaceExpr, SyntaxConfig, SyntaxEngine};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn engine() -> (SyntaxEngine, systean_core::semantics::Environment) {
    let repo = repository();
    let config = SyntaxConfig::from_toml(
        &fs::read_to_string(repo.join("tests/fixtures/syntax/surface.toml")).unwrap(),
    )
    .unwrap();
    let environment = compile_sources([
        (
            "language/semantics/core.semsys".to_owned(),
            fs::read_to_string(repo.join("language/semantics/core.semsys")).unwrap(),
        ),
        (
            "tests/fixtures/semantics/syntax.semsys".to_owned(),
            fs::read_to_string(repo.join("tests/fixtures/semantics/syntax.semsys")).unwrap(),
        ),
    ])
    .unwrap();
    (SyntaxEngine::new(config), environment)
}

fn semantics(source: &str) -> String {
    let (engine, environment) = engine();
    engine.analyze(source, &environment).unwrap().canonical_semantics
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
        semantics("ra pe vi mu du"),
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
    let (engine, _) = engine();
    let parsed = engine.parse("ki mi si zo tu si ku va mi vi tu").unwrap();
    assert_eq!(
        engine.linearize(&parsed).unwrap(),
        "ki mi si zo tu si ku va mi vi tu"
    );

    let redundant = engine.parse("ki mi si ku").unwrap();
    assert_eq!(engine.linearize(&redundant).unwrap(), "mi si");
}

#[test]
fn same_logical_operator_chain_is_flattened() {
    let (engine, _) = engine();
    let parsed = engine.parse("mi si va tu si va mi vi tu").unwrap();
    let SurfaceExpr::Infix { operator, operands } = parsed else {
        panic!("expected flattened infix expression");
    };
    assert_eq!(operator, "va");
    assert_eq!(operands.len(), 3);
}

#[test]
fn redundant_grouping_inside_associative_chains_is_normalized() {
    let (engine, _) = engine();
    let grouped = engine.parse("mi si va ki tu si va mi vi tu ku").unwrap();
    let flat = engine.parse("mi si va tu si va mi vi tu").unwrap();
    assert_eq!(grouped, flat);
    assert_eq!(engine.linearize(&grouped).unwrap(), "mi si va tu si va mi vi tu");
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
fn required_frame_arguments_are_not_omitted_without_discourse_resolution() {
    let (engine, _) = engine();
    assert!(engine.parse("mi vi").is_err());
}

#[test]
fn noncanonical_argument_order_is_rejected() {
    let (engine, _) = engine();
    assert!(engine.parse("vi mi tu").is_err());
}

#[test]
fn parse_linearize_round_trip_preserves_surface_ast() {
    let (engine, _) = engine();
    for source in [
        "mi vi tu",
        "ne ra pe si",
        "ra pe ne si",
        "ra pe vi mu du",
        "mi si zo tu si va mi vi tu",
        "ki mi si zo tu si ku va mi vi tu",
        "ke ne ki mi si va tu si ku",
    ] {
        let parsed = engine.parse(source).unwrap();
        let canonical = engine.linearize(&parsed).unwrap();
        let reparsed = engine.parse(&canonical).unwrap();
        assert_eq!(parsed, reparsed, "source={source}, canonical={canonical}");
    }
}

#[test]
fn surface_lowering_is_type_checked() {
    let (engine, environment) = engine();
    let analysis = engine.analyze("ra pe vi mu du", &environment).unwrap();
    assert_eq!(analysis.inferred_type, "Proposition");
    let lowered = engine.lower(&analysis.syntax, &environment).unwrap();
    assert_eq!(canonicalize(&lowered.term).to_string(), analysis.canonical_semantics);
}
