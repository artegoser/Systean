use std::collections::BTreeMap;
use std::fmt;

use num_traits::ToPrimitive;

use crate::semantics::{
    Checker, Environment, InformationStatus, Literal, StructuredValue, Term, Type, canonicalize,
};
use crate::spec::{
    CompiledActKind, CompiledEffectInstruction, CompiledRepairKind, SymbolDebugInfo,
    TypedSemanticPackage,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestedValue {
    pub ty: Type,
    pub status: InformationStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuestionKind {
    Truth,
    Value { requested: Vec<RequestedValue> },
    Choice { alternatives: Vec<Term> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommunicativeAct {
    Assertion { content: Term },
    Question { kind: QuestionKind, content: Term },
    Command { content: Term },
    Request { content: Term },
    Expressive { state: Term },
    Focus { target: Term, content: Term },
    Topic { target: Term, content: Term },
    Retraction { target: u64 },
    Correction { target: u64, replacement: Term },
    Clarification { target: u64, content: Term },
}

impl CommunicativeAct {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Assertion { .. } => "assertion",
            Self::Question { kind: QuestionKind::Truth, .. } => "truth_question",
            Self::Question { kind: QuestionKind::Value { .. }, .. } => "value_question",
            Self::Question { kind: QuestionKind::Choice { .. }, .. } => "choice_question",
            Self::Command { .. } => "command",
            Self::Request { .. } => "request",
            Self::Expressive { .. } => "expressive",
            Self::Focus { .. } => "focus",
            Self::Topic { .. } => "topic",
            Self::Retraction { .. } => "retraction",
            Self::Correction { .. } => "correction",
            Self::Clarification { .. } => "clarification",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscourseEffect {
    Commit { content: Term },
    Retract { target: u64 },
    Replace { target: u64, replacement: Term },
    Clarify { target: u64, content: Term },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PragmaticAnalysis {
    pub act: CommunicativeAct,
    pub effects: Vec<DiscourseEffect>,
    pub utterance: Term,
    pub inferred_type: Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PragmaticError {
    MissingOperator { operator: String },
    MissingRole { operator: String, role: String },
    MissingDefaultEffect,
    MissingEffect { operator: String },
    InvalidEffect { operator: String, message: String },
    NotCommunicative { ty: Type },
    UnknownUtteranceOperator { operator: String },
    MixedQuestionStructure,
    FocusTargetAbsent { operator: String },
    InvalidRepairTarget { operator: String, value: Term },
    InvalidSemanticTerm(String),
}

pub fn validate_pragmatics(
    package: &TypedSemanticPackage,
    environment: &Environment,
) -> Result<(), PragmaticError> {
    let default = package.default_effect().ok_or(PragmaticError::MissingDefaultEffect)?;
    if package.effect_program(default).is_none() {
        return Err(PragmaticError::InvalidEffect {
            operator: package.source_name_for_symbol(default).unwrap_or("<unknown>").to_owned(),
            message: "default effect target has no effect program".into(),
        });
    }
    for program in package.effect_programs() {
        let name = package
            .source_name_for_symbol(program.symbol)
            .ok_or_else(|| PragmaticError::InvalidEffect {
                operator: "<unknown>".into(),
                message: "effect target has no retained source symbol".into(),
            })?;
        if environment.operator(name).is_none() {
            return Err(PragmaticError::MissingOperator { operator: name.to_owned() });
        }
    }
    Ok(())
}

pub fn interpret_pragmatics(
    term: &Term,
    inferred_type: &Type,
    package: &TypedSemanticPackage,
    environment: &Environment,
) -> Result<PragmaticAnalysis, PragmaticError> {
    validate_pragmatics(package, environment)?;

    let proposition = Type::named("Proposition");
    let utterance = Type::named("Utterance");
    if environment.is_assignable(inferred_type, &proposition) {
        let symbol = package.default_effect().ok_or(PragmaticError::MissingDefaultEffect)?;
        let debug = effect_debug(package, symbol)?;
        if debug.parameter_names.len() != 1 {
            return Err(PragmaticError::InvalidEffect {
                operator: debug.source_name.clone(),
                message: "default assertion effect must accept exactly one proposition argument".into(),
            });
        }
        let arguments = vec![term.clone()];
        let wrapped = Term::Call {
            function: debug.source_name.clone(),
            arguments: BTreeMap::from([(debug.parameter_names[0].clone(), term.clone())]),
        };
        let ty = Checker::new(environment)
            .infer(&wrapped)
            .map_err(|error| PragmaticError::InvalidSemanticTerm(error.to_string()))?;
        return execute_effect_program(symbol, &arguments, wrapped, ty, package);
    }

    if !environment.is_assignable(inferred_type, &utterance) {
        return Err(PragmaticError::NotCommunicative { ty: inferred_type.clone() });
    }

    let Term::Call { function, arguments } = term else {
        return Err(PragmaticError::InvalidSemanticTerm(
            "an Utterance value must be an explicit semantic call".into(),
        ));
    };
    let symbol = package.symbol_id(function).ok_or_else(|| PragmaticError::UnknownUtteranceOperator {
        operator: function.clone(),
    })?;
    if package.effect_program(symbol).is_none() {
        return Err(PragmaticError::MissingEffect { operator: function.clone() });
    }
    let debug = effect_debug(package, symbol)?;
    let ordered = debug
        .parameter_names
        .iter()
        .map(|role| {
            arguments.get(role).cloned().ok_or_else(|| PragmaticError::MissingRole {
                operator: function.clone(),
                role: role.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    execute_effect_program(symbol, &ordered, term.clone(), inferred_type.clone(), package)
}

fn execute_effect_program(
    symbol: crate::semantics::SymbolId,
    arguments: &[Term],
    utterance: Term,
    inferred_type: Type,
    package: &TypedSemanticPackage,
) -> Result<PragmaticAnalysis, PragmaticError> {
    let program = package.effect_program(symbol).ok_or_else(|| PragmaticError::MissingEffect {
        operator: package.source_name_for_symbol(symbol).unwrap_or("<unknown>").to_owned(),
    })?;
    let operator = package.source_name_for_symbol(symbol).unwrap_or("<unknown>");
    let arg = |index: u32| -> Result<&Term, PragmaticError> {
        arguments.get(index as usize).ok_or_else(|| PragmaticError::InvalidEffect {
            operator: operator.to_owned(),
            message: format!("effect references absent argument slot {index}"),
        })
    };

    let mut act = None;
    let mut effects = Vec::new();
    let mut choice_operator = None;
    for instruction in &program.instructions {
        match instruction {
            CompiledEffectInstruction::Choice { operator } => choice_operator = Some(*operator),
            CompiledEffectInstruction::RequireContains { content, target } => {
                let content = arg(*content)?;
                let target = arg(*target)?;
                if !contains_subterm(content, target) {
                    return Err(PragmaticError::FocusTargetAbsent { operator: operator.to_owned() });
                }
            }
            CompiledEffectInstruction::Commit { argument } => {
                effects.push(DiscourseEffect::Commit { content: arg(*argument)?.clone() });
            }
            CompiledEffectInstruction::Repair { kind, target, value } => {
                let target_term = arg(*target)?;
                let target = repair_target_id(operator, target_term)?;
                effects.push(match kind {
                    CompiledRepairKind::Retract => DiscourseEffect::Retract { target },
                    CompiledRepairKind::Replace => DiscourseEffect::Replace {
                        target,
                        replacement: arg(value.expect("replace effect has a value"))?.clone(),
                    },
                    CompiledRepairKind::Clarify => DiscourseEffect::Clarify {
                        target,
                        content: arg(value.expect("clarify effect has a value"))?.clone(),
                    },
                });
            }
            CompiledEffectInstruction::Act { .. } => {}
        }
    }

    for instruction in &program.instructions {
        let CompiledEffectInstruction::Act { kind, arguments: act_arguments } = instruction else { continue; };
        if act.is_some() {
            return Err(PragmaticError::InvalidEffect {
                operator: operator.to_owned(),
                message: "effect program contains more than one act instruction".into(),
            });
        }
        let values = act_arguments.iter().map(|index| arg(*index)).collect::<Result<Vec<_>, _>>()?;
        act = Some(match kind {
            CompiledActKind::Assertion => CommunicativeAct::Assertion { content: one(operator, &values)?.clone() },
            CompiledActKind::Question => {
                let content = one(operator, &values)?.clone();
                let requested = requested_values(&content);
                let choice = choice_operator.is_some_and(|candidate| is_symbol_call(&content, candidate, package));
                if !requested.is_empty() && choice { return Err(PragmaticError::MixedQuestionStructure); }
                let kind = if !requested.is_empty() {
                    QuestionKind::Value { requested }
                } else if let Some(choice_operator) = choice_operator.filter(|_| choice) {
                    QuestionKind::Choice { alternatives: collect_binary_alternatives(&content, choice_operator, package)? }
                } else {
                    QuestionKind::Truth
                };
                CommunicativeAct::Question { kind, content }
            }
            CompiledActKind::Command => CommunicativeAct::Command { content: one(operator, &values)?.clone() },
            CompiledActKind::Request => CommunicativeAct::Request { content: one(operator, &values)?.clone() },
            CompiledActKind::Expressive => CommunicativeAct::Expressive { state: one(operator, &values)?.clone() },
            CompiledActKind::Focus => {
                let [target, content] = two(operator, &values)?;
                CommunicativeAct::Focus { target: (*target).clone(), content: (*content).clone() }
            }
            CompiledActKind::Topic => {
                let [target, content] = two(operator, &values)?;
                CommunicativeAct::Topic { target: (*target).clone(), content: (*content).clone() }
            }
            CompiledActKind::Retraction => CommunicativeAct::Retraction { target: repair_target_id(operator, one(operator, &values)?)? },
            CompiledActKind::Correction => {
                let [target, replacement] = two(operator, &values)?;
                CommunicativeAct::Correction { target: repair_target_id(operator, target)?, replacement: (*replacement).clone() }
            }
            CompiledActKind::Clarification => {
                let [target, content] = two(operator, &values)?;
                CommunicativeAct::Clarification { target: repair_target_id(operator, target)?, content: (*content).clone() }
            }
        });
    }

    let act = act.ok_or_else(|| PragmaticError::InvalidEffect {
        operator: operator.to_owned(),
        message: "effect program has no act instruction".into(),
    })?;
    Ok(PragmaticAnalysis { act, effects, utterance, inferred_type })
}

fn effect_debug(package: &TypedSemanticPackage, symbol: crate::semantics::SymbolId) -> Result<&SymbolDebugInfo, PragmaticError> {
    package.debug_symbol(symbol).ok_or_else(|| PragmaticError::InvalidEffect {
        operator: package.source_name_for_symbol(symbol).unwrap_or("<unknown>").to_owned(),
        message: "effect target has no debug parameter table".into(),
    })
}

fn one<'a>(operator: &str, values: &[&'a Term]) -> Result<&'a Term, PragmaticError> {
    if let [value] = values { Ok(*value) } else { Err(PragmaticError::InvalidEffect { operator: operator.into(), message: format!("act requires one argument, got {}", values.len()) }) }
}

fn two<'a>(operator: &str, values: &[&'a Term]) -> Result<[&'a Term; 2], PragmaticError> {
    if let [left, right] = values { Ok([*left, *right]) } else { Err(PragmaticError::InvalidEffect { operator: operator.into(), message: format!("act requires two arguments, got {}", values.len()) }) }
}

fn is_symbol_call(term: &Term, operator: crate::semantics::SymbolId, package: &TypedSemanticPackage) -> bool {
    let Some(name) = package.source_name_for_symbol(operator) else { return false; };
    matches!(term, Term::Call { function, .. } if function == name)
}

fn requested_values(term: &Term) -> Vec<RequestedValue> {
    let mut output = Vec::new();
    collect_requested_values(term, &mut output);
    output
}

fn collect_requested_values(term: &Term, output: &mut Vec<RequestedValue>) {
    match term {
        Term::Literal(Literal::Structured(value)) => {
            if let StructuredValue::Information { status: InformationStatus::Unknown, .. } = &value.value {
                output.push(RequestedValue { ty: value.ty.clone(), status: InformationStatus::Unknown });
            }
        }
        Term::Call { arguments, .. } | Term::Record(arguments) => {
            for value in arguments.values() { collect_requested_values(value, output); }
        }
        Term::Bind { body, .. } => collect_requested_values(body, output),
        Term::Field { record, .. } => collect_requested_values(record, output),
        Term::Const(_) | Term::Var(_) | Term::Literal(_) => {}
    }
}

fn collect_binary_alternatives(
    term: &Term,
    operator: crate::semantics::SymbolId,
    package: &TypedSemanticPackage,
) -> Result<Vec<Term>, PragmaticError> {
    let debug = effect_debug(package, operator)?;
    if debug.parameter_names.len() != 2 {
        return Err(PragmaticError::InvalidEffect { operator: debug.source_name.clone(), message: "choice operator must be binary".into() });
    }
    let mut output = Vec::new();
    collect_binary(term, operator, package, debug, &mut output)?;
    Ok(output)
}

fn collect_binary(
    term: &Term,
    operator: crate::semantics::SymbolId,
    package: &TypedSemanticPackage,
    debug: &SymbolDebugInfo,
    output: &mut Vec<Term>,
) -> Result<(), PragmaticError> {
    if let Term::Call { function, arguments } = term {
        if package.source_name_for_symbol(operator).is_some_and(|name| name == function) {
            let left = arguments.get(&debug.parameter_names[0]).ok_or_else(|| PragmaticError::MissingRole { operator: function.clone(), role: debug.parameter_names[0].clone() })?;
            let right = arguments.get(&debug.parameter_names[1]).ok_or_else(|| PragmaticError::MissingRole { operator: function.clone(), role: debug.parameter_names[1].clone() })?;
            collect_binary(left, operator, package, debug, output)?;
            collect_binary(right, operator, package, debug, output)?;
            return Ok(());
        }
    }
    output.push(term.clone());
    Ok(())
}

fn contains_subterm(content: &Term, target: &Term) -> bool {
    let target = canonicalize(target);
    contains_canonical(content, &target)
}

fn contains_canonical(content: &Term, target: &Term) -> bool {
    if canonicalize(content) == *target { return true; }
    match content {
        Term::Call { arguments, .. } | Term::Record(arguments) => arguments.values().any(|value| contains_canonical(value, target)),
        Term::Bind { body, .. } => contains_canonical(body, target),
        Term::Field { record, .. } => contains_canonical(record, target),
        Term::Const(_) | Term::Var(_) | Term::Literal(_) => false,
    }
}

fn repair_target_id(operator: &str, term: &Term) -> Result<u64, PragmaticError> {
    let value = match term {
        Term::Literal(Literal::Integer(value)) if *value > 0 => u64::try_from(*value).ok(),
        Term::Literal(Literal::Structured(value)) if value.ty == Type::named("Number") => match &value.value {
            StructuredValue::Number(number) if number.denom() == &num_bigint::BigInt::from(1u8) => number.numer().to_u64().filter(|value| *value > 0),
            _ => None,
        },
        _ => None,
    };
    value.ok_or_else(|| PragmaticError::InvalidRepairTarget { operator: operator.to_owned(), value: term.clone() })
}

impl fmt::Display for PragmaticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOperator { operator } => write!(f, "pragmatics references unknown semantic operator `{operator}`"),
            Self::MissingRole { operator, role } => write!(f, "pragmatic operator `{operator}` has no role `{role}`"),
            Self::MissingDefaultEffect => f.write_str("typed semantic package declares no default discourse effect"),
            Self::MissingEffect { operator } => write!(f, "Utterance operator `{operator}` has no package-declared discourse effect"),
            Self::InvalidEffect { operator, message } => write!(f, "invalid discourse effect for `{operator}`: {message}"),
            Self::NotCommunicative { ty } => write!(f, "top-level semantic value of type `{ty}` is neither a proposition nor an utterance"),
            Self::UnknownUtteranceOperator { operator } => write!(f, "Utterance operator `{operator}` is not a typed semantic symbol"),
            Self::MixedQuestionStructure => f.write_str("question cannot simultaneously request an unknown value and declare a top-level choice"),
            Self::FocusTargetAbsent { operator } => write!(f, "`{operator}` target is not structurally present in its declared content"),
            Self::InvalidRepairTarget { operator, value } => write!(f, "`{operator}` repair target `{value}` must be a positive whole utterance number"),
            Self::InvalidSemanticTerm(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for PragmaticError {}
