use std::collections::BTreeMap;
use std::fmt;

use crate::semantics::{Checker, Environment, Term, Type};
use crate::spec::parse_type;

use super::{Argument, Clause, LexemeConfig, SurfaceExpr, SurfaceLexicon};

#[derive(Clone, Debug, PartialEq)]
pub struct LoweredSurface {
    pub term: Term,
    pub inferred_type: Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceLowerError {
    MissingLexeme(String),
    WrongLexemeKind { surface: String, expected: &'static str },
    MissingPrecedence(String),
    InvalidType { source: String, message: String },
    InvalidSemanticTerm(String),
}

pub fn lower_surface(
    expression: &SurfaceExpr,
    lexicon: &SurfaceLexicon,
    environment: &Environment,
) -> Result<LoweredSurface, SurfaceLowerError> {
    let mut lowerer = Lowerer { lexicon, quantifier_index: 0 };
    let term = lowerer.lower_expr(expression)?;
    let inferred_type = Checker::new(environment)
        .infer(&term)
        .map_err(|error| SurfaceLowerError::InvalidSemanticTerm(error.to_string()))?;
    Ok(LoweredSurface { term, inferred_type })
}

struct Lowerer<'a> {
    lexicon: &'a SurfaceLexicon,
    quantifier_index: usize,
}

#[derive(Clone)]
struct QuantifierIntroduction {
    quantifier_surface: String,
    restriction_surface: String,
    variable: String,
    variable_type: Type,
}

#[derive(Clone)]
enum ScopeIntroduction {
    Prefix(String),
    Quantifier(QuantifierIntroduction),
}

impl Lowerer<'_> {
    fn lower_expr(&mut self, expression: &SurfaceExpr) -> Result<Term, SurfaceLowerError> {
        match expression {
            SurfaceExpr::Atom(surface) => self.lower_atom(surface),
            SurfaceExpr::Clause(clause) => self.lower_clause(clause),
            SurfaceExpr::Prefix { operator, operand } => {
                let operand = self.lower_expr(operand)?;
                self.call_prefix(operator, operand)
            }
            SurfaceExpr::SpeechAct { operator, content } => {
                let content = self.lower_expr(content)?;
                self.call_speech_act(operator, content)
            }
            SurfaceExpr::Infix { operator, operands } => {
                let mut operands = operands.iter();
                let first = operands
                    .next()
                    .ok_or_else(|| SurfaceLowerError::InvalidSemanticTerm("infix expression has no operands".into()))?;
                let mut term = self.lower_expr(first)?;
                for operand in operands {
                    let right = self.lower_expr(operand)?;
                    term = self.call_infix(operator, term, right)?;
                }
                Ok(term)
            }
        }
    }

    fn lower_atom(&self, surface: &str) -> Result<Term, SurfaceLowerError> {
        let LexemeConfig::Atom { semantic } = self.lexeme(surface)? else {
            return Err(SurfaceLowerError::WrongLexemeKind {
                surface: surface.to_owned(),
                expected: "atom",
            });
        };
        Ok(Term::Const(semantic.clone()))
    }

    fn lower_clause(&mut self, clause: &Clause) -> Result<Term, SurfaceLowerError> {
        let predicate = self.lexeme(&clause.predicate)?.clone();
        let LexemeConfig::Predicate { semantic, primary_role, rest_roles } = predicate else {
            return Err(SurfaceLowerError::WrongLexemeKind { surface: clause.predicate.clone(), expected: "predicate" });
        };

        let mut arguments = BTreeMap::new();
        let mut introductions = Vec::new();

        if let (Some(role), Some(argument)) = (primary_role.as_ref(), clause.primary.as_ref()) {
            let (term, intro) = self.lower_argument(argument)?;
            arguments.insert(role.clone(), term);
            if let Some(intro) = intro {
                introductions.push(ScopeIntroduction::Quantifier(intro));
            }
        }

        for prefix in &clause.inner_prefixes {
            introductions.push(ScopeIntroduction::Prefix(prefix.clone()));
        }

        if rest_roles.len() != clause.rest.len() {
            return Err(SurfaceLowerError::InvalidSemanticTerm(format!(
                "predicate `{}` expects {} rest argument(s), surface clause has {}",
                clause.predicate,
                rest_roles.len(),
                clause.rest.len()
            )));
        }

        for (role, argument) in rest_roles.iter().zip(&clause.rest) {
            let (term, intro) = self.lower_argument(argument)?;
            arguments.insert(role.clone(), term);
            if let Some(intro) = intro {
                introductions.push(ScopeIntroduction::Quantifier(intro));
            }
        }

        let mut term = Term::Call { function: semantic.clone(), arguments };
        for introduction in introductions.into_iter().rev() {
            term = match introduction {
                ScopeIntroduction::Prefix(surface) => self.call_prefix(&surface, term)?,
                ScopeIntroduction::Quantifier(quantifier) => self.wrap_quantifier(quantifier, term)?,
            };
        }
        Ok(term)
    }

    fn lower_argument(&mut self, argument: &Argument) -> Result<(Term, Option<QuantifierIntroduction>), SurfaceLowerError> {
        match argument {
            Argument::Atom(surface) => {
                let LexemeConfig::Atom { semantic } = self.lexeme(surface)? else {
                    return Err(SurfaceLowerError::WrongLexemeKind { surface: surface.clone(), expected: "atom" });
                };
                Ok((Term::Const(semantic.clone()), None))
            }
            Argument::Quantified { quantifier, restriction } => {
                let quantifier_config = self.lexeme(quantifier)?.clone();
                let LexemeConfig::Quantifier { variable_type, .. } = quantifier_config else {
                    return Err(SurfaceLowerError::WrongLexemeKind { surface: quantifier.clone(), expected: "quantifier" });
                };
                let restriction_config = self.lexeme(restriction)?.clone();
                let LexemeConfig::Class { .. } = restriction_config else {
                    return Err(SurfaceLowerError::WrongLexemeKind { surface: restriction.clone(), expected: "class" });
                };
                let variable_type = parse_type(&variable_type)
                    .map_err(|errors| SurfaceLowerError::InvalidType {
                        source: variable_type.clone(),
                        message: errors.into_iter().map(|error| error.to_string()).collect::<Vec<_>>().join("; "),
                    })?;
                let variable = format!("surface_q{}", self.quantifier_index);
                self.quantifier_index += 1;
                Ok((
                    Term::Var(variable.clone()),
                    Some(QuantifierIntroduction {
                        quantifier_surface: quantifier.clone(),
                        restriction_surface: restriction.clone(),
                        variable,
                        variable_type,
                    }),
                ))
            }
        }
    }

    fn wrap_quantifier(&self, intro: QuantifierIntroduction, body: Term) -> Result<Term, SurfaceLowerError> {
        let LexemeConfig::Quantifier {
            semantic,
            binder_role,
            restriction_operator,
            restriction_role,
            body_role,
            ..
        } = self.lexeme(&intro.quantifier_surface)? else {
            return Err(SurfaceLowerError::WrongLexemeKind { surface: intro.quantifier_surface, expected: "quantifier" });
        };
        let LexemeConfig::Class { semantic: restriction_semantic, role: class_role } = self.lexeme(&intro.restriction_surface)? else {
            return Err(SurfaceLowerError::WrongLexemeKind { surface: intro.restriction_surface, expected: "class" });
        };

        let restriction = Term::Call {
            function: restriction_semantic.clone(),
            arguments: BTreeMap::from([(class_role.clone(), Term::Var(intro.variable.clone()))]),
        };
        let combined = Term::Call {
            function: restriction_operator.clone(),
            arguments: BTreeMap::from([
                (restriction_role.clone(), restriction),
                (body_role.clone(), body),
            ]),
        };
        let binder = Term::Bind {
            variable: intro.variable,
            variable_type: intro.variable_type,
            body: Box::new(combined),
        };
        Ok(Term::Call {
            function: semantic.clone(),
            arguments: BTreeMap::from([(binder_role.clone(), binder)]),
        })
    }

    fn call_prefix(&self, surface: &str, operand: Term) -> Result<Term, SurfaceLowerError> {
        let LexemeConfig::Prefix { semantic, role } = self.lexeme(surface)? else {
            return Err(SurfaceLowerError::WrongLexemeKind { surface: surface.to_owned(), expected: "prefix" });
        };
        Ok(Term::Call {
            function: semantic.clone(),
            arguments: BTreeMap::from([(role.clone(), operand)]),
        })
    }

    fn call_infix(&self, surface: &str, left: Term, right: Term) -> Result<Term, SurfaceLowerError> {
        let LexemeConfig::Infix { semantic, left_role, right_role } = self.lexeme(surface)? else {
            return Err(SurfaceLowerError::WrongLexemeKind { surface: surface.to_owned(), expected: "infix" });
        };
        Ok(Term::Call {
            function: semantic.clone(),
            arguments: BTreeMap::from([
                (left_role.clone(), left),
                (right_role.clone(), right),
            ]),
        })
    }

    fn call_speech_act(&self, surface: &str, content: Term) -> Result<Term, SurfaceLowerError> {
        let LexemeConfig::SpeechAct { semantic, role } = self.lexeme(surface)? else {
            return Err(SurfaceLowerError::WrongLexemeKind { surface: surface.to_owned(), expected: "speech_act" });
        };
        Ok(Term::Call {
            function: semantic.clone(),
            arguments: BTreeMap::from([(role.clone(), content)]),
        })
    }

    fn lexeme(&self, surface: &str) -> Result<&LexemeConfig, SurfaceLowerError> {
        self.lexicon
            .get(surface)
            .ok_or_else(|| SurfaceLowerError::MissingLexeme(surface.to_owned()))
    }
}

impl fmt::Display for SurfaceLowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLexeme(surface) => write!(f, "surface lexeme `{surface}` is not declared"),
            Self::WrongLexemeKind { surface, expected } => write!(f, "surface lexeme `{surface}` is not a {expected}"),
            Self::MissingPrecedence(operator) => write!(f, "surface operator `{operator}` has no precedence"),
            Self::InvalidType { source, message } => write!(f, "invalid surface binder type `{source}`: {message}"),
            Self::InvalidSemanticTerm(message) => write!(f, "surface semantics: {message}"),
        }
    }
}

impl std::error::Error for SurfaceLowerError {}
