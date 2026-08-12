use systean_core::semantics::Type;
use systean_core::spec::{compile_specification, parse_specification};

#[test]
fn core_spec_parses_and_compiles() {
    let source = include_str!("../../../language/legacy/semantics/core.semsys");
    let spec = parse_specification(source).expect("core semantic spec should parse");
    let environment = compile_specification(&spec).expect("core semantic spec should compile");

    assert!(environment.operator("cease").is_some());
    assert!(environment.operator("forall").is_some());
    assert_eq!(
        environment.literal_type(systean_core::semantics::LiteralKind::Integer),
        Some(&Type::named("Number"))
    );
}


#[test]
fn generic_type_arity_is_declared_and_checked() {
    let source = r#"
        type Entity;
        type Pair<A, B>;
        const pair: Pair<Entity, Entity>;
    "#;
    let spec = parse_specification(source).unwrap();
    compile_specification(&spec).unwrap();

    let invalid = r#"
        type Entity;
        type Pair<A, B>;
        const pair: Pair<Entity>;
    "#;
    let spec = parse_specification(invalid).unwrap();
    assert!(compile_specification(&spec).is_err());
}

#[test]
fn subtype_cycles_are_rejected() {
    let source = r#"
        type Entity;
        type Person;
        subtype Person: Entity;
        subtype Entity: Person;
    "#;
    let spec = parse_specification(source).unwrap();
    assert!(compile_specification(&spec).is_err());
}

#[test]
fn semantic_package_directory_compiles_with_provenance() {
    let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let core = std::fs::read_to_string(repository.join("language/legacy/semantics/core.semsys")).unwrap();
    let demo = std::fs::read_to_string(repository.join("tests/fixtures/semantics/demo.semsys")).unwrap();
    let environment = systean_core::spec::compile_sources([
        ("language/legacy/semantics/core.semsys".to_owned(), core),
        ("tests/fixtures/semantics/demo.semsys".to_owned(), demo),
    ])
    .expect("semantic fixtures should compile");
    let origin = environment.operator_origin("smoke").expect("origin");
    assert!(origin.source.ends_with("demo.semsys"));
    assert_eq!(origin.declaration, 1);
}
