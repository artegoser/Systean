use std::fmt;

use crate::literals::SurfaceLiteral;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceExpr {
    Atom(String),
    Context(String),
    Alias(String),
    Name {
        marker: String,
        payload: String,
    },
    Quote(String),
    Literal(SurfaceLiteral),
    Clause(Clause),
    Prefix {
        operator: String,
        operand: Box<SurfaceExpr>,
    },
    Infix {
        operator: String,
        operands: Vec<SurfaceExpr>,
    },
    SpeechAct {
        operator: String,
        content: Box<SurfaceExpr>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clause {
    pub primary: Option<Argument>,
    pub inner_prefixes: Vec<String>,
    pub predicate: String,
    pub rest: Vec<Argument>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Argument {
    Atom(String),
    Context(String),
    Reference(String),
    Alias(String),
    Name {
        marker: String,
        payload: String,
    },
    Quote(String),
    Literal(SurfaceLiteral),
    Information {
        marker: String,
        knower: Option<InformationKnower>,
    },
    Omitted,
    Quantified {
        quantifier: String,
        restriction: String,
    },
    CountedQuantified {
        quantifier: String,
        count: SurfaceLiteral,
        restriction: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InformationKnower {
    Context(String),
    Name { marker: String, payload: String },
}

impl fmt::Display for SurfaceExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atom(surface) | Self::Context(surface) | Self::Alias(surface) => {
                write!(f, "{surface}")
            }
            Self::Name { marker, payload } => write!(f, "{marker}<{payload}>"),
            Self::Quote(payload) => write!(f, "quote({payload:?})"),
            Self::Literal(literal) => write!(f, "literal({})", literal.semantic),
            Self::Clause(clause) => write!(f, "{clause:?}"),
            Self::Prefix { operator, operand } => write!(f, "{operator}({operand})"),
            Self::Infix { operator, operands } => {
                write!(f, "{operator}[")?;
                for (index, operand) in operands.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{operand}")?;
                }
                write!(f, "]")
            }
            Self::SpeechAct { operator, content } => write!(f, "{operator}<{content}>"),
        }
    }
}
