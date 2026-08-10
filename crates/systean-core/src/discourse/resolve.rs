use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::semantics::{Checker, Environment, Term, Type};
use crate::syntax::{ContextSlot, ReferenceSlot, SurfaceExpr, TypedSurfaceAst};

use super::{ContextValue, DiscourseError, DiscourseState, ResolvedReference};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedReferenceBinding {
    pub slot: ReferenceSlot,
    pub referent: ResolvedReference,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedContextBinding {
    pub slot: ContextSlot,
    pub value: ContextValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSurfaceAst {
    pub surface: SurfaceExpr,
    pub term: Term,
    pub inferred_type: Type,
    pub references: Vec<ResolvedReferenceBinding>,
    pub contexts: Vec<ResolvedContextBinding>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscourseResolutionError {
    Discourse(DiscourseError),
    InvalidSemanticTerm(String),
}

pub fn resolve_surface(
    typed: &TypedSurfaceAst,
    discourse: &DiscourseState,
    environment: &Environment,
) -> Result<ResolvedSurfaceAst, DiscourseResolutionError> {
    let mut replacements = BTreeMap::<String, Term>::new();
    let mut contexts = Vec::with_capacity(typed.contexts.len());
    for slot in &typed.contexts {
        let value = discourse
            .resolve_context(
                &slot.key,
                &slot.declared_type,
                &slot.expected_type,
                environment,
            )
            .map_err(DiscourseResolutionError::Discourse)?;
        replacements.insert(slot.placeholder.clone(), value.value.clone());
        contexts.push(ResolvedContextBinding {
            slot: slot.clone(),
            value,
        });
    }

    let mut references = Vec::with_capacity(typed.references.len());
    for slot in &typed.references {
        let referent = discourse
            .resolve_reference(&slot.role, &slot.expected_type, environment)
            .map_err(DiscourseResolutionError::Discourse)?;
        replacements.insert(slot.placeholder.clone(), referent.value.clone());
        references.push(ResolvedReferenceBinding {
            slot: slot.clone(),
            referent,
        });
    }

    let term = substitute_placeholders(&typed.template, &replacements, &mut BTreeSet::new());
    let inferred_type = Checker::new(environment)
        .infer(&term)
        .map_err(|error| DiscourseResolutionError::InvalidSemanticTerm(error.to_string()))?;
    Ok(ResolvedSurfaceAst {
        surface: typed.surface.clone(),
        term,
        inferred_type,
        references,
        contexts,
    })
}

fn substitute_placeholders(
    term: &Term,
    replacements: &BTreeMap<String, Term>,
    bound: &mut BTreeSet<String>,
) -> Term {
    match term {
        Term::Const(_) | Term::Literal(_) => term.clone(),
        Term::Var(name) => {
            if bound.contains(name) {
                term.clone()
            } else {
                replacements
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| term.clone())
            }
        }
        Term::Call {
            function,
            arguments,
        } => Term::Call {
            function: function.clone(),
            arguments: arguments
                .iter()
                .map(|(role, value)| {
                    (
                        role.clone(),
                        substitute_placeholders(value, replacements, bound),
                    )
                })
                .collect(),
        },
        Term::Bind {
            variable,
            variable_type,
            body,
        } => {
            let inserted = bound.insert(variable.clone());
            let body = substitute_placeholders(body, replacements, bound);
            if inserted {
                bound.remove(variable);
            }
            Term::Bind {
                variable: variable.clone(),
                variable_type: variable_type.clone(),
                body: Box::new(body),
            }
        }
        Term::Record(fields) => Term::Record(
            fields
                .iter()
                .map(|(name, value)| {
                    (
                        name.clone(),
                        substitute_placeholders(value, replacements, bound),
                    )
                })
                .collect(),
        ),
        Term::Field { record, field } => Term::Field {
            record: Box::new(substitute_placeholders(record, replacements, bound)),
            field: field.clone(),
        },
    }
}

impl fmt::Display for DiscourseResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Discourse(error) => error.fmt(f),
            Self::InvalidSemanticTerm(message) => {
                write!(f, "resolved surface semantics: {message}")
            }
        }
    }
}

impl std::error::Error for DiscourseResolutionError {}
