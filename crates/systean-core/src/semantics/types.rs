use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Named(String),
    Generic {
        name: String,
        arguments: Vec<Type>,
    },
    Variable(String),
    Function {
        parameters: Vec<FunctionParameter>,
        returns: Box<Type>,
    },
    Record(BTreeMap<String, Type>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FunctionParameter {
    pub name: Option<String>,
    pub ty: Type,
}

impl Type {
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }

    pub fn variable(name: impl Into<String>) -> Self {
        Self::Variable(name.into())
    }

    pub fn substitute(&self, bindings: &BTreeMap<String, Type>) -> Self {
        match self {
            Self::Named(_) => self.clone(),
            Self::Generic { name, arguments } => Self::Generic {
                name: name.clone(),
                arguments: arguments
                    .iter()
                    .map(|argument| argument.substitute(bindings))
                    .collect(),
            },
            Self::Variable(name) => bindings
                .get(name)
                .cloned()
                .unwrap_or_else(|| self.clone()),
            Self::Function {
                parameters,
                returns,
            } => Self::Function {
                parameters: parameters
                    .iter()
                    .map(|parameter| FunctionParameter {
                        name: parameter.name.clone(),
                        ty: parameter.ty.substitute(bindings),
                    })
                    .collect(),
                returns: Box::new(returns.substitute(bindings)),
            },
            Self::Record(fields) => Self::Record(
                fields
                    .iter()
                    .map(|(name, ty)| (name.clone(), ty.substitute(bindings)))
                    .collect(),
            ),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(name) => write!(f, "{name}"),
            Self::Generic { name, arguments } => {
                write!(f, "{name}<")?;
                for (index, argument) in arguments.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{argument}")?;
                }
                write!(f, ">")
            }
            Self::Variable(name) => write!(f, "${name}"),
            Self::Function {
                parameters,
                returns,
            } => {
                write!(f, "fn(")?;
                for (index, parameter) in parameters.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    if let Some(name) = &parameter.name {
                        write!(f, "{name}: ")?;
                    }
                    write!(f, "{}", parameter.ty)?;
                }
                write!(f, ") -> {returns}")
            }
            Self::Record(fields) => {
                write!(f, "{{")?;
                for (index, (name, ty)) in fields.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{name}: {ty}")?;
                }
                write!(f, "}}")
            }
        }
    }
}
