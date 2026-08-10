use std::collections::BTreeMap;
use std::fmt;

use chumsky::error::Rich;
use chumsky::prelude::*;

use crate::semantics::{FunctionParameter, Literal, LiteralKind, Parameter, Type};

use super::{Declaration, ParsedTerm, Specification};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
}

type Extra<'src> = extra::Err<Rich<'src, char>>;

pub fn parse_type(source: &str) -> Result<Type, Vec<ParseError>> {
    type_parser()
        .then_ignore(end())
        .parse(source)
        .into_result()
        .map_err(render_errors)
}

pub fn parse_term(source: &str) -> Result<ParsedTerm, Vec<ParseError>> {
    term_parser()
        .then_ignore(end())
        .parse(source)
        .into_result()
        .map_err(render_errors)
}

pub fn parse_specification(source: &str) -> Result<Specification, Vec<ParseError>> {
    declaration_parser()
        .repeated()
        .collect::<Vec<_>>()
        .map(|declarations| Specification { declarations })
        .then_ignore(end())
        .parse(source)
        .into_result()
        .map_err(render_errors)
}

fn identifier<'src>() -> impl Parser<'src, &'src str, String, Extra<'src>> + Clone {
    text::ident::<_, Extra<'src>>().map(str::to_owned).padded()
}

fn type_parser<'src>() -> impl Parser<'src, &'src str, Type, Extra<'src>> + Clone {
    recursive(|ty| {
        let type_variable = just('$')
            .padded()
            .ignore_then(identifier())
            .map(Type::Variable);

        let function_parameter = identifier()
            .then_ignore(just(':').padded())
            .then(ty.clone())
            .map(|(name, ty)| FunctionParameter {
                name: Some(name),
                ty,
            });

        let function = just("fn")
            .padded()
            .ignore_then(
                function_parameter
                    .separated_by(just(',').padded())
                    .allow_trailing()
                    .collect::<Vec<_>>()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .then_ignore(just("->").padded())
            .then(ty.clone())
            .map(|(parameters, returns)| Type::Function {
                parameters,
                returns: Box::new(returns),
            });

        let record_field = identifier()
            .then_ignore(just(':').padded())
            .then(ty.clone());

        let record = record_field
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<_>>()
            .try_map(|fields, span| {
                let mut by_name = BTreeMap::new();
                for (name, ty) in fields {
                    if by_name.insert(name.clone(), ty).is_some() {
                        return Err(Rich::custom(
                            span,
                            format!("record type field `{name}` is specified more than once"),
                        ));
                    }
                }
                Ok(Type::Record(by_name))
            })
            .delimited_by(just('{').padded(), just('}').padded());

        let generic_arguments = ty
            .clone()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just('<').padded(), just('>').padded());

        let named = identifier()
            .then(generic_arguments.or_not())
            .map(|(name, arguments)| match arguments {
                Some(arguments) => Type::Generic { name, arguments },
                None => Type::Named(name),
            });

        choice((function, record, type_variable, named))
    })
}

fn term_parser<'src>() -> impl Parser<'src, &'src str, ParsedTerm, Extra<'src>> + Clone {
    recursive(|term| {
        let integer = just('-')
            .or_not()
            .then(text::digits::<_, Extra<'src>>(10).to_slice())
            .try_map(|(sign, digits): (Option<char>, &str), span| {
                let value = digits.parse::<i64>().map_err(|_| {
                    Rich::custom(span, "integer literal is outside the supported i64 range".to_owned())
                })?;
                Ok(ParsedTerm::Literal(Literal::Integer(if sign.is_some() {
                    -value
                } else {
                    value
                })))
            })
            .padded();

        let boolean = choice((
            just("true").to(ParsedTerm::Literal(Literal::Boolean(true))),
            just("false").to(ParsedTerm::Literal(Literal::Boolean(false))),
        ))
        .padded();

        let string = none_of('"')
            .repeated()
            .collect::<String>()
            .delimited_by(just('"'), just('"'))
            .map(|value| ParsedTerm::Literal(Literal::String(value)))
            .padded();

        let bind = just("bind")
            .padded()
            .ignore_then(identifier())
            .then_ignore(just(':').padded())
            .then(type_parser())
            .then_ignore(just("=>").padded())
            .then(term.clone())
            .map(|((variable, variable_type), body)| ParsedTerm::Bind {
                variable,
                variable_type,
                body: Box::new(body),
            });

        let record_field = identifier()
            .then_ignore(just('=').padded())
            .then(term.clone());

        let record = record_field
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<_>>()
            .try_map(|fields, span| {
                let mut by_name = BTreeMap::new();
                for (name, value) in fields {
                    if by_name.insert(name.clone(), value).is_some() {
                        return Err(Rich::custom(
                            span,
                            format!("record field `{name}` is specified more than once"),
                        ));
                    }
                }
                Ok(ParsedTerm::Record(by_name))
            })
            .delimited_by(just('{').padded(), just('}').padded());

        let argument = identifier()
            .then_ignore(just('=').padded())
            .then(term.clone());

        let arguments = argument
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<_>>()
            .try_map(|arguments, span| {
                let mut by_name = BTreeMap::new();
                for (name, value) in arguments {
                    if by_name.insert(name.clone(), value).is_some() {
                        return Err(Rich::custom(
                            span,
                            format!("argument `{name}` is specified more than once"),
                        ));
                    }
                }
                Ok(by_name)
            })
            .delimited_by(just('(').padded(), just(')').padded());

        let call_or_name = identifier()
            .then(arguments.or_not())
            .map(|(name, arguments)| match arguments {
                Some(arguments) => ParsedTerm::Call {
                    function: name,
                    arguments,
                },
                None => ParsedTerm::Name(name),
            });

        let atom = choice((bind, record, integer, boolean, string, call_or_name))
            .or(term.clone().delimited_by(just('(').padded(), just(')').padded()));

        atom.clone().foldl(
            just('.')
                .padded()
                .ignore_then(identifier())
                .repeated(),
            |record, field| ParsedTerm::Field {
                record: Box::new(record),
                field,
            },
        )
    })
}

fn declaration_parser<'src>() -> impl Parser<'src, &'src str, Declaration, Extra<'src>> + Clone {
    let ty = type_parser();
    let ident = identifier();

    let type_declaration = just("type")
        .padded()
        .ignore_then(ident.clone())
        .then_ignore(just(';').padded())
        .map(|name| Declaration::Type { name });

    let subtype = just("subtype")
        .padded()
        .ignore_then(ident.clone())
        .then_ignore(just(':').padded())
        .then(ident.clone())
        .then_ignore(just(';').padded())
        .map(|(child, parent)| Declaration::Subtype { child, parent });

    let literal_kind = choice((
        just("integer").to(LiteralKind::Integer),
        just("boolean").to(LiteralKind::Boolean),
        just("string").to(LiteralKind::String),
    ))
    .padded();

    let literal = just("literal")
        .padded()
        .ignore_then(literal_kind)
        .then_ignore(just(':').padded())
        .then(ty.clone())
        .then_ignore(just(';').padded())
        .map(|(kind, ty)| Declaration::Literal { kind, ty });

    let constant = just("const")
        .padded()
        .ignore_then(ident.clone())
        .then_ignore(just(':').padded())
        .then(ty.clone())
        .then_ignore(just(';').padded())
        .map(|(name, ty)| Declaration::Const { name, ty });

    let type_parameters = ident
        .clone()
        .separated_by(just(',').padded())
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just('<').padded(), just('>').padded());

    let parameter = ident
        .clone()
        .then_ignore(just(':').padded())
        .then(ty.clone())
        .map(|(name, ty)| Parameter { name, ty });

    let operator = just("operator")
        .padded()
        .ignore_then(ident)
        .then(type_parameters.or_not())
        .then(
            parameter
                .separated_by(just(',').padded())
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just('(').padded(), just(')').padded()),
        )
        .then_ignore(just("->").padded())
        .then(ty)
        .then_ignore(just(';').padded())
        .map(|(((name, type_parameters), parameters), returns)| Declaration::Operator {
            name,
            type_parameters: type_parameters.unwrap_or_default(),
            parameters,
            returns,
        });

    choice((
        type_declaration,
        subtype,
        literal,
        constant,
        operator,
    ))
}

fn render_errors(errors: Vec<Rich<'_, char>>) -> Vec<ParseError> {
    errors
        .into_iter()
        .map(|error| ParseError {
            message: error.to_string(),
        })
        .collect()
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ParseError {}
