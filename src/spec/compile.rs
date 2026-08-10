use std::collections::BTreeSet;
use std::fmt;

use crate::semantics::{Environment, EnvironmentError, Origin, Signature, Term};
use super::{Declaration, ParsedTerm, Specification};

#[derive(Clone, Debug, PartialEq)]
pub struct SourceSpecification {
    pub source: String,
    pub specification: Specification,
}

impl SourceSpecification {
    pub fn new(source: impl Into<String>, specification: Specification) -> Self {
        Self { source: source.into(), specification }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompileError {
    Environment { origin: Origin, error: EnvironmentError },
}

pub fn compile_specification(specification: &Specification) -> Result<Environment, Vec<CompileError>> {
    compile_specifications(&[SourceSpecification::new("<memory>", specification.clone())])
}

pub fn compile_specifications(specifications: &[SourceSpecification]) -> Result<Environment, Vec<CompileError>> {
    let mut environment = Environment::new();
    let mut errors = Vec::new();

    for source in specifications {
        for (index, declaration) in source.specification.declarations.iter().enumerate() {
            let Declaration::Type { name, type_parameters } = declaration else { continue; };
            let origin = Origin::new(source.source.clone(), index + 1);
            match environment.define_type(name.clone(), type_parameters.clone()) {
                Ok(()) => environment.set_type_origin(name.clone(), origin),
                Err(error) => errors.push(CompileError::Environment { origin, error }),
            }
        }
    }

    for source in specifications {
        for (index, declaration) in source.specification.declarations.iter().enumerate() {
            let origin = Origin::new(source.source.clone(), index + 1);
            let result = match declaration {
                Declaration::Type { .. } => continue,
                Declaration::Subtype { child, parent } => environment.define_subtype(child.clone(), parent.clone()),
                Declaration::Literal { kind, ty } => environment.define_literal_type(*kind, ty.clone()),
                Declaration::Const { name, ty } => environment.define_constant(name.clone(), ty.clone()),
                Declaration::Operator { name, type_parameters, parameters, returns } => environment.define_operator(
                    name.clone(),
                    Signature { type_parameters: type_parameters.clone(), parameters: parameters.clone(), returns: returns.clone() },
                ),
            };
            match result {
                Ok(()) => match declaration {
                    Declaration::Literal { kind, .. } => environment.set_literal_origin(*kind, origin),
                    Declaration::Const { name, .. } => environment.set_constant_origin(name.clone(), origin),
                    Declaration::Operator { name, .. } => environment.set_operator_origin(name.clone(), origin),
                    Declaration::Subtype { .. } | Declaration::Type { .. } => {}
                },
                Err(error) => errors.push(CompileError::Environment { origin, error }),
            }
        }
    }

    if errors.is_empty() { Ok(environment) } else { Err(errors) }
}

pub fn lower_term(term: ParsedTerm) -> Term {
    lower_term_with_scope(term, &mut BTreeSet::new())
}

fn lower_term_with_scope(term: ParsedTerm, scope: &mut BTreeSet<String>) -> Term {
    match term {
        ParsedTerm::Name(name) => if scope.contains(&name) { Term::Var(name) } else { Term::Const(name) },
        ParsedTerm::Literal(literal) => Term::Literal(literal),
        ParsedTerm::Call { function, arguments } => Term::Call {
            function,
            arguments: arguments.into_iter().map(|(name, value)| (name, lower_term_with_scope(value, scope))).collect(),
        },
        ParsedTerm::Bind { variable, variable_type, body } => {
            let inserted = scope.insert(variable.clone());
            let body = lower_term_with_scope(*body, scope);
            if inserted { scope.remove(&variable); }
            Term::Bind { variable, variable_type, body: Box::new(body) }
        }
        ParsedTerm::Record(fields) => Term::Record(fields.into_iter().map(|(name, term)| (name, lower_term_with_scope(term, scope))).collect()),
        ParsedTerm::Field { record, field } => Term::Field { record: Box::new(lower_term_with_scope(*record, scope)), field },
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self { Self::Environment { origin, error } => write!(f, "{origin}: {error}") }
    }
}
impl std::error::Error for CompileError {}
