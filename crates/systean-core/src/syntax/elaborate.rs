use std::collections::BTreeMap;
use std::fmt;

use crate::semantics::{Checker, Environment, Literal, StructuredLiteral, Term, Type};
use crate::spec::parse_type;

use super::{Argument, Clause, InformationKnower, LexemeConfig, SurfaceExpr, SurfaceLexicon};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferenceSource {
    Explicit { surface: String },
    Omitted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceSlot {
    pub placeholder: String,
    pub role: String,
    pub expected_type: Type,
    pub source: ReferenceSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextSlot {
    pub placeholder: String,
    pub surface: String,
    pub key: String,
    pub declared_type: Type,
    pub expected_type: Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AliasSlot {
    pub placeholder: String,
    pub surface: String,
    pub alias: String,
    pub declared_type: Type,
    pub expected_type: Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedSurfaceAst {
    pub surface: SurfaceExpr,
    pub template: Term,
    pub inferred_type: Type,
    pub references: Vec<ReferenceSlot>,
    pub contexts: Vec<ContextSlot>,
    pub aliases: Vec<AliasSlot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceElaborationError {
    MissingLexeme(String),
    WrongLexemeKind {
        surface: String,
        expected: &'static str,
    },
    UnknownSemanticOperator(String),
    MissingSemanticRole {
        operator: String,
        role: String,
    },
    InvalidType {
        source: String,
        message: String,
    },
    UnconstrainedReferenceType {
        role: String,
        ty: Type,
    },
    InvalidSemanticTerm(String),
}

pub fn elaborate_surface(
    expression: &SurfaceExpr,
    lexicon: &SurfaceLexicon,
    environment: &Environment,
) -> Result<TypedSurfaceAst, SurfaceElaborationError> {
    let mut elaborator = Elaborator {
        lexicon,
        environment,
        placeholder_index: 0,
        quantifier_index: 0,
        references: Vec::new(),
        contexts: Vec::new(),
        aliases: Vec::new(),
    };
    let template = elaborator.lower_expr(expression)?;
    let mut variables = BTreeMap::new();
    for slot in &elaborator.references {
        variables.insert(slot.placeholder.clone(), slot.expected_type.clone());
    }
    for slot in &elaborator.contexts {
        variables.insert(slot.placeholder.clone(), slot.expected_type.clone());
    }
    for slot in &elaborator.aliases {
        variables.insert(slot.placeholder.clone(), slot.expected_type.clone());
    }
    let inferred_type = Checker::new(environment)
        .infer_with_scope(&template, &mut variables)
        .map_err(|error| SurfaceElaborationError::InvalidSemanticTerm(error.to_string()))?;
    Ok(TypedSurfaceAst {
        surface: expression.clone(),
        template,
        inferred_type,
        references: elaborator.references,
        contexts: elaborator.contexts,
        aliases: elaborator.aliases,
    })
}

struct Elaborator<'a> {
    lexicon: &'a SurfaceLexicon,
    environment: &'a Environment,
    placeholder_index: usize,
    quantifier_index: usize,
    references: Vec<ReferenceSlot>,
    contexts: Vec<ContextSlot>,
    aliases: Vec<AliasSlot>,
}

#[derive(Clone)]
struct QuantifierIntroduction {
    quantifier_surface: String,
    restriction_surface: String,
    count: Option<Term>,
    variable: String,
    variable_type: Type,
}

#[derive(Clone)]
enum ScopeIntroduction {
    Prefix(String),
    Quantifier(QuantifierIntroduction),
}

impl Elaborator<'_> {
    fn lower_expr(&mut self, expression: &SurfaceExpr) -> Result<Term, SurfaceElaborationError> {
        match expression {
            SurfaceExpr::Atom(surface) => self.lower_atom(surface),
            SurfaceExpr::Context(surface) => self.lower_standalone_context(surface),
            SurfaceExpr::Alias(surface) => self.lower_standalone_alias(surface),
            SurfaceExpr::Name { marker, payload } => self.lower_name(marker, payload),
            SurfaceExpr::Quote(payload) => Ok(self.lower_quote(payload)),
            SurfaceExpr::Literal(literal) => Ok(Term::Literal(Literal::Structured(literal.semantic.clone()))),
            SurfaceExpr::Clause(clause) => self.lower_clause(clause),
            SurfaceExpr::Prefix { operator, operand } => {
                let operand = self.lower_expr(operand)?;
                self.call_prefix(operator, operand)
            }
            SurfaceExpr::SpeechAct { operator, content } => {
                let content = self.lower_expr(content)?;
                self.call_speech_act(operator, content)
            }
            SurfaceExpr::Infix { operator, operands } => {
                let mut operands = operands.iter();
                let first = operands.next().ok_or_else(|| {
                    SurfaceElaborationError::InvalidSemanticTerm(
                        "infix expression has no operands".into(),
                    )
                })?;
                let mut term = self.lower_expr(first)?;
                for operand in operands {
                    let right = self.lower_expr(operand)?;
                    term = self.call_infix(operator, term, right)?;
                }
                Ok(term)
            }
        }
    }

    fn lower_atom(&self, surface: &str) -> Result<Term, SurfaceElaborationError> {
        let LexemeConfig::Atom { semantic } = self.lexeme(surface)? else {
            return Err(SurfaceElaborationError::WrongLexemeKind {
                surface: surface.to_owned(),
                expected: "atom",
            });
        };
        Ok(Term::Const(semantic.clone()))
    }

    fn lower_name(
        &self,
        marker: &str,
        payload: &str,
    ) -> Result<Term, SurfaceElaborationError> {
        let LexemeConfig::Name { semantic, role } = self.lexeme(marker)? else {
            return Err(SurfaceElaborationError::WrongLexemeKind {
                surface: marker.to_owned(),
                expected: "name",
            });
        };
        let mut arguments = BTreeMap::new();
        arguments.insert(
            role.clone(),
            Term::Literal(Literal::String(payload.to_owned())),
        );
        Ok(Term::Call {
            function: semantic.clone(),
            arguments,
        })
    }

    fn lower_quote(&self, payload: &str) -> Term {
        Term::Literal(Literal::String(payload.to_owned()))
    }

    fn lower_standalone_context(
        &mut self,
        surface: &str,
    ) -> Result<Term, SurfaceElaborationError> {
        let LexemeConfig::Context { key, ty } = self.lexeme(surface)?.clone() else {
            return Err(SurfaceElaborationError::WrongLexemeKind {
                surface: surface.to_owned(),
                expected: "context",
            });
        };
        let declared_type = self.parse_declared_type(&ty)?;
        let placeholder = self.next_placeholder("context");
        self.contexts.push(ContextSlot {
            placeholder: placeholder.clone(),
            surface: surface.to_owned(),
            key,
            declared_type: declared_type.clone(),
            expected_type: declared_type,
        });
        Ok(Term::Var(placeholder))
    }

    fn lower_standalone_alias(
        &mut self,
        surface: &str,
    ) -> Result<Term, SurfaceElaborationError> {
        let LexemeConfig::Alias { name, ty } = self.lexeme(surface)?.clone() else {
            return Err(SurfaceElaborationError::WrongLexemeKind {
                surface: surface.to_owned(),
                expected: "alias",
            });
        };
        let declared_type = self.parse_declared_type(&ty)?;
        let placeholder = self.next_placeholder("alias");
        self.aliases.push(AliasSlot {
            placeholder: placeholder.clone(),
            surface: surface.to_owned(),
            alias: name,
            declared_type: declared_type.clone(),
            expected_type: declared_type,
        });
        Ok(Term::Var(placeholder))
    }

    fn lower_clause(&mut self, clause: &Clause) -> Result<Term, SurfaceElaborationError> {
        let predicate = self.lexeme(&clause.predicate)?.clone();
        let (semantic, primary_role, rest_roles) = match predicate {
            LexemeConfig::Predicate {
                semantic,
                primary_role,
                rest_roles,
            } => (semantic, primary_role, rest_roles),
            LexemeConfig::Class { semantic, role } => {
                (semantic, Some(role), Vec::new())
            }
            _ => {
                return Err(SurfaceElaborationError::WrongLexemeKind {
                    surface: clause.predicate.clone(),
                    expected: "predicate or class",
                });
            }
        };
        let signature = self
            .environment
            .operator(&semantic)
            .cloned()
            .ok_or_else(|| SurfaceElaborationError::UnknownSemanticOperator(semantic.clone()))?;

        let reference_start = self.references.len();
        let context_start = self.contexts.len();
        let alias_start = self.aliases.len();
        let mut type_bindings = BTreeMap::new();
        let mut arguments = BTreeMap::new();
        let mut introductions = Vec::new();

        if let Some(role) = primary_role.as_ref() {
            let argument = clause.primary.as_ref().ok_or_else(|| {
                SurfaceElaborationError::InvalidSemanticTerm(format!(
                    "predicate `{}` is missing its primary argument slot",
                    clause.predicate
                ))
            })?;
            let expected = parameter_type(&signature, &semantic, role)?;
            let (term, intro) = self.lower_argument(
                argument,
                role,
                expected,
                &mut type_bindings,
            )?;
            arguments.insert(role.clone(), term);
            if let Some(intro) = intro {
                introductions.push(ScopeIntroduction::Quantifier(intro));
            }
        } else if clause.primary.is_some() {
            return Err(SurfaceElaborationError::InvalidSemanticTerm(format!(
                "predicate `{}` has no primary semantic role",
                clause.predicate
            )));
        }

        for prefix in &clause.inner_prefixes {
            introductions.push(ScopeIntroduction::Prefix(prefix.clone()));
        }

        if rest_roles.len() != clause.rest.len() {
            return Err(SurfaceElaborationError::InvalidSemanticTerm(format!(
                "predicate `{}` expects {} rest argument slot(s), surface clause has {}",
                clause.predicate,
                rest_roles.len(),
                clause.rest.len()
            )));
        }

        for (role, argument) in rest_roles.iter().zip(&clause.rest) {
            let expected = parameter_type(&signature, &semantic, role)?;
            let (term, intro) = self.lower_argument(
                argument,
                role,
                expected,
                &mut type_bindings,
            )?;
            arguments.insert(role.clone(), term);
            if let Some(intro) = intro {
                introductions.push(ScopeIntroduction::Quantifier(intro));
            }
        }

        self.finalize_reference_types(reference_start, &type_bindings)?;
        self.finalize_context_types(context_start, &type_bindings)?;
        self.finalize_alias_types(alias_start, &type_bindings)?;

        let mut term = Term::Call {
            function: semantic,
            arguments,
        };
        for introduction in introductions.into_iter().rev() {
            term = match introduction {
                ScopeIntroduction::Prefix(surface) => self.call_prefix(&surface, term)?,
                ScopeIntroduction::Quantifier(quantifier) => {
                    self.wrap_quantifier(quantifier, term)?
                }
            };
        }
        Ok(term)
    }

    fn lower_argument(
        &mut self,
        argument: &Argument,
        role: &str,
        expected: &Type,
        type_bindings: &mut BTreeMap<String, Type>,
    ) -> Result<(Term, Option<QuantifierIntroduction>), SurfaceElaborationError> {
        match argument {
            Argument::Atom(surface) => {
                let LexemeConfig::Atom { semantic } = self.lexeme(surface)? else {
                    return Err(SurfaceElaborationError::WrongLexemeKind {
                        surface: surface.clone(),
                        expected: "atom",
                    });
                };
                let actual = self
                    .environment
                    .constant_type(semantic)
                    .cloned()
                    .ok_or_else(|| {
                        SurfaceElaborationError::InvalidSemanticTerm(format!(
                            "unknown semantic constant `{semantic}`"
                        ))
                    })?;
                self.match_expected(expected, &actual, type_bindings, role)?;
                Ok((Term::Const(semantic.clone()), None))
            }
            Argument::Context(surface) => {
                let LexemeConfig::Context { key, ty } = self.lexeme(surface)?.clone() else {
                    return Err(SurfaceElaborationError::WrongLexemeKind {
                        surface: surface.clone(),
                        expected: "context",
                    });
                };
                let declared_type = self.parse_declared_type(&ty)?;
                self.match_expected(expected, &declared_type, type_bindings, role)?;
                let placeholder = self.next_placeholder("context");
                self.contexts.push(ContextSlot {
                    placeholder: placeholder.clone(),
                    surface: surface.clone(),
                    key,
                    declared_type,
                    expected_type: expected.clone(),
                });
                Ok((Term::Var(placeholder), None))
            }
            Argument::Alias(surface) => {
                let LexemeConfig::Alias { name, ty } = self.lexeme(surface)?.clone() else {
                    return Err(SurfaceElaborationError::WrongLexemeKind {
                        surface: surface.clone(),
                        expected: "alias",
                    });
                };
                let declared_type = self.parse_declared_type(&ty)?;
                self.match_expected(expected, &declared_type, type_bindings, role)?;
                let placeholder = self.next_placeholder("alias");
                self.aliases.push(AliasSlot {
                    placeholder: placeholder.clone(),
                    surface: surface.clone(),
                    alias: name,
                    declared_type,
                    expected_type: expected.clone(),
                });
                Ok((Term::Var(placeholder), None))
            }

            Argument::Name { marker, payload } => {
                let term = self.lower_name(marker, payload)?;
                let actual = Checker::new(self.environment)
                    .infer(&term)
                    .map_err(|error| {
                        SurfaceElaborationError::InvalidSemanticTerm(error.to_string())
                    })?;
                self.match_expected(expected, &actual, type_bindings, role)?;
                Ok((term, None))
            }
            Argument::Quote(payload) => {
                let term = self.lower_quote(payload);
                let actual = Checker::new(self.environment)
                    .infer(&term)
                    .map_err(|error| {
                        SurfaceElaborationError::InvalidSemanticTerm(error.to_string())
                    })?;
                self.match_expected(expected, &actual, type_bindings, role)?;
                Ok((term, None))
            }
            Argument::Literal(literal) => {
                let term = Term::Literal(Literal::Structured(literal.semantic.clone()));
                let actual = Checker::new(self.environment)
                    .infer(&term)
                    .map_err(|error| SurfaceElaborationError::InvalidSemanticTerm(error.to_string()))?;
                self.match_expected(expected, &actual, type_bindings, role)?;
                Ok((term, None))
            }
            Argument::Information { marker, knower } => {
                let LexemeConfig::Information { status, knower_type } = self.lexeme(marker)?.clone() else {
                    return Err(SurfaceElaborationError::WrongLexemeKind {
                        surface: marker.clone(),
                        expected: "information marker",
                    });
                };
                if knower_type.is_some() != knower.is_some() {
                    return Err(SurfaceElaborationError::InvalidSemanticTerm(format!(
                        "information marker `{marker}` knower shape does not match its lexical declaration"
                    )));
                }
                let concrete = expected.substitute(type_bindings);
                if contains_type_variable(&concrete) {
                    return Err(SurfaceElaborationError::UnconstrainedReferenceType {
                        role: format!("information {status} for {role}"),
                        ty: concrete,
                    });
                }
                let declared_knower_type = knower_type
                    .as_deref()
                    .map(|source| self.parse_declared_type(source))
                    .transpose()?;
                let canonical = match knower {
                    None => status.clone(),
                    Some(InformationKnower::Context(surface)) => {
                        let LexemeConfig::Context { key, ty } = self.lexeme(surface)? else {
                            return Err(SurfaceElaborationError::WrongLexemeKind {
                                surface: surface.clone(),
                                expected: "context knower",
                            });
                        };
                        if let Some(expected_knower) = &declared_knower_type {
                            let actual_knower = self.parse_declared_type(ty)?;
                            let mut knower_bindings = BTreeMap::new();
                            self.match_expected(
                                expected_knower,
                                &actual_knower,
                                &mut knower_bindings,
                                "information knower",
                            )?;
                        }
                        format!("{status}:context:{key}")
                    }
                    Some(InformationKnower::Name { marker: name_marker, payload }) => {
                        let LexemeConfig::Name { semantic, role: name_role } = self.lexeme(name_marker)? else {
                            return Err(SurfaceElaborationError::WrongLexemeKind {
                                surface: name_marker.clone(),
                                expected: "proper-name knower",
                            });
                        };
                        if let Some(expected_knower) = &declared_knower_type {
                            let name_term = self.lower_name(name_marker, payload)?;
                            let actual_knower = Checker::new(self.environment)
                                .infer(&name_term)
                                .map_err(|error| SurfaceElaborationError::InvalidSemanticTerm(error.to_string()))?;
                            let mut knower_bindings = BTreeMap::new();
                            self.match_expected(
                                expected_knower,
                                &actual_knower,
                                &mut knower_bindings,
                                name_role,
                            )?;
                        }
                        format!("{status}:{semantic}:{payload}")
                    }
                };
                Ok((
                    Term::Literal(Literal::Structured(StructuredLiteral::new(
                        "information",
                        canonical,
                        concrete,
                    ))),
                    None,
                ))
            }
            Argument::CountedQuantified { quantifier, count, restriction } => {
                let quantifier_config = self.lexeme(quantifier)?.clone();
                let LexemeConfig::CountedQuantifier { variable_type, count_role, semantic, .. } = quantifier_config else {
                    return Err(SurfaceElaborationError::WrongLexemeKind {
                        surface: quantifier.clone(),
                        expected: "counted quantifier",
                    });
                };
                let restriction_config = self.lexeme(restriction)?.clone();
                let LexemeConfig::Class { .. } = restriction_config else {
                    return Err(SurfaceElaborationError::WrongLexemeKind {
                        surface: restriction.clone(),
                        expected: "class",
                    });
                };
                let variable_type = self.parse_declared_type(&variable_type)?;
                self.match_expected(expected, &variable_type, type_bindings, role)?;
                let signature = self.environment.operator(&semantic).ok_or_else(|| {
                    SurfaceElaborationError::UnknownSemanticOperator(semantic.clone())
                })?;
                let count_expected = parameter_type(signature, &semantic, &count_role)?;
                let count_term = Term::Literal(Literal::Structured(count.semantic.clone()));
                let count_actual = Checker::new(self.environment).infer(&count_term).map_err(|error| {
                    SurfaceElaborationError::InvalidSemanticTerm(error.to_string())
                })?;
                let mut count_bindings = BTreeMap::new();
                self.match_expected(count_expected, &count_actual, &mut count_bindings, &count_role)?;
                let variable = format!("surface_q{}", self.quantifier_index);
                self.quantifier_index += 1;
                Ok((
                    Term::Var(variable.clone()),
                    Some(QuantifierIntroduction {
                        quantifier_surface: quantifier.clone(),
                        restriction_surface: restriction.clone(),
                        count: Some(count_term),
                        variable,
                        variable_type,
                    }),
                ))
            }
            Argument::Reference(surface) => {
                let LexemeConfig::Reference = self.lexeme(surface)? else {
                    return Err(SurfaceElaborationError::WrongLexemeKind {
                        surface: surface.clone(),
                        expected: "reference",
                    });
                };
                let placeholder = self.next_placeholder("reference");
                self.references.push(ReferenceSlot {
                    placeholder: placeholder.clone(),
                    role: role.to_owned(),
                    expected_type: expected.clone(),
                    source: ReferenceSource::Explicit {
                        surface: surface.clone(),
                    },
                });
                Ok((Term::Var(placeholder), None))
            }
            Argument::Omitted => {
                let placeholder = self.next_placeholder("reference");
                self.references.push(ReferenceSlot {
                    placeholder: placeholder.clone(),
                    role: role.to_owned(),
                    expected_type: expected.clone(),
                    source: ReferenceSource::Omitted,
                });
                Ok((Term::Var(placeholder), None))
            }
            Argument::Quantified {
                quantifier,
                restriction,
            } => {
                let quantifier_config = self.lexeme(quantifier)?.clone();
                let LexemeConfig::Quantifier { variable_type, .. } = quantifier_config else {
                    return Err(SurfaceElaborationError::WrongLexemeKind {
                        surface: quantifier.clone(),
                        expected: "quantifier",
                    });
                };
                let restriction_config = self.lexeme(restriction)?.clone();
                let LexemeConfig::Class { .. } = restriction_config else {
                    return Err(SurfaceElaborationError::WrongLexemeKind {
                        surface: restriction.clone(),
                        expected: "class",
                    });
                };
                let variable_type = self.parse_declared_type(&variable_type)?;
                self.match_expected(expected, &variable_type, type_bindings, role)?;
                let variable = format!("surface_q{}", self.quantifier_index);
                self.quantifier_index += 1;
                Ok((
                    Term::Var(variable.clone()),
                    Some(QuantifierIntroduction {
                        quantifier_surface: quantifier.clone(),
                        restriction_surface: restriction.clone(),
                        count: None,
                        variable,
                        variable_type,
                    }),
                ))
            }
        }
    }

    fn match_expected(
        &self,
        expected: &Type,
        actual: &Type,
        bindings: &mut BTreeMap<String, Type>,
        role: &str,
    ) -> Result<(), SurfaceElaborationError> {
        Checker::new(self.environment)
            .match_expected_type(
                expected,
                actual,
                bindings,
                &format!("surface role `{role}`"),
            )
            .map_err(|error| SurfaceElaborationError::InvalidSemanticTerm(error.to_string()))
    }

    fn finalize_reference_types(
        &mut self,
        start: usize,
        bindings: &BTreeMap<String, Type>,
    ) -> Result<(), SurfaceElaborationError> {
        for slot in &mut self.references[start..] {
            let resolved = slot.expected_type.substitute(bindings);
            if contains_type_variable(&resolved) {
                return Err(SurfaceElaborationError::UnconstrainedReferenceType {
                    role: slot.role.clone(),
                    ty: resolved,
                });
            }
            slot.expected_type = resolved;
        }
        Ok(())
    }

    fn finalize_context_types(
        &mut self,
        start: usize,
        bindings: &BTreeMap<String, Type>,
    ) -> Result<(), SurfaceElaborationError> {
        for slot in &mut self.contexts[start..] {
            let resolved = slot.expected_type.substitute(bindings);
            if contains_type_variable(&resolved) {
                return Err(SurfaceElaborationError::UnconstrainedReferenceType {
                    role: format!("context {}", slot.key),
                    ty: resolved,
                });
            }
            slot.expected_type = resolved;
        }
        Ok(())
    }

    fn finalize_alias_types(
        &mut self,
        start: usize,
        bindings: &BTreeMap<String, Type>,
    ) -> Result<(), SurfaceElaborationError> {
        for slot in &mut self.aliases[start..] {
            let resolved = slot.expected_type.substitute(bindings);
            if contains_type_variable(&resolved) {
                return Err(SurfaceElaborationError::UnconstrainedReferenceType {
                    role: format!("alias {}", slot.alias),
                    ty: resolved,
                });
            }
            slot.expected_type = resolved;
        }
        Ok(())
    }

    fn parse_declared_type(&self, source: &str) -> Result<Type, SurfaceElaborationError> {
        let ty = parse_type(source).map_err(|errors| SurfaceElaborationError::InvalidType {
            source: source.to_owned(),
            message: errors
                .into_iter()
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("; "),
        })?;
        if !self.environment.is_well_formed_type(&ty) {
            return Err(SurfaceElaborationError::InvalidType {
                source: source.to_owned(),
                message: "type is not declared in the semantic environment".into(),
            });
        }
        Ok(ty)
    }

    fn wrap_quantifier(
        &self,
        intro: QuantifierIntroduction,
        body: Term,
    ) -> Result<Term, SurfaceElaborationError> {
        let (semantic, binder_role, count_role, restriction_operator, restriction_role, body_role) =
            match self.lexeme(&intro.quantifier_surface)? {
                LexemeConfig::Quantifier {
                    semantic,
                    binder_role,
                    restriction_operator,
                    restriction_role,
                    body_role,
                    ..
                } => (
                    semantic.clone(), binder_role.clone(), None,
                    restriction_operator.clone(), restriction_role.clone(), body_role.clone(),
                ),
                LexemeConfig::CountedQuantifier {
                    semantic,
                    binder_role,
                    count_role,
                    restriction_operator,
                    restriction_role,
                    body_role,
                    ..
                } => (
                    semantic.clone(), binder_role.clone(), Some(count_role.clone()),
                    restriction_operator.clone(), restriction_role.clone(), body_role.clone(),
                ),
                _ => {
                    return Err(SurfaceElaborationError::WrongLexemeKind {
                        surface: intro.quantifier_surface,
                        expected: "quantifier",
                    });
                }
            };
        let LexemeConfig::Class {
            semantic: restriction_semantic,
            role: class_role,
        } = self.lexeme(&intro.restriction_surface)?
        else {
            return Err(SurfaceElaborationError::WrongLexemeKind {
                surface: intro.restriction_surface,
                expected: "class",
            });
        };

        let restriction = Term::Call {
            function: restriction_semantic.clone(),
            arguments: BTreeMap::from([(
                class_role.clone(),
                Term::Var(intro.variable.clone()),
            )]),
        };
        let combined = Term::Call {
            function: restriction_operator,
            arguments: BTreeMap::from([
                (restriction_role, restriction),
                (body_role, body),
            ]),
        };
        let binder = Term::Bind {
            variable: intro.variable,
            variable_type: intro.variable_type,
            body: Box::new(combined),
        };
        let mut arguments = BTreeMap::from([(binder_role, binder)]);
        match (count_role, intro.count) {
            (Some(role), Some(count)) => {
                arguments.insert(role, count);
            }
            (None, None) => {}
            _ => {
                return Err(SurfaceElaborationError::InvalidSemanticTerm(
                    "quantifier count shape does not match lexical declaration".into(),
                ));
            }
        }
        Ok(Term::Call {
            function: semantic,
            arguments,
        })
    }

    fn call_prefix(
        &self,
        surface: &str,
        operand: Term,
    ) -> Result<Term, SurfaceElaborationError> {
        let LexemeConfig::Prefix { semantic, role } = self.lexeme(surface)? else {
            return Err(SurfaceElaborationError::WrongLexemeKind {
                surface: surface.to_owned(),
                expected: "prefix",
            });
        };
        Ok(Term::Call {
            function: semantic.clone(),
            arguments: BTreeMap::from([(role.clone(), operand)]),
        })
    }

    fn call_infix(
        &self,
        surface: &str,
        left: Term,
        right: Term,
    ) -> Result<Term, SurfaceElaborationError> {
        let LexemeConfig::Infix {
            semantic,
            left_role,
            right_role,
        } = self.lexeme(surface)?
        else {
            return Err(SurfaceElaborationError::WrongLexemeKind {
                surface: surface.to_owned(),
                expected: "infix",
            });
        };
        Ok(Term::Call {
            function: semantic.clone(),
            arguments: BTreeMap::from([
                (left_role.clone(), left),
                (right_role.clone(), right),
            ]),
        })
    }

    fn call_speech_act(
        &self,
        surface: &str,
        content: Term,
    ) -> Result<Term, SurfaceElaborationError> {
        let LexemeConfig::SpeechAct { semantic, role } = self.lexeme(surface)? else {
            return Err(SurfaceElaborationError::WrongLexemeKind {
                surface: surface.to_owned(),
                expected: "speech_act",
            });
        };
        Ok(Term::Call {
            function: semantic.clone(),
            arguments: BTreeMap::from([(role.clone(), content)]),
        })
    }

    fn lexeme(&self, surface: &str) -> Result<&LexemeConfig, SurfaceElaborationError> {
        self.lexicon
            .get(surface)
            .ok_or_else(|| SurfaceElaborationError::MissingLexeme(surface.to_owned()))
    }

    fn next_placeholder(&mut self, kind: &str) -> String {
        let placeholder = format!("__surface_{kind}_{}", self.placeholder_index);
        self.placeholder_index += 1;
        placeholder
    }
}

fn parameter_type<'a>(
    signature: &'a crate::semantics::Signature,
    operator: &str,
    role: &str,
) -> Result<&'a Type, SurfaceElaborationError> {
    signature
        .parameters
        .iter()
        .find(|parameter| parameter.name == role)
        .map(|parameter| &parameter.ty)
        .ok_or_else(|| SurfaceElaborationError::MissingSemanticRole {
            operator: operator.to_owned(),
            role: role.to_owned(),
        })
}

fn contains_type_variable(ty: &Type) -> bool {
    match ty {
        Type::Variable(_) => true,
        Type::Named(_) => false,
        Type::Generic { arguments, .. } => arguments.iter().any(contains_type_variable),
        Type::Function {
            parameters,
            returns,
        } => {
            parameters
                .iter()
                .any(|parameter| contains_type_variable(&parameter.ty))
                || contains_type_variable(returns)
        }
        Type::Record(fields) => fields.values().any(contains_type_variable),
    }
}

impl fmt::Display for SurfaceElaborationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLexeme(surface) => {
                write!(f, "surface lexeme `{surface}` is not declared")
            }
            Self::WrongLexemeKind { surface, expected } => {
                write!(f, "surface lexeme `{surface}` is not a {expected}")
            }
            Self::UnknownSemanticOperator(operator) => {
                write!(f, "surface references unknown semantic operator `{operator}`")
            }
            Self::MissingSemanticRole { operator, role } => {
                write!(f, "semantic operator `{operator}` has no role `{role}`")
            }
            Self::InvalidType { source, message } => {
                write!(f, "invalid surface type `{source}`: {message}")
            }
            Self::UnconstrainedReferenceType { role, ty } => write!(
                f,
                "reference slot `{role}` has underconstrained semantic type `{ty}`"
            ),
            Self::InvalidSemanticTerm(message) => write!(f, "surface semantics: {message}"),
        }
    }
}

impl std::error::Error for SurfaceElaborationError {}
