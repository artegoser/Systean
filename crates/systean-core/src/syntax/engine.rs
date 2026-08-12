use std::fmt;

use crate::literals::LiteralEngine;
use crate::semantics::{Environment, LiteralKind, canonicalize};
use crate::spec::parse_type;

use super::{
    CompiledSurfaceBinding, LoweredSurface, SurfaceElaborationError, SurfaceExpr, SurfaceGenerationError,
    CompiledSurfaceLexicon, SurfaceLowerError, SurfaceParseError, SyntaxConfig, TypedSurfaceAst,
    elaborate_surface, linearize_surface, lower_surface, parse_surface_with_literals,
};

#[derive(Clone, Debug)]
pub struct SyntaxEngine {
    config: SyntaxConfig,
    lexicon: CompiledSurfaceLexicon,
    literals: Option<LiteralEngine>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceAnalysis {
    pub syntax: SurfaceExpr,
    pub canonical_surface: String,
    pub inferred_type: String,
    pub canonical_semantics: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceError {
    Parse(Vec<SurfaceParseError>),
    Elaborate(SurfaceElaborationError),
    Lower(SurfaceLowerError),
    Generate(SurfaceGenerationError),
    InvalidBinding(String),
}

impl SyntaxEngine {
    pub fn new(config: SyntaxConfig, lexicon: CompiledSurfaceLexicon) -> Self {
        Self { config, lexicon, literals: None }
    }

    pub fn new_with_literals(config: SyntaxConfig, lexicon: CompiledSurfaceLexicon, literals: LiteralEngine) -> Self {
        Self { config, lexicon, literals: Some(literals) }
    }

    pub fn literals(&self) -> Option<&LiteralEngine> {
        self.literals.as_ref()
    }

    pub fn config(&self) -> &SyntaxConfig {
        &self.config
    }

    pub fn lexicon(&self) -> &CompiledSurfaceLexicon {
        &self.lexicon
    }

    pub fn validate_environment(&self, environment: &Environment) -> Result<(), SurfaceError> {
        let semantic_name = |id| self.lexicon.semantic_name(id).ok_or_else(|| {
            SurfaceError::InvalidBinding(format!("compiled surface symbol `{id}` has no semantic debug name"))
        });
        for (surface, lexeme) in self.lexicon.iter() {
            match lexeme {
                CompiledSurfaceBinding::Atom { semantic } => {
                    let semantic_name = semantic_name(*semantic)?;
                    if environment.constant_type(semantic_name).is_none() {
                        return Err(SurfaceError::InvalidBinding(format!(
                            "lexical root `{surface}` references unknown semantic constant `{semantic_name}`"
                        )));
                    }
                }
                CompiledSurfaceBinding::Reference => {}
                CompiledSurfaceBinding::Information { knower_type, .. } => {
                    if let Some(ty) = knower_type {
                        if !environment.is_well_formed_type(ty) {
                            return Err(SurfaceError::InvalidBinding(format!(
                                "information lexical root `{surface}` uses unknown knower type `{ty}`"
                            )));
                        }
                    }
                }
                CompiledSurfaceBinding::Alias { ty, .. } => {
                    let ty = parse_type(ty).map_err(|errors| {
                        SurfaceError::InvalidBinding(format!(
                            "discourse alias `{surface}` has invalid type: {}",
                            errors
                                .into_iter()
                                .map(|error| error.to_string())
                                .collect::<Vec<_>>()
                                .join("; ")
                        ))
                    })?;
                    if !environment.is_well_formed_type(&ty) {
                        return Err(SurfaceError::InvalidBinding(format!(
                            "discourse alias `{surface}` uses unknown type `{ty}`"
                        )));
                    }
                }
                CompiledSurfaceBinding::Context { ty, .. } => {
                    if !environment.is_well_formed_type(ty) {
                        return Err(SurfaceError::InvalidBinding(format!(
                            "context lexical root `{surface}` uses unknown type `{ty}`"
                        )));
                    }
                }
                CompiledSurfaceBinding::Class { semantic, parameter } => {
                    validate_operator_parameters(environment, surface, semantic_name(*semantic)?, [*parameter])?;
                }
                CompiledSurfaceBinding::Predicate {
                    semantic,
                    primary_parameter,
                    rest_parameters,
                } => {
                    let parameters = primary_parameter
                        .iter()
                        .copied()
                        .chain(rest_parameters.iter().copied())
                        .collect::<Vec<_>>();
                    validate_operator_parameters(environment, surface, semantic_name(*semantic)?, parameters)?;
                }
                CompiledSurfaceBinding::Prefix { semantic, parameter, .. } => {
                    validate_operator_parameters(environment, surface, semantic_name(*semantic)?, [*parameter])?;
                }
                CompiledSurfaceBinding::Infix {
                    semantic,
                    left_parameter,
                    right_parameter,
                    ..
                } => {
                    validate_operator_parameters(
                        environment,
                        surface,
                        semantic_name(*semantic)?,
                        [*left_parameter, *right_parameter],
                    )?;
                }
                CompiledSurfaceBinding::Binder {
                    semantic,
                    binder_parameter,
                    direct_parameters,
                    variable_type,
                    combiner_semantic,
                    combiner_left_parameter,
                    combiner_right_parameter,
                } => {
                    let parameters = std::iter::once(*binder_parameter)
                        .chain(direct_parameters.iter().copied())
                        .collect::<Vec<_>>();
                    validate_operator_parameters(environment, surface, semantic_name(*semantic)?, parameters)?;
                    validate_operator_parameters(
                        environment,
                        surface,
                        semantic_name(*combiner_semantic)?,
                        [*combiner_left_parameter, *combiner_right_parameter],
                    )?;
                    if !environment.is_well_formed_type(variable_type) {
                        return Err(SurfaceError::InvalidBinding(format!(
                            "binder lexical root `{surface}` uses unknown binder type `{variable_type}`"
                        )));
                    }
                }
                CompiledSurfaceBinding::Capture { semantic, parameter } => {
                    let semantic_name = semantic_name(*semantic)?;
                    validate_operator_parameters(environment, surface, semantic_name, [*parameter])?;
                    let signature = environment
                        .operator(semantic_name)
                        .expect("operator existence validated above");
                    let declared_parameter = signature
                        .parameters
                        .get(*parameter as usize)
                        .expect("capture parameter existence validated above");
                    let text = environment.literal_type(LiteralKind::String).ok_or_else(|| {
                        SurfaceError::InvalidBinding(format!(
                            "capture lexical root `{surface}` requires a declared string literal type"
                        ))
                    })?;
                    if !environment.is_assignable(text, &declared_parameter.ty) {
                        return Err(SurfaceError::InvalidBinding(format!(
                            "capture lexical root `{surface}` parameter `{parameter}` must accept the string literal type `{text}`, but `{semantic_name}` requires `{}`",
                            declared_parameter.ty
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn parse(&self, source: &str) -> Result<SurfaceExpr, SurfaceError> {
        parse_surface_with_literals(source, &self.config, &self.lexicon, self.literals.as_ref()).map_err(SurfaceError::Parse)
    }

    pub fn linearize(&self, syntax: &SurfaceExpr) -> Result<String, SurfaceError> {
        linearize_surface(syntax, &self.config, &self.lexicon).map_err(SurfaceError::Generate)
    }

    pub fn elaborate(
        &self,
        syntax: &SurfaceExpr,
        environment: &Environment,
    ) -> Result<TypedSurfaceAst, SurfaceError> {
        elaborate_surface(syntax, &self.lexicon, environment).map_err(SurfaceError::Elaborate)
    }

    pub fn lower(
        &self,
        syntax: &SurfaceExpr,
        environment: &Environment,
    ) -> Result<LoweredSurface, SurfaceError> {
        lower_surface(syntax, &self.lexicon, environment).map_err(SurfaceError::Lower)
    }

    pub fn analyze(
        &self,
        source: &str,
        environment: &Environment,
    ) -> Result<SurfaceAnalysis, SurfaceError> {
        self.validate_environment(environment)?;
        let syntax = self.parse(source)?;
        let canonical_surface = self.linearize(&syntax)?;
        let lowered = self.lower(&syntax, environment)?;
        Ok(SurfaceAnalysis {
            syntax,
            canonical_surface,
            inferred_type: lowered.inferred_type.to_string(),
            canonical_semantics: canonicalize(&lowered.term).to_string(),
        })
    }
}

impl fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(errors) => {
                for (index, error) in errors.iter().enumerate() {
                    if index > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "{error}")?;
                }
                Ok(())
            }
            Self::Elaborate(error) => error.fmt(f),
            Self::Lower(error) => error.fmt(f),
            Self::Generate(error) => error.fmt(f),
            Self::InvalidBinding(message) => write!(f, "surface binding: {message}"),
        }
    }
}

impl std::error::Error for SurfaceError {}

fn validate_operator_parameters(
    environment: &Environment,
    surface: &str,
    semantic: &str,
    parameters: impl IntoIterator<Item = u32>,
) -> Result<(), SurfaceError> {
    let signature = environment.operator(semantic).ok_or_else(|| {
        SurfaceError::InvalidBinding(format!(
            "lexical root `{surface}` references unknown semantic operator `{semantic}`"
        ))
    })?;
    let mut configured = parameters.into_iter().collect::<Vec<_>>();
    configured.sort_unstable();
    configured.dedup();
    let declared = (0..signature.parameters.len() as u32).collect::<Vec<_>>();
    if configured != declared {
        return Err(SurfaceError::InvalidBinding(format!(
            "lexical root `{surface}` maps `{semantic}` parameter slots {:?}, but semantic signature requires {:?}",
            configured, declared
        )));
    }
    Ok(())
}
