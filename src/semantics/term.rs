use std::collections::BTreeMap;

use super::Type;

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Integer(i64),
    Boolean(bool),
    String(String),
}
