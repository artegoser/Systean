use std::fmt;

use super::{Argument, LexemeConfig, SurfaceExpr, SyntaxConfig};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceGenerationError {
    MissingLexeme(String),
    MissingPrecedence(String),
}

pub fn linearize_surface(expression: &SurfaceExpr, config: &SyntaxConfig) -> Result<String, SurfaceGenerationError> {
    Ok(Generator { config }.render(expression, None)?.join(" "))
}

struct Generator<'a> {
    config: &'a SyntaxConfig,
}

impl Generator<'_> {
    fn render(&self, expression: &SurfaceExpr, parent: Option<ParentContext<'_>>) -> Result<Vec<String>, SurfaceGenerationError> {
        let own = self.precedence(expression)?;
        let mut tokens = match expression {
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
                    tokens.extend(self.render(operand, Some(ParentContext::Infix { operator, precedence: own }))?);
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

    fn needs_group(&self, expression: &SurfaceExpr, own: u16, parent: Option<ParentContext<'_>>) -> Result<bool, SurfaceGenerationError> {
        let Some(parent) = parent else { return Ok(false) };
        match parent {
            ParentContext::Prefix => Ok(matches!(expression, SurfaceExpr::Infix { .. })),
            ParentContext::Infix { operator: parent_operator, precedence: parent_precedence } => {
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
                let Some(LexemeConfig::Infix { semantic, .. }) = self.config.lexemes.get(operator) else {
                    return Err(SurfaceGenerationError::MissingLexeme(operator.clone()));
                };
                self.config.precedence(semantic).ok_or_else(|| SurfaceGenerationError::MissingPrecedence(semantic.clone()))
            }
            _ => Ok(u16::MAX),
        }
    }

    fn render_argument(&self, argument: &Argument) -> Vec<String> {
        match argument {
            Argument::Atom(surface) => vec![surface.clone()],
            Argument::Quantified { quantifier, restriction } => vec![quantifier.clone(), restriction.clone()],
        }
    }
}

#[derive(Clone, Copy)]
enum ParentContext<'a> {
    Prefix,
    Infix { operator: &'a str, precedence: u16 },
}

impl fmt::Display for SurfaceGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLexeme(surface) => write!(f, "surface lexeme `{surface}` is not declared"),
            Self::MissingPrecedence(operator) => write!(f, "surface operator `{operator}` has no precedence"),
        }
    }
}

impl std::error::Error for SurfaceGenerationError {}
