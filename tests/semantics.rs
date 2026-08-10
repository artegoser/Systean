use systean::semantics::{CheckError, Checker, Term, Type};
use systean::spec::{
    compile_specification, lower_term, parse_specification, parse_term, parse_type,
};

const FIXTURE: &str = r#"
type Entity;
type Person;
subtype Person: Entity;
type Activity;
type Proposition;
type Number;
type Boolean;
type Text;

literal integer: Number;
literal boolean: Boolean;
literal string: Text;

operator cease(target: Activity) -> Proposition;
operator habitual(activity: Activity) -> Activity;
operator smoke(agent: Entity, object: Entity) -> Activity;
operator person(entity: Entity) -> Proposition;
operator arrived(entity: Entity) -> Proposition;
operator forall(predicate: fn(value: Entity) -> Proposition) -> Proposition;
operator not(value: Proposition) -> Proposition;
operator equal<T>(left: $T, right: $T) -> Proposition;
operator needs_entity_predicate(predicate: fn(value: Entity) -> Proposition) -> Proposition;
operator needs_person_predicate(predicate: fn(value: Person) -> Proposition) -> Proposition;

const john: Entity;
const alice: Person;
const entity_predicate: fn(value: Entity) -> Proposition;
const person_predicate: fn(value: Person) -> Proposition;
const mary: Entity;
const cigarette_x: Entity;
const cigarette_kind: Entity;
"#;

fn environment() -> systean::semantics::Environment {
    let specification = parse_specification(FIXTURE).expect("fixture spec should parse");
    compile_specification(&specification).expect("fixture spec should compile")
}

fn checked(source: &str) -> (Term, Type) {
    let environment = environment();
    let parsed = parse_term(source).expect("term should parse");
    let term = lower_term(parsed);
    let ty = Checker::new(&environment)
        .infer(&term)
        .expect("term should type-check");
    (term, ty)
}

#[test]
fn chumsky_parser_handles_function_and_generic_types() {
    assert_eq!(
        parse_type("fn(value: Entity) -> Proposition").unwrap(),
        Type::Function {
            parameters: vec![systean::semantics::FunctionParameter {
                name: Some("value".into()),
                ty: Type::named("Entity"),
            }],
            returns: Box::new(Type::named("Proposition")),
        }
    );

    assert_eq!(
        parse_type("Pair<Entity, Number>").unwrap(),
        Type::Generic {
            name: "Pair".into(),
            arguments: vec![Type::named("Entity"), Type::named("Number")],
        }
    );
}

#[test]
fn current_smoking_and_habitual_smoking_are_different_terms() {
    let (current, current_type) = checked(
        "cease(target = smoke(agent = john, object = cigarette_x))",
    );
    let (habitual, habitual_type) = checked(
        "cease(target = habitual(activity = smoke(agent = john, object = cigarette_kind)))",
    );

    assert_eq!(current_type, Type::named("Proposition"));
    assert_eq!(habitual_type, Type::named("Proposition"));
    assert_ne!(current, habitual);
}

#[test]
fn quantifier_scope_is_structural() {
    let (not_every, _) = checked(
        "not(value = forall(predicate = bind x: Entity => arrived(entity = x)))",
    );
    let (every_not, _) = checked(
        "forall(predicate = bind x: Entity => not(value = arrived(entity = x)))",
    );

    assert_ne!(not_every, every_not);
}

#[test]
fn bound_names_lower_to_variables_and_unbound_names_to_constants() {
    let parsed = parse_term("bind x: Entity => equal(left = x, right = john)").unwrap();
    let lowered = lower_term(parsed);

    let Term::Bind { body, .. } = lowered else {
        panic!("expected bind");
    };
    let Term::Call { arguments, .. } = *body else {
        panic!("expected call");
    };

    assert!(matches!(arguments.get("left"), Some(Term::Var(name)) if name == "x"));
    assert!(matches!(arguments.get("right"), Some(Term::Const(name)) if name == "john"));
}

#[test]
fn generic_operator_requires_one_consistent_type() {
    let environment = environment();
    let parsed = parse_term("equal(left = john, right = 7)").unwrap();
    let term = lower_term(parsed);
    let error = Checker::new(&environment).infer(&term).unwrap_err();

    assert!(matches!(error, CheckError::ConflictingTypeVariable { .. }));
}

#[test]
fn named_roles_are_order_independent() {
    let (first, first_type) = checked(
        "smoke(agent = john, object = cigarette_x)",
    );
    let (second, second_type) = checked(
        "smoke(object = cigarette_x, agent = john)",
    );

    assert_eq!(first_type, Type::named("Activity"));
    assert_eq!(second_type, Type::named("Activity"));

    // Named roles are canonicalized into an ordered map during parsing/lowering.
    assert_eq!(first, second);
}

#[test]
fn checker_rejects_unknown_named_roles() {
    let environment = environment();
    let term = lower_term(parse_term("smoke(actor = john, object = cigarette_x)").unwrap());
    let error = Checker::new(&environment).infer(&term).unwrap_err();

    assert!(matches!(error, CheckError::UnknownArgument { .. }));
}

#[test]
fn semantic_calls_reject_duplicate_named_roles_during_parsing() {
    let error = parse_term("smoke(agent = john, agent = mary)").unwrap_err();
    assert!(error.iter().any(|error| error.message.contains("agent")));
}

#[test]
fn semantic_calls_do_not_accept_positional_arguments() {
    assert!(parse_term("smoke(john, cigarette_x)").is_err());
}


#[test]
fn alpha_equivalent_binders_have_one_canonical_term() {
    let x = lower_term(parse_term("forall(predicate = bind x: Entity => arrived(entity = x))").unwrap());
    let person = lower_term(parse_term("forall(predicate = bind person: Entity => arrived(entity = person))").unwrap());
    assert_ne!(x, person);
    assert_eq!(systean::semantics::canonicalize(&x), systean::semantics::canonicalize(&person));
}

#[test]
fn record_fields_are_typed_and_field_access_is_checked() {
    let environment = environment();
    let term = lower_term(parse_term("{owner = john, count = 7}.owner").unwrap());
    assert_eq!(Checker::new(&environment).infer(&term).unwrap(), Type::named("Entity"));
    let missing = lower_term(parse_term("{owner = john}.missing").unwrap());
    assert!(matches!(Checker::new(&environment).infer(&missing).unwrap_err(), CheckError::UnknownField { .. }));
}

#[test]
fn subtype_values_flow_only_toward_supertypes() {
    let (_, ty) = checked("smoke(agent = alice, object = cigarette_x)");
    assert_eq!(ty, Type::named("Activity"));
}

#[test]
fn function_assignability_is_contravariant_in_parameters() {
    let environment = environment();
    let accepted = lower_term(parse_term("needs_person_predicate(predicate = entity_predicate)").unwrap());
    Checker::new(&environment).infer(&accepted).unwrap();
    let rejected = lower_term(parse_term("needs_entity_predicate(predicate = person_predicate)").unwrap());
    assert!(matches!(Checker::new(&environment).infer(&rejected).unwrap_err(), CheckError::TypeMismatch { .. }));
}

#[test]
fn explainer_reports_types_roles_and_definition_origins() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("spec/semantics");
    let environment = systean::spec::compile_path(root).expect("semantic package should compile");
    let term = lower_term(parse_term("cease(target = smoke(agent = john, object = cigarette_x))").unwrap());
    let rendered = systean::semantics::Explainer::new(&environment).explain(&term).unwrap().render();
    assert!(rendered.contains("call cease : Proposition"));
    assert!(rendered.contains("target: call smoke : Process"));
    assert!(rendered.contains("agent: const john : Entity"));
    assert!(rendered.contains("core.semsys"));
    assert!(rendered.contains("demo.semsys"));
}
