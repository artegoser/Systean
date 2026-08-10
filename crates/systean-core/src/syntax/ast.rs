use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceExpr {
    Atom(String),
    Context(String),
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
    Omitted,
    Quantified {
        quantifier: String,
        restriction: String,
    },
}

impl fmt::Display for SurfaceExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atom(surface) | Self::Context(surface) => write!(f, "{surface}"),
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
