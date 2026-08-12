use std::fmt;

use chumsky::error::Rich;
use chumsky::prelude::*;

use crate::semantics::LiteralKind;
use super::parser::type_parser;
use super::typed_ast::{
    DataConstructorDeclaration, SourceActKind, SourceBinderRule, SourceCaptureKind, SourceCaptureRule,
    SourceEffectDirective, SourceExpr, SourceParameter, SourceRepairKind, SourceSurfaceItem,
    SourceSurfaceRule, SourceUnitDefinition, TypedDeclaration, TypedSpecification,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedParseError {
    pub message: String,
}

type Extra<'src> = extra::Err<Rich<'src, char>>;

pub fn parse_typed_specification(source: &str) -> Result<TypedSpecification, Vec<TypedParseError>> {
    let stripped = strip_line_comments(source);
    typed_declaration_parser()
        .repeated()
        .collect::<Vec<_>>()
        .map(|declarations| TypedSpecification { declarations })
        .then_ignore(end())
        .parse(stripped.as_str())
        .into_result()
        .map_err(render_errors)
}

fn identifier<'src>() -> impl Parser<'src, &'src str, String, Extra<'src>> + Clone {
    text::ident::<_, Extra<'src>>().map(str::to_owned).padded()
}

fn qualified_identifier<'src>() -> impl Parser<'src, &'src str, String, Extra<'src>> + Clone {
    text::ident::<_, Extra<'src>>()
        .map(str::to_owned)
        .separated_by(just('.'))
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|parts| parts.join("."))
        .padded()
}

fn generic_parameters<'src>() -> impl Parser<'src, &'src str, Vec<String>, Extra<'src>> + Clone {
    identifier()
        .separated_by(just(',').padded())
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just('<').padded(), just('>').padded())
}

fn source_parameter<'src>() -> impl Parser<'src, &'src str, SourceParameter, Extra<'src>> + Clone {
    just('$')
        .padded()
        .ignore_then(identifier())
        .then_ignore(just(':').padded())
        .then(type_parser())
        .map(|(name, ty)| SourceParameter { name, ty })
}

fn parameter_list<'src>() -> impl Parser<'src, &'src str, Vec<SourceParameter>, Extra<'src>> + Clone {
    source_parameter()
        .separated_by(just(',').padded())
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just('(').padded(), just(')').padded())
}

fn surface_rule_parser<'src>() -> impl Parser<'src, &'src str, SourceSurfaceRule, Extra<'src>> + Clone {
    #[derive(Clone, Debug)]
    enum Directive {
        Form(Vec<SourceSurfaceItem>),
        Precedence(u16),
        Associative,
        Bind(SourceBinderRule),
        Capture(SourceCaptureRule),
        OuterOnly,
    }

    let item = choice((
        just('_').to(SourceSurfaceItem::Root),
        just("@restriction").to(SourceSurfaceItem::Restriction),
        just('$')
            .padded()
            .ignore_then(identifier())
            .map(SourceSurfaceItem::Argument),
    ))
    .padded();

    let form = just("form")
        .padded()
        .ignore_then(item.repeated().at_least(1).collect::<Vec<_>>())
        .then_ignore(just(';').padded())
        .map(Directive::Form);

    let precedence = just("precedence")
        .padded()
        .ignore_then(text::digits::<_, Extra<'src>>(10).to_slice())
        .try_map(|digits: &str, span| {
            digits
                .parse::<u16>()
                .map(Directive::Precedence)
                .map_err(|_| Rich::custom(span, "surface precedence is outside the supported u16 range"))
        })
        .then_ignore(just(';').padded());

    let associative = just("associative")
        .padded()
        .then_ignore(just(';').padded())
        .to(Directive::Associative);

    let bind = just("bind")
        .padded()
        .ignore_then(just('$').padded().ignore_then(identifier()))
        .then_ignore(just("using").padded())
        .then(qualified_identifier())
        .then_ignore(just(';').padded())
        .map(|(parameter, combiner)| Directive::Bind(SourceBinderRule { parameter, combiner }));

    let capture = just("capture")
        .padded()
        .ignore_then(just('$').padded().ignore_then(identifier()))
        .then_ignore(just("bare").padded())
        .then_ignore(just(';').padded())
        .map(|parameter| Directive::Capture(SourceCaptureRule {
            parameter,
            kind: SourceCaptureKind::BareToken,
        }));

    let outer_only = just("outer")
        .padded()
        .then_ignore(just(';').padded())
        .to(Directive::OuterOnly);

    choice((form, precedence, associative, bind, capture, outer_only))
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>()
        .delimited_by(just('{').padded(), just('}').padded())
        .try_map(|directives, span| {
            let mut form = None;
            let mut precedence = None;
            let mut associative = false;
            let mut binder = None;
            let mut captures = Vec::new();
            let mut outer_only = false;
            for directive in directives {
                match directive {
                    Directive::Form(items) => {
                        if form.replace(items).is_some() {
                            return Err(Rich::custom(span, "surface block contains more than one `form` declaration"));
                        }
                    }
                    Directive::Precedence(value) => {
                        if precedence.replace(value).is_some() {
                            return Err(Rich::custom(span, "surface block contains more than one `precedence` declaration"));
                        }
                    }
                    Directive::Associative => {
                        if associative {
                            return Err(Rich::custom(span, "surface block contains duplicate `associative` declaration"));
                        }
                        associative = true;
                    }
                    Directive::Bind(rule) => {
                        if binder.replace(rule).is_some() {
                            return Err(Rich::custom(span, "surface block contains more than one `bind` declaration"));
                        }
                    }
                    Directive::Capture(rule) => captures.push(rule),
                    Directive::OuterOnly => {
                        if outer_only {
                            return Err(Rich::custom(span, "surface block contains duplicate `outer` declaration"));
                        }
                        outer_only = true;
                    }
                }
            }
            let Some(items) = form else {
                return Err(Rich::custom(span, "surface block requires exactly one `form` declaration"));
            };
            Ok(SourceSurfaceRule { items, precedence, associative, binder, captures, outer_only })
        })
}

fn source_expr_parser<'src>() -> impl Parser<'src, &'src str, SourceExpr, Extra<'src>> + Clone {
    recursive(|expr| {
        let integer = just('-')
            .or_not()
            .then(text::digits::<_, Extra<'src>>(10).to_slice())
            .try_map(|(sign, digits): (Option<char>, &str), span| {
                let value = digits.parse::<i64>().map_err(|_| {
                    Rich::custom(span, "integer literal is outside the supported i64 range")
                })?;
                Ok(SourceExpr::Integer(if sign.is_some() { -value } else { value }))
            })
            .padded();

        let boolean = choice((
            just("true").to(SourceExpr::Boolean(true)),
            just("false").to(SourceExpr::Boolean(false)),
        ))
        .padded();

        let string = none_of('"')
            .repeated()
            .collect::<String>()
            .delimited_by(just('"'), just('"'))
            .map(SourceExpr::String)
            .padded();

        let local_name = just('$')
            .padded()
            .ignore_then(identifier());

        let lambda_parameter = source_parameter();
        let lambda = just("fn")
            .padded()
            .ignore_then(
                lambda_parameter
                    .separated_by(just(',').padded())
                    .at_least(1)
                    .collect::<Vec<_>>()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .then_ignore(just("=>").padded())
            .then(expr.clone())
            .map(|(parameters, body)| {
                parameters.into_iter().rev().fold(body, |body, parameter| {
                    SourceExpr::Lambda {
                        parameter,
                        body: Box::new(body),
                    }
                })
            });

        let arguments = expr
            .clone()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just('(').padded(), just(')').padded());

        let local = local_name
            .then(arguments.clone().or_not())
            .map(|(name, arguments)| match arguments {
                Some(arguments) => SourceExpr::Invoke {
                    function: Box::new(SourceExpr::Local(name)),
                    arguments,
                },
                None => SourceExpr::Local(name),
            });

        let call_or_name = qualified_identifier()
            .then(arguments.or_not())
            .map(|(name, arguments)| match arguments {
                Some(arguments) => SourceExpr::Call { function: name, arguments },
                None => SourceExpr::Name(name),
            });

        choice((lambda, integer, boolean, string, local, call_or_name))
            .or(expr.clone().delimited_by(just('(').padded(), just(')').padded()))
    })
}

fn effect_directives_parser<'src>() -> impl Parser<'src, &'src str, Vec<SourceEffectDirective>, Extra<'src>> + Clone {
    let argument = just('$').padded().ignore_then(identifier());
    let arguments = argument
        .clone()
        .separated_by(just(',').padded())
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just('(').padded(), just(')').padded());

    let act_kind = choice((
        just("assertion").to(SourceActKind::Assertion),
        just("question").to(SourceActKind::Question),
        just("command").to(SourceActKind::Command),
        just("request").to(SourceActKind::Request),
        just("expressive").to(SourceActKind::Expressive),
        just("focus").to(SourceActKind::Focus),
        just("topic").to(SourceActKind::Topic),
        just("retraction").to(SourceActKind::Retraction),
        just("correction").to(SourceActKind::Correction),
        just("clarification").to(SourceActKind::Clarification),
    ))
    .padded();

    let act = just("act")
        .padded()
        .ignore_then(act_kind)
        .then(arguments)
        .then_ignore(just(';').padded())
        .map(|(kind, arguments)| SourceEffectDirective::Act { kind, arguments });

    let commit = just("commit")
        .padded()
        .ignore_then(argument.clone())
        .then_ignore(just(';').padded())
        .map(|argument| SourceEffectDirective::Commit { argument });

    let contains = just("require_contains")
        .padded()
        .ignore_then(argument.clone())
        .then(argument.clone())
        .then_ignore(just(';').padded())
        .map(|(content, target)| SourceEffectDirective::RequireContains { content, target });

    let choice_operator = just("choice")
        .padded()
        .ignore_then(qualified_identifier())
        .then_ignore(just(';').padded())
        .map(|operator| SourceEffectDirective::Choice { operator });

    let repair_kind = choice((
        just("retract").to(SourceRepairKind::Retract),
        just("replace").to(SourceRepairKind::Replace),
        just("clarify").to(SourceRepairKind::Clarify),
    ))
    .padded();
    let repair = just("repair")
        .padded()
        .ignore_then(repair_kind)
        .then(argument.clone())
        .then(argument.or_not())
        .then_ignore(just(';').padded())
        .map(|((kind, target), value)| SourceEffectDirective::Repair { kind, target, value });

    choice((act, commit, contains, choice_operator, repair))
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>()
        .delimited_by(just('{').padded(), just('}').padded())
}

fn typed_declaration_parser<'src>() -> impl Parser<'src, &'src str, TypedDeclaration, Extra<'src>> + Clone {
    let ident = identifier();
    let generics = generic_parameters();
    let parameters = parameter_list();
    let ty = type_parser();
    let expr = source_expr_parser();
    let surface = surface_rule_parser();

    let type_declaration = just("type")
        .padded()
        .ignore_then(ident.clone())
        .then(generics.clone().or_not())
        .then_ignore(just(';').padded())
        .map(|(name, type_parameters)| TypedDeclaration::Type {
            name,
            type_parameters: type_parameters.unwrap_or_default(),
        });

    let subtype = just("subtype")
        .padded()
        .ignore_then(ident.clone())
        .then_ignore(just(':').padded())
        .then(ident.clone())
        .then_ignore(just(';').padded())
        .map(|(child, parent)| TypedDeclaration::Subtype { child, parent });

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
        .map(|(kind, ty)| TypedDeclaration::Literal { kind, ty });

    let callable_shape = choice((
        parameters
            .clone()
            .then_ignore(just("->").padded())
            .then(ty.clone()),
        just(':').padded().ignore_then(ty.clone()).map(|returns| (Vec::new(), returns)),
    ));

    let word_terminator = choice((
        surface.clone().map(Some),
        just(';').padded().to(None),
    ));
    let word = just("word")
        .padded()
        .ignore_then(ident.clone())
        .then(generics.clone().or_not())
        .then(callable_shape.clone())
        .then(just('=').padded().ignore_then(expr.clone()).or_not())
        .then(word_terminator)
        .map(|((((name, type_parameters), (parameters, returns)), definition), surface)| {
            TypedDeclaration::Word {
                name,
                type_parameters: type_parameters.unwrap_or_default(),
                parameters,
                returns,
                definition,
                surface,
            }
        });

    let primitive = just("primitive")
        .padded()
        .ignore_then(qualified_identifier())
        .then(generics.clone().or_not())
        .then(callable_shape.clone())
        .then_ignore(just(';').padded())
        .map(|((name, type_parameters), (parameters, returns))| TypedDeclaration::Primitive {
            name,
            type_parameters: type_parameters.unwrap_or_default(),
            parameters,
            returns,
        });

    let definition = just("def")
        .padded()
        .ignore_then(qualified_identifier())
        .then(generics.clone().or_not())
        .then(callable_shape.clone())
        .then_ignore(just('=').padded())
        .then(expr.clone())
        .then_ignore(just(';').padded())
        .map(|(((name, type_parameters), (parameters, returns)), definition)| TypedDeclaration::Def {
            name,
            type_parameters: type_parameters.unwrap_or_default(),
            parameters,
            returns,
            definition,
        });

    let intrinsic = just("intrinsic")
        .padded()
        .ignore_then(qualified_identifier())
        .then(generics.clone().or_not())
        .then(callable_shape)
        .then_ignore(just(';').padded())
        .map(|((name, type_parameters), (parameters, returns))| TypedDeclaration::Intrinsic {
            name,
            type_parameters: type_parameters.unwrap_or_default(),
            parameters,
            returns,
        });

    let constructor = ident
        .clone()
        .then(parameters.clone().or_not())
        .then_ignore(just(';').padded())
        .map(|(name, parameters)| DataConstructorDeclaration {
            name,
            parameters: parameters.unwrap_or_default(),
        });

    let data = just("data")
        .padded()
        .ignore_then(ident.clone())
        .then(generics.or_not())
        .then(
            constructor
                .repeated()
                .at_least(1)
                .collect::<Vec<_>>()
                .delimited_by(just('{').padded(), just('}').padded()),
        )
        .map(|((name, type_parameters), constructors)| TypedDeclaration::Data {
            name,
            type_parameters: type_parameters.unwrap_or_default(),
            constructors,
        });

    let context = just("context")
        .padded()
        .ignore_then(ident.clone())
        .then_ignore(just(':').padded())
        .then(ty)
        .then_ignore(just(';').padded())
        .map(|(name, ty)| TypedDeclaration::Context { name, ty });

    let dimension = just("dimension")
        .padded()
        .ignore_then(ident.clone())
        .then_ignore(just(':').padded())
        .then(ty.clone())
        .then_ignore(just(';').padded())
        .map(|(name, ty)| TypedDeclaration::Dimension { name, ty });

    let effect = just("effect")
        .padded()
        .ignore_then(qualified_identifier())
        .then(effect_directives_parser())
        .map(|(target, directives)| TypedDeclaration::Effect { target, directives });

    let default_effect = just("default")
        .padded()
        .ignore_then(just("effect").padded())
        .ignore_then(qualified_identifier())
        .then_ignore(just(';').padded())
        .map(|target| TypedDeclaration::DefaultEffect { target });

    let unit_scale = text::digits::<_, Extra<'src>>(10)
        .to_slice()
        .try_map(|digits: &str, span| {
            digits
                .parse::<u64>()
                .map_err(|_| Rich::custom(span, "unit scale is outside the supported u64 range"))
        })
        .then(
            just('/')
                .padded()
                .ignore_then(text::digits::<_, Extra<'src>>(10).to_slice())
                .try_map(|digits: &str, span| {
                    digits.parse::<u64>().map_err(|_| {
                        Rich::custom(span, "unit scale denominator is outside the supported u64 range")
                    })
                })
                .or_not(),
        )
        .map(|(numerator, denominator)| (numerator, denominator.unwrap_or(1)))
        .padded();

    let unit_definition = unit_scale
        .then_ignore(just('*').padded())
        .then(identifier())
        .map(|((numerator, denominator), base)| SourceUnitDefinition {
            numerator,
            denominator,
            base,
        });

    let unit = just("unit")
        .padded()
        .ignore_then(ident)
        .then_ignore(just(':').padded())
        .then(identifier())
        .then(just('=').padded().ignore_then(unit_definition).or_not())
        .then_ignore(just(';').padded())
        .map(|((name, dimension), definition)| TypedDeclaration::Unit {
            name,
            dimension,
            definition,
        });

    choice((
        type_declaration,
        subtype,
        literal,
        word,
        primitive,
        definition,
        intrinsic,
        data,
        context,
        dimension,
        effect,
        default_effect,
        unit,
    ))
}

fn strip_line_comments(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    let mut in_string = false;
    while let Some(ch) = chars.next() {
        if ch == '"' {
            in_string = !in_string;
            output.push(ch);
            continue;
        }
        if !in_string && ch == '/' && chars.peek() == Some(&'/') {
            chars.next();
            output.push(' ');
            output.push(' ');
            for next in chars.by_ref() {
                if next == '\n' {
                    output.push('\n');
                    break;
                }
                output.push(' ');
            }
            continue;
        }
        output.push(ch);
    }
    output
}

fn render_errors(errors: Vec<Rich<'_, char>>) -> Vec<TypedParseError> {
    errors
        .into_iter()
        .map(|error| TypedParseError { message: error.to_string() })
        .collect()
}

impl fmt::Display for TypedParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for TypedParseError {}
