use std::fmt;

use crate::semantics::{Checker, Environment, Literal, Term, Type, canonicalize};
use crate::syntax::PragmaticsConfig;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestedValue {
    pub ty: Type,
    pub status: String,
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

    pub fn committed_content(&self) -> Option<&Term> {
        match self {
            Self::Assertion { content }
            | Self::Focus { content, .. }
            | Self::Topic { content, .. } => Some(content),
            Self::Correction { replacement, .. } => Some(replacement),
            Self::Clarification { content, .. } => Some(content),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PragmaticAnalysis {
    pub act: CommunicativeAct,
    pub utterance: Term,
    pub inferred_type: Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PragmaticError {
    MissingOperator { operator: String },
    MissingRole { operator: String, role: String },
    InvalidConfiguredRole { operator: String, role: String },
    NotCommunicative { ty: Type },
    UnknownUtteranceOperator { operator: String },
    MixedQuestionStructure,
    FocusTargetAbsent { operator: String },
    InvalidRepairTarget { operator: String, value: Term },
    InvalidSemanticTerm(String),
}

pub fn validate_pragmatics(
    config: &PragmaticsConfig,
    environment: &Environment,
) -> Result<(), PragmaticError> {
    for (operator, roles) in [
        (
            config.default_assertion_operator.as_str(),
            vec![config.default_assertion_role.as_str()],
        ),
        (
            config.question_operator.as_str(),
            vec![config.question_content_role.as_str()],
        ),
        (
            config.command_operator.as_str(),
            vec![config.command_content_role.as_str()],
        ),
        (
            config.request_operator.as_str(),
            vec![config.request_content_role.as_str()],
        ),
        (
            config.expressive_operator.as_str(),
            vec![config.expressive_state_role.as_str()],
        ),
        (
            config.focus_operator.as_str(),
            vec![config.focus_target_role.as_str(), config.focus_content_role.as_str()],
        ),
        (
            config.topic_operator.as_str(),
            vec![config.topic_target_role.as_str(), config.topic_content_role.as_str()],
        ),
        (
            config.retract_operator.as_str(),
            vec![config.repair_target_role.as_str()],
        ),
        (
            config.correction_operator.as_str(),
            vec![
                config.repair_target_role.as_str(),
                config.correction_replacement_role.as_str(),
            ],
        ),
        (
            config.clarification_operator.as_str(),
            vec![
                config.repair_target_role.as_str(),
                config.clarification_content_role.as_str(),
            ],
        ),
        (
            config.disjunction_operator.as_str(),
            vec![
                config.disjunction_left_role.as_str(),
                config.disjunction_right_role.as_str(),
            ],
        ),
    ] {
        let signature = environment
            .operator(operator)
            .ok_or_else(|| PragmaticError::MissingOperator {
                operator: operator.to_owned(),
            })?;
        for role in roles {
            if !signature.parameters.iter().any(|parameter| parameter.name == role) {
                return Err(PragmaticError::InvalidConfiguredRole {
                    operator: operator.to_owned(),
                    role: role.to_owned(),
                });
            }
        }
    }
    Ok(())
}

pub fn interpret_pragmatics(
    term: &Term,
    inferred_type: &Type,
    config: &PragmaticsConfig,
    environment: &Environment,
) -> Result<PragmaticAnalysis, PragmaticError> {
    validate_pragmatics(config, environment)?;

    let proposition = Type::named("Proposition");
    let utterance = Type::named("Utterance");
    if environment.is_assignable(inferred_type, &proposition) {
        let wrapped = Term::Call {
            function: config.default_assertion_operator.clone(),
            arguments: std::collections::BTreeMap::from([(
                config.default_assertion_role.clone(),
                term.clone(),
            )]),
        };
        let ty = Checker::new(environment)
            .infer(&wrapped)
            .map_err(|error| PragmaticError::InvalidSemanticTerm(error.to_string()))?;
        return Ok(PragmaticAnalysis {
            act: CommunicativeAct::Assertion {
                content: term.clone(),
            },
            utterance: wrapped,
            inferred_type: ty,
        });
    }

    if !environment.is_assignable(inferred_type, &utterance) {
        return Err(PragmaticError::NotCommunicative {
            ty: inferred_type.clone(),
        });
    }

    let Term::Call { function, arguments } = term else {
        return Err(PragmaticError::InvalidSemanticTerm(
            "an Utterance value must be an explicit semantic call".into(),
        ));
    };

    let act = if function == &config.question_operator {
        let content = role(arguments, function, &config.question_content_role)?.clone();
        let requested = requested_values(&content, config);
        let choice = is_call(&content, &config.disjunction_operator);
        if !requested.is_empty() && choice {
            return Err(PragmaticError::MixedQuestionStructure);
        }
        let kind = if !requested.is_empty() {
            QuestionKind::Value { requested }
        } else if choice {
            QuestionKind::Choice {
                alternatives: collect_disjunction_alternatives(&content, config)?,
            }
        } else {
            QuestionKind::Truth
        };
        CommunicativeAct::Question { kind, content }
    } else if function == &config.command_operator {
        CommunicativeAct::Command {
            content: role(arguments, function, &config.command_content_role)?.clone(),
        }
    } else if function == &config.request_operator {
        CommunicativeAct::Request {
            content: role(arguments, function, &config.request_content_role)?.clone(),
        }
    } else if function == &config.expressive_operator {
        CommunicativeAct::Expressive {
            state: role(arguments, function, &config.expressive_state_role)?.clone(),
        }
    } else if function == &config.focus_operator {
        let target = role(arguments, function, &config.focus_target_role)?.clone();
        let content = role(arguments, function, &config.focus_content_role)?.clone();
        if !contains_subterm(&content, &target) {
            return Err(PragmaticError::FocusTargetAbsent {
                operator: function.clone(),
            });
        }
        CommunicativeAct::Focus { target, content }
    } else if function == &config.topic_operator {
        let target = role(arguments, function, &config.topic_target_role)?.clone();
        let content = role(arguments, function, &config.topic_content_role)?.clone();
        if !contains_subterm(&content, &target) {
            return Err(PragmaticError::FocusTargetAbsent {
                operator: function.clone(),
            });
        }
        CommunicativeAct::Topic { target, content }
    } else if function == &config.retract_operator {
        let target_term = role(arguments, function, &config.repair_target_role)?.clone();
        CommunicativeAct::Retraction {
            target: repair_target_id(function, &target_term)?,
        }
    } else if function == &config.correction_operator {
        let target_term = role(arguments, function, &config.repair_target_role)?.clone();
        CommunicativeAct::Correction {
            target: repair_target_id(function, &target_term)?,
            replacement: role(arguments, function, &config.correction_replacement_role)?.clone(),
        }
    } else if function == &config.clarification_operator {
        let target_term = role(arguments, function, &config.repair_target_role)?.clone();
        CommunicativeAct::Clarification {
            target: repair_target_id(function, &target_term)?,
            content: role(arguments, function, &config.clarification_content_role)?.clone(),
        }
    } else {
        return Err(PragmaticError::UnknownUtteranceOperator {
            operator: function.clone(),
        });
    };

    Ok(PragmaticAnalysis {
        act,
        utterance: term.clone(),
        inferred_type: inferred_type.clone(),
    })
}

fn role<'a>(
    arguments: &'a std::collections::BTreeMap<String, Term>,
    operator: &str,
    role: &str,
) -> Result<&'a Term, PragmaticError> {
    arguments
        .get(role)
        .ok_or_else(|| PragmaticError::MissingRole {
            operator: operator.to_owned(),
            role: role.to_owned(),
        })
}

fn is_call(term: &Term, operator: &str) -> bool {
    matches!(term, Term::Call { function, .. } if function == operator)
}

fn requested_values(term: &Term, config: &PragmaticsConfig) -> Vec<RequestedValue> {
    let mut output = Vec::new();
    collect_requested_values(term, config, &mut output);
    output
}

fn collect_requested_values(
    term: &Term,
    config: &PragmaticsConfig,
    output: &mut Vec<RequestedValue>,
) {
    match term {
        Term::Literal(Literal::Structured(value))
            if value.family == config.information_family
                && (value.canonical == config.unknown_status
                    || value
                        .canonical
                        .strip_prefix(config.unknown_status.as_str())
                        .is_some_and(|suffix| suffix.starts_with(':'))) =>
        {
            output.push(RequestedValue {
                ty: value.ty.clone(),
                status: value.canonical.clone(),
            });
        }
        Term::Call { arguments, .. } | Term::Record(arguments) => {
            for value in arguments.values() {
                collect_requested_values(value, config, output);
            }
        }
        Term::Bind { body, .. } => collect_requested_values(body, config, output),
        Term::Field { record, .. } => collect_requested_values(record, config, output),
        Term::Const(_) | Term::Var(_) | Term::Literal(_) => {}
    }
}

fn collect_disjunction_alternatives(
    term: &Term,
    config: &PragmaticsConfig,
) -> Result<Vec<Term>, PragmaticError> {
    let mut output = Vec::new();
    collect_disjunction(term, config, &mut output)?;
    Ok(output)
}

fn collect_disjunction(
    term: &Term,
    config: &PragmaticsConfig,
    output: &mut Vec<Term>,
) -> Result<(), PragmaticError> {
    if let Term::Call { function, arguments } = term {
        if function == &config.disjunction_operator {
            let left = role(arguments, function, &config.disjunction_left_role)?;
            let right = role(arguments, function, &config.disjunction_right_role)?;
            collect_disjunction(left, config, output)?;
            collect_disjunction(right, config, output)?;
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
    if canonicalize(content) == *target {
        return true;
    }
    match content {
        Term::Call { arguments, .. } | Term::Record(arguments) => {
            arguments.values().any(|value| contains_canonical(value, target))
        }
        Term::Bind { body, .. } => contains_canonical(body, target),
        Term::Field { record, .. } => contains_canonical(record, target),
        Term::Const(_) | Term::Var(_) | Term::Literal(_) => false,
    }
}

fn repair_target_id(operator: &str, term: &Term) -> Result<u64, PragmaticError> {
    let value = match term {
        Term::Literal(Literal::Integer(value)) if *value > 0 => u64::try_from(*value).ok(),
        Term::Literal(Literal::Structured(value)) if value.ty == Type::named("Number") => {
            value.canonical.parse::<u64>().ok().filter(|value| *value > 0)
        }
        _ => None,
    };
    value.ok_or_else(|| PragmaticError::InvalidRepairTarget {
        operator: operator.to_owned(),
        value: term.clone(),
    })
}

impl fmt::Display for PragmaticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOperator { operator } => {
                write!(f, "pragmatics references unknown semantic operator `{operator}`")
            }
            Self::MissingRole { operator, role } => {
                write!(f, "pragmatic operator `{operator}` has no role `{role}`")
            }
            Self::InvalidConfiguredRole { operator, role } => {
                write!(f, "pragmatics config maps undeclared role `{role}` on `{operator}`")
            }
            Self::NotCommunicative { ty } => {
                write!(f, "top-level semantic value of type `{ty}` is neither a proposition nor an utterance")
            }
            Self::UnknownUtteranceOperator { operator } => {
                write!(f, "Utterance operator `{operator}` is not declared by the pragmatics protocol")
            }
            Self::MixedQuestionStructure => {
                f.write_str("question cannot simultaneously request an unknown value and declare a top-level choice")
            }
            Self::FocusTargetAbsent { operator } => {
                write!(f, "`{operator}` target is not structurally present in its declared content")
            }
            Self::InvalidRepairTarget { operator, value } => {
                write!(f, "`{operator}` repair target `{value}` must be a positive whole utterance number")
            }
            Self::InvalidSemanticTerm(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for PragmaticError {}
