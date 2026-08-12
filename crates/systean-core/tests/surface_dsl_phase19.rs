use std::path::{Path, PathBuf};

use systean_core::language::LanguagePackage;
use systean_core::spec::{
    CompiledSurfaceItem, CompiledTerm, SourceSurfaceItem, TypedCompileError,
    compile_typed_sources, parse_typed_specification,
};
use systean_core::syntax::{LexemeConfig, SurfaceFormConfig};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    LanguagePackage::load(repository().join("language")).expect("Phase 19A language package")
}

#[test]
fn typed_word_form_syntax_parses_root_arguments_precedence_and_associativity() {
    let source = r#"
        type Proposition;
        word ne($value: Proposition) -> Proposition {
            form _ $value;
        }
        word va($left: Proposition, $right: Proposition) -> Proposition {
            form $left _ $right;
            precedence 20;
            associative;
        }
    "#;
    let parsed = parse_typed_specification(source).expect("surface DSL parses");
    let va = parsed
        .declarations
        .iter()
        .find(|declaration| declaration.source_name() == "va")
        .expect("va declaration");
    let systean_core::spec::TypedDeclaration::Word { surface: Some(surface), .. } = va else {
        panic!("va must carry one typed surface rule")
    };
    assert_eq!(surface.precedence, Some(20));
    assert!(surface.associative);
    assert_eq!(
        surface.items,
        vec![
            SourceSurfaceItem::Argument("left".into()),
            SourceSurfaceItem::Root,
            SourceSurfaceItem::Argument("right".into()),
        ]
    );
}

#[test]
fn compiled_surface_rules_use_argument_indices_not_parameter_names() {
    let left = r#"
        type Proposition;
        word va($left: Proposition, $right: Proposition) -> Proposition {
            form $left _ $right;
            precedence 20;
            associative;
        }
    "#;
    let right = r#"
        type Proposition;
        word va($a: Proposition, $b: Proposition) -> Proposition {
            form $a _ $b;
            precedence 20;
            associative;
        }
    "#;
    let left = compile_typed_sources([("left.semsys".into(), left.into())]).unwrap();
    let right = compile_typed_sources([("right.semsys".into(), right.into())]).unwrap();
    let id = left.symbol_id("va").unwrap();
    assert_eq!(id, right.symbol_id("va").unwrap());
    let left_rule = left.surface_rule(id).unwrap();
    let right_rule = right.surface_rule(id).unwrap();
    assert_eq!(left_rule.items, right_rule.items);
    assert_eq!(
        left_rule.items,
        vec![
            CompiledSurfaceItem::Argument(0),
            CompiledSurfaceItem::Root,
            CompiledSurfaceItem::Argument(1),
        ]
    );
    assert_eq!(left.surface_fingerprint(), right.surface_fingerprint());
}

#[test]
fn default_word_frames_and_explicit_forms_share_one_compiled_rule_model() {
    let source = r#"
        type Entity;
        type Proposition;
        word per($entity: Entity) -> Proposition;
        word vid($observer: Entity, $observed: Entity) -> Proposition;
        word ne($value: Proposition) -> Proposition {
            form _ $value;
        }
    "#;
    let package = compile_typed_sources([("surface.semsys".into(), source.into())]).unwrap();
    let per = package.surface_rule(package.symbol_id("per").unwrap()).unwrap();
    let vid = package.surface_rule(package.symbol_id("vid").unwrap()).unwrap();
    let ne = package.surface_rule(package.symbol_id("ne").unwrap()).unwrap();
    assert_eq!(per.items, vec![CompiledSurfaceItem::Argument(0), CompiledSurfaceItem::Root]);
    assert_eq!(
        vid.items,
        vec![
            CompiledSurfaceItem::Argument(0),
            CompiledSurfaceItem::Root,
            CompiledSurfaceItem::Argument(1),
        ]
    );
    assert_eq!(ne.items, vec![CompiledSurfaceItem::Root, CompiledSurfaceItem::Argument(0)]);
}

#[test]
fn surface_only_changes_do_not_change_semantic_fingerprint() {
    let prefix = r#"
        type Proposition;
        word op($value: Proposition) -> Proposition {
            form _ $value;
        }
    "#;
    let postfix = r#"
        type Proposition;
        word op($value: Proposition) -> Proposition {
            form $value _;
        }
    "#;
    let prefix = compile_typed_sources([("a.semsys".into(), prefix.into())]).unwrap();
    let postfix = compile_typed_sources([("b.semsys".into(), postfix.into())]).unwrap();
    assert_eq!(prefix.semantic_fingerprint(), postfix.semantic_fingerprint());
    assert_ne!(prefix.surface_fingerprint(), postfix.surface_fingerprint());
}

#[test]
fn malformed_or_noninvertible_surface_forms_fail_package_compilation() {
    for (source, expected) in [
        (
            r#"type Entity; word x($a: Entity) -> Entity { form _ $missing; }"#,
            "unknown",
        ),
        (
            r#"type Entity; word x($a: Entity) -> Entity { form _ $a $a; }"#,
            "duplicate",
        ),
        (
            r#"type Entity; word x($a: Entity, $b: Entity) -> Entity { form $a _; }"#,
            "missing",
        ),
        (
            r#"type Entity; word x($a: Entity) -> Entity { form $a; }"#,
            "root",
        ),
    ] {
        let errors = compile_typed_sources([("bad.semsys".into(), source.into())])
            .expect_err("invalid surface form must fail");
        assert!(
            errors.iter().any(|error| match (expected, error) {
                ("unknown", TypedCompileError::UnknownSurfaceParameter { .. })
                | ("duplicate", TypedCompileError::DuplicateSurfaceParameter { .. })
                | ("missing", TypedCompileError::MissingSurfaceParameter { .. })
                | ("root", TypedCompileError::InvalidSurfaceRootCount { .. }) => true,
                _ => false,
            }),
            "unexpected errors: {errors:?}"
        );
    }

    let errors = compile_typed_sources([(
        "bad-assoc.semsys".into(),
        r#"
            type Entity;
            type Proposition;
            word x($a: Entity, $b: Entity) -> Proposition {
                form $a _ $b;
                precedence 10;
                associative;
            }
        "#
        .into(),
    )])
    .expect_err("associativity requires an endomorphism over both operands");
    assert!(errors.iter().any(|error| matches!(error, TypedCompileError::InvalidAssociativeSurface { .. })));
}

#[test]
fn production_surface_ownership_moved_out_of_dictionary_and_syntax_precedence_table() {
    let language = language();
    let overlays = language
        .dictionary()
        .entries()
        .iter()
        .filter_map(|entry| entry.syntax.as_ref())
        .collect::<Vec<_>>();
    assert_eq!(
        overlays.len(),
        9,
        "only name, quantifier, counted-quantifier, and outer speech-act compatibility overlays remain in 19A",
    );
    assert!(overlays.iter().all(|surface| matches!(
        surface,
        SurfaceFormConfig::Name { .. }
            | SurfaceFormConfig::Quantifier { .. }
            | SurfaceFormConfig::CountedQuantifier { .. }
            | SurfaceFormConfig::SpeechAct { .. }
    )));

    let syntax_source = std::fs::read_to_string(repository().join("language/syntax.toml")).unwrap();
    let syntax: toml::Value = toml::from_str(&syntax_source).unwrap();
    assert!(syntax.get("logic").and_then(|logic| logic.get("precedence")).is_none());
}

#[test]
fn production_parser_and_linearizer_are_projected_from_the_same_typed_rules() {
    let language = language();
    let lexicon = language.syntax().lexicon();

    assert!(matches!(lexicon.get("per"), Some(LexemeConfig::Class { .. })));
    assert!(matches!(lexicon.get("vid"), Some(LexemeConfig::Predicate { .. })));
    assert!(matches!(lexicon.get("ne"), Some(LexemeConfig::Prefix { .. })));
    assert!(matches!(lexicon.get("ke"), Some(LexemeConfig::SpeechAct { .. })));
    let ke = language.typed_semantics().symbol_id("ke").unwrap();
    assert_eq!(
        language.typed_semantics().surface_rule(ke).unwrap().items,
        vec![CompiledSurfaceItem::Root, CompiledSurfaceItem::Argument(0)],
    );
    assert!(matches!(
        lexicon.get("va"),
        Some(LexemeConfig::Infix { precedence: 20, associative: true, .. })
    ));
    assert!(matches!(
        lexicon.get("imp"),
        Some(LexemeConfig::Infix { precedence: 5, associative: false, .. })
    ));

    for source in [
        "mi viv",
        "ne mi viv",
        "mi viv va tu viv",
        "ki mi viv zo tu viv ku va mi viv",
        "ke mi viv",
    ] {
        let parsed = language.syntax().parse(source).unwrap_or_else(|errors| {
            panic!("{source}: {errors:?}")
        });
        let canonical = language.syntax().linearize(&parsed).unwrap();
        let reparsed = language.syntax().parse(&canonical).unwrap();
        assert_eq!(parsed, reparsed, "round trip drift for {source} -> {canonical}");
    }

    assert!(
        language.syntax().parse("mi ke viv").is_err(),
        "Phase 19A must preserve outer-only speech-act placement until the generic typed parser replaces the backend category",
    );
}

#[test]
fn ne_va_zo_have_abstract_semantic_definitions_separate_from_surface_roots() {
    let language = language();
    let typed = language.typed_semantics();
    for (root, primitive) in [("ne", "logic.not"), ("va", "logic.and"), ("zo", "logic.or")] {
        let symbol = typed.symbol(typed.symbol_id(root).unwrap()).unwrap();
        let mut body = symbol.definition.as_ref().expect("defined surface word");
        while let CompiledTerm::Lambda { body: nested, .. } = body {
            body = nested;
        }
        let CompiledTerm::Apply { function, .. } = body else {
            panic!("{root} should lower to an abstract primitive")
        };
        assert_eq!(*function, typed.symbol_id(primitive).unwrap());
    }
}
