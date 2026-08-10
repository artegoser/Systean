use std::collections::BTreeMap;

use super::Term;

/// Convert a semantic term to the representation-level canonical form.
///
/// This normalization is deliberately conservative. It performs only
/// equivalences that are guaranteed by the core representation itself:
///
/// - bound variables are alpha-renamed deterministically;
/// - call roles and record fields are already stored in `BTreeMap`s and are
///   therefore emitted in canonical key order.
///
/// It does not assume algebraic laws such as commutativity, associativity, or
/// logical equivalence. Those properties belong to explicit semantic rules,
/// not to the generic IR.
pub fn canonicalize(term: &Term) -> Term {
    let mut normalizer = Normalizer::default();
    normalizer.term(term)
}

#[derive(Default)]
struct Normalizer {
    next_variable: usize,
    bindings: BTreeMap<String, Vec<String>>,
}

impl Normalizer {
    fn term(&mut self, term: &Term) -> Term {
        match term {
            Term::Const(name) => Term::Const(name.clone()),
            Term::Var(name) => Term::Var(self.resolve_variable(name)),
            Term::Literal(literal) => Term::Literal(literal.clone()),
            Term::Call { function, arguments } => Term::Call {
                function: function.clone(),
                arguments: arguments
                    .iter()
                    .map(|(role, value)| (role.clone(), self.term(value)))
                    .collect(),
            },
            Term::Bind { variable, variable_type, body } => {
                let canonical = format!("v{}", self.next_variable);
                self.next_variable += 1;
                self.bindings
                    .entry(variable.clone())
                    .or_default()
                    .push(canonical.clone());

                let body = self.term(body);

                let stack = self
                    .bindings
                    .get_mut(variable)
                    .expect("binding was inserted before normalizing its body");
                stack.pop();
                if stack.is_empty() {
                    self.bindings.remove(variable);
                }

                Term::Bind {
                    variable: canonical,
                    variable_type: variable_type.clone(),
                    body: Box::new(body),
                }
            }
            Term::Record(fields) => Term::Record(
                fields
                    .iter()
                    .map(|(name, value)| (name.clone(), self.term(value)))
                    .collect(),
            ),
            Term::Field { record, field } => Term::Field {
                record: Box::new(self.term(record)),
                field: field.clone(),
            },
        }
    }

    fn resolve_variable(&self, name: &str) -> String {
        self.bindings
            .get(name)
            .and_then(|stack| stack.last())
            .cloned()
            .unwrap_or_else(|| name.to_owned())
    }
}
