use std::fmt;

use super::{Argument, InformationKnower, LexemeConfig, SurfaceExpr, SurfaceLexicon, SyntaxConfig};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceGenerationError {
    MissingLexeme(String),
    MissingPrecedence(String),
}

pub fn linearize_surface(
    expression: &SurfaceExpr,
    config: &SyntaxConfig,
    lexicon: &SurfaceLexicon,
) -> Result<String, SurfaceGenerationError> {
    Ok(Generator { config, lexicon }
        .render(expression, None)?
        .join(" "))
}

struct Generator<'a> {
    config: &'a SyntaxConfig,
    lexicon: &'a SurfaceLexicon,
}

impl Generator<'_> {
    fn render(
        &self,
        expression: &SurfaceExpr,
        parent: Option<ParentContext<'_>>,
    ) -> Result<Vec<String>, SurfaceGenerationError> {
        let own = self.precedence(expression)?;
        let mut tokens = match expression {
            SurfaceExpr::Atom(surface) | SurfaceExpr::Context(surface) | SurfaceExpr::Alias(surface) => vec![surface.clone()],
            SurfaceExpr::Name { marker, payload } => vec![marker.clone(), payload.clone()],
            SurfaceExpr::Quote(payload) => vec![self.render_quote(payload)],
            SurfaceExpr::Literal(literal) => vec![literal.canonical_surface().to_owned()],
            SurfaceExpr::Clause(clause) => {
                let mut tokens = Vec::new();
                match self.config.order.frame {
                    super::FrameOrder::PrimaryPredicateRest => {
                        if let Some(primary) = &clause.primary {
                            tokens.extend(self.render_argument(primary));
                        }
                        tokens.extend(clause.inner_prefixes.iter().cloned());
                        tokens.push(clause.predicate.clone());
                        for argument in &clause.rest {
                            tokens.extend(self.render_argument(argument));
                        }
                    }
                    super::FrameOrder::PredicateArguments => {
                        tokens.push(clause.predicate.clone());
                        if let Some(primary) = &clause.primary {
                            tokens.extend(self.render_argument(primary));
                        }
                        for argument in &clause.rest {
                            tokens.extend(self.render_argument(argument));
                        }
                    }
                }
                tokens
            }
            SurfaceExpr::Prefix { operator, operand } => {
                let mut tokens = vec![operator.clone()];
                tokens.extend(self.render(operand, Some(ParentContext::Prefix))?);
                tokens
            }
            SurfaceExpr::SpeechAct { operator, content } => {
                let mut tokens = vec![operator.clone()];
                tokens.extend(self.render(content, Some(ParentContext::Prefix))?);
                tokens
            }
            SurfaceExpr::Infix { operator, operands } => {
                let mut tokens = Vec::new();
                for (index, operand) in operands.iter().enumerate() {
                    if index > 0 {
                        tokens.push(operator.clone());
                    }
                    tokens.extend(self.render(
                        operand,
                        Some(ParentContext::Infix {
                            operator,
                            precedence: own,
                        }),
                    )?);
                }
                tokens
            }
        };
        if self.needs_group(expression, own, parent)? {
            tokens.insert(0, self.config.scope.open.clone());
            tokens.push(self.config.scope.close.clone());
        }
        Ok(tokens)
    }

    fn needs_group(
        &self,
        expression: &SurfaceExpr,
        own: u16,
        parent: Option<ParentContext<'_>>,
    ) -> Result<bool, SurfaceGenerationError> {
        let Some(parent) = parent else {
            return Ok(false);
        };
        match parent {
            ParentContext::Prefix => Ok(matches!(expression, SurfaceExpr::Infix { .. })),
            ParentContext::Infix {
                operator: parent_operator,
                precedence: parent_precedence,
            } => {
                if own < parent_precedence {
                    return Ok(true);
                }
                if own > parent_precedence {
                    return Ok(false);
                }
                match expression {
                    SurfaceExpr::Infix { operator, .. } => Ok(operator != parent_operator),
                    _ => Ok(false),
                }
            }
        }
    }

    fn precedence(&self, expression: &SurfaceExpr) -> Result<u16, SurfaceGenerationError> {
        match expression {
            SurfaceExpr::Infix { operator, .. } => {
                let Some(LexemeConfig::Infix { semantic, .. }) = self.lexicon.get(operator) else {
                    return Err(SurfaceGenerationError::MissingLexeme(operator.clone()));
                };
                self.config
                    .precedence(semantic)
                    .ok_or_else(|| SurfaceGenerationError::MissingPrecedence(semantic.clone()))
            }
            _ => Ok(u16::MAX),
        }
    }

    fn render_quote(&self, payload: &str) -> String {
        if payload.is_empty() {
            format!("{} {}", self.config.quotation.open, self.config.quotation.close)
        } else {
            format!(
                "{} {} {}",
                self.config.quotation.open, payload, self.config.quotation.close
            )
        }
    }

    fn render_argument(&self, argument: &Argument) -> Vec<String> {
        match argument {
            Argument::Atom(surface)
            | Argument::Context(surface)
            | Argument::Reference(surface)
            | Argument::Alias(surface) => {
                vec![surface.clone()]
            }
            Argument::Name { marker, payload } => vec![marker.clone(), payload.clone()],
            Argument::Quote(payload) => vec![self.render_quote(payload)],
            Argument::Literal(literal) => vec![literal.canonical_surface().to_owned()],
            Argument::Information { marker, knower } => {
                let mut tokens = vec![marker.clone()];
                if let Some(knower) = knower {
                    match knower {
                        InformationKnower::Context(surface) => tokens.push(surface.clone()),
                        InformationKnower::Name { marker, payload } => {
                            tokens.push(marker.clone());
                            tokens.push(payload.clone());
                        }
                    }
                }
                tokens
            }
            Argument::Omitted => Vec::new(),
            Argument::Quantified {
                quantifier,
                restriction,
            } => vec![quantifier.clone(), restriction.clone()],
            Argument::CountedQuantified {
                quantifier,
                count,
                restriction,
            } => vec![
                quantifier.clone(),
                count.canonical_surface().to_owned(),
                restriction.clone(),
            ],
        }
    }
}

#[derive(Clone, Copy)]
enum ParentContext<'a> {
    Prefix,
    Infix {
        operator: &'a str,
        precedence: u16,
    },
}

impl fmt::Display for SurfaceGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLexeme(surface) => {
                write!(f, "surface lexeme `{surface}` is not declared")
            }
            Self::MissingPrecedence(operator) => {
                write!(f, "surface operator `{operator}` has no precedence")
            }
        }
    }
}

impl std::error::Error for SurfaceGenerationError {}
