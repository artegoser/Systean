use std::fmt;

use crate::semantics::Environment;
use crate::syntax::{Argument, Clause, SurfaceExpr, TypedSurfaceAst};

use super::{DiscourseState, ResolvedSurfaceAst};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscourseGenerationError {
    ReferenceCountMismatch,
    UnsafeReference {
        role: String,
        referent: String,
    },
}

pub fn materialize_resolved_surface(
    typed: &TypedSurfaceAst,
    resolved: &ResolvedSurfaceAst,
    discourse: &DiscourseState,
    environment: &Environment,
    reference_surface: &str,
) -> Result<SurfaceExpr, DiscourseGenerationError> {
    let mut reference_index = 0usize;
    let surface = rewrite_expr(
        &typed.surface,
        resolved,
        discourse,
        environment,
        reference_surface,
        &mut reference_index,
    )?;
    if reference_index != resolved.references.len() {
        return Err(DiscourseGenerationError::ReferenceCountMismatch);
    }
    Ok(surface)
}

fn rewrite_expr(
    expression: &SurfaceExpr,
    resolved: &ResolvedSurfaceAst,
    discourse: &DiscourseState,
    environment: &Environment,
    reference_surface: &str,
    reference_index: &mut usize,
) -> Result<SurfaceExpr, DiscourseGenerationError> {
    Ok(match expression {
        SurfaceExpr::Atom(_)
        | SurfaceExpr::Context(_)
        | SurfaceExpr::Alias(_)
        | SurfaceExpr::Name { .. }
        | SurfaceExpr::Quote(_)
        | SurfaceExpr::Literal(_) => expression.clone(),
        SurfaceExpr::Clause(clause) => SurfaceExpr::Clause(Clause {
            primary: clause
                .primary
                .as_ref()
                .map(|argument| {
                    rewrite_argument(
                        argument,
                        resolved,
                        discourse,
                        environment,
                        reference_surface,
                        reference_index,
                    )
                })
                .transpose()?,
            inner_prefixes: clause.inner_prefixes.clone(),
            predicate: clause.predicate.clone(),
            rest: clause
                .rest
                .iter()
                .map(|argument| {
                    rewrite_argument(
                        argument,
                        resolved,
                        discourse,
                        environment,
                        reference_surface,
                        reference_index,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        }),
        SurfaceExpr::Prefix { operator, operand } => SurfaceExpr::Prefix {
            operator: operator.clone(),
            operand: Box::new(rewrite_expr(
                operand,
                resolved,
                discourse,
                environment,
                reference_surface,
                reference_index,
            )?),
        },
        SurfaceExpr::SpeechAct { operator, content } => SurfaceExpr::SpeechAct {
            operator: operator.clone(),
            content: Box::new(rewrite_expr(
                content,
                resolved,
                discourse,
                environment,
                reference_surface,
                reference_index,
            )?),
        },
        SurfaceExpr::Infix { operator, operands } => SurfaceExpr::Infix {
            operator: operator.clone(),
            operands: operands
                .iter()
                .map(|operand| {
                    rewrite_expr(
                        operand,
                        resolved,
                        discourse,
                        environment,
                        reference_surface,
                        reference_index,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
    })
}

fn rewrite_argument(
    argument: &Argument,
    resolved: &ResolvedSurfaceAst,
    discourse: &DiscourseState,
    environment: &Environment,
    reference_surface: &str,
    reference_index: &mut usize,
) -> Result<Argument, DiscourseGenerationError> {
    if !matches!(argument, Argument::Reference(_) | Argument::Omitted) {
        return Ok(argument.clone());
    }
    let Some(binding) = resolved.references.get(*reference_index) else {
        return Err(DiscourseGenerationError::ReferenceCountMismatch);
    };
    *reference_index += 1;

    if discourse
        .resolve_reference(&binding.slot.role, &binding.slot.expected_type, environment)
        .is_ok_and(|candidate| candidate.id == binding.referent.id)
    {
        return Ok(Argument::Reference(reference_surface.to_owned()));
    }

    if let Some(alias) = discourse
        .aliases_for_referent(binding.referent.id)
        .into_iter()
        .find(|alias| environment.is_assignable(&alias.ty, &binding.slot.expected_type))
    {
        return Ok(Argument::Alias(alias.surface.clone()));
    }

    Err(DiscourseGenerationError::UnsafeReference {
        role: binding.slot.role.clone(),
        referent: binding.referent.id.to_string(),
    })
}

impl fmt::Display for DiscourseGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReferenceCountMismatch => write!(
                f,
                "resolved surface reference bindings do not match the source surface structure"
            ),
            Self::UnsafeReference { role, referent } => write!(
                f,
                "cannot generate an unambiguous reference for role `{role}` to `{referent}`"
            ),
        }
    }
}

impl std::error::Error for DiscourseGenerationError {}
