use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

use super::{Signature, Type};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiteralKind {
    Integer,
    Boolean,
    String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeDefinition {
    pub type_parameters: Vec<String>,
    pub supertypes: BTreeSet<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Environment {
    types: BTreeMap<String, TypeDefinition>,
    literal_types: BTreeMap<LiteralKind, Type>,
    constants: BTreeMap<String, Type>,
    operators: BTreeMap<String, Signature>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvironmentError {
    DuplicateType(String),
    UnknownType(String),
    DuplicateLiteralKind(LiteralKind),
    DuplicateConstant(String),
    DuplicateOperator(String),
    DuplicateParameter { operator: String, parameter: String },
    DuplicateTypeParameter { owner: String, parameter: String },
    UndeclaredTypeVariable { operator: String, variable: String },
    WrongTypeArity { name: String, expected: usize, actual: usize },
    EmptyGenericArguments(String),
    CyclicSubtype { child: String, parent: String },
}

impl Environment {
    pub fn new() -> Self { Self::default() }

    pub fn define_type(
        &mut self,
        name: impl Into<String>,
        type_parameters: Vec<String>,
    ) -> Result<(), EnvironmentError> {
        let name = name.into();
        if self.types.contains_key(&name) {
            return Err(EnvironmentError::DuplicateType(name));
        }
        let mut seen = BTreeSet::new();
        for parameter in &type_parameters {
            if !seen.insert(parameter.clone()) {
                return Err(EnvironmentError::DuplicateTypeParameter {
                    owner: name,
                    parameter: parameter.clone(),
                });
            }
        }
        self.types.insert(
            name,
            TypeDefinition { type_parameters, supertypes: BTreeSet::new() },
        );
        Ok(())
    }

    pub fn define_subtype(
        &mut self,
        child: impl Into<String>,
        parent: impl Into<String>,
    ) -> Result<(), EnvironmentError> {
        let child = child.into();
        let parent = parent.into();
        self.require_type_arity(&child, 0)?;
        self.require_type_arity(&parent, 0)?;
        if child == parent || self.is_named_subtype(&parent, &child) {
            return Err(EnvironmentError::CyclicSubtype { child, parent });
        }
        self.types
            .get_mut(&child)
            .expect("child existence checked above")
            .supertypes
            .insert(parent);
        Ok(())
    }

    pub fn define_literal_type(&mut self, kind: LiteralKind, ty: Type) -> Result<(), EnvironmentError> {
        self.validate_type(&ty, &BTreeSet::new(), None)?;
        if self.literal_types.contains_key(&kind) {
            return Err(EnvironmentError::DuplicateLiteralKind(kind));
        }
        self.literal_types.insert(kind, ty);
        Ok(())
    }

    pub fn define_constant(&mut self, name: impl Into<String>, ty: Type) -> Result<(), EnvironmentError> {
        let name = name.into();
        self.validate_type(&ty, &BTreeSet::new(), None)?;
        if self.constants.contains_key(&name) {
            return Err(EnvironmentError::DuplicateConstant(name));
        }
        self.constants.insert(name, ty);
        Ok(())
    }

    pub fn define_operator(&mut self, name: impl Into<String>, signature: Signature) -> Result<(), EnvironmentError> {
        let name = name.into();
        if self.operators.contains_key(&name) {
            return Err(EnvironmentError::DuplicateOperator(name));
        }
        let mut type_parameters = BTreeSet::new();
        for parameter in &signature.type_parameters {
            if !type_parameters.insert(parameter.clone()) {
                return Err(EnvironmentError::DuplicateTypeParameter {
                    owner: name,
                    parameter: parameter.clone(),
                });
            }
        }
        let mut parameters = BTreeSet::new();
        for parameter in &signature.parameters {
            if !parameters.insert(parameter.name.clone()) {
                return Err(EnvironmentError::DuplicateParameter {
                    operator: name,
                    parameter: parameter.name.clone(),
                });
            }
            self.validate_type(&parameter.ty, &type_parameters, Some(&name))?;
        }
        self.validate_type(&signature.returns, &type_parameters, Some(&name))?;
        self.operators.insert(name, signature);
        Ok(())
    }

    pub fn type_definition(&self, name: &str) -> Option<&TypeDefinition> { self.types.get(name) }
    pub fn literal_type(&self, kind: LiteralKind) -> Option<&Type> { self.literal_types.get(&kind) }
    pub fn constant_type(&self, name: &str) -> Option<&Type> { self.constants.get(name) }
    pub fn operator(&self, name: &str) -> Option<&Signature> { self.operators.get(name) }
    pub fn is_well_formed_type(&self, ty: &Type) -> bool {
        self.validate_type(ty, &BTreeSet::new(), None).is_ok()
    }

    pub fn is_assignable(&self, actual: &Type, expected: &Type) -> bool {
        if actual == expected { return true; }
        match (actual, expected) {
            (Type::Named(actual), Type::Named(expected)) => self.is_named_subtype(actual, expected),
            (Type::Generic { .. }, Type::Generic { .. }) => false,
            (
                Type::Function { parameters: actual_parameters, returns: actual_returns },
                Type::Function { parameters: expected_parameters, returns: expected_returns },
            ) => {
                actual_parameters.len() == expected_parameters.len()
                    && actual_parameters.iter().zip(expected_parameters).all(|(actual, expected)| {
                        self.is_assignable(&expected.ty, &actual.ty)
                    })
                    && self.is_assignable(actual_returns, expected_returns)
            }
            (Type::Record(actual), Type::Record(expected)) => expected.iter().all(|(name, expected)| {
                actual.get(name).is_some_and(|actual| self.is_assignable(actual, expected))
            }),
            _ => false,
        }
    }

    fn is_named_subtype(&self, actual: &str, expected: &str) -> bool {
        if actual == expected { return true; }
        let mut queue = VecDeque::from([actual.to_owned()]);
        let mut seen = BTreeSet::new();
        while let Some(current) = queue.pop_front() {
            if !seen.insert(current.clone()) { continue; }
            let Some(definition) = self.types.get(&current) else { continue; };
            for parent in &definition.supertypes {
                if parent == expected { return true; }
                queue.push_back(parent.clone());
            }
        }
        false
    }

    fn validate_type(
        &self,
        ty: &Type,
        type_parameters: &BTreeSet<String>,
        operator: Option<&str>,
    ) -> Result<(), EnvironmentError> {
        match ty {
            Type::Named(name) => self.require_type_arity(name, 0),
            Type::Generic { name, arguments } => {
                if arguments.is_empty() {
                    return Err(EnvironmentError::EmptyGenericArguments(name.clone()));
                }
                self.require_type_arity(name, arguments.len())?;
                for argument in arguments {
                    self.validate_type(argument, type_parameters, operator)?;
                }
                Ok(())
            }
            Type::Variable(variable) => {
                if type_parameters.contains(variable) { Ok(()) } else {
                    Err(EnvironmentError::UndeclaredTypeVariable {
                        operator: operator.unwrap_or("<non-operator>").to_owned(),
                        variable: variable.clone(),
                    })
                }
            }
            Type::Function { parameters, returns } => {
                for parameter in parameters {
                    self.validate_type(&parameter.ty, type_parameters, operator)?;
                }
                self.validate_type(returns, type_parameters, operator)
            }
            Type::Record(fields) => {
                for ty in fields.values() {
                    self.validate_type(ty, type_parameters, operator)?;
                }
                Ok(())
            }
        }
    }

    fn require_type_arity(&self, name: &str, actual: usize) -> Result<(), EnvironmentError> {
        let definition = self.types.get(name).ok_or_else(|| EnvironmentError::UnknownType(name.to_owned()))?;
        let expected = definition.type_parameters.len();
        if actual == expected { Ok(()) } else {
            Err(EnvironmentError::WrongTypeArity { name: name.to_owned(), expected, actual })
        }
    }
}

impl fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateType(name) => write!(f, "type `{name}` is already defined"),
            Self::UnknownType(name) => write!(f, "unknown type `{name}`"),
            Self::DuplicateLiteralKind(kind) => write!(f, "literal kind `{kind:?}` is already typed"),
            Self::DuplicateConstant(name) => write!(f, "constant `{name}` is already defined"),
            Self::DuplicateOperator(name) => write!(f, "operator `{name}` is already defined"),
            Self::DuplicateParameter { operator, parameter } => write!(f, "operator `{operator}` declares parameter `{parameter}` more than once"),
            Self::DuplicateTypeParameter { owner, parameter } => write!(f, "`{owner}` declares type parameter `{parameter}` more than once"),
            Self::UndeclaredTypeVariable { operator, variable } => write!(f, "operator `{operator}` uses undeclared type variable `${variable}`"),
            Self::WrongTypeArity { name, expected, actual } => write!(f, "type `{name}` expects {expected} type argument(s) but received {actual}"),
            Self::EmptyGenericArguments(name) => write!(f, "generic type `{name}` must have at least one type argument"),
            Self::CyclicSubtype { child, parent } => write!(f, "subtype edge `{child}: {parent}` would create a subtype cycle"),
        }
    }
}

impl std::error::Error for EnvironmentError {}
