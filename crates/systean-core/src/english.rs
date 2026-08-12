use std::fmt;

use serde::Serialize;

use crate::documentation::DocumentationPackage;
use crate::rational::ExactRational;
use crate::semantics::{InformationKnowerValue, InformationStatus, Literal, StructuredValue, Term, canonicalize};
use crate::spec::{CompiledSurfaceItem, TypedSemanticPackage};
use crate::units::UnitRegistry;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct EnglishAlignment {
    pub semantic_path: String,
    pub root: Option<String>,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct EnglishRendering {
    pub text: String,
    pub alignments: Vec<EnglishAlignment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnglishRenderError(pub String);

impl fmt::Display for EnglishRenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for EnglishRenderError {}

pub struct EnglishRenderer<'a> {
    documentation: &'a DocumentationPackage,
    semantics: &'a TypedSemanticPackage,
    units: Option<&'a UnitRegistry>,
}

impl<'a> EnglishRenderer<'a> {
    pub fn new(
        documentation: &'a DocumentationPackage,
        semantics: &'a TypedSemanticPackage,
        units: Option<&'a UnitRegistry>,
    ) -> Self {
        Self { documentation, semantics, units }
    }

    pub fn render(&self, term: &Term) -> Result<EnglishRendering, EnglishRenderError> {
        let canonical = canonicalize(term);
        let mut state = RenderState::default();
        self.render_term(&canonical, "$", &mut state)?;
        Ok(EnglishRendering {
            text: state.text,
            alignments: state.alignments,
        })
    }

    fn render_term(
        &self,
        term: &Term,
        path: &str,
        state: &mut RenderState,
    ) -> Result<(), EnglishRenderError> {
        let start = state.text.len();
        let root = match term {
            Term::Call { function, .. } => Some(function.clone()),
            Term::Const(name) if self.documentation.entry(name).is_some() => Some(name.clone()),
            _ => None,
        };
        match term {
            Term::Const(name) => self.render_const(name, state),
            Term::Var(name) => state.push(&humanize(name)),
            Term::Literal(literal) => self.render_literal(literal, path, state)?,
            Term::Call { function, arguments } => {
                self.render_call(function, arguments, path, state)?;
            }
            Term::Bind { variable, variable_type, body } => {
                state.push("for ");
                state.push(&humanize(variable));
                state.push(": ");
                state.push(&variable_type.to_string());
                state.push(" => (");
                self.render_term(body, &format!("{path}.body"), state)?;
                state.push(")");
            }
            Term::Record(fields) => {
                state.push("record {");
                for (index, (name, value)) in fields.iter().enumerate() {
                    if index > 0 {
                        state.push("; ");
                    }
                    state.push(&humanize(name));
                    state.push(": ");
                    self.render_term(value, &format!("{path}.{name}"), state)?;
                }
                state.push("}");
            }
            Term::Field { record, field } => {
                state.push("field ");
                state.push(&humanize(field));
                state.push(" of (");
                self.render_term(record, &format!("{path}.record"), state)?;
                state.push(")");
            }
        }
        let end = state.text.len();
        state.alignments.push(EnglishAlignment {
            semantic_path: path.to_owned(),
            root,
            start,
            end,
        });
        Ok(())
    }

    fn render_const(&self, name: &str, state: &mut RenderState) {
        if let Some(entry) = self.documentation.entry(name) {
            state.push(&entry.gloss);
            return;
        }
        state.push(&humanize(name));
    }

    fn render_call(
        &self,
        function: &str,
        arguments: &std::collections::BTreeMap<String, Term>,
        path: &str,
        state: &mut RenderState,
    ) -> Result<(), EnglishRenderError> {
        let Some(symbol_id) = self.semantics.symbol_id(function) else {
            return self.render_generic_call(function, arguments, path, state);
        };
        let Some(entry) = self.documentation.entry(function) else {
            return self.render_generic_call(function, arguments, path, state);
        };
        if self.semantics.symbol(symbol_id).is_none() {
            return Err(EnglishRenderError(format!("compiled symbol for `{function}` is missing")));
        }
        let debug = self.semantics.debug_symbol(symbol_id).ok_or_else(|| {
            EnglishRenderError(format!("compiled debug signature for `{function}` is missing"))
        })?;
        if let Some(rule) = self.semantics.surface_rule(symbol_id) {
            if let Some(binder) = &rule.binder {
                if let Some(rendered) = self.render_binder_call(
                    &entry.gloss,
                    arguments,
                    binder.parameter as usize,
                    binder.combiner,
                    debug,
                    path,
                    state,
                )? {
                    return Ok(rendered);
                }
            }

            if rule.items.as_slice()
                == [CompiledSurfaceItem::Argument(0), CompiledSurfaceItem::Root, CompiledSurfaceItem::Argument(1)]
                && debug.parameter_names.len() == 2
            {
                let left_role = &debug.parameter_names[0];
                let right_role = &debug.parameter_names[1];
                if let (Some(left), Some(right)) = (arguments.get(left_role), arguments.get(right_role)) {
                    if rule.outer_only {
                        state.push(&entry.gloss);
                        state.push(" [");
                        state.push(&humanize(left_role));
                        state.push(": ");
                        self.render_term(left, &format!("{path}.{left_role}"), state)?;
                        state.push("]: (");
                        self.render_term(right, &format!("{path}.{right_role}"), state)?;
                        state.push(")");
                        return Ok(());
                    }
                    state.push("(");
                    self.render_term(left, &format!("{path}.{left_role}"), state)?;
                    state.push(") ");
                    state.push(&entry.gloss);
                    state.push(" (");
                    self.render_term(right, &format!("{path}.{right_role}"), state)?;
                    state.push(")");
                    return Ok(());
                }
            }

            if rule.items.as_slice()
                == [CompiledSurfaceItem::Root, CompiledSurfaceItem::Argument(0)]
                && debug.parameter_names.len() == 1
            {
                let role = &debug.parameter_names[0];
                if let Some(value) = arguments.get(role) {
                    state.push(&entry.gloss);
                    if rule.outer_only {
                        state.push(": (");
                    } else {
                        state.push(" (");
                    }
                    self.render_term(value, &format!("{path}.{role}"), state)?;
                    state.push(")");
                    return Ok(());
                }
            }
        }

        state.push(&entry.gloss);
        if arguments.is_empty() {
            return Ok(());
        }
        state.push(" [");
        let mut wrote = false;
        for role in &debug.parameter_names {
            let Some(value) = arguments.get(role) else {
                continue;
            };
            if wrote {
                state.push("; ");
            }
            wrote = true;
            state.push(&humanize(role));
            state.push(": ");
            self.render_term(value, &format!("{path}.{role}"), state)?;
        }
        for (role, value) in arguments {
            if debug.parameter_names.contains(role) {
                continue;
            }
            if wrote {
                state.push("; ");
            }
            wrote = true;
            state.push(&humanize(role));
            state.push(": ");
            self.render_term(value, &format!("{path}.{role}"), state)?;
        }
        state.push("]");
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn render_binder_call(
        &self,
        gloss: &str,
        arguments: &std::collections::BTreeMap<String, Term>,
        binder_parameter: usize,
        combiner: crate::semantics::SymbolId,
        debug: &crate::spec::SymbolDebugInfo,
        path: &str,
        state: &mut RenderState,
    ) -> Result<Option<()>, EnglishRenderError> {
        let Some(role) = debug.parameter_names.get(binder_parameter) else {
            return Ok(None);
        };
        let Some(Term::Bind { variable, body, .. }) = arguments.get(role) else {
            return Ok(None);
        };
        let Some(combiner_name) = self.semantics.source_name_for_symbol(combiner) else {
            return Ok(None);
        };
        let Some((body_function, body_arguments)) = body.as_ref().into_call() else {
            return Ok(None);
        };
        if body_function.as_str() != combiner_name {
            return Ok(None);
        }
        let Some(combiner_debug) = self.semantics.debug_symbol(combiner) else {
            return Ok(None);
        };
        if combiner_debug.parameter_names.len() != 2 {
            return Ok(None);
        }
        let Some(restriction) = body_arguments.get(&combiner_debug.parameter_names[0]) else {
            return Ok(None);
        };
        let Some(body_value) = body_arguments.get(&combiner_debug.parameter_names[1]) else {
            return Ok(None);
        };

        state.push(gloss);
        for parameter in debug.parameter_names.iter().take(binder_parameter) {
            if let Some(value) = arguments.get(parameter) {
                state.push(" ");
                self.render_term(value, &format!("{path}.{parameter}"), state)?;
            }
        }
        state.push(" value ");
        state.push(&humanize(variable));
        state.push(" such that (");
        self.render_term(restriction, &format!("{path}.{role}.restriction"), state)?;
        state.push(") => (");
        self.render_term(body_value, &format!("{path}.{role}.body"), state)?;
        state.push(")");
        Ok(Some(()))
    }

    fn render_generic_call(
        &self,
        function: &str,
        arguments: &std::collections::BTreeMap<String, Term>,
        path: &str,
        state: &mut RenderState,
    ) -> Result<(), EnglishRenderError> {
        state.push(&humanize(function));
        if arguments.is_empty() {
            return Ok(());
        }
        state.push(" [");
        for (index, (role, value)) in arguments.iter().enumerate() {
            if index > 0 {
                state.push("; ");
            }
            state.push(&humanize(role));
            state.push(": ");
            self.render_term(value, &format!("{path}.{role}"), state)?;
        }
        state.push("]");
        Ok(())
    }

    fn render_literal(
        &self,
        literal: &Literal,
        path: &str,
        state: &mut RenderState,
    ) -> Result<(), EnglishRenderError> {
        match literal {
            Literal::Integer(value) => state.push(&value.to_string()),
            Literal::Boolean(value) => state.push(if *value { "true" } else { "false" }),
            Literal::String(value) => {
                state.push("\"");
                state.push(value);
                state.push("\"");
            }
            Literal::Structured(value) => self.render_structured(&value.value, path, state)?,
        }
        Ok(())
    }

    fn render_structured(
        &self,
        value: &StructuredValue,
        path: &str,
        state: &mut RenderState,
    ) -> Result<(), EnglishRenderError> {
        match value {
            StructuredValue::Number(value) => state.push(&render_rational(value)),
            StructuredValue::ApproximateNumber { value, tolerance } => {
                state.push("approximately ");
                state.push(&render_rational(value));
                if let Some(tolerance) = tolerance {
                    state.push(" ± ");
                    state.push(&render_rational(tolerance));
                }
            }
            StructuredValue::Digit(value) => state.push(&value.to_string()),
            StructuredValue::DigitSequence(values) => {
                state.push(&values.iter().map(u8::to_string).collect::<String>())
            }
            StructuredValue::Unit(unit) => {
                state.push(&self.unit_label(*unit));
            }
            StructuredValue::Quantity { value, unit, approximate, uncertainty } => {
                if *approximate {
                    state.push("approximately ");
                }
                state.push(&render_rational(value));
                state.push(" ");
                state.push(&self.unit_label(*unit));
                if let Some(uncertainty) = uncertainty {
                    state.push(" ± ");
                    state.push(&render_rational(uncertainty));
                }
            }
            StructuredValue::CalendarDate { year, month, day } => {
                state.push(&format!("{year:04}-{month:02}-{day:02}"));
            }
            StructuredValue::TimeOfDay { hour, minute, second } => {
                state.push(&format!("{hour:02}:{minute:02}:{second:02}"));
            }
            StructuredValue::TimeZone { offset_minutes } => {
                state.push(&render_timezone(*offset_minutes));
            }
            StructuredValue::Instant { year, month, day, hour, minute, second, offset_minutes } => {
                state.push(&format!(
                    "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}{}",
                    render_timezone(*offset_minutes)
                ));
            }
            StructuredValue::Duration { seconds } => {
                state.push("duration ");
                state.push(&render_rational(seconds));
                state.push(" seconds");
            }
            StructuredValue::Interval { start, end } => {
                state.push("interval from (");
                self.render_structured(start, &format!("{path}.start"), state)?;
                state.push(") to (");
                self.render_structured(end, &format!("{path}.end"), state)?;
                state.push(")");
            }
            StructuredValue::Information { status, knower, .. } => {
                match status {
                    InformationStatus::Unknown => state.push("unknown information"),
                    InformationStatus::Unspecified => state.push("unspecified information"),
                    InformationStatus::Withheld => state.push("withheld information"),
                }
                if let Some(knower) = knower {
                    state.push(" [knower: ");
                    match knower {
                        InformationKnowerValue::Context(slot) => {
                            state.push(self.semantics.source_name_for_context(*slot).unwrap_or("context"));
                        }
                        InformationKnowerValue::Value(value) => {
                            self.render_term(value, &format!("{path}.knower"), state)?;
                        }
                    }
                    state.push("]");
                }
            }
        }
        Ok(())
    }

    fn unit_label(&self, unit: crate::semantics::UnitId) -> String {
        self.units
            .and_then(|units| units.unit_by_id(unit))
            .map(|unit| unit.legacy_name.clone())
            .or_else(|| self.semantics.source_name_for_unit(unit).map(str::to_owned))
            .unwrap_or_else(|| "unit".into())
    }
}

#[derive(Default)]
struct RenderState {
    text: String,
    alignments: Vec<EnglishAlignment>,
}

impl RenderState {
    fn push(&mut self, text: &str) {
        self.text.push_str(text);
    }
}

trait TermCallExt {
    fn into_call(&self) -> Option<(&String, &std::collections::BTreeMap<String, Term>)>;
}

impl TermCallExt for Term {
    fn into_call(&self) -> Option<(&String, &std::collections::BTreeMap<String, Term>)> {
        match self {
            Term::Call { function, arguments } => Some((function, arguments)),
            _ => None,
        }
    }
}

fn humanize(value: &str) -> String {
    value.replace(['_', '.'], " ")
}

fn render_rational(value: &ExactRational) -> String {
    if value.denom() == &num_bigint::BigInt::from(1u8) {
        value.numer().to_string()
    } else {
        format!("{}/{}", value.numer(), value.denom())
    }
}

fn render_timezone(offset_minutes: i16) -> String {
    if offset_minutes == 0 {
        return "Z".into();
    }
    let sign = if offset_minutes < 0 { '-' } else { '+' };
    let absolute = offset_minutes.unsigned_abs();
    format!("{sign}{:02}:{:02}", absolute / 60, absolute % 60)
}
