use std::collections::BTreeMap;

use crate::semantics::{Literal, LiteralKind, Parameter, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Specification {
    pub declarations: Vec<Declaration>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Declaration {
    Type {
        name: String,
    },
    Subtype {
        child: String,
        parent: String,
    },
    Literal {
        kind: LiteralKind,
        ty: Type,
    },
    Const {
        name: String,
        ty: Type,
    },
    Operator {
        name: String,
        type_parameters: Vec<String>,
        parameters: Vec<Parameter>,
        returns: Type,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParsedTerm {
    Name(String),
    Literal(Literal),
    Call {
        function: String,
        arguments: BTreeMap<String, ParsedTerm>,
    },
    Bind {
        variable: String,
        variable_type: Type,
        body: Box<ParsedTerm>,
    },
    Record(BTreeMap<String, ParsedTerm>),
    Field {
        record: Box<ParsedTerm>,
        field: String,
    },
}
