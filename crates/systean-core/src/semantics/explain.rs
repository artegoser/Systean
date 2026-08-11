use std::collections::BTreeMap;
use std::fmt::Write;
use super::{CheckError, Checker, Environment, Literal, LiteralKind, Origin, Term, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Explanation { pub label: String, pub ty: Type, pub origin: Option<Origin>, pub children: Vec<ExplanationEdge> }
#[derive(Clone, Debug, PartialEq)]
pub struct ExplanationEdge { pub relation: String, pub node: Explanation }
pub struct Explainer<'env> { environment: &'env Environment }

impl<'env> Explainer<'env> {
    pub fn new(environment: &'env Environment) -> Self { Self { environment } }
    pub fn explain(&self, term: &Term) -> Result<Explanation, CheckError> { self.explain_with_scope(term, &mut BTreeMap::new()) }
    fn explain_with_scope(&self, term: &Term, variables: &mut BTreeMap<String, Type>) -> Result<Explanation, CheckError> {
        let checker = Checker::new(self.environment);
        let ty = checker.infer_with_scope(term, variables)?;
        match term {
            Term::Const(name) => Ok(Explanation { label: format!("const {name}"), ty, origin: self.environment.constant_origin(name).cloned(), children: vec![] }),
            Term::Var(name) => Ok(Explanation { label: format!("var {name}"), ty, origin: None, children: vec![] }),
            Term::Literal(literal) => Ok(Explanation {
                label: format!("literal {literal}"),
                ty,
                origin: match literal {
                    Literal::Structured(_) => None,
                    _ => self.environment.literal_origin(literal_kind(literal)).cloned(),
                },
                children: vec![],
            }),
            Term::Call { function, arguments } => {
                let mut children=Vec::new();
                for (role, argument) in arguments { children.push(ExplanationEdge { relation: role.clone(), node: self.explain_with_scope(argument, variables)? }); }
                Ok(Explanation { label: format!("call {function}"), ty, origin: self.environment.operator_origin(function).cloned(), children })
            }
            Term::Bind { variable, variable_type, body } => {
                let previous=variables.insert(variable.clone(), variable_type.clone());
                let body=self.explain_with_scope(body, variables);
                match previous { Some(previous)=>{variables.insert(variable.clone(), previous);}, None=>{variables.remove(variable);} }
                Ok(Explanation { label: format!("bind {variable}: {variable_type}"), ty, origin: type_origin(self.environment, variable_type), children: vec![ExplanationEdge { relation:"body".into(), node:body? }] })
            }
            Term::Record(fields) => {
                let mut children=Vec::new();
                for (field,value) in fields { children.push(ExplanationEdge { relation:field.clone(), node:self.explain_with_scope(value, variables)? }); }
                Ok(Explanation { label:"record".into(), ty, origin:None, children })
            }
            Term::Field { record, field } => Ok(Explanation { label:format!("field {field}"), ty, origin:None, children:vec![ExplanationEdge { relation:"record".into(), node:self.explain_with_scope(record, variables)? }] }),
        }
    }
}

impl Explanation {
    pub fn render(&self) -> String {
        let mut output=String::new(); self.render_into(&mut output,0,None).expect("String write"); output
    }
    fn render_into(&self, output:&mut String, depth:usize, relation:Option<&str>) -> std::fmt::Result {
        for _ in 0..depth { output.push_str("  "); }
        if let Some(relation)=relation { write!(output,"{relation}: ")?; }
        write!(output,"{} : {}",self.label,self.ty)?;
        if let Some(origin)=&self.origin { write!(output," @ {origin}")?; }
        output.push('\n');
        for child in &self.children { child.node.render_into(output, depth+1, Some(&child.relation))?; }
        Ok(())
    }
}
fn literal_kind(literal:&Literal)->LiteralKind { match literal {
    Literal::Integer(_)=>LiteralKind::Integer,
    Literal::Boolean(_)=>LiteralKind::Boolean,
    Literal::String(_)=>LiteralKind::String,
    Literal::Structured(_) => LiteralKind::String,
} }
fn type_origin(environment:&Environment, ty:&Type)->Option<Origin> { match ty { Type::Named(name)|Type::Generic{name,..}=>environment.type_origin(name).cloned(), _=>None } }
