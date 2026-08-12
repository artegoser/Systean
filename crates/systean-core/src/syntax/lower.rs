use std::fmt;

use crate::semantics::{Checker, Environment, Term, Type};

use super::{SurfaceElaborationError, SurfaceExpr, CompiledSurfaceLexicon, elaborate_surface};

#[derive(Clone, Debug, PartialEq)]
pub struct LoweredSurface {
    pub term: Term,
    pub inferred_type: Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceLowerError {
    Elaborate(SurfaceElaborationError),
    DiscourseRequired {
        references: usize,
        contexts: usize,
        aliases: usize,
    },
    InvalidSemanticTerm(String),
}

pub fn lower_surface(
    expression: &SurfaceExpr,
    lexicon: &CompiledSurfaceLexicon,
    environment: &Environment,
) -> Result<LoweredSurface, SurfaceLowerError> {
    let typed = elaborate_surface(expression, lexicon, environment)
        .map_err(SurfaceLowerError::Elaborate)?;
    if !typed.references.is_empty() || !typed.contexts.is_empty() || !typed.aliases.is_empty() {
        return Err(SurfaceLowerError::DiscourseRequired {
            references: typed.references.len(),
            contexts: typed.contexts.len(),
            aliases: typed.aliases.len(),
        });
    }
    let inferred_type = Checker::new(environment)
        .infer(&typed.template)
        .map_err(|error| SurfaceLowerError::InvalidSemanticTerm(error.to_string()))?;
    Ok(LoweredSurface {
        term: typed.template,
        inferred_type,
    })
}

impl fmt::Display for SurfaceLowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Elaborate(error) => error.fmt(f),
            Self::DiscourseRequired {
                references,
                contexts,
                aliases,
            } => write!(
                f,
                "surface expression requires discourse resolution ({references} reference slot(s), {contexts} context value(s), {aliases} alias slot(s))"
            ),
            Self::InvalidSemanticTerm(message) => write!(f, "surface semantics: {message}"),
        }
    }
}

impl std::error::Error for SurfaceLowerError {}
