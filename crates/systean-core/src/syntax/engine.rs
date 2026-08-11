use std::collections::BTreeSet;
use std::fmt;

use crate::literals::LiteralEngine;
use crate::semantics::{Environment, LiteralKind, canonicalize};
use crate::spec::parse_type;

use super::{
    LexemeConfig, LoweredSurface, SurfaceElaborationError, SurfaceExpr, SurfaceGenerationError,
    SurfaceLexicon, SurfaceLowerError, SurfaceParseError, SyntaxConfig, TypedSurfaceAst,
    elaborate_surface, linearize_surface, lower_surface, parse_surface_with_literals,
};

#[derive(Clone, Debug)]
pub struct SyntaxEngine {
    config: SyntaxConfig,
    lexicon: SurfaceLexicon,
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
    pub fn new(config: SyntaxConfig, lexicon: SurfaceLexicon) -> Self {
        Self { config, lexicon, literals: None }
    }

    pub fn new_with_literals(config: SyntaxConfig, lexicon: SurfaceLexicon, literals: LiteralEngine) -> Self {
        Self { config, lexicon, literals: Some(literals) }
    }

    pub fn literals(&self) -> Option<&LiteralEngine> {
        self.literals.as_ref()
    }

    pub fn config(&self) -> &SyntaxConfig {
        &self.config
    }

    pub fn lexicon(&self) -> &SurfaceLexicon {
        &self.lexicon
    }

    pub fn validate_environment(&self, environment: &Environment) -> Result<(), SurfaceError> {
        for (surface, lexeme) in self.lexicon.iter() {
            match lexeme {
                LexemeConfig::Atom { semantic } => {
                    if environment.constant_type(semantic).is_none() {
                        return Err(SurfaceError::InvalidBinding(format!(
                            "lexical root `{surface}` references unknown semantic constant `{semantic}`"
                        )));
                    }
                }
                LexemeConfig::Reference => {}
                LexemeConfig::Information { status, knower_type } => {
                    if status.trim().is_empty() {
                        return Err(SurfaceError::InvalidBinding(format!(
                            "information lexical root `{surface}` has an empty status identity"
                        )));
                    }
                    if let Some(source) = knower_type {
                        let ty = parse_type(source).map_err(|errors| {
                            SurfaceError::InvalidBinding(format!(
                                "information lexical root `{surface}` has invalid knower type `{source}`: {}",
                                errors
                                    .into_iter()
                                    .map(|error| error.to_string())
                                    .collect::<Vec<_>>()
                                    .join("; ")
                            ))
                        })?;
                        if !environment.is_well_formed_type(&ty) {
                            return Err(SurfaceError::InvalidBinding(format!(
                                "information lexical root `{surface}` uses unknown knower type `{source}`"
                            )));
                        }
                    }
                }
                LexemeConfig::Alias { ty, .. } => {
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
                LexemeConfig::Context { ty, .. } => {
                    let ty = parse_type(ty).map_err(|errors| {
                        SurfaceError::InvalidBinding(format!(
                            "context lexical root `{surface}` has invalid type: {}",
                            errors
                                .into_iter()
                                .map(|error| error.to_string())
                                .collect::<Vec<_>>()
                                .join("; ")
                        ))
                    })?;
                    if !environment.is_well_formed_type(&ty) {
                        return Err(SurfaceError::InvalidBinding(format!(
                            "context lexical root `{surface}` uses unknown type `{ty}`"
                        )));
                    }
                }
                LexemeConfig::Class { semantic, role } => {
                    validate_operator_roles(environment, surface, semantic, [role.as_str()])?;
                }
                LexemeConfig::Predicate {
                    semantic,
                    primary_role,
                    rest_roles,
                } => {
                    let roles = primary_role
                        .iter()
                        .map(String::as_str)
                        .chain(rest_roles.iter().map(String::as_str))
                        .collect::<Vec<_>>();
                    validate_operator_roles(environment, surface, semantic, roles)?;
                }
                LexemeConfig::Prefix { semantic, role } => {
                    validate_operator_roles(environment, surface, semantic, [role.as_str()])?;
                }
                LexemeConfig::Infix {
                    semantic,
                    left_role,
                    right_role,
                } => {
                    validate_operator_roles(
                        environment,
                        surface,
                        semantic,
                        [left_role.as_str(), right_role.as_str()],
                    )?;
                    if self.config.precedence(semantic).is_none() {
                        return Err(SurfaceError::InvalidBinding(format!(
                            "infix lexical root `{surface}` semantic operator `{semantic}` has no precedence"
                        )));
                    }
                }
                LexemeConfig::Quantifier {
                    semantic,
                    binder_role,
                    variable_type,
                    restriction_operator,
                    restriction_role,
                    body_role,
                } => {
                    validate_operator_roles(environment, surface, semantic, [binder_role.as_str()])?;
                    validate_operator_roles(
                        environment,
                        surface,
                        restriction_operator,
                        [restriction_role.as_str(), body_role.as_str()],
                    )?;
                    let ty = parse_type(variable_type).map_err(|errors| {
                        SurfaceError::InvalidBinding(format!(
                            "quantifier lexical root `{surface}` has invalid binder type `{variable_type}`: {}",
                            errors
                                .into_iter()
                                .map(|error| error.to_string())
                                .collect::<Vec<_>>()
                                .join("; ")
                        ))
                    })?;
                    if !environment.is_well_formed_type(&ty) {
                        return Err(SurfaceError::InvalidBinding(format!(
                            "quantifier lexical root `{surface}` uses unknown binder type `{variable_type}`"
                        )));
                    }
                }
                LexemeConfig::CountedQuantifier {
                    semantic,
                    binder_role,
                    count_role,
                    variable_type,
                    restriction_operator,
                    restriction_role,
                    body_role,
                } => {
                    validate_operator_roles(
                        environment,
                        surface,
                        semantic,
                        [binder_role.as_str(), count_role.as_str()],
                    )?;
                    validate_operator_roles(
                        environment,
                        surface,
                        restriction_operator,
                        [restriction_role.as_str(), body_role.as_str()],
                    )?;
                    let ty = parse_type(variable_type).map_err(|errors| {
                        SurfaceError::InvalidBinding(format!(
                            "counted quantifier lexical root `{surface}` has invalid binder type `{variable_type}`: {}",
                            errors.into_iter().map(|error| error.to_string()).collect::<Vec<_>>().join("; ")
                        ))
                    })?;
                    if !environment.is_well_formed_type(&ty) {
                        return Err(SurfaceError::InvalidBinding(format!(
                            "counted quantifier lexical root `{surface}` uses unknown binder type `{variable_type}`"
                        )));
                    }
                }
                LexemeConfig::SpeechAct { semantic, role } => {
                    validate_operator_roles(environment, surface, semantic, [role.as_str()])?;
                }
                LexemeConfig::Name { semantic, role } => {
                    validate_operator_roles(environment, surface, semantic, [role.as_str()])?;
                    let signature = environment
                        .operator(semantic)
                        .expect("operator existence validated above");
                    let parameter = signature
                        .parameters
                        .iter()
                        .find(|parameter| parameter.name == *role)
                        .expect("name role existence validated above");
                    let text = environment.literal_type(LiteralKind::String).ok_or_else(|| {
                        SurfaceError::InvalidBinding(format!(
                            "name lexical root `{surface}` requires a declared string literal type"
                        ))
                    })?;
                    if !environment.is_assignable(text, &parameter.ty) {
                        return Err(SurfaceError::InvalidBinding(format!(
                            "name lexical root `{surface}` role `{role}` must accept the string literal type `{text}`, but `{semantic}` requires `{}`",
                            parameter.ty
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

fn validate_operator_roles<'a>(
    environment: &Environment,
    surface: &str,
    semantic: &str,
    roles: impl IntoIterator<Item = &'a str>,
) -> Result<(), SurfaceError> {
    let signature = environment.operator(semantic).ok_or_else(|| {
        SurfaceError::InvalidBinding(format!(
            "lexical root `{surface}` references unknown semantic operator `{semantic}`"
        ))
    })?;
    let configured = roles.into_iter().collect::<BTreeSet<_>>();
    let declared = signature
        .parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<BTreeSet<_>>();
    if configured != declared {
        return Err(SurfaceError::InvalidBinding(format!(
            "lexical root `{surface}` maps `{semantic}` roles {:?}, but semantic signature requires {:?}",
            configured, declared
        )));
    }
    Ok(())
}
