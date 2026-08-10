use super::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    pub type_parameters: Vec<String>,
    pub parameters: Vec<Parameter>,
    pub returns: Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
}
