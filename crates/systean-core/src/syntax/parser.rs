use std::collections::BTreeMap;
use std::fmt;

use super::{
    Argument, ArgumentOmission, Clause, FrameOrder, LexemeConfig, SurfaceExpr, SurfaceLexicon,
    SyntaxConfig,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceParseError {
    pub token: usize,
    pub message: String,
}

pub fn tokenize(source: &str) -> Result<Vec<String>, Vec<SurfaceParseError>> {
    let mut tokens = Vec::new();
    for (index, token) in source.split_whitespace().enumerate() {
        if !is_identifier(token) {
            return Err(vec![SurfaceParseError {
                token: index,
                message: format!("invalid surface token `{token}`"),
            }]);
        }
        tokens.push(token.to_lowercase());
    }
    Ok(tokens)
}

pub fn parse_surface(
    source: &str,
    config: &SyntaxConfig,
    lexicon: &SurfaceLexicon,
) -> Result<SurfaceExpr, Vec<SurfaceParseError>> {
    let tokenized = tokenize_with_quotes(source, &config.quotation.open, &config.quotation.close)?;
    if tokenized.tokens.is_empty() {
        return Err(vec![SurfaceParseError {
            token: 0,
            message: "surface expression is empty".into(),
        }]);
    }
    let mut parser = ParserState {
        tokens: tokenized.tokens,
        quotes: tokenized.quotes,
        index: 0,
        config,
        lexicon,
    };
    let expression = parser.parse_expression(0)?;
    if parser.index != parser.tokens.len() {
        let token = parser.display_token(parser.index);
        return Err(vec![parser.error(format!("unexpected token `{token}`"))]);
    }
    Ok(expression)
}

#[derive(Default)]
struct TokenizedSurface {
    tokens: Vec<String>,
    quotes: BTreeMap<String, String>,
}

fn tokenize_with_quotes(
    source: &str,
    quote_open: &str,
    quote_close: &str,
) -> Result<TokenizedSurface, Vec<SurfaceParseError>> {
    let spans = whitespace_tokens(source);
    let mut output = TokenizedSurface::default();
    let mut index = 0usize;
    while index < spans.len() {
        let (start, end) = spans[index];
        let token = &source[start..end];
        if token.eq_ignore_ascii_case(quote_open) {
            let payload_start = end;
            let mut depth = 1usize;
            let mut cursor = index + 1;
            let mut close_start = None;
            while cursor < spans.len() {
                let (nested_start, nested_end) = spans[cursor];
                let nested = &source[nested_start..nested_end];
                if nested.eq_ignore_ascii_case(quote_open) {
                    depth += 1;
                } else if nested.eq_ignore_ascii_case(quote_close) {
                    depth -= 1;
                    if depth == 0 {
                        close_start = Some(nested_start);
                        break;
                    }
                }
                cursor += 1;
            }
            let Some(close_start) = close_start else {
                return Err(vec![SurfaceParseError {
                    token: output.tokens.len(),
                    message: format!("quotation `{quote_open}` is missing close marker `{quote_close}`"),
                }]);
            };
            let payload = source[payload_start..close_start].trim().to_owned();
            let placeholder = format!("\u{e000}quote{}", output.quotes.len());
            output.quotes.insert(placeholder.clone(), payload);
            output.tokens.push(placeholder);
            index = cursor + 1;
            continue;
        }
        if token.eq_ignore_ascii_case(quote_close) {
            return Err(vec![SurfaceParseError {
                token: output.tokens.len(),
                message: format!("unexpected quotation close marker `{quote_close}`"),
            }]);
        }
        if !is_identifier(token) {
            return Err(vec![SurfaceParseError {
                token: output.tokens.len(),
                message: format!("invalid surface token `{token}`"),
            }]);
        }
        output.tokens.push(token.to_lowercase());
        index += 1;
    }
    Ok(output)
}

fn whitespace_tokens(source: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = None;
    for (offset, character) in source.char_indices() {
        if character.is_whitespace() {
            if let Some(token_start) = start.take() {
                spans.push((token_start, offset));
            }
        } else if start.is_none() {
            start = Some(offset);
        }
    }
    if let Some(token_start) = start {
        spans.push((token_start, source.len()));
    }
    spans
}

fn is_identifier(token: &str) -> bool {
    let mut characters = token.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    (first == '_' || first.is_alphabetic())
        && characters.all(|character| character == '_' || character.is_alphanumeric())
}

struct ParserState<'a> {
    tokens: Vec<String>,
    quotes: BTreeMap<String, String>,
    index: usize,
    config: &'a SyntaxConfig,
    lexicon: &'a SurfaceLexicon,
}

impl ParserState<'_> {
    fn parse_expression(
        &mut self,
        minimum_precedence: u16,
    ) -> Result<SurfaceExpr, Vec<SurfaceParseError>> {
        let mut left = self.parse_operand()?;
        loop {
            if self
                .peek()
                .is_some_and(|token| token.as_str() == self.config.scope.close.as_str())
            {
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
        if let Some(payload) = self.quote_payload(&token).cloned() {
            if self.argument_starts_clause() {
                return self.parse_clause();
            }
            self.index += 1;
            return Ok(SurfaceExpr::Quote(payload));
        }
        if token == self.config.scope.open {
            self.index += 1;
            let expression = self.parse_expression(0)?;
            match self.peek() {
                Some(close) if close.as_str() == self.config.scope.close.as_str() => {
                    self.index += 1;
                    return Ok(expression);
                }
                _ => {
                    return Err(vec![self.error(format!(
                        "expected scope close marker `{}`",
                        self.config.scope.close
                    ))]);
                }
            }
        }
        if token == self.config.scope.close {
            return Err(vec![self.error(format!(
                "unexpected scope close marker `{token}`"
            ))]);
        }

        match self.lexicon.get(&token) {
            Some(LexemeConfig::Prefix { .. }) => {
                self.index += 1;
                let operand = self.parse_operand()?;
                Ok(SurfaceExpr::Prefix {
                    operator: token,
                    operand: Box::new(operand),
                })
            }
            Some(LexemeConfig::SpeechAct { .. }) => {
                self.index += 1;
                let content = self.parse_operand()?;
                Ok(SurfaceExpr::SpeechAct {
                    operator: token,
                    content: Box::new(content),
                })
            }
            Some(LexemeConfig::Atom { .. }) if !self.argument_starts_clause() => {
                self.index += 1;
                Ok(SurfaceExpr::Atom(token))
            }
            Some(LexemeConfig::Context { .. }) if !self.argument_starts_clause() => {
                self.index += 1;
                Ok(SurfaceExpr::Context(token))
            }
            Some(LexemeConfig::Alias { .. }) if !self.argument_starts_clause() => {
                self.index += 1;
                Ok(SurfaceExpr::Alias(token))
            }
            Some(LexemeConfig::Name { .. }) if !self.argument_starts_clause() => {
                let Argument::Name { marker, payload } = self.parse_argument()? else {
                    unreachable!("name lexeme must parse as a name argument");
                };
                Ok(SurfaceExpr::Name { marker, payload })
            }
            Some(LexemeConfig::Reference) if !self.argument_starts_clause() => Err(vec![self.error(
                format!("reference `{token}` requires a typed argument slot"),
            )]),
            Some(LexemeConfig::Atom { .. })
            | Some(LexemeConfig::Context { .. })
            | Some(LexemeConfig::Alias { .. })
            | Some(LexemeConfig::Reference)
            | Some(LexemeConfig::Quantifier { .. })
            | Some(LexemeConfig::Predicate { .. })
            | Some(LexemeConfig::Class { .. })
            | Some(LexemeConfig::Name { .. }) => self.parse_clause(),
            Some(other) => Err(vec![self.error(format!(
                "`{token}` cannot start a standalone surface expression ({})",
                lexeme_kind(other)
            ))]),
            None => Err(vec![self.error(format!("unknown lexical root `{token}`"))]),
        }
    }

    fn argument_starts_clause(&self) -> bool {
        if self.config.order.frame != FrameOrder::PrimaryPredicateRest {
            return false;
        }
        let Some(mut index) = self.argument_end_index(self.index) else {
            return false;
        };
        while self
            .tokens
            .get(index)
            .is_some_and(|token| matches!(self.lexicon.get(token), Some(LexemeConfig::Prefix { .. })))
        {
            index += 1;
        }
        matches!(
            self.tokens.get(index).and_then(|token| self.lexicon.get(token)),
            Some(LexemeConfig::Predicate {
                primary_role: Some(_),
                ..
            }) | Some(LexemeConfig::Class { .. })
        )
    }

    fn argument_end_index(&self, start: usize) -> Option<usize> {
        let token = self.tokens.get(start)?;
        if self.quote_payload(token).is_some() {
            return Some(start + 1);
        }
        match self.lexicon.get(token)? {
            LexemeConfig::Atom { .. }
            | LexemeConfig::Context { .. }
            | LexemeConfig::Alias { .. }
            | LexemeConfig::Reference => Some(start + 1),
            LexemeConfig::Name { .. } => self
                .tokens
                .get(start + 1)
                .filter(|payload| self.quote_payload(payload).is_none())
                .map(|_| start + 2),
            LexemeConfig::Quantifier { .. } => {
                let restriction = self.tokens.get(start + 1)?;
                matches!(self.lexicon.get(restriction), Some(LexemeConfig::Class { .. }))
                    .then_some(start + 2)
            }
            _ => None,
        }
    }

    fn parse_clause(&mut self) -> Result<SurfaceExpr, Vec<SurfaceParseError>> {
        match self.config.order.frame {
            FrameOrder::PrimaryPredicateRest => self.parse_primary_predicate_rest(),
            FrameOrder::PredicateArguments => self.parse_predicate_arguments(),
        }
    }

    fn parse_primary_predicate_rest(&mut self) -> Result<SurfaceExpr, Vec<SurfaceParseError>> {
        let primary = if self.peek_is_predicate() {
            match self.peek_predicate_primary_role() {
                Some(true) if self.omission_allowed() => Some(Argument::Omitted),
                Some(true) => {
                    return Err(vec![self.error("predicate requires an explicit primary argument".into())]);
                }
                Some(false) => None,
                None => unreachable!(),
            }
        } else {
            Some(self.parse_argument()?)
        };
        let mut inner_prefixes = Vec::new();
        while self
            .peek()
            .is_some_and(|token| matches!(self.lexicon.get(token), Some(LexemeConfig::Prefix { .. })))
        {
            inner_prefixes.push(self.tokens[self.index].clone());
            self.index += 1;
        }
        let predicate = self.take_predicate()?;
        let rest_count = match self.lexicon.get(&predicate) {
            Some(LexemeConfig::Predicate {
                primary_role,
                rest_roles,
                ..
            }) => {
                let surface_has_primary_slot = primary.is_some();
                if primary_role.is_some() != surface_has_primary_slot {
                    return Err(vec![self.error(format!(
                        "predicate `{predicate}` primary participant shape does not match the surface clause"
                    ))]);
                }
                rest_roles.len()
            }
            Some(LexemeConfig::Class { .. }) => {
                if primary.is_none() {
                    return Err(vec![self.error(format!(
                        "class predicate `{predicate}` requires a primary participant"
                    ))]);
                }
                0
            }
            _ => unreachable!(),
        };
        let mut rest = Vec::with_capacity(rest_count);
        for _ in 0..rest_count {
            if self.can_parse_argument_here() {
                rest.push(self.parse_argument()?);
            } else if self.argument_can_be_omitted_here() && self.omission_allowed() {
                rest.push(Argument::Omitted);
            } else {
                return Err(vec![self.error("expected argument".into())]);
            }
        }
        Ok(SurfaceExpr::Clause(Clause {
            primary,
            inner_prefixes,
            predicate,
            rest,
        }))
    }

    fn parse_predicate_arguments(&mut self) -> Result<SurfaceExpr, Vec<SurfaceParseError>> {
        let predicate = self.take_predicate()?;
        let (primary_count, rest_count) = match self.lexicon.get(&predicate) {
            Some(LexemeConfig::Predicate {
                primary_role,
                rest_roles,
                ..
            }) => (usize::from(primary_role.is_some()), rest_roles.len()),
            Some(LexemeConfig::Class { .. }) => (1, 0),
            _ => unreachable!(),
        };
        let primary = if primary_count == 1 {
            if self.can_parse_argument_here() {
                Some(self.parse_argument()?)
            } else if self.argument_can_be_omitted_here() && self.omission_allowed() {
                Some(Argument::Omitted)
            } else {
                return Err(vec![self.error("expected primary argument".into())]);
            }
        } else {
            None
        };
        let mut rest = Vec::with_capacity(rest_count);
        for _ in 0..rest_count {
            if self.can_parse_argument_here() {
                rest.push(self.parse_argument()?);
            } else if self.argument_can_be_omitted_here() && self.omission_allowed() {
                rest.push(Argument::Omitted);
            } else {
                return Err(vec![self.error("expected argument".into())]);
            }
        }
        Ok(SurfaceExpr::Clause(Clause {
            primary,
            inner_prefixes: Vec::new(),
            predicate,
            rest,
        }))
    }

    fn parse_argument(&mut self) -> Result<Argument, Vec<SurfaceParseError>> {
        let Some(token) = self.peek().cloned() else {
            return Err(vec![self.error("expected argument".into())]);
        };
        if let Some(payload) = self.quote_payload(&token).cloned() {
            self.index += 1;
            return Ok(Argument::Quote(payload));
        }
        match self.lexicon.get(&token) {
            Some(LexemeConfig::Atom { .. }) => {
                self.index += 1;
                Ok(Argument::Atom(token))
            }
            Some(LexemeConfig::Context { .. }) => {
                self.index += 1;
                Ok(Argument::Context(token))
            }
            Some(LexemeConfig::Reference) => {
                self.index += 1;
                Ok(Argument::Reference(token))
            }
            Some(LexemeConfig::Alias { .. }) => {
                self.index += 1;
                Ok(Argument::Alias(token))
            }
            Some(LexemeConfig::Name { .. }) => {
                self.index += 1;
                let Some(payload) = self.peek().cloned() else {
                    return Err(vec![self.error(format!(
                        "proper-name marker `{token}` requires one canonical name payload token"
                    ))]);
                };
                if self.quote_payload(&payload).is_some() {
                    return Err(vec![self.error(format!(
                        "proper-name marker `{token}` requires a Systean name payload, not quotation"
                    ))]);
                }
                self.index += 1;
                Ok(Argument::Name {
                    marker: token,
                    payload,
                })
            }
            Some(LexemeConfig::Quantifier { .. }) => {
                self.index += 1;
                let Some(restriction) = self.peek().cloned() else {
                    return Err(vec![self.error(format!(
                        "quantifier `{token}` requires a restriction"
                    ))]);
                };
                match self.lexicon.get(&restriction) {
                    Some(LexemeConfig::Class { .. }) => {
                        self.index += 1;
                        Ok(Argument::Quantified {
                            quantifier: token,
                            restriction,
                        })
                    }
                    _ => Err(vec![self.error(format!(
                        "quantifier `{token}` must be followed by a class expression, found `{restriction}`"
                    ))]),
                }
            }
            Some(other) => Err(vec![self.error(format!(
                "`{token}` is not an argument lexeme ({})",
                lexeme_kind(other)
            ))]),
            None => Err(vec![self.error(format!("unknown lexical root `{token}`"))]),
        }
    }

    fn can_parse_argument_here(&self) -> bool {
        self.peek().is_some_and(|token| {
            self.quote_payload(token).is_some()
                || matches!(
                    self.lexicon.get(token),
                    Some(LexemeConfig::Atom { .. })
                        | Some(LexemeConfig::Context { .. })
                        | Some(LexemeConfig::Alias { .. })
                        | Some(LexemeConfig::Reference)
                        | Some(LexemeConfig::Quantifier { .. })
                        | Some(LexemeConfig::Name { .. })
                )
        })
    }

    fn argument_can_be_omitted_here(&self) -> bool {
        match self.peek() {
            None => true,
            Some(token) if token == &self.config.scope.close => true,
            Some(_) if self.peek_infix().is_some() => true,
            _ => false,
        }
    }

    fn omission_allowed(&self) -> bool {
        self.config.arguments.omission == ArgumentOmission::UniqueReferenceOnly
    }

    fn take_predicate(&mut self) -> Result<String, Vec<SurfaceParseError>> {
        let Some(token) = self.peek().cloned() else {
            return Err(vec![self.error("expected predicate".into())]);
        };
        if matches!(
            self.lexicon.get(&token),
            Some(LexemeConfig::Predicate { .. }) | Some(LexemeConfig::Class { .. })
        ) {
            self.index += 1;
            Ok(token)
        } else {
            Err(vec![self.error(format!("expected predicate, found `{token}`"))])
        }
    }

    fn peek_is_predicate(&self) -> bool {
        self.peek().is_some_and(|token| {
            matches!(
                self.lexicon.get(token),
                Some(LexemeConfig::Predicate { .. }) | Some(LexemeConfig::Class { .. })
            )
        })
    }

    fn peek_predicate_primary_role(&self) -> Option<bool> {
        let token = self.peek()?;
        match self.lexicon.get(token)? {
            LexemeConfig::Predicate { primary_role, .. } => Some(primary_role.is_some()),
            LexemeConfig::Class { .. } => Some(true),
            _ => None,
        }
    }

    fn peek_infix(&self) -> Option<(String, u16)> {
        let token = self.peek()?;
        let LexemeConfig::Infix { semantic, .. } = self.lexicon.get(token)? else {
            return None;
        };
        let precedence = self.config.precedence(semantic)?;
        Some((token.clone(), precedence))
    }

    fn combine_infix(
        &self,
        surface: String,
        left: SurfaceExpr,
        right: SurfaceExpr,
    ) -> SurfaceExpr {
        if self.config.logic.flatten_same_operator {
            let mut operands = Vec::new();
            push_flattened(&surface, left, &mut operands);
            push_flattened(&surface, right, &mut operands);
            return SurfaceExpr::Infix {
                operator: surface,
                operands,
            };
        }
        SurfaceExpr::Infix {
            operator: surface,
            operands: vec![left, right],
        }
    }

    fn peek(&self) -> Option<&String> {
        self.tokens.get(self.index)
    }

    fn quote_payload(&self, token: &str) -> Option<&String> {
        self.quotes.get(token)
    }

    fn display_token(&self, index: usize) -> String {
        let Some(token) = self.tokens.get(index) else {
            return "<end>".into();
        };
        if self.quote_payload(token).is_some() {
            return format!("{} ... {}", self.config.quotation.open, self.config.quotation.close);
        }
        token.clone()
    }

    fn error(&self, message: String) -> SurfaceParseError {
        SurfaceParseError {
            token: self.index,
            message,
        }
    }
}

fn push_flattened(operator: &str, expression: SurfaceExpr, output: &mut Vec<SurfaceExpr>) {
    match expression {
        SurfaceExpr::Infix {
            operator: nested_operator,
            operands,
        } => {
            if nested_operator == operator {
                output.extend(operands);
            } else {
                output.push(SurfaceExpr::Infix {
                    operator: nested_operator,
                    operands,
                });
            }
        }
        other => output.push(other),
    }
}

fn lexeme_kind(lexeme: &LexemeConfig) -> &'static str {
    match lexeme {
        LexemeConfig::Atom { .. } => "atom",
        LexemeConfig::Reference => "reference",
        LexemeConfig::Alias { .. } => "alias",
        LexemeConfig::Context { .. } => "context",
        LexemeConfig::Class { .. } => "class",
        LexemeConfig::Predicate { .. } => "predicate",
        LexemeConfig::Prefix { .. } => "prefix",
        LexemeConfig::Infix { .. } => "infix",
        LexemeConfig::Quantifier { .. } => "quantifier",
        LexemeConfig::SpeechAct { .. } => "speech_act",
        LexemeConfig::Name { .. } => "name",
    }
}

impl fmt::Display for SurfaceParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "surface token {}: {}", self.token + 1, self.message)
    }
}

impl std::error::Error for SurfaceParseError {}
