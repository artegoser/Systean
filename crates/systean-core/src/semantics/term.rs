use std::collections::BTreeMap;
use std::fmt;

use super::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Term {
    Const(String),
    Var(String),
    Literal(Literal),
    Call {
        function: String,
        arguments: BTreeMap<String, Term>,
    },
    Bind {
        variable: String,
        variable_type: Type,
        body: Box<Term>,
    },
    Record(BTreeMap<String, Term>),
    Field {
        record: Box<Term>,
        field: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Literal {
    Integer(i64),
    Boolean(bool),
    String(String),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{value}"),
            Self::Boolean(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "{value:?}"),
        }
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Const(name) | Self::Var(name) => f.write_str(name),
            Self::Literal(literal) => literal.fmt(f),
            Self::Call { function, arguments } => {
                write!(f, "{function}(")?;
                for (index, (role, value)) in arguments.iter().enumerate() {
                    if index > 0 { write!(f, ", ")?; }
                    write!(f, "{role} = {value}")?;
                }
                write!(f, ")")
            }
            Self::Bind { variable, variable_type, body } => {
                write!(f, "bind {variable}: {variable_type} => {body}")
            }
            Self::Record(fields) => {
                write!(f, "{{")?;
                for (index, (name, value)) in fields.iter().enumerate() {
                    if index > 0 { write!(f, ", ")?; }
                    write!(f, "{name} = {value}")?;
                }
                write!(f, "}}")
            }
            Self::Field { record, field } => write!(f, "({record}).{field}"),
        }
    }
}
