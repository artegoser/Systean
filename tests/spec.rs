use systean::semantics::Type;
use systean::spec::{compile_specification, parse_specification};

#[test]
fn core_spec_parses_and_compiles() {
    let source = include_str!("../spec/semantics/core.sys");
    let spec = parse_specification(source).expect("core semantic spec should parse");
    let environment = compile_specification(&spec).expect("core semantic spec should compile");

    assert!(environment.operator("cease").is_some());
    assert!(environment.operator("forall").is_some());
    assert_eq!(
        environment.literal_type(systean::semantics::LiteralKind::Integer),
        Some(&Type::named("Number"))
    );
}
