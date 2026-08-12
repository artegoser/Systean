use std::path::Path;

use systean_core::language::{LanguagePackage, LexicalSemantic};
use systean_core::semantics::Type;
use systean_core::spec::{
    CompiledScalar, CompiledSymbolKind, CompiledTerm, CompiledType, DefaultSurfaceItem,
    TypedCompileError,
    compile_typed_sources, parse_typed_specification,
};

fn representative_source() -> &'static str {
    include_str!("../../../tests/fixtures/phase18/representative.semsys")
}

#[test]
fn phase18_dsl_parses_all_core_declaration_families() {
    let specification = parse_typed_specification(representative_source())
        .expect("representative Phase 18 DSL should parse");
    let names = specification
        .declarations
        .iter()
        .map(|declaration| declaration.source_name())
        .collect::<Vec<_>>();

    for expected in [
        "Entity",
        "Proposition",
        "InfoValue",
        "Int",
        "Date",
        "InfoMode",
        "reference_date",
        "speaker",
        "addressee",
        "Time",
        "sek",
        "milisek",
        "hole",
        "mi",
        "tu",
        "per",
        "viv",
        "vid",
        "unk",
        "vak",
        "hid",
        "keep",
        "constant_predicate",
        "apply_predicate",
        "opaque_relation",
    ] {
        assert!(names.contains(&expected), "missing declaration {expected}");
    }
}

#[test]
fn compiled_semantics_resolve_source_names_to_ids_and_discard_parameter_names() {
    let package = compile_typed_sources([(
        "representative.semsys".to_owned(),
        representative_source().to_owned(),
    )])
    .expect("typed semantic package should compile");

    let vid = package.symbol_id("vid").expect("vid id");
    let vid_symbol = package.symbol(vid).expect("vid symbol");
    assert_eq!(vid_symbol.kind, CompiledSymbolKind::Word);
    assert_eq!(vid_symbol.signature.parameters.len(), 2);
    assert_eq!(
        vid_symbol.signature.returns,
        CompiledType::Named(package.type_id("Proposition").expect("Proposition id")),
    );

    let debug = package.debug_symbol(vid).expect("debug symbol");
    assert_eq!(debug.parameter_names, vec!["observer".to_owned(), "observed".to_owned()]);
    assert!(vid_symbol.provenance.span.is_some(), "source location should survive compilation");
    let entity = package.type_id("Entity").expect("Entity id");
    assert!(package.type_provenance(entity).and_then(|provenance| provenance.span).is_some());
    let time = package.dimension_id("Time").expect("Time dimension");
    assert!(package.dimension_provenance(time).and_then(|provenance| provenance.span).is_some());
    assert!(!format!("{:?}", vid_symbol.signature).contains("observer"));
    assert!(!format!("{:?}", vid_symbol.signature).contains("observed"));
}

#[test]
fn local_parameter_and_lambda_renames_are_alpha_equivalent() {
    let left = r#"
        type Entity;
        type Proposition;
        def constant($content: Proposition) -> fn(value: Entity) -> Proposition =
            fn ($person: Entity) => $content;
    "#;
    let right = r#"
        // Documentation and formatting are intentionally different.
        type Entity;
        type Proposition;

        def constant($p: Proposition) -> fn(item: Entity) -> Proposition =
            fn ($x: Entity) => $p;
    "#;

    let left = compile_typed_sources([("left.semsys".to_owned(), left.to_owned())]).unwrap();
    let right = compile_typed_sources([("right.semsys".to_owned(), right.to_owned())]).unwrap();

    assert_eq!(left.symbol_id("constant"), right.symbol_id("constant"));
    let left_symbol = left.symbol(left.symbol_id("constant").unwrap()).unwrap();
    let right_symbol = right.symbol(right.symbol_id("constant").unwrap()).unwrap();
    assert_eq!(left_symbol.signature, right_symbol.signature);
    assert_eq!(left_symbol.definition, right_symbol.definition);
    assert_eq!(left.semantic_fingerprint(), right.semantic_fingerprint());
    assert_eq!(left.surface_fingerprint(), right.surface_fingerprint());
}

#[test]
fn declaration_order_does_not_change_ids_or_semantic_fingerprint() {
    let left = r#"
        type Entity;
        type Proposition;
        word per($x: Entity) -> Proposition;
        word vid($a: Entity, $b: Entity) -> Proposition;
    "#;
    let right = r#"
        word vid($left: Entity, $right: Entity) -> Proposition;
        type Proposition;
        word per($value: Entity) -> Proposition;
        type Entity;
    "#;
    let left = compile_typed_sources([("left.semsys".to_owned(), left.to_owned())]).unwrap();
    let right = compile_typed_sources([("right.semsys".to_owned(), right.to_owned())]).unwrap();
    assert_eq!(left.symbol_id("vid"), right.symbol_id("vid"));
    assert_eq!(left.symbol_id("per"), right.symbol_id("per"));
    assert_eq!(left.semantic_fingerprint(), right.semantic_fingerprint());
    assert_eq!(left.surface_fingerprint(), right.surface_fingerprint());
}

#[test]
fn semantic_signature_changes_change_semantic_but_not_unrelated_source_formatting() {
    let baseline = r#"
        type Entity;
        type Proposition;
        word vid($a: Entity, $b: Entity) -> Proposition;
    "#;
    let changed = r#"
        type Entity;
        type Proposition;
        word vid($a: Entity) -> Proposition;
    "#;

    let baseline = compile_typed_sources([("a.semsys".to_owned(), baseline.to_owned())]).unwrap();
    let changed = compile_typed_sources([("b.semsys".to_owned(), changed.to_owned())]).unwrap();
    assert_ne!(baseline.semantic_fingerprint(), changed.semantic_fingerprint());
    assert_ne!(baseline.surface_fingerprint(), changed.surface_fingerprint());
}

#[test]
fn algebraic_constructors_are_scoped_by_their_data_type() {
    let source = r#"
        data Left { none; }
        data Right { none; }
        def left : Left = Left.none;
        def right : Right = Right.none;
    "#;
    let package = compile_typed_sources([("constructors.semsys".to_owned(), source.to_owned())]).unwrap();
    assert_ne!(
        package.constructor_id("Left.none").unwrap(),
        package.constructor_id("Right.none").unwrap(),
    );
}

#[test]
fn information_values_are_structural_in_the_new_ir() {
    let package = compile_typed_sources([(
        "representative.semsys".to_owned(),
        representative_source().to_owned(),
    )])
    .unwrap();

    let hidden = package.symbol(package.symbol_id("hid").unwrap()).unwrap();
    let Some(CompiledTerm::Apply { function, arguments }) = peel_zero_parameter_definition(hidden.definition.as_ref().unwrap()) else {
        panic!("hid should lower to a structural hole application")
    };
    assert_eq!(*function, package.symbol_id("hole").unwrap());
    assert_eq!(arguments.len(), 1);
    let CompiledTerm::Constructor { constructor, fields } = &arguments[0] else {
        panic!("hid mode should be an algebraic constructor")
    };
    assert_eq!(*constructor, package.constructor_id("InfoMode.withheld").unwrap());
    assert!(fields.is_empty());

    let unknown = package.symbol(package.symbol_id("unk").unwrap()).unwrap();
    assert!(!format!("{:?}", unknown.definition).contains("unknown:"));
    assert!(!format!("{:?}", unknown.definition).contains("context:"));
}

#[test]
fn representative_date_is_structural_in_the_new_path_and_matches_the_phase17_value() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let legacy = LanguagePackage::load(repository.join("language")).expect("Phase 17 package");
    let old = legacy
        .literals()
        .expect("literal engine")
        .parse_complete("2026-08-12")
        .expect("legacy date literal");
    assert_eq!(old.semantic.family, "calendar_date");
    assert_eq!(old.semantic.canonical, "2026-08-12");

    let typed = compile_typed_sources([(
        "representative.semsys".to_owned(),
        representative_source().to_owned(),
    )])
    .unwrap();
    let reference_date = typed
        .symbol(typed.symbol_id("reference_date").unwrap())
        .unwrap();
    let Some(CompiledTerm::Constructor { constructor, fields }) = reference_date.definition.as_ref() else {
        panic!("reference_date should be a structural Date constructor")
    };
    assert_eq!(*constructor, typed.constructor_id("Date.date").unwrap());
    assert_eq!(
        fields,
        &[
            CompiledTerm::Scalar(CompiledScalar::Integer(2026)),
            CompiledTerm::Scalar(CompiledScalar::Integer(8)),
            CompiledTerm::Scalar(CompiledScalar::Integer(12)),
        ],
    );
    assert!(!format!("{reference_date:?}").contains("2026-08-12"));
}

#[test]
fn context_and_units_are_resolved_without_runtime_source_strings() {
    let package = compile_typed_sources([(
        "representative.semsys".to_owned(),
        representative_source().to_owned(),
    )])
    .unwrap();

    let mi = package.symbol(package.symbol_id("mi").unwrap()).unwrap();
    assert_eq!(
        mi.definition.as_ref(),
        Some(&CompiledTerm::Context(package.context_slot_id("speaker").unwrap())),
    );

    let millis = package.unit(package.unit_id("milisek").unwrap()).unwrap();
    assert_eq!(millis.dimension, package.dimension_id("Time").unwrap());
    assert_eq!(millis.scale_numerator, 1);
    assert_eq!(millis.scale_denominator, 1000);
    assert_eq!(millis.base, Some(package.unit_id("sek").unwrap()));
}


#[test]
fn unit_definitions_are_exact_id_based_relations_and_reject_invalid_graphs() {
    let normalized = r#"
        dimension Time;
        unit sek : Time;
        unit half : Time = 2/4 * sek;
    "#;
    let package = compile_typed_sources([("units.semsys".to_owned(), normalized.to_owned())]).unwrap();
    let half = package.unit(package.unit_id("half").unwrap()).unwrap();
    assert_eq!((half.scale_numerator, half.scale_denominator), (1, 2));
    assert_eq!(half.base, Some(package.unit_id("sek").unwrap()));

    let mismatch = r#"
        dimension Time;
        dimension Length;
        unit sek : Time;
        unit broken : Length = 1 * sek;
    "#;
    let errors = compile_typed_sources([("mismatch.semsys".to_owned(), mismatch.to_owned())])
        .expect_err("cross-dimension unit derivation must fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypedCompileError::UnitDimensionMismatch { unit, base }
            if unit == "broken" && base == "sek"
    )));

    let cycle = r#"
        dimension Time;
        unit a : Time = 1 * b;
        unit b : Time = 1 * a;
    "#;
    let errors = compile_typed_sources([("cycle.semsys".to_owned(), cycle.to_owned())])
        .expect_err("unit derivation cycle must fail");
    assert!(errors.iter().any(|error| matches!(error, TypedCompileError::CyclicUnitDefinition(_))));
}

#[test]
fn higher_order_parameters_compile_to_generic_invocation_over_bound_ids() {
    let package = compile_typed_sources([(
        "representative.semsys".to_owned(),
        representative_source().to_owned(),
    )])
    .unwrap();
    let apply = package.symbol(package.symbol_id("apply_predicate").unwrap()).unwrap();
    let Some(CompiledTerm::Lambda { body, .. }) = &apply.definition else {
        panic!("first source parameter should compile to a lambda")
    };
    let CompiledTerm::Lambda { body, .. } = body.as_ref() else {
        panic!("second source parameter should compile to a nested lambda")
    };
    let CompiledTerm::Invoke { function, arguments } = body.as_ref() else {
        panic!("bound function application must be structural")
    };
    assert_eq!(function.as_ref(), &CompiledTerm::Bound(1));
    assert_eq!(arguments, &[CompiledTerm::Bound(0)]);
}


#[test]
fn duplicate_declarations_and_parameter_names_are_rejected() {
    let duplicate_symbol = r#"
        type Entity;
        word lum : Entity;
        word lum : Entity;
    "#;
    let errors = compile_typed_sources([(
        "duplicate.semsys".to_owned(),
        duplicate_symbol.to_owned(),
    )])
    .expect_err("duplicate symbol should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypedCompileError::DuplicateDeclaration { namespace: "symbol", name } if name == "lum"
    )));

    let duplicate_parameter = r#"
        type Entity;
        word vid($x: Entity, $x: Entity) -> Entity;
    "#;
    let errors = compile_typed_sources([(
        "duplicate-parameter.semsys".to_owned(),
        duplicate_parameter.to_owned(),
    )])
    .expect_err("duplicate parameter should fail");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypedCompileError::DuplicateParameter { declaration, parameter }
            if declaration == "vid" && parameter == "x"
    )));
}

#[test]
fn forward_references_do_not_bypass_compiled_arity_validation() {
    let source = r#"
        type Entity;
        type Proposition;
        def first($x: Entity) -> Proposition = later($x, $x);
        word later($x: Entity) -> Proposition;
    "#;
    let errors = compile_typed_sources([("forward.semsys".to_owned(), source.to_owned())])
        .expect_err("forward call with the wrong arity should fail after all signatures resolve");
    assert!(errors.iter().any(|error| matches!(
        error,
        TypedCompileError::InvalidCallArity { name, expected: 1, actual: 2 }
            if name == "later"
    )));
}

#[test]
fn definition_cycles_are_rejected() {
    let source = r#"
        type Entity;
        def a : Entity = b;
        def b : Entity = a;
    "#;
    let errors = compile_typed_sources([("cycle.semsys".to_owned(), source.to_owned())])
        .expect_err("definition cycle should fail");
    assert!(errors.iter().any(|error| matches!(error, TypedCompileError::CyclicDefinition(_))));
}

#[test]
fn changing_a_word_root_changes_the_surface_fingerprint() {
    let left = r#"
        type Entity;
        type Proposition;
        word vid($a: Entity, $b: Entity) -> Proposition;
    "#;
    let right = r#"
        type Entity;
        type Proposition;
        word vis($a: Entity, $b: Entity) -> Proposition;
    "#;
    let left = compile_typed_sources([("left.semsys".to_owned(), left.to_owned())]).unwrap();
    let right = compile_typed_sources([("right.semsys".to_owned(), right.to_owned())]).unwrap();
    assert_ne!(left.surface_fingerprint(), right.surface_fingerprint());
}

#[test]
fn ordinary_word_growth_needs_one_dsl_declaration() {
    let source = r#"
        type Entity;
        type Proposition;
        word lum($entity: Entity) -> Proposition;
    "#;
    let package = compile_typed_sources([("word.semsys".to_owned(), source.to_owned())]).unwrap();
    let lum = package.symbol(package.symbol_id("lum").unwrap()).unwrap();
    assert_eq!(lum.kind, CompiledSymbolKind::Word);
    assert_eq!(lum.signature.parameters.len(), 1);
}


#[test]
fn ordinary_words_derive_the_phase17_default_surface_frame_from_arity() {
    let package = compile_typed_sources([(
        "representative.semsys".to_owned(),
        representative_source().to_owned(),
    )])
    .unwrap();

    let per = package.default_surface_frame(package.symbol_id("per").unwrap()).unwrap();
    assert_eq!(per.root, "per");
    assert_eq!(
        per.items,
        vec![DefaultSurfaceItem::Argument(0), DefaultSurfaceItem::Root],
    );

    let vid = package.default_surface_frame(package.symbol_id("vid").unwrap()).unwrap();
    assert_eq!(
        vid.items,
        vec![
            DefaultSurfaceItem::Argument(0),
            DefaultSurfaceItem::Root,
            DefaultSurfaceItem::Argument(1),
        ],
    );
}

#[test]
fn representative_words_match_phase17_signatures_while_owning_new_systean_ids() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let legacy = LanguagePackage::load(repository.join("language")).expect("Phase 17 package");
    let typed = compile_typed_sources([(
        "representative.semsys".to_owned(),
        representative_source().to_owned(),
    )])
    .unwrap();

    for (root, old_operator, arity) in [
        ("per", "person", 1usize),
        ("viv", "alive", 1usize),
        ("vid", "see", 2usize),
    ] {
        let entry = legacy
            .dictionary()
            .entries()
            .iter()
            .find(|entry| entry.root == root)
            .expect("legacy dictionary entry");
        let LexicalSemantic::Operator { name } = &entry.semantic else {
            panic!("{root} should be a legacy operator")
        };
        assert_eq!(name, old_operator);
        assert_eq!(legacy.semantics().operator(old_operator).unwrap().parameters.len(), arity);

        let id = typed.symbol_id(root).expect("new Systean-owned symbol id");
        let symbol = typed.symbol(id).unwrap();
        assert_eq!(symbol.signature.parameters.len(), arity);
        assert_ne!(format!("{id}"), old_operator);
    }

    for (root, old_key) in [("mi", "speaker"), ("tu", "addressee")] {
        let entry = legacy.dictionary().entries().iter().find(|entry| entry.root == root).unwrap();
        let LexicalSemantic::Context { key, .. } = &entry.semantic else {
            panic!("{root} should be a legacy context value")
        };
        assert_eq!(key, old_key);
        let symbol = typed.symbol(typed.symbol_id(root).unwrap()).unwrap();
        assert!(matches!(symbol.definition.as_ref(), Some(CompiledTerm::Context(_))));
    }
}

#[test]
fn compiled_scalars_are_values_not_canonical_string_envelopes() {
    let source = r#"
        type Number;
        def one : Number = 1;
        def yes : Number = true;
        def text : Number = "opaque";
    "#;
    let package = compile_typed_sources([("scalar.semsys".to_owned(), source.to_owned())]).unwrap();
    let one = package.symbol(package.symbol_id("one").unwrap()).unwrap();
    assert_eq!(
        one.definition.as_ref(),
        Some(&CompiledTerm::Scalar(CompiledScalar::Integer(1))),
    );
}

fn peel_zero_parameter_definition(term: &CompiledTerm) -> Option<&CompiledTerm> {
    match term {
        CompiledTerm::Lambda { .. } => None,
        other => Some(other),
    }
}

#[test]
fn legacy_environment_still_keeps_phase17_behavior_during_dual_path_migration() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let package = LanguagePackage::load(repository.join("language")).expect("legacy package loads");
    assert_eq!(package.semantics().operator("see").unwrap().returns, Type::named("Proposition"));
    assert!(package.analyze("mi vid tu").is_ok());
}
