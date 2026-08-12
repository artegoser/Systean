use std::path::Path;

use systean_core::language::LanguagePackage;
use systean_core::semantics::{
    ContextSlotId, InformationKnowerValue, InformationStatus, Literal, StructuredValue, Term, Type,
};
use systean_core::spec::{
    CompiledScalar, CompiledSymbolKind, CompiledTerm, CompiledType, DefaultSurfaceItem,
    TypedCompileError,
    compile_typed_sources, parse_typed_specification, resolve_legacy_lexical_map,
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
    assert_eq!(old.semantic.family(), "calendar_date");
    assert!(matches!(
        old.semantic.value,
        StructuredValue::CalendarDate { year: 2026, month: 8, day: 12 }
    ));

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
fn production_words_are_owned_by_typed_systean_root_symbols() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let language = LanguagePackage::load(repository.join("language")).expect("Phase 18 package");
    let typed = language.typed_semantics();

    for (root, arity) in [("per", 1usize), ("viv", 1usize), ("vid", 2usize)] {
        let id = typed.symbol_id(root).expect("Systean-owned symbol id");
        let symbol = typed.symbol(id).unwrap();
        assert_eq!(symbol.kind, CompiledSymbolKind::Word);
        assert_eq!(symbol.signature.parameters.len(), arity);
        assert_eq!(typed.source_name_for_symbol(id), Some(root));
        assert!(language.semantics().operator(root).is_some());
    }

    for (root, context) in [("mi", "speaker"), ("tu", "addressee")] {
        let symbol = typed.symbol(typed.symbol_id(root).unwrap()).unwrap();
        assert_eq!(
            symbol.definition.as_ref(),
            Some(&CompiledTerm::Context(typed.context_slot_id(context).unwrap())),
        );
    }
}

#[test]
fn archived_phase17_aliases_resolve_to_phase18_ids_without_becoming_semantic_input() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let language = LanguagePackage::load(repository.join("language")).expect("Phase 18 package");
    let legacy = std::fs::read_to_string(repository.join("language/legacy/lexical-map.tsv")).unwrap();
    let mappings = resolve_legacy_lexical_map(language.typed_semantics(), &legacy).unwrap();
    assert_eq!(mappings.len(), language.dictionary().entries().len());

    for (root, old) in [("vid", "see"), ("per", "person"), ("viv", "alive")] {
        let mapping = mappings.iter().find(|mapping| mapping.root == root).unwrap();
        assert_eq!(mapping.legacy_semantic, old);
        assert_eq!(mapping.symbol, language.typed_semantics().symbol_id(root).unwrap());
        assert!(language.semantics().operator(old).is_none());
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
fn production_type_subtype_and_literal_ownership_is_in_the_typed_package() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let language = LanguagePackage::load(repository.join("language")).expect("Phase 18 package");
    let typed = language.typed_semantics();

    let event = typed.type_id("Event").unwrap();
    let occurrence = typed.type_id("Occurrence").unwrap();
    assert!(typed.subtypes().any(|edge| edge == (event, occurrence)));
    assert_eq!(
        typed.literal_types().find(|(kind, _)| *kind == systean_core::semantics::LiteralKind::String).map(|(_, ty)| ty),
        Some(&CompiledType::Named(typed.type_id("Text").unwrap())),
    );
}

#[test]
fn typed_subtype_cycles_are_rejected_before_environment_projection() {
    let source = r#"
        type A;
        type B;
        subtype A: B;
        subtype B: A;
    "#;
    let errors = compile_typed_sources([("subtypes.semsys".to_owned(), source.to_owned())])
        .expect_err("subtype cycle must fail");
    assert!(errors.iter().any(|error| matches!(error, TypedCompileError::CyclicSubtype(_))));
}

#[test]
fn phase17_checker_projection_is_derived_from_phase18_root_symbols() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let package = LanguagePackage::load(repository.join("language")).expect("Phase 18 package loads");
    assert_eq!(package.semantics().operator("vid").unwrap().returns, Type::named("Proposition"));
    assert!(package.semantics().operator("see").is_none());
    assert!(package.analyze_surface("na artemi vid na mari").is_ok());
}


#[test]
fn production_structured_literals_store_typed_values_not_canonical_string_envelopes() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let language = LanguagePackage::load(repository.join("language")).expect("language package");
    let literals = language.literals().expect("literal engine");

    let number = literals.parse_complete("10.5").expect("number");
    assert!(matches!(number.semantic.value, StructuredValue::Number(_)));

    let quantity = literals.parse_complete("1 km").expect("quantity");
    let StructuredValue::Quantity { unit, approximate: false, .. } = quantity.semantic.value else {
        panic!("quantity should store a typed quantity value")
    };
    assert_eq!(unit, literals.units().unit_by_name("kilometr").unwrap().id);

    let date = literals.parse_complete("2026-08-12").expect("date");
    assert!(matches!(
        date.semantic.value,
        StructuredValue::CalendarDate { year: 2026, month: 8, day: 12 }
    ));

    let duration = literals.parse_complete("PT60S").expect("duration");
    assert!(matches!(duration.semantic.value, StructuredValue::Duration { .. }));

    let interval = literals
        .parse_complete("2026-08-12/2026-08-13")
        .expect("interval");
    assert!(matches!(interval.semantic.value, StructuredValue::Interval { .. }));
}

#[test]
fn runtime_unit_ids_match_the_typed_phase18_unit_bridge() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let language = LanguagePackage::load(repository.join("language")).expect("language package");
    let runtime = language.literals().expect("literal engine").units();
    let typed = compile_typed_sources([(
        "units.semsys".to_owned(),
        include_str!("../../../language/typed/units.semsys").to_owned(),
    )])
    .expect("typed unit bridge should compile");

    for (legacy_name, root) in [
        ("meter", "metr"),
        ("kilometer", "kilometr"),
        ("centimeter", "santimetr"),
        ("second", "sek"),
        ("millisecond", "milisek"),
        ("hour", "hor"),
        ("kilogram", "kilogram"),
        ("gram", "gram"),
        ("kelvin", "kelvin"),
        ("ampere", "amper"),
        ("mole", "mol"),
        ("candela", "kandela"),
    ] {
        let runtime_unit = runtime.unit_by_name(legacy_name).unwrap_or_else(|| panic!("runtime unit {legacy_name}"));
        let typed_id = typed.unit_id(root).unwrap_or_else(|| panic!("typed unit {root}"));
        let typed_unit = typed.unit(typed_id).unwrap();
        assert_eq!(runtime_unit.id, typed_id, "runtime identity drift for {root}");
        assert_eq!(runtime_unit.dimension, typed_unit.dimension, "dimension drift for {root}");
        assert_eq!(runtime_unit.scale.numer().to_string(), typed_unit.scale_numerator.to_string());
        assert_eq!(runtime_unit.scale.denom().to_string(), typed_unit.scale_denominator.to_string());
    }
}

#[test]
fn information_context_is_structural_before_discourse_resolution() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let language = LanguagePackage::load(repository.join("language")).expect("language package");
    let syntax = language.syntax().parse("na artemi vid unk tu").expect("surface parse");
    let typed = language
        .syntax()
        .elaborate(&syntax, language.semantics())
        .expect("surface elaboration");
    let information = find_information(&typed.template).expect("information value");
    let StructuredValue::Information { mode, status, knower } = information else {
        unreachable!()
    };
    let typed = compile_typed_sources([(
        "representative.semsys".to_owned(),
        representative_source().to_owned(),
    )])
    .expect("typed representative package");
    assert_eq!(*mode, typed.constructor_id("InfoMode.unknown").unwrap());
    assert_eq!(*status, InformationStatus::Unknown);
    assert_eq!(
        knower.as_ref(),
        Some(&InformationKnowerValue::Context(ContextSlotId::from_source(
            "context",
            "addressee",
        ))),
    );
    assert!(!format!("{information:?}").contains("unknown:context:"));
}


#[test]
fn phase18_typed_sources_are_normative_package_provenance_and_legacy_sources_are_not() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let language = LanguagePackage::load(repository.join("language")).expect("language package");
    let sources = language
        .provenance()
        .sources
        .iter()
        .map(|source| source.path.as_str())
        .collect::<Vec<_>>();
    for required in ["typed/core.semsys", "typed/lexicon.semsys", "typed/units.semsys"] {
        assert!(sources.contains(&required), "missing normative source {required}");
    }
    assert!(sources.iter().all(|source| !source.starts_with("legacy/")));
}

#[test]
fn phase18_runtime_has_no_structured_value_string_reparse_or_duration_name_hardcode() {
    let literals = include_str!("../src/literals/mod.rs");
    let pragmatics = include_str!("../src/pragmatics.rs");
    let workbench = include_str!("../src/workbench.rs");

    for source in [literals, pragmatics, workbench] {
        assert!(!source.contains(".canonical.split"));
        assert!(!source.contains("canonical.splitn"));
        assert!(!source.contains("semantic.canonical"));
    }
    assert!(!literals.contains("unit.dimension != \"time\""));
    assert!(!literals.contains("unit_by_id(\"second\")"));
}

#[test]
fn archived_phase17_operator_signatures_match_phase18_projection() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let language = LanguagePackage::load(repository.join("language")).expect("Phase 18 package");
    let old = systean_core::spec::compile_path(repository.join("language/legacy/semantics"))
        .expect("archived Phase 17 semantics");
    let source = std::fs::read_to_string(repository.join("language/legacy/lexical-map.tsv")).unwrap();
    let mappings = resolve_legacy_lexical_map(language.typed_semantics(), &source).unwrap();

    let mut lexical_operators = 0usize;
    for mapping in &mappings {
        let Some(old_signature) = old.operator(&mapping.legacy_semantic) else { continue; };
        let new_signature = language
            .semantics()
            .operator(&mapping.root)
            .unwrap_or_else(|| panic!("typed root `{}` lost its compatibility signature", mapping.root));
        assert_signatures_alpha_equivalent(old_signature, new_signature, &mapping.root);
        lexical_operators += 1;
    }
    assert_eq!(lexical_operators, 167);

    for primitive in [
        "activity_of",
        "assert",
        "duration_of",
        "equal",
        "event_of",
        "interval",
        "process_of",
        "state_of",
    ] {
        assert_signatures_alpha_equivalent(
            old.operator(primitive).unwrap(),
            language.semantics().operator(primitive).unwrap(),
            primitive,
        );
    }
}

fn assert_signatures_alpha_equivalent(
    old: &systean_core::semantics::Signature,
    new: &systean_core::semantics::Signature,
    owner: &str,
) {
    assert_eq!(old.type_parameters.len(), new.type_parameters.len(), "generic arity drift for {owner}");
    assert_eq!(old.parameters.len(), new.parameters.len(), "parameter arity drift for {owner}");
    for (old, new) in old.parameters.iter().zip(&new.parameters) {
        assert_eq!(old.name, new.name, "parameter role drift for {owner}");
        assert_eq!(normalized_type(&old.ty), normalized_type(&new.ty), "parameter type drift for {owner}/{}", old.name);
    }
    assert_eq!(normalized_type(&old.returns), normalized_type(&new.returns), "return type drift for {owner}");
}

fn normalized_type(ty: &Type) -> Type {
    use systean_core::semantics::FunctionParameter;
    match ty {
        Type::Named(name) => Type::Named(name.clone()),
        Type::Generic { name, arguments } => Type::Generic {
            name: name.clone(),
            arguments: arguments.iter().map(normalized_type).collect(),
        },
        Type::Variable(_) => Type::Variable("_".into()),
        Type::Function { parameters, returns } => Type::Function {
            parameters: parameters
                .iter()
                .map(|parameter| FunctionParameter {
                    name: None,
                    ty: normalized_type(&parameter.ty),
                })
                .collect(),
            returns: Box::new(normalized_type(returns)),
        },
        Type::Record(fields) => Type::Record(
            fields
                .iter()
                .map(|(name, ty)| (name.clone(), normalized_type(ty)))
                .collect(),
        ),
    }
}

fn find_information(term: &Term) -> Option<&StructuredValue> {
    match term {
        Term::Literal(Literal::Structured(literal))
            if matches!(&literal.value, StructuredValue::Information { .. }) =>
        {
            Some(&literal.value)
        }
        Term::Call { arguments, .. } | Term::Record(arguments) => {
            arguments.values().find_map(find_information)
        }
        Term::Bind { body, .. } => find_information(body),
        Term::Field { record, .. } => find_information(record),
        _ => None,
    }
}
