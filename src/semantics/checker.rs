use std::collections::BTreeMap;
use std::fmt;

use super::{Environment, FunctionParameter, Literal, LiteralKind, Term, Type};

pub struct Checker<'env> {
    environment: &'env Environment,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CheckError {
    UnknownConstant(String),
    UnknownOperator(String),
    UnknownVariable(String),
    UnknownBinderType(Type),
    MissingLiteralType(LiteralKind),
    MissingArgument {
        operator: String,
        parameter: String,
    },
    UnknownArgument {
        operator: String,
        argument: String,
    },
    TypeMismatch {
        context: String,
        expected: Type,
        actual: Type,
    },
    ConflictingTypeVariable {
        variable: String,
        first: Type,
        second: Type,
    },
    UnknownField {
        field: String,
        record_type: Type,
    },
    FieldAccessOnNonRecord(Type),
}

impl<'env> Checker<'env> {
    pub fn new(environment: &'env Environment) -> Self {
        Self { environment }
    }

    pub fn infer(&self, term: &Term) -> Result<Type, CheckError> {
        self.infer_with_scope(term, &mut BTreeMap::new())
    }

    fn infer_with_scope(
        &self,
        term: &Term,
        variables: &mut BTreeMap<String, Type>,
    ) -> Result<Type, CheckError> {
        match term {
            Term::Const(name) => self
                .environment
                .constant_type(name)
                .cloned()
                .ok_or_else(|| CheckError::UnknownConstant(name.clone())),
            Term::Var(name) => variables
                .get(name)
                .cloned()
                .ok_or_else(|| CheckError::UnknownVariable(name.clone())),
            Term::Literal(literal) => {
                let kind = match literal {
                    Literal::Integer(_) => LiteralKind::Integer,
                    Literal::Boolean(_) => LiteralKind::Boolean,
                    Literal::String(_) => LiteralKind::String,
                };
                self.environment
                    .literal_type(kind)
                    .cloned()
                    .ok_or(CheckError::MissingLiteralType(kind))
            }
            Term::Call {
                function,
                arguments,
            } => self.infer_call(function, arguments, variables),
            Term::Bind {
                variable,
                variable_type,
                body,
            } => {
                if !self.environment.is_well_formed_type(variable_type) {
                    return Err(CheckError::UnknownBinderType(variable_type.clone()));
                }

                let previous = variables.insert(variable.clone(), variable_type.clone());
                let result = self.infer_with_scope(body, variables);
                match previous {
                    Some(previous) => {
                        variables.insert(variable.clone(), previous);
                    }
                    None => {
                        variables.remove(variable);
                    }
                }
                let body_type = result?;
                Ok(Type::Function {
                    parameters: vec![FunctionParameter {
                        name: Some(variable.clone()),
                        ty: variable_type.clone(),
                    }],
                    returns: Box::new(body_type),
                })
            }
            Term::Record(fields) => {
                let mut typed = BTreeMap::new();
                for (name, value) in fields {
                    typed.insert(name.clone(), self.infer_with_scope(value, variables)?);
                }
                Ok(Type::Record(typed))
            }
            Term::Field { record, field } => {
                let record_type = self.infer_with_scope(record, variables)?;
                match &record_type {
                    Type::Record(fields) => fields
                        .get(field)
                        .cloned()
                        .ok_or_else(|| CheckError::UnknownField {
                            field: field.clone(),
                            record_type,
                        }),
                    _ => Err(CheckError::FieldAccessOnNonRecord(record_type)),
                }
            }
        }
    }

    fn infer_call(
        &self,
        function: &str,
        arguments: &BTreeMap<String, Term>,
        variables: &mut BTreeMap<String, Type>,
    ) -> Result<Type, CheckError> {
        let signature = self
            .environment
            .operator(function)
            .ok_or_else(|| CheckError::UnknownOperator(function.to_owned()))?;

        for argument in arguments.keys() {
            if !signature.parameters.iter().any(|parameter| parameter.name == *argument) {
                return Err(CheckError::UnknownArgument {
                    operator: function.to_owned(),
                    argument: argument.clone(),
                });
            }
        }

        let mut type_bindings = BTreeMap::new();
        for parameter in &signature.parameters {
            let argument = arguments
                .get(&parameter.name)
                .ok_or_else(|| CheckError::MissingArgument {
                    operator: function.to_owned(),
                    parameter: parameter.name.clone(),
                })?;
            let actual = self.infer_with_scope(argument, variables)?;
            self.match_type(
                &parameter.ty,
                &actual,
                &mut type_bindings,
                &format!("argument `{}` of `{function}`", parameter.name),
            )?;
        }

        Ok(signature.returns.substitute(&type_bindings))
    }

    fn match_type(
        &self,
        expected: &Type,
        actual: &Type,
        bindings: &mut BTreeMap<String, Type>,
        context: &str,
    ) -> Result<(), CheckError> {
        match expected {
            Type::Variable(variable) => match bindings.get(variable) {
                Some(bound) if bound != actual => Err(CheckError::ConflictingTypeVariable {
                    variable: variable.clone(),
                    first: bound.clone(),
                    second: actual.clone(),
                }),
                Some(_) => Ok(()),
                None => {
                    bindings.insert(variable.clone(), actual.clone());
                    Ok(())
                }
            },
            Type::Generic {
                name: expected_name,
                arguments: expected_arguments,
            } => match actual {
                Type::Generic {
                    name: actual_name,
                    arguments: actual_arguments,
                } if expected_name == actual_name
                    && expected_arguments.len() == actual_arguments.len() => {
                    for (expected, actual) in expected_arguments.iter().zip(actual_arguments) {
                        self.match_type(expected, actual, bindings, context)?;
                    }
                    Ok(())
                }
                _ => self.require_assignable(expected, actual, context),
            },
            Type::Function {
                parameters: expected_parameters,
                returns: expected_returns,
            } => match actual {
                Type::Function {
                    parameters: actual_parameters,
                    returns: actual_returns,
                } if expected_parameters.len() == actual_parameters.len() => {
                    // Parameter names are documentation/role names; alpha-renaming a binder
                    // does not change the function's semantic type.
                    for (expected, actual) in expected_parameters.iter().zip(actual_parameters) {
                        self.match_type(&expected.ty, &actual.ty, bindings, context)?;
                    }
                    self.match_type(expected_returns, actual_returns, bindings, context)
                }
                _ => self.require_assignable(expected, actual, context),
            },
            Type::Record(expected_fields) => match actual {
                Type::Record(actual_fields) => {
                    for (name, expected_field) in expected_fields {
                        let Some(actual_field) = actual_fields.get(name) else {
                            return Err(CheckError::TypeMismatch {
                                context: context.to_owned(),
                                expected: expected.clone(),
                                actual: actual.clone(),
                            });
                        };
                        self.match_type(expected_field, actual_field, bindings, context)?;
                    }
                    Ok(())
                }
                _ => self.require_assignable(expected, actual, context),
            },
            Type::Named(_) => self.require_assignable(expected, actual, context),
        }
    }

    fn require_assignable(
        &self,
        expected: &Type,
        actual: &Type,
        context: &str,
    ) -> Result<(), CheckError> {
        if self.environment.is_assignable(actual, expected) {
            Ok(())
        } else {
            Err(CheckError::TypeMismatch {
                context: context.to_owned(),
                expected: expected.clone(),
                actual: actual.clone(),
            })
        }
    }
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownConstant(name) => write!(f, "unknown constant `{name}`"),
            Self::UnknownOperator(name) => write!(f, "unknown operator `{name}`"),
            Self::UnknownVariable(name) => write!(f, "unknown variable `{name}`"),
            Self::UnknownBinderType(ty) => write!(f, "binder uses unknown or invalid type `{ty}`"),
            Self::MissingLiteralType(kind) => write!(f, "literal kind `{kind:?}` has no configured type"),
            Self::MissingArgument { operator, parameter } => {
                write!(f, "operator `{operator}` is missing argument `{parameter}`")
            }
            Self::UnknownArgument { operator, argument } => {
                write!(f, "operator `{operator}` has no argument named `{argument}`")
            }
            Self::TypeMismatch {
                context,
                expected,
                actual,
            } => write!(f, "{context} expects `{expected}` but received `{actual}`"),
            Self::ConflictingTypeVariable {
                variable,
                first,
                second,
            } => write!(
                f,
                "type variable `${variable}` was inferred as both `{first}` and `{second}`"
            ),
            Self::UnknownField { field, record_type } => {
                write!(f, "record type `{record_type}` has no field `{field}`")
            }
            Self::FieldAccessOnNonRecord(ty) => {
                write!(f, "cannot access a field on non-record type `{ty}`")
            }
        }
    }
}

impl std::error::Error for CheckError {}
