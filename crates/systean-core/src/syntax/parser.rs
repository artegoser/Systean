use std::fmt;

use chumsky::error::Rich;
use chumsky::prelude::*;

use super::{Argument, Clause, FrameOrder, LexemeConfig, SurfaceExpr, SyntaxConfig};

type Extra<'src> = extra::Err<Rich<'src, char>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceParseError {
    pub token: usize,
    pub message: String,
}

fn token_parser<'src>() -> impl Parser<'src, &'src str, Vec<String>, Extra<'src>> + Clone {
    text::ident::<_, Extra<'src>>()
        .map(str::to_lowercase)
        .padded()
        .repeated()
        .collect::<Vec<_>>()
}

pub fn tokenize(source: &str) -> Result<Vec<String>, Vec<SurfaceParseError>> {
    token_parser()
        .then_ignore(end())
        .parse(source)
        .into_result()
        .map_err(|errors| {
            errors
                .into_iter()
                .map(|error| SurfaceParseError {
                    token: 0,
                    message: error.to_string(),
                })
                .collect()
        })
}

pub fn parse_surface(source: &str, config: &SyntaxConfig) -> Result<SurfaceExpr, Vec<SurfaceParseError>> {
    let tokens = tokenize(source)?;
    if tokens.is_empty() {
        return Err(vec![SurfaceParseError { token: 0, message: "surface expression is empty".into() }]);
    }
    let mut parser = ParserState { tokens, index: 0, config };
    let expression = parser.parse_expression(0)?;
    if parser.index != parser.tokens.len() {
        return Err(vec![parser.error(format!("unexpected token `{}`", parser.tokens[parser.index]))]);
    }
    Ok(expression)
}

struct ParserState<'a> {
    tokens: Vec<String>,
    index: usize,
    config: &'a SyntaxConfig,
}

impl ParserState<'_> {
    fn parse_expression(&mut self, minimum_precedence: u16) -> Result<SurfaceExpr, Vec<SurfaceParseError>> {
        let mut left = self.parse_operand()?;
        loop {
            if self.peek().is_some_and(|token| token == self.config.scope.close) {
                break;
            }
            let Some((surface, precedence)) = self.peek_infix() else {
                break;
            };
            if precedence < minimum_precedence {
                break;
            }
            self.index += 1;
            let right = self.parse_expression(precedence.saturating_add(1))?;
            left = self.combine_infix(surface, left, right);
        }
        Ok(left)
    }

    fn parse_operand(&mut self) -> Result<SurfaceExpr, Vec<SurfaceParseError>> {
        let Some(token) = self.peek().cloned() else {
            return Err(vec![self.error("expected expression".into())]);
        };
        if token == self.config.scope.open {
            self.index += 1;
            let expression = self.parse_expression(0)?;
            match self.peek() {
                Some(close) if close == self.config.scope.close => {
                    self.index += 1;
                    return Ok(expression);
                }
                _ => return Err(vec![self.error(format!("expected scope close marker `{}`", self.config.scope.close))]),
            }
        }
        if token == self.config.scope.close {
            return Err(vec![self.error(format!("unexpected scope close marker `{token}`"))]);
        }
        match self.config.lexemes.get(&token) {
            Some(LexemeConfig::Prefix { .. }) => {
                self.index += 1;
                let operand = self.parse_operand()?;
                Ok(SurfaceExpr::Prefix { operator: token, operand: Box::new(operand) })
            }
            Some(LexemeConfig::SpeechAct { .. }) => {
                self.index += 1;
                let content = self.parse_operand()?;
                Ok(SurfaceExpr::SpeechAct { operator: token, content: Box::new(content) })
            }
            _ => self.parse_clause(),
        }
    }

    fn parse_clause(&mut self) -> Result<SurfaceExpr, Vec<SurfaceParseError>> {
        match self.config.order.frame {
            FrameOrder::PrimaryPredicateRest => self.parse_primary_predicate_rest(),
            FrameOrder::PredicateArguments => self.parse_predicate_arguments(),
        }
    }

    fn parse_primary_predicate_rest(&mut self) -> Result<SurfaceExpr, Vec<SurfaceParseError>> {
        let primary = if self.peek_is_zero_primary_predicate() {
            None
        } else {
            Some(self.parse_argument()?)
        };
        let mut inner_prefixes = Vec::new();
        while self.peek().is_some_and(|token| matches!(self.config.lexemes.get(token), Some(LexemeConfig::Prefix { .. }))) {
            inner_prefixes.push(self.tokens[self.index].clone());
            self.index += 1;
        }
        let predicate = self.take_predicate()?;
        let rest_count = match self.config.lexemes.get(&predicate) {
            Some(LexemeConfig::Predicate { primary_role, rest_roles, .. }) => {
                if primary_role.is_some() != primary.is_some() {
                    return Err(vec![self.error(format!("predicate `{predicate}` primary participant shape does not match the surface clause"))]);
                }
                rest_roles.len()
            }
            _ => unreachable!(),
        };
        let mut rest = Vec::with_capacity(rest_count);
        for _ in 0..rest_count {
            rest.push(self.parse_argument()?);
        }
        Ok(SurfaceExpr::Clause(Clause { primary, inner_prefixes, predicate, rest }))
    }

    fn parse_predicate_arguments(&mut self) -> Result<SurfaceExpr, Vec<SurfaceParseError>> {
        let predicate = self.take_predicate()?;
        let (primary_count, rest_count) = match self.config.lexemes.get(&predicate) {
            Some(LexemeConfig::Predicate { primary_role, rest_roles, .. }) => (if primary_role.is_some() { 1 } else { 0 }, rest_roles.len()),
            _ => unreachable!(),
        };
        let primary = if primary_count == 1 { Some(self.parse_argument()?) } else { None };
        let mut rest = Vec::with_capacity(rest_count);
        for _ in 0..rest_count {
            rest.push(self.parse_argument()?);
        }
        Ok(SurfaceExpr::Clause(Clause { primary, inner_prefixes: Vec::new(), predicate, rest }))
    }

    fn parse_argument(&mut self) -> Result<Argument, Vec<SurfaceParseError>> {
        let Some(token) = self.peek().cloned() else {
            return Err(vec![self.error("expected argument".into())]);
        };
        match self.config.lexemes.get(&token) {
            Some(LexemeConfig::Atom { .. }) => {
                self.index += 1;
                Ok(Argument::Atom(token))
            }
            Some(LexemeConfig::Quantifier { .. }) => {
                self.index += 1;
                let Some(restriction) = self.peek().cloned() else {
                    return Err(vec![self.error(format!("quantifier `{token}` requires a restriction"))]);
                };
                match self.config.lexemes.get(&restriction) {
                    Some(LexemeConfig::Class { .. }) => {
                        self.index += 1;
                        Ok(Argument::Quantified { quantifier: token, restriction })
                    }
                    _ => Err(vec![self.error(format!("quantifier `{token}` must be followed by a class expression, found `{restriction}`"))]),
                }
            }
            Some(other) => Err(vec![self.error(format!("`{token}` is not an argument lexeme ({})", lexeme_kind(other)))]),
            None => Err(vec![self.error(format!("unknown surface word `{token}`"))]),
        }
    }

    fn take_predicate(&mut self) -> Result<String, Vec<SurfaceParseError>> {
        let Some(token) = self.peek().cloned() else {
            return Err(vec![self.error("expected predicate".into())]);
        };
        if matches!(self.config.lexemes.get(&token), Some(LexemeConfig::Predicate { .. })) {
            self.index += 1;
            Ok(token)
        } else {
            Err(vec![self.error(format!("expected predicate, found `{token}`"))])
        }
    }

    fn peek_is_zero_primary_predicate(&self) -> bool {
        self.peek().is_some_and(|token| matches!(
            self.config.lexemes.get(token),
            Some(LexemeConfig::Predicate { primary_role: None, .. })
        ))
    }

    fn peek_infix(&self) -> Option<(String, u16)> {
        let token = self.peek()?;
        let LexemeConfig::Infix { semantic, .. } = self.config.lexemes.get(token)? else {
            return None;
        };
        let precedence = self.config.precedence(semantic)?;
        Some((token.clone(), precedence))
    }

    fn combine_infix(&self, surface: String, left: SurfaceExpr, right: SurfaceExpr) -> SurfaceExpr {
        if self.config.logic.flatten_same_operator {
            let mut operands = Vec::new();
            push_flattened(&surface, left, &mut operands);
            push_flattened(&surface, right, &mut operands);
            return SurfaceExpr::Infix { operator: surface, operands };
        }
        SurfaceExpr::Infix { operator: surface, operands: vec![left, right] }
    }

    fn peek(&self) -> Option<&String> {
        self.tokens.get(self.index)
    }

    fn error(&self, message: String) -> SurfaceParseError {
        SurfaceParseError { token: self.index, message }
    }
}

fn push_flattened(operator: &str, expression: SurfaceExpr, output: &mut Vec<SurfaceExpr>) {
    match expression {
        SurfaceExpr::Infix { operator: nested_operator, operands } => {
            if nested_operator == operator {
                output.extend(operands);
            } else {
                output.push(SurfaceExpr::Infix { operator: nested_operator, operands });
            }
        }
        other => output.push(other),
    }
}

fn lexeme_kind(lexeme: &LexemeConfig) -> &'static str {
    match lexeme {
        LexemeConfig::Atom { .. } => "atom",
        LexemeConfig::Class { .. } => "class",
        LexemeConfig::Predicate { .. } => "predicate",
        LexemeConfig::Prefix { .. } => "prefix",
        LexemeConfig::Infix { .. } => "infix",
        LexemeConfig::Quantifier { .. } => "quantifier",
        LexemeConfig::SpeechAct { .. } => "speech_act",
    }
}

impl fmt::Display for SurfaceParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "surface token {}: {}", self.token + 1, self.message)
    }
}

impl std::error::Error for SurfaceParseError {}
