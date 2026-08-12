use std::collections::BTreeMap;
use std::fmt;

use crate::semantics::{
    Checker, Environment, InformationKnowerValue, Literal, StructuredLiteral, StructuredValue,
    Term, Type,
};
use crate::spec::parse_type;

use super::{Argument, Clause, InformationKnower, CompiledSurfaceBinding, SurfaceExpr, CompiledSurfaceLexicon};

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
    lexicon: &CompiledSurfaceLexicon,
    environment: &Environment,
) -> Result<TypedSurfaceAst, SurfaceElaborationError> {
    let mut elaborator = Elaborator {
        lexicon,
        environment,
        placeholder_index: 0,
        binder_index: 0,
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
    lexicon: &'a CompiledSurfaceLexicon,
    environment: &'a Environment,
    placeholder_index: usize,
    binder_index: usize,
    references: Vec<ReferenceSlot>,
    contexts: Vec<ContextSlot>,
    aliases: Vec<AliasSlot>,
}

#[derive(Clone)]
struct BinderIntroduction {
    binder_surface: String,
    restriction_surface: String,
    direct_arguments: Vec<(u32, Term)>,
    variable: String,
    variable_type: Type,
}

#[derive(Clone)]
enum ScopeIntroduction {
    Prefix(String),
    Binder(BinderIntroduction),
}

impl Elaborator<'_> {
    fn lower_expr(&mut self, expression: &SurfaceExpr) -> Result<Term, SurfaceElaborationError> {
        match expression {
            SurfaceExpr::Atom(surface) => self.lower_atom(surface),
            SurfaceExpr::Context(surface) => self.lower_standalone_context(surface),
            SurfaceExpr::Alias(surface) => self.lower_standalone_alias(surface),
            SurfaceExpr::Captured { marker, payload } => self.lower_capture(marker, payload),
            SurfaceExpr::Quote(payload) => Ok(self.lower_quote(payload)),
            SurfaceExpr::Literal(literal) => Ok(Term::Literal(Literal::Structured(literal.semantic.clone()))),
            SurfaceExpr::Clause(clause) => self.lower_clause(clause),
            SurfaceExpr::Prefix { operator, operand } => {
                let operand = self.lower_expr(operand)?;
                self.call_prefix(operator, operand)
            }
            SurfaceExpr::Outer { operator, content } => {
                let content = self.lower_expr(content)?;
                self.call_prefix(operator, content)
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
        let CompiledSurfaceBinding::Atom { semantic } = self.lexeme(surface)? else {
            return Err(SurfaceElaborationError::WrongLexemeKind {
                surface: surface.to_owned(),
                expected: "atom",
            });
        };
        Ok(Term::Const(self.semantic_name(*semantic)?.to_owned()))
    }

    fn lower_capture(
        &self,
        marker: &str,
        payload: &str,
    ) -> Result<Term, SurfaceElaborationError> {
        let CompiledSurfaceBinding::Capture { semantic, parameter } = self.lexeme(marker)? else {
            return Err(SurfaceElaborationError::WrongLexemeKind {
                surface: marker.to_owned(),
                expected: "name",
            });
        };
        let semantic_name = self.semantic_name(*semantic)?.to_owned();
        let signature = self.environment.operator(&semantic_name).ok_or_else(|| {
            SurfaceElaborationError::UnknownSemanticOperator(semantic_name.clone())
        })?;
        let parameter = parameter_at(signature, &semantic_name, *parameter)?;
        let mut arguments = BTreeMap::new();
        arguments.insert(
            parameter.name.clone(),
            Term::Literal(Literal::String(payload.to_owned())),
        );
        Ok(Term::Call {
            function: semantic_name,
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
        let CompiledSurfaceBinding::Context { key, ty, .. } = self.lexeme(surface)?.clone() else {
            return Err(SurfaceElaborationError::WrongLexemeKind {
                surface: surface.to_owned(),
                expected: "context",
            });
        };
        let declared_type = ty;
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
        let CompiledSurfaceBinding::Alias { name, ty } = self.lexeme(surface)?.clone() else {
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
        let (semantic_id, primary_parameter, rest_parameters) = match predicate {
            CompiledSurfaceBinding::Predicate { semantic, primary_parameter, rest_parameters } => {
                (semantic, primary_parameter, rest_parameters)
            }
            CompiledSurfaceBinding::Class { semantic, parameter } => (semantic, Some(parameter), Vec::new()),
            _ => {
                return Err(SurfaceElaborationError::WrongLexemeKind {
                    surface: clause.predicate.clone(),
                    expected: "predicate or class",
                });
            }
        };
        let semantic = self.semantic_name(semantic_id)?.to_owned();
        let signature = self.environment.operator(&semantic).cloned().ok_or_else(|| {
            SurfaceElaborationError::UnknownSemanticOperator(semantic.clone())
        })?;

        let reference_start = self.references.len();
        let context_start = self.contexts.len();
        let alias_start = self.aliases.len();
        let mut type_bindings = BTreeMap::new();
        let mut arguments = BTreeMap::new();
        let mut introductions = Vec::new();

        if let Some(index) = primary_parameter {
            let argument = clause.primary.as_ref().ok_or_else(|| {
                SurfaceElaborationError::InvalidSemanticTerm(format!(
                    "predicate `{}` is missing its primary argument slot", clause.predicate
                ))
            })?;
            let parameter = parameter_at(&signature, &semantic, index)?;
            let (term, intro) = self.lower_argument(
                argument,
                &parameter.name,
                &parameter.ty,
                &mut type_bindings,
            )?;
            arguments.insert(parameter.name.clone(), term);
            if let Some(intro) = intro { introductions.push(ScopeIntroduction::Binder(intro)); }
        } else if clause.primary.is_some() {
            return Err(SurfaceElaborationError::InvalidSemanticTerm(format!(
                "predicate `{}` has no primary semantic parameter", clause.predicate
            )));
        }

        for prefix in &clause.inner_prefixes {
            introductions.push(ScopeIntroduction::Prefix(prefix.clone()));
        }

        if rest_parameters.len() != clause.rest.len() {
            return Err(SurfaceElaborationError::InvalidSemanticTerm(format!(
                "predicate `{}` expects {} rest argument slot(s), surface clause has {}",
                clause.predicate, rest_parameters.len(), clause.rest.len()
            )));
        }

        for (index, argument) in rest_parameters.iter().copied().zip(&clause.rest) {
            let parameter = parameter_at(&signature, &semantic, index)?;
            let (term, intro) = self.lower_argument(
                argument,
                &parameter.name,
                &parameter.ty,
                &mut type_bindings,
            )?;
            arguments.insert(parameter.name.clone(), term);
            if let Some(intro) = intro { introductions.push(ScopeIntroduction::Binder(intro)); }
        }

        self.finalize_reference_types(reference_start, &type_bindings)?;
        self.finalize_context_types(context_start, &type_bindings)?;
        self.finalize_alias_types(alias_start, &type_bindings)?;

        let mut term = Term::Call { function: semantic, arguments };
        for introduction in introductions.into_iter().rev() {
            term = match introduction {
                ScopeIntroduction::Prefix(surface) => self.call_prefix(&surface, term)?,
                ScopeIntroduction::Binder(binder) => self.wrap_binder(binder, term)?,
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
    ) -> Result<(Term, Option<BinderIntroduction>), SurfaceElaborationError> {
        match argument {
            Argument::Atom(surface) => {
                let CompiledSurfaceBinding::Atom { semantic } = self.lexeme(surface)? else {
                    return Err(SurfaceElaborationError::WrongLexemeKind {
                        surface: surface.clone(),
                        expected: "atom",
                    });
                };
                let semantic_name = self.semantic_name(*semantic)?.to_owned();
                let actual = self
                    .environment
                    .constant_type(&semantic_name)
                    .cloned()
                    .ok_or_else(|| {
                        SurfaceElaborationError::InvalidSemanticTerm(format!(
                            "unknown semantic constant `{semantic_name}`"
                        ))
                    })?;
                self.match_expected(expected, &actual, type_bindings, role)?;
                Ok((Term::Const(semantic_name), None))
            }
            Argument::Context(surface) => {
                let CompiledSurfaceBinding::Context { key, ty, .. } = self.lexeme(surface)?.clone() else {
                    return Err(SurfaceElaborationError::WrongLexemeKind {
                        surface: surface.clone(),
                        expected: "context",
                    });
                };
                let declared_type = ty;
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
                let CompiledSurfaceBinding::Alias { name, ty } = self.lexeme(surface)?.clone() else {
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

            Argument::Captured { marker, payload } => {
                let term = self.lower_capture(marker, payload)?;
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
                let CompiledSurfaceBinding::Information { mode, status, knower_type } = self.lexeme(marker)?.clone() else {
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
                        role: format!("information {status:?} for {role}"),
                        ty: concrete,
                    });
                }
                let declared_knower_type = knower_type;
                let knower = match knower {
                    None => None,
                    Some(InformationKnower::Context(surface)) => {
                        let CompiledSurfaceBinding::Context { slot, ty, .. } = self.lexeme(surface)? else {
                            return Err(SurfaceElaborationError::WrongLexemeKind {
                                surface: surface.clone(),
                                expected: "context knower",
                            });
                        };
                        if let Some(expected_knower) = &declared_knower_type {
                            let actual_knower = ty.clone();
                            let mut knower_bindings = BTreeMap::new();
                            self.match_expected(
                                expected_knower,
                                &actual_knower,
                                &mut knower_bindings,
                                "information knower",
                            )?;
                        }
                        Some(InformationKnowerValue::Context(*slot))
                    }
                    Some(InformationKnower::Captured { marker: name_marker, payload }) => {
                        let CompiledSurfaceBinding::Capture { .. } = self.lexeme(name_marker)? else {
                            return Err(SurfaceElaborationError::WrongLexemeKind {
                                surface: name_marker.clone(),
                                expected: "proper-name knower",
                            });
                        };
                        let name_term = self.lower_capture(name_marker, payload)?;
                        if let Some(expected_knower) = &declared_knower_type {
                            let actual_knower = Checker::new(self.environment)
                                .infer(&name_term)
                                .map_err(|error| SurfaceElaborationError::InvalidSemanticTerm(error.to_string()))?;
                            let mut knower_bindings = BTreeMap::new();
                            self.match_expected(
                                expected_knower,
                                &actual_knower,
                                &mut knower_bindings,
                                "information knower",
                            )?;
                        }
                        Some(InformationKnowerValue::Value(Box::new(name_term)))
                    }
                };
                Ok((
                    Term::Literal(Literal::Structured(StructuredLiteral::new(
                        StructuredValue::Information { mode, status, knower },
                        concrete,
                    ))),
                    None,
                ))
            }
            Argument::Scoped { binder, direct, restriction } => {
                let binder_config = self.lexeme(binder)?.clone();
                let CompiledSurfaceBinding::Binder {
                    semantic,
                    binder_parameter,
                    direct_parameters,
                    variable_type,
                    ..
                } = binder_config else {
                    return Err(SurfaceElaborationError::WrongLexemeKind {
                        surface: binder.clone(), expected: "scoped binder",
                    });
                };
                if direct_parameters.len() != direct.len() {
                    return Err(SurfaceElaborationError::InvalidSemanticTerm(format!(
                        "scoped binder `{binder}` expects {} direct argument(s), surface has {}",
                        direct_parameters.len(), direct.len(),
                    )));
                }
                let restriction_config = self.lexeme(restriction)?.clone();
                let is_unary_restriction = match restriction_config {
                    CompiledSurfaceBinding::Class { .. } => true,
                    CompiledSurfaceBinding::Predicate { primary_parameter: Some(_), ref rest_parameters, .. } => rest_parameters.is_empty(),
                    _ => false,
                };
                if !is_unary_restriction {
                    return Err(SurfaceElaborationError::WrongLexemeKind {
                        surface: restriction.clone(), expected: "unary predicate restriction",
                    });
                }
                self.match_expected(expected, &variable_type, type_bindings, role)?;
                let semantic_name = self.semantic_name(semantic)?.to_owned();
                let signature = self.environment.operator(&semantic_name).ok_or_else(|| {
                    SurfaceElaborationError::UnknownSemanticOperator(semantic_name.clone())
                })?;
                let mut direct_arguments = Vec::with_capacity(direct.len());
                for (index, argument) in direct_parameters.iter().copied().zip(direct) {
                    let parameter = parameter_at(signature, &semantic_name, index)?;
                    let mut direct_bindings = BTreeMap::new();
                    let (term, nested) = self.lower_argument(
                        argument, &parameter.name, &parameter.ty, &mut direct_bindings,
                    )?;
                    if nested.is_some() {
                        return Err(SurfaceElaborationError::InvalidSemanticTerm(format!(
                            "scoped binder `{binder}` direct argument slot `{index}` cannot introduce another scope"
                        )));
                    }
                    direct_arguments.push((index, term));
                }
                let binder_parameter_value = parameter_at(signature, &semantic_name, binder_parameter)?;
                let Type::Function { parameters, returns } = &binder_parameter_value.ty else {
                    return Err(SurfaceElaborationError::InvalidSemanticTerm(format!(
                        "scoped binder `{binder}` binder parameter `{binder_parameter}` must be a function"
                    )));
                };
                if parameters.len() != 1 || returns.as_ref() != &Type::named("Proposition") {
                    return Err(SurfaceElaborationError::InvalidSemanticTerm(format!(
                        "scoped binder `{binder}` binder parameter must be a unary predicate"
                    )));
                }
                let variable = format!("surface_b{}", self.binder_index);
                self.binder_index += 1;
                Ok((
                    Term::Var(variable.clone()),
                    Some(BinderIntroduction {
                        binder_surface: binder.clone(),
                        restriction_surface: restriction.clone(),
                        direct_arguments,
                        variable,
                        variable_type,
                    }),
                ))
            }
            Argument::Reference(surface) => {
                let CompiledSurfaceBinding::Reference = self.lexeme(surface)? else {
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

    fn wrap_binder(
        &self,
        intro: BinderIntroduction,
        body: Term,
    ) -> Result<Term, SurfaceElaborationError> {
        let CompiledSurfaceBinding::Binder {
            semantic,
            binder_parameter,
            combiner_semantic,
            combiner_left_parameter,
            combiner_right_parameter,
            ..
        } = self.lexeme(&intro.binder_surface)? else {
            return Err(SurfaceElaborationError::WrongLexemeKind {
                surface: intro.binder_surface, expected: "scoped binder",
            });
        };
        let semantic = self.semantic_name(*semantic)?.to_owned();
        let combiner_semantic = self.semantic_name(*combiner_semantic)?.to_owned();

        let restriction_lexeme = self.lexeme(&intro.restriction_surface)?;
        let (restriction_id, restriction_parameter) = match restriction_lexeme {
            CompiledSurfaceBinding::Class { semantic, parameter } => (*semantic, *parameter),
            CompiledSurfaceBinding::Predicate { semantic, primary_parameter: Some(parameter), rest_parameters } if rest_parameters.is_empty() => {
                (*semantic, *parameter)
            }
            _ => {
                return Err(SurfaceElaborationError::WrongLexemeKind {
                    surface: intro.restriction_surface, expected: "unary predicate restriction",
                });
            }
        };
        let restriction_semantic = self.semantic_name(restriction_id)?.to_owned();
        let restriction_signature = self.environment.operator(&restriction_semantic).ok_or_else(|| {
            SurfaceElaborationError::UnknownSemanticOperator(restriction_semantic.clone())
        })?;
        let restriction_parameter = parameter_at(restriction_signature, &restriction_semantic, restriction_parameter)?;
        let restriction = Term::Call {
            function: restriction_semantic,
            arguments: BTreeMap::from([(restriction_parameter.name.clone(), Term::Var(intro.variable.clone()))]),
        };

        let combiner_signature = self.environment.operator(&combiner_semantic).ok_or_else(|| {
            SurfaceElaborationError::UnknownSemanticOperator(combiner_semantic.clone())
        })?;
        let left = parameter_at(combiner_signature, &combiner_semantic, *combiner_left_parameter)?;
        let right = parameter_at(combiner_signature, &combiner_semantic, *combiner_right_parameter)?;
        let combined = Term::Call {
            function: combiner_semantic,
            arguments: BTreeMap::from([(left.name.clone(), restriction), (right.name.clone(), body)]),
        };
        let binder = Term::Bind {
            variable: intro.variable, variable_type: intro.variable_type, body: Box::new(combined),
        };
        let signature = self.environment.operator(&semantic).ok_or_else(|| {
            SurfaceElaborationError::UnknownSemanticOperator(semantic.clone())
        })?;
        let binder_parameter_value = parameter_at(signature, &semantic, *binder_parameter)?;
        let mut arguments = BTreeMap::from([(binder_parameter_value.name.clone(), binder)]);
        for (index, term) in intro.direct_arguments {
            let parameter = parameter_at(signature, &semantic, index)?;
            arguments.insert(parameter.name.clone(), term);
        }
        Ok(Term::Call { function: semantic, arguments })
    }

    fn call_prefix(&self, surface: &str, operand: Term) -> Result<Term, SurfaceElaborationError> {
        let CompiledSurfaceBinding::Prefix { semantic, parameter, .. } = self.lexeme(surface)? else {
            return Err(SurfaceElaborationError::WrongLexemeKind { surface: surface.to_owned(), expected: "prefix" });
        };
        let semantic = self.semantic_name(*semantic)?.to_owned();
        let signature = self.environment.operator(&semantic).ok_or_else(|| {
            SurfaceElaborationError::UnknownSemanticOperator(semantic.clone())
        })?;
        let parameter = parameter_at(signature, &semantic, *parameter)?;
        Ok(Term::Call { function: semantic, arguments: BTreeMap::from([(parameter.name.clone(), operand)]) })
    }

    fn call_infix(&self, surface: &str, left: Term, right: Term) -> Result<Term, SurfaceElaborationError> {
        let CompiledSurfaceBinding::Infix { semantic, left_parameter, right_parameter, .. } = self.lexeme(surface)? else {
            return Err(SurfaceElaborationError::WrongLexemeKind { surface: surface.to_owned(), expected: "infix" });
        };
        let semantic = self.semantic_name(*semantic)?.to_owned();
        let signature = self.environment.operator(&semantic).ok_or_else(|| {
            SurfaceElaborationError::UnknownSemanticOperator(semantic.clone())
        })?;
        let left_parameter = parameter_at(signature, &semantic, *left_parameter)?;
        let right_parameter = parameter_at(signature, &semantic, *right_parameter)?;
        Ok(Term::Call {
            function: semantic,
            arguments: BTreeMap::from([(left_parameter.name.clone(), left), (right_parameter.name.clone(), right)]),
        })
    }

    fn semantic_name(&self, id: crate::semantics::SymbolId) -> Result<&str, SurfaceElaborationError> {
        self.lexicon.semantic_name(id).ok_or_else(|| {
            SurfaceElaborationError::UnknownSemanticOperator(id.to_string())
        })
    }

    fn lexeme(&self, surface: &str) -> Result<&CompiledSurfaceBinding, SurfaceElaborationError> {
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

fn parameter_at<'a>(
    signature: &'a crate::semantics::Signature,
    operator: &str,
    index: u32,
) -> Result<&'a crate::semantics::Parameter, SurfaceElaborationError> {
    signature.parameters.get(index as usize).ok_or_else(|| SurfaceElaborationError::MissingSemanticRole {
        operator: operator.to_owned(),
        role: format!("#{index}"),
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
