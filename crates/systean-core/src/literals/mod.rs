mod config;
mod number;
mod time;

use std::collections::BTreeSet;
use std::fmt;

use num_bigint::BigInt;
use num_traits::ToPrimitive;

pub use config::{CalendarConfig, LiteralConfig, LiteralConfigError, NumberConfig};
pub use number::{ExactNumber, NumberParse, canonical_spoken as canonical_spoken_number, integer_to_spoken, parse_spoken_integer, parse_spoken_number};
pub use time::TemporalLiteralValue;

use crate::semantics::{DimensionId, StructuredLiteral, StructuredValue, Type, UnitId};
use crate::spec::parse_type;
use crate::units::{QuantityValue, UnitRegistry};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiteralRealization {
    Written,
    Spoken,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceLiteral {
    pub semantic: StructuredLiteral,
    pub canonical_written: String,
    pub canonical_spoken: String,
    pub realization: LiteralRealization,
}

impl SurfaceLiteral {
    pub fn canonical_surface(&self) -> &str {
        match self.realization {
            LiteralRealization::Written => &self.canonical_written,
            LiteralRealization::Spoken => &self.canonical_spoken,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LiteralEngine {
    config: LiteralConfig,
    units: UnitRegistry,
    scope_open: String,
    scope_close: String,
    number_type: Type,
    approximate_number_type: Type,
    digit_type: Type,
    digit_sequence_type: Type,
    date_type: Type,
    time_of_day_type: Type,
    instant_type: Type,
    interval_type: Type,
    duration_type: Type,
    timezone_type: Type,
    duration_base_unit: UnitId,
    duration_dimension: DimensionId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralMatch {
    pub literal: SurfaceLiteral,
    pub consumed: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralError(pub String);

/// Generic parse/generate contract shared by every package-backed structured literal family.
/// The language package owns one engine implementing this boundary; callers never need
/// family-specific parsing logic.
pub trait StructuredLiteralCodec {
    fn parse_at(&self, tokens: &[String], start: usize) -> Result<Option<LiteralMatch>, LiteralError>;
    fn render_written(&self, literal: &StructuredLiteral) -> Result<String, LiteralError>;
    fn render_spoken(&self, literal: &StructuredLiteral) -> Result<String, LiteralError>;
}

impl LiteralEngine {
    pub fn new(
        config: LiteralConfig,
        units: UnitRegistry,
        scope_open: impl Into<String>,
        scope_close: impl Into<String>,
    ) -> Result<Self, LiteralError> {
        let number_type = parse_config_type(&config.number.semantic_type)?;
        let approximate_number_type = Type::Generic {
            name: "Approximate".into(),
            arguments: vec![number_type.clone()],
        };
        let digit_type = parse_config_type(&config.number.digit_type)?;
        let digit_sequence_type = parse_config_type(&config.number.digit_sequence_type)?;
        let date_type = parse_config_type(&config.calendar.date_type)?;
        let time_of_day_type = parse_config_type(&config.calendar.time_of_day_type)?;
        let instant_type = parse_config_type(&config.calendar.instant_type)?;
        let interval_type = parse_config_type(&config.calendar.interval_type)?;
        let duration_type = parse_config_type(&config.calendar.duration_type)?;
        let timezone_type = parse_config_type(&config.calendar.timezone_type)?;
        let duration_base_unit = units.unit_id_by_name(&config.calendar.duration_base_unit)
            .ok_or_else(|| LiteralError(format!(
                "duration base unit `{}` is not declared by the resolved unit registry",
                config.calendar.duration_base_unit
            )))?;
        let duration_dimension = units.unit_by_id(duration_base_unit)
            .map(|unit| unit.dimension)
            .ok_or_else(|| LiteralError("duration base unit was not resolved".into()))?;
        let literal_forms = config.reserved_forms().into_iter().collect::<BTreeSet<_>>();
        for unit in &units.config().units {
            if literal_forms.contains(unit.spoken.as_str()) {
                return Err(LiteralError(format!(
                    "spoken unit form `{}` collides with another structured-literal form",
                    unit.spoken
                )));
            }
        }
        Ok(Self {
            config,
            units,
            scope_open: scope_open.into(),
            scope_close: scope_close.into(),
            number_type,
            approximate_number_type,
            digit_type,
            digit_sequence_type,
            date_type,
            time_of_day_type,
            instant_type,
            interval_type,
            duration_type,
            timezone_type,
            duration_base_unit,
            duration_dimension,
        })
    }

    pub fn config(&self) -> &LiteralConfig { &self.config }
    pub fn units(&self) -> &UnitRegistry { &self.units }

    /// Construct one typed Digit component. Bare spoken digit words remain Numbers;
    /// Digit has no competing standalone surface form and is used inside DigitSequence.
    pub fn digit_literal(&self, digit: u8) -> Result<StructuredLiteral, LiteralError> {
        if digit > 9 {
            return Err(LiteralError(format!("digit {digit} is outside 0..9")));
        }
        Ok(StructuredLiteral::new(StructuredValue::Digit(digit), self.digit_type.clone()))
    }

    pub fn digit_sequence_elements(&self, digits: &str) -> Result<Vec<StructuredLiteral>, LiteralError> {
        if digits.is_empty() || !digits.chars().all(|value| value.is_ascii_digit()) {
            return Err(LiteralError("digit sequence must contain one or more decimal digits".into()));
        }
        digits
            .chars()
            .map(|value| self.digit_literal(value.to_digit(10).expect("ASCII digit") as u8))
            .collect()
    }

    pub fn parse_complete(&self, source: &str) -> Result<SurfaceLiteral, LiteralError> {
        let tokens = source
            .split_whitespace()
            .map(str::to_lowercase)
            .collect::<Vec<_>>();
        if tokens.is_empty() {
            return Err(LiteralError("structured literal source is empty".into()));
        }
        let matched = self
            .parse_at(&tokens, 0)?
            .ok_or_else(|| LiteralError(format!("`{source}` is not a structured literal")))?;
        if matched.consumed != tokens.len() {
            return Err(LiteralError(format!(
                "structured literal consumed {} of {} tokens",
                matched.consumed,
                tokens.len()
            )));
        }
        Ok(matched.literal)
    }

    pub fn convert_quantity_literal(
        &self,
        literal: &StructuredLiteral,
        target_unit: &str,
    ) -> Result<SurfaceLiteral, LiteralError> {
        let StructuredValue::Quantity { value, unit, approximate, uncertainty } = &literal.value else {
            return Err(LiteralError(format!(
                "cannot convert structured literal family `{}` as a quantity",
                literal.family()
            )));
        };
        let quantity = QuantityValue {
            value: ExactNumber(value.clone()),
            unit_id: *unit,
            approximate: *approximate,
            uncertainty: uncertainty.clone().map(ExactNumber),
        };
        let converted = self.units.convert_to_name(&quantity, target_unit).map_err(LiteralError)?;
        self.quantity_match(converted, 0, LiteralRealization::Written)
            .map(|matched| matched.literal)
    }

    pub fn reserved_forms(&self) -> Vec<&str> {
        let mut forms = self.config.reserved_forms();
        forms.extend(self.units.config().units.iter().map(|unit| unit.spoken.as_str()));
        forms.sort_unstable();
        forms.dedup();
        forms
    }

    pub fn parse_at(&self, tokens: &[String], start: usize) -> Result<Option<LiteralMatch>, LiteralError> {
        if start >= tokens.len() { return Ok(None); }

        if let Some(value) = time::parse_written_temporal(&tokens[start], self.config.number.max_explicit_exponent).map_err(LiteralError)? {
            return self.temporal_match(value, 1, LiteralRealization::Written).map(Some);
        }

        if tokens[start] == self.config.calendar.date_marker {
            if let Some(result) = self.parse_spoken_interval(tokens, start)? {
                return Ok(Some(result));
            }
            if let Some(result) = self.parse_spoken_instant(tokens, start)? {
                return Ok(Some(result));
            }
            if let Some(result) = self.parse_spoken_duration(tokens, start)? {
                return Ok(Some(result));
            }
            if let Some((value, consumed)) = time::parse_spoken_time(
                tokens, start, &self.config.number, &self.config.calendar.date_marker,
                &self.scope_open, &self.scope_close
            ).map_err(LiteralError)? {
                return self.temporal_match(value, consumed, LiteralRealization::Spoken).map(Some);
            }
            if let Some((value, consumed)) = time::parse_spoken_date(
                tokens, start, &self.config.number, &self.config.calendar.date_marker,
                &self.scope_open, &self.scope_close
            ).map_err(LiteralError)? {
                return self.temporal_match(value, consumed, LiteralRealization::Spoken).map(Some);
            }
        }

        if tokens[start] == self.config.calendar.timezone_marker {
            if let Some(result) = self.parse_spoken_timezone(tokens, start)? {
                return Ok(Some(result));
            }
        }

        if let Some(quantity) = self.parse_quantity(tokens, start)? {
            return Ok(Some(quantity));
        }
        if let Some(unit) = self.parse_unit(tokens, start)? {
            return Ok(Some(unit));
        }

        if tokens[start] == self.config.number.digit_sequence_marker {
            return self.parse_digit_sequence(tokens, start).map(Some);
        }

        if let Some(approximate) = self.parse_approximate_number(tokens, start)? {
            return Ok(Some(approximate));
        }

        if let Ok(value) = ExactNumber::parse_written(&tokens[start], self.config.number.max_explicit_exponent) {
            return self.number_match(value, 1, LiteralRealization::Written).map(Some);
        }

        match parse_spoken_number(
            tokens,
            start,
            &self.config.number,
            &self.scope_open,
            &self.scope_close,
        ).map_err(LiteralError)? {
            Some(parsed) => self.number_match(parsed.value, parsed.consumed, LiteralRealization::Spoken).map(Some),
            None => Ok(None),
        }
    }

    pub fn render_written(&self, literal: &StructuredLiteral) -> Result<String, LiteralError> {
        match &literal.value {
            StructuredValue::Number(value) => Ok(ExactNumber(value.clone()).canonical_written()),
            StructuredValue::ApproximateNumber { value, tolerance } => {
                let value = ExactNumber(value.clone()).canonical_written();
                Ok(match tolerance {
                    Some(tolerance) => format!("{value}±{}", ExactNumber(tolerance.clone()).canonical_written()),
                    None => format!("~{value}"),
                })
            }
            StructuredValue::Digit(value) => Ok(value.to_string()),
            StructuredValue::DigitSequence(values) => Ok(format!(
                "{} {}",
                self.config.number.digit_sequence_marker,
                values.iter().map(u8::to_string).collect::<String>()
            )),
            StructuredValue::Unit(id) => self.units.unit_by_id(*id)
                .map(|unit| unit.symbol.clone())
                .ok_or_else(|| LiteralError(format!("unknown unit `{id}`"))),
            StructuredValue::Quantity { value, unit, approximate, uncertainty } => {
                let unit = self.units.unit_by_id(*unit)
                    .ok_or_else(|| LiteralError(format!("unknown unit `{unit}`")))?;
                let value = ExactNumber(value.clone()).canonical_written();
                Ok(match uncertainty {
                    Some(uncertainty) => format!("{value}±{} {}", ExactNumber(uncertainty.clone()).canonical_written(), unit.symbol),
                    None if *approximate => format!("~{value} {}", unit.symbol),
                    None => format!("{value} {}", unit.symbol),
                })
            }
            value @ (
                StructuredValue::CalendarDate { .. }
                | StructuredValue::TimeOfDay { .. }
                | StructuredValue::TimeZone { .. }
                | StructuredValue::Instant { .. }
                | StructuredValue::Duration { .. }
                | StructuredValue::Interval { .. }
            ) => temporal_written(value),
            StructuredValue::Information { .. } => Err(LiteralError(
                "information values are semantic slot values, not standalone structured-literal surfaces".into()
            )),
        }
    }

    pub fn render_spoken(&self, literal: &StructuredLiteral) -> Result<String, LiteralError> {
        match &literal.value {
            StructuredValue::Number(value) => Ok(canonical_spoken_number(
                &ExactNumber(value.clone()),
                &self.config.number,
                &self.scope_open,
                &self.scope_close,
            ).map_err(LiteralError)?.join(" ")),
            StructuredValue::ApproximateNumber { value, tolerance } => {
                self.render_approximate_number_value(value, tolerance.as_ref())
            }
            StructuredValue::Digit(value) => Ok(self.digit_surface(*value)?.to_owned()),
            StructuredValue::DigitSequence(values) => {
                let mut tokens = vec![self.config.number.digit_sequence_marker.clone()];
                for digit in values {
                    tokens.push(self.digit_surface(*digit)?.to_owned());
                }
                Ok(tokens.join(" "))
            }
            StructuredValue::Unit(id) => self.units.unit_by_id(*id)
                .map(|unit| unit.spoken.clone())
                .ok_or_else(|| LiteralError(format!("unknown unit `{id}`"))),
            StructuredValue::Quantity { value, unit, approximate, uncertainty } => {
                self.quantity_to_spoken(&QuantityValue {
                    value: ExactNumber(value.clone()),
                    unit_id: *unit,
                    approximate: *approximate,
                    uncertainty: uncertainty.clone().map(ExactNumber),
                })
            }
            StructuredValue::Duration { seconds } => self.render_duration_spoken_value(seconds),
            value @ (
                StructuredValue::CalendarDate { .. }
                | StructuredValue::TimeOfDay { .. }
                | StructuredValue::TimeZone { .. }
                | StructuredValue::Instant { .. }
                | StructuredValue::Interval { .. }
            ) => self.render_temporal_spoken(value),
            StructuredValue::Information { .. } => Err(LiteralError(
                "information values are semantic slot values, not standalone structured-literal surfaces".into()
            )),
        }
    }

    fn parse_approximate_number(&self, tokens: &[String], start: usize) -> Result<Option<LiteralMatch>, LiteralError> {
        let token = &tokens[start];
        if let Some(value_source) = token.strip_prefix('~') {
            if value_source.is_empty() {
                return Ok(None);
            }
            let value = ExactNumber::parse_written(value_source, self.config.number.max_explicit_exponent)
                .map_err(LiteralError)?;
            return self.approximate_number_match(value, None, 1, LiteralRealization::Written).map(Some);
        }
        if let Some((value_source, tolerance_source)) = token.split_once('±') {
            let value = ExactNumber::parse_written(value_source, self.config.number.max_explicit_exponent)
                .map_err(LiteralError)?;
            let tolerance = ExactNumber::parse_written(tolerance_source, self.config.number.max_explicit_exponent)
                .map_err(LiteralError)?;
            return self.approximate_number_match(value, Some(tolerance), 1, LiteralRealization::Written).map(Some);
        }
        if token != &self.config.number.approximation {
            return Ok(None);
        }
        if tokens.get(start + 1).is_some_and(|value| value == &self.scope_open) {
            let (value_tokens, after_value) = take_scope_group(tokens, start + 1, &self.scope_open, &self.scope_close)?
                .ok_or_else(|| LiteralError("approximate number requires a scoped value".into()))?;
            let value = parse_spoken_number(&value_tokens, 0, &self.config.number, &self.scope_open, &self.scope_close)
                .map_err(LiteralError)?
                .ok_or_else(|| LiteralError("approximate number requires a numeric value".into()))?;
            if value.consumed != value_tokens.len() {
                return Err(LiteralError("approximate number value scope contains trailing tokens".into()));
            }
            let (tolerance_tokens, after_tolerance) = take_scope_group(tokens, after_value, &self.scope_open, &self.scope_close)?
                .ok_or_else(|| LiteralError("scoped approximate number requires an explicit tolerance scope".into()))?;
            let tolerance = parse_spoken_number(&tolerance_tokens, 0, &self.config.number, &self.scope_open, &self.scope_close)
                .map_err(LiteralError)?
                .ok_or_else(|| LiteralError("approximate number tolerance must be numeric".into()))?;
            if tolerance.consumed != tolerance_tokens.len() {
                return Err(LiteralError("approximate number tolerance scope contains trailing tokens".into()));
            }
            return self.approximate_number_match(
                value.value,
                Some(tolerance.value),
                after_tolerance - start,
                LiteralRealization::Spoken,
            ).map(Some);
        }
        let Some(value) = parse_spoken_number(
            tokens,
            start + 1,
            &self.config.number,
            &self.scope_open,
            &self.scope_close,
        ).map_err(LiteralError)? else {
            return Err(LiteralError(format!("`{}` requires a numeric value", self.config.number.approximation)));
        };
        self.approximate_number_match(
            value.value,
            None,
            value.consumed + 1,
            LiteralRealization::Spoken,
        ).map(Some)
    }

    fn approximate_number_match(
        &self,
        value: ExactNumber,
        tolerance: Option<ExactNumber>,
        consumed: usize,
        realization: LiteralRealization,
    ) -> Result<LiteralMatch, LiteralError> {
        if tolerance.as_ref().is_some_and(|value| value.0.is_negative()) {
            return Err(LiteralError("approximation tolerance cannot be negative".into()));
        }
        let canonical_written = match &tolerance {
            Some(tolerance) => format!("{}±{}", value.canonical_written(), tolerance.canonical_written()),
            None => format!("~{}", value.canonical_written()),
        };
        let canonical_spoken = match &tolerance {
            Some(tolerance) => {
                let mut out = vec![self.config.number.approximation.clone(), self.scope_open.clone()];
                out.extend(canonical_spoken_number(&value, &self.config.number, &self.scope_open, &self.scope_close).map_err(LiteralError)?);
                out.push(self.scope_close.clone());
                out.push(self.scope_open.clone());
                out.extend(canonical_spoken_number(tolerance, &self.config.number, &self.scope_open, &self.scope_close).map_err(LiteralError)?);
                out.push(self.scope_close.clone());
                out.join(" ")
            }
            None => {
                let mut out = vec![self.config.number.approximation.clone()];
                out.extend(canonical_spoken_number(&value, &self.config.number, &self.scope_open, &self.scope_close).map_err(LiteralError)?);
                out.join(" ")
            }
        };
        Ok(LiteralMatch {
            literal: SurfaceLiteral {
                semantic: StructuredLiteral::new(
                    StructuredValue::ApproximateNumber {
                        value: value.0.clone(),
                        tolerance: tolerance.as_ref().map(|value| value.0.clone()),
                    },
                    self.approximate_number_type.clone(),
                ),
                canonical_written,
                canonical_spoken,
                realization,
            },
            consumed,
        })
    }

    fn render_approximate_number_value(
        &self,
        value: &crate::rational::ExactRational,
        tolerance: Option<&crate::rational::ExactRational>,
    ) -> Result<String, LiteralError> {
        let value = ExactNumber(value.clone());
        let tolerance = tolerance.cloned().map(ExactNumber);
        Ok(match tolerance.as_ref() {
            Some(tolerance) => {
                let mut out = vec![self.config.number.approximation.clone(), self.scope_open.clone()];
                out.extend(canonical_spoken_number(&value, &self.config.number, &self.scope_open, &self.scope_close).map_err(LiteralError)?);
                out.push(self.scope_close.clone());
                out.push(self.scope_open.clone());
                out.extend(canonical_spoken_number(tolerance, &self.config.number, &self.scope_open, &self.scope_close).map_err(LiteralError)?);
                out.push(self.scope_close.clone());
                out.join(" ")
            }
            None => {
                let mut out = vec![self.config.number.approximation.clone()];
                out.extend(canonical_spoken_number(&value, &self.config.number, &self.scope_open, &self.scope_close).map_err(LiteralError)?);
                out.join(" ")
            }
        })
    }

    fn number_match(&self, value: ExactNumber, consumed: usize, realization: LiteralRealization) -> Result<LiteralMatch, LiteralError> {
        let canonical_written = value.canonical_written();
        let canonical_spoken = canonical_spoken_number(&value, &self.config.number, &self.scope_open, &self.scope_close).map_err(LiteralError)?.join(" ");
        Ok(LiteralMatch {
            literal: SurfaceLiteral {
                semantic: StructuredLiteral::new(StructuredValue::Number(value.0.clone()), self.number_type.clone()),
                canonical_written,
                canonical_spoken,
                realization,
            },
            consumed,
        })
    }

    fn parse_digit_sequence(&self, tokens: &[String], start: usize) -> Result<LiteralMatch, LiteralError> {
        let Some(first) = tokens.get(start + 1) else {
            return Err(LiteralError(format!(
                "`{}` requires at least one digit",
                self.config.number.digit_sequence_marker
            )));
        };

        if !first.is_empty() && first.chars().all(|value| value.is_ascii_digit()) {
            let mut spoken = vec![self.config.number.digit_sequence_marker.clone()];
            for value in first.chars() {
                spoken.push(self.digit_surface(value.to_digit(10).expect("ASCII digit") as u8)?.to_owned());
            }
            return Ok(LiteralMatch {
                literal: SurfaceLiteral {
                    semantic: StructuredLiteral::new(
                        StructuredValue::DigitSequence(first.chars().map(|value| value.to_digit(10).expect("ASCII digit") as u8).collect()),
                        self.digit_sequence_type.clone(),
                    ),
                    canonical_written: format!("{} {}", self.config.number.digit_sequence_marker, first),
                    canonical_spoken: spoken.join(" "),
                    realization: LiteralRealization::Written,
                },
                consumed: 2,
            });
        }

        let mut index = start + 1;
        let mut digits = String::new();
        while let Some(token) = tokens.get(index) {
            let Some(value) = self.config.number.digits.get(token) else { break };
            digits.push(char::from(b'0' + *value));
            index += 1;
        }
        if digits.is_empty() {
            return Err(LiteralError(format!(
                "`{}` requires decimal digit words or one written digit sequence",
                self.config.number.digit_sequence_marker
            )));
        }
        let mut values = vec![self.config.number.digit_sequence_marker.clone()];
        values.extend(tokens[start + 1..index].iter().cloned());
        Ok(LiteralMatch {
            literal: SurfaceLiteral {
                semantic: StructuredLiteral::new(StructuredValue::DigitSequence(digits.chars().map(|value| value.to_digit(10).expect("digit") as u8).collect()), self.digit_sequence_type.clone()),
                canonical_written: format!("{} {}", self.config.number.digit_sequence_marker, digits),
                canonical_spoken: values.join(" "),
                realization: LiteralRealization::Spoken,
            },
            consumed: index - start,
        })
    }

    fn parse_unit(&self, tokens: &[String], start: usize) -> Result<Option<LiteralMatch>, LiteralError> {
        let (unit, realization) = if let Some(unit) = self.units.unit_by_symbol(&tokens[start]) {
            (unit, LiteralRealization::Written)
        } else if let Some(unit) = self.units.unit_by_spoken(&tokens[start]) {
            (unit, LiteralRealization::Spoken)
        } else {
            return Ok(None);
        };
        Ok(Some(LiteralMatch {
            literal: SurfaceLiteral {
                semantic: StructuredLiteral::new(StructuredValue::Unit(unit.id), self.units.unit_type(unit)),
                canonical_written: unit.symbol.clone(),
                canonical_spoken: unit.spoken.clone(),
                realization,
            },
            consumed: 1,
        }))
    }

    fn parse_quantity(&self, tokens: &[String], start: usize) -> Result<Option<LiteralMatch>, LiteralError> {
        if let Some(result) = self.parse_written_quantity(tokens, start)? {
            return Ok(Some(result));
        }
        self.parse_spoken_quantity(tokens, start)
    }

    fn parse_written_quantity(&self, tokens: &[String], start: usize) -> Result<Option<LiteralMatch>, LiteralError> {
        let Some(value_token) = tokens.get(start) else { return Ok(None) };
        let Some(unit_token) = tokens.get(start + 1) else { return Ok(None) };
        let Some(unit) = self.units.unit_by_symbol(unit_token) else { return Ok(None) };

        let (value_source, approximate, uncertainty_source) = if let Some(value) = value_token.strip_prefix('~') {
            (value, true, None)
        } else if let Some((value, uncertainty)) = value_token.split_once('±') {
            (value, true, Some(uncertainty))
        } else {
            (value_token.as_str(), false, None)
        };
        let value = match ExactNumber::parse_written(value_source, self.config.number.max_explicit_exponent) {
            Ok(value) => value,
            Err(_) => return Ok(None),
        };
        let uncertainty = uncertainty_source
            .map(|source| ExactNumber::parse_written(source, self.config.number.max_explicit_exponent).map_err(LiteralError))
            .transpose()?;
        let quantity = QuantityValue { value, unit_id: unit.id.clone(), approximate, uncertainty };
        self.quantity_match(quantity, 2, LiteralRealization::Written).map(Some)
    }

    fn parse_spoken_quantity(&self, tokens: &[String], start: usize) -> Result<Option<LiteralMatch>, LiteralError> {
        if tokens.get(start).is_some_and(|token| token == &self.config.number.approximation)
            && tokens.get(start + 1).is_some_and(|token| token == &self.scope_open)
        {
            if let Some(quantity) = self.parse_spoken_uncertain_quantity(tokens, start)? {
                return Ok(Some(quantity));
            }
            return Ok(None);
        }

        let approximate = tokens.get(start).is_some_and(|token| token == &self.config.number.approximation);
        let number_start = start + usize::from(approximate);
        let Some(number) = parse_spoken_number(
            tokens, number_start, &self.config.number, &self.scope_open, &self.scope_close
        ).map_err(LiteralError)? else { return Ok(None) };
        let unit_index = number_start + number.consumed;
        let Some(unit) = tokens.get(unit_index).and_then(|surface| self.units.unit_by_spoken(surface)) else {
            return Ok(None);
        };
        let quantity = QuantityValue { value: number.value, unit_id: unit.id.clone(), approximate, uncertainty: None };
        self.quantity_match(quantity, unit_index + 1 - start, LiteralRealization::Spoken).map(Some)
    }

    fn parse_spoken_uncertain_quantity(&self, tokens: &[String], start: usize) -> Result<Option<LiteralMatch>, LiteralError> {
        let mut index = start + 1;
        require(tokens, &mut index, &self.scope_open, "approximate quantity value")?;
        let value = parse_spoken_number(tokens, index, &self.config.number, &self.scope_open, &self.scope_close)
            .map_err(LiteralError)?.ok_or_else(|| LiteralError("approximate quantity requires numeric value".into()))?;
        index += value.consumed;
        require(tokens, &mut index, &self.scope_close, "approximate quantity value")?;
        require(tokens, &mut index, &self.scope_open, "approximate quantity uncertainty")?;
        let uncertainty = parse_spoken_number(tokens, index, &self.config.number, &self.scope_open, &self.scope_close)
            .map_err(LiteralError)?.ok_or_else(|| LiteralError("approximate quantity requires numeric uncertainty".into()))?;
        index += uncertainty.consumed;
        require(tokens, &mut index, &self.scope_close, "approximate quantity uncertainty")?;
        let Some(unit) = tokens.get(index).and_then(|surface| self.units.unit_by_spoken(surface)) else {
            // `apro ki VALUE ku ki TOLERANCE ku` is also the complete spoken form of an
            // Approximate<Number>. Do not commit to the quantity grammar until a unit
            // follows the two numeric scopes; the number parser gets the same token
            // stream if no unit is present.
            return Ok(None);
        };
        index += 1;
        self.quantity_match(
            QuantityValue {
                value: value.value,
                unit_id: unit.id.clone(),
                approximate: true,
                uncertainty: Some(uncertainty.value),
            },
            index - start,
            LiteralRealization::Spoken,
        )
        .map(Some)
    }

    fn quantity_match(&self, quantity: QuantityValue, consumed: usize, realization: LiteralRealization) -> Result<LiteralMatch, LiteralError> {
        if quantity.uncertainty.as_ref().is_some_and(|value| value.0.is_negative()) {
            return Err(LiteralError("measurement uncertainty cannot be negative".into()));
        }
        let unit = self.units.unit_by_id(quantity.unit_id)
            .ok_or_else(|| LiteralError(format!("unknown unit `{}`", quantity.unit_id)))?;
        let canonical_written = if let Some(uncertainty) = &quantity.uncertainty {
            format!("{}±{} {}", quantity.value.canonical_written(), uncertainty.canonical_written(), unit.symbol)
        } else if quantity.approximate {
            format!("~{} {}", quantity.value.canonical_written(), unit.symbol)
        } else {
            format!("{} {}", quantity.value.canonical_written(), unit.symbol)
        };
        let canonical_spoken = self.quantity_to_spoken(&quantity)?;
        Ok(LiteralMatch {
            literal: SurfaceLiteral {
                semantic: StructuredLiteral::new(
                    StructuredValue::Quantity {
                        value: quantity.value.0.clone(),
                        unit: quantity.unit_id,
                        approximate: quantity.approximate,
                        uncertainty: quantity.uncertainty.as_ref().map(|value| value.0.clone()),
                    },
                    self.units.quantity_type(unit, quantity.approximate),
                ),
                canonical_written,
                canonical_spoken,
                realization,
            },
            consumed,
        })
    }

    fn quantity_to_spoken(&self, quantity: &QuantityValue) -> Result<String, LiteralError> {
        let unit = self.units.unit_by_id(quantity.unit_id)
            .ok_or_else(|| LiteralError(format!("unknown unit `{}`", quantity.unit_id)))?;
        if let Some(uncertainty) = &quantity.uncertainty {
            let mut out = vec![self.config.number.approximation.clone(), self.scope_open.clone()];
            out.extend(canonical_spoken_number(&quantity.value, &self.config.number, &self.scope_open, &self.scope_close).map_err(LiteralError)?);
            out.push(self.scope_close.clone());
            out.push(self.scope_open.clone());
            out.extend(canonical_spoken_number(uncertainty, &self.config.number, &self.scope_open, &self.scope_close).map_err(LiteralError)?);
            out.push(self.scope_close.clone());
            out.push(unit.spoken.clone());
            return Ok(out.join(" "));
        }
        let mut out = Vec::new();
        if quantity.approximate { out.push(self.config.number.approximation.clone()); }
        out.extend(canonical_spoken_number(&quantity.value, &self.config.number, &self.scope_open, &self.scope_close).map_err(LiteralError)?);
        out.push(unit.spoken.clone());
        Ok(out.join(" "))
    }

    fn temporal_match(&self, value: TemporalLiteralValue, consumed: usize, realization: LiteralRealization) -> Result<LiteralMatch, LiteralError> {
        let structured = temporal_to_structured(&value);
        let ty = match &structured {
            StructuredValue::CalendarDate { .. } => self.date_type.clone(),
            StructuredValue::TimeOfDay { .. } => self.time_of_day_type.clone(),
            StructuredValue::TimeZone { .. } => self.timezone_type.clone(),
            StructuredValue::Instant { .. } => self.instant_type.clone(),
            StructuredValue::Duration { .. } => self.duration_type.clone(),
            StructuredValue::Interval { .. } => self.interval_type.clone(),
            _ => unreachable!("temporal parser produced a non-temporal semantic value"),
        };
        let semantic = StructuredLiteral::new(structured, ty);
        let canonical_written = self.render_written(&semantic)?;
        let canonical_spoken = self.render_spoken(&semantic)?;
        Ok(LiteralMatch {
            literal: SurfaceLiteral {
                semantic,
                canonical_written,
                canonical_spoken,
                realization,
            },
            consumed,
        })
    }

    fn parse_spoken_timezone(&self, tokens: &[String], start: usize) -> Result<Option<LiteralMatch>, LiteralError> {
        let mut index = start + 1;
        let Some(next) = tokens.get(index) else { return Err(LiteralError("timezone marker requires offset".into())) };
        let offset_minutes = if self.config.number.digits.get(next) == Some(&0) {
            index += 1;
            0i16
        } else {
            let negative = next == &self.config.number.minus;
            let positive = next == &self.config.number.plus;
            if !negative && !positive { return Ok(None); }
            index += 1;
            let Some((hours, consumed)) = parse_spoken_integer(tokens, index, &self.config.number).map_err(LiteralError)? else {
                return Err(LiteralError("timezone offset requires hours".into()));
            };
            index += consumed;
            require(tokens, &mut index, &self.config.number.decimal, "timezone offset")?;
            let Some((minutes, consumed)) = parse_spoken_integer(tokens, index, &self.config.number).map_err(LiteralError)? else {
                return Err(LiteralError("timezone offset requires minutes".into()));
            };
            index += consumed;
            let hours = hours.to_i16().ok_or_else(|| LiteralError("timezone hours outside range".into()))?;
            let minutes = minutes.to_i16().ok_or_else(|| LiteralError("timezone minutes outside range".into()))?;
            if hours > 14 || minutes > 59 || (hours == 14 && minutes != 0) {
                return Err(LiteralError("timezone offset outside ±14:00".into()));
            }
            let value = hours * 60 + minutes;
            if negative { -value } else { value }
        };
        self.temporal_match(
            TemporalLiteralValue::TimeZone { offset_minutes },
            index - start,
            LiteralRealization::Spoken,
        ).map(Some)
    }

    fn parse_spoken_interval(&self, tokens: &[String], start: usize) -> Result<Option<LiteralMatch>, LiteralError> {
        if tokens.get(start + 1).is_none_or(|token| token != &self.scope_open) {
            return Ok(None);
        }
        let Some((left_tokens, mut index)) = take_scope_group(tokens, start + 1, &self.scope_open, &self.scope_close)? else {
            return Ok(None);
        };
        if left_tokens.first().is_none_or(|token| token != &self.config.calendar.date_marker) {
            return Ok(None);
        }
        let Some((right_tokens, next)) = take_scope_group(tokens, index, &self.scope_open, &self.scope_close)? else {
            return Ok(None);
        };
        index = next;
        if right_tokens.first().is_none_or(|token| token != &self.config.calendar.date_marker) {
            return Ok(None);
        }
        let left = self.parse_spoken_instant(&left_tokens, 0)?
            .ok_or_else(|| LiteralError("interval start must be a complete instant".into()))?;
        let right = self.parse_spoken_instant(&right_tokens, 0)?
            .ok_or_else(|| LiteralError("interval end must be a complete instant".into()))?;
        if left.consumed != left_tokens.len() || right.consumed != right_tokens.len() {
            return Err(LiteralError("interval instant scopes contain trailing tokens".into()));
        }
        let left_value = structured_to_temporal(&left.literal.semantic.value)?;
        let right_value = structured_to_temporal(&right.literal.semantic.value)?;
        let value = TemporalLiteralValue::Interval {
            start: Box::new(left_value),
            end: Box::new(right_value),
        };
        self.temporal_match(value, index - start, LiteralRealization::Spoken).map(Some)
    }

    fn parse_spoken_instant(&self, tokens: &[String], start: usize) -> Result<Option<LiteralMatch>, LiteralError> {
        if tokens.get(start).is_none_or(|token| token != &self.config.calendar.date_marker)
            || tokens.get(start + 1).is_none_or(|token| token != &self.scope_open)
        {
            return Ok(None);
        }
        let mut index = start + 1;
        let mut fields = Vec::with_capacity(6);
        for _ in 0..6 {
            let Some((field_tokens, next)) = take_scope_group(tokens, index, &self.scope_open, &self.scope_close)? else {
                return Ok(None);
            };
            let Some((value, consumed)) = parse_spoken_integer(&field_tokens, 0, &self.config.number).map_err(LiteralError)? else {
                return Ok(None);
            };
            if consumed != field_tokens.len() {
                return Err(LiteralError("instant field must contain exactly one canonical integer".into()));
            }
            fields.push(value);
            index = next;
        }
        if tokens.get(index).is_none_or(|token| token != &self.config.calendar.timezone_marker) {
            return Ok(None);
        }
        let zone = self.parse_spoken_timezone(tokens, index)?
            .ok_or_else(|| LiteralError("invalid instant timezone".into()))?;
        let StructuredValue::TimeZone { offset_minutes } = &zone.literal.semantic.value else {
            return Err(LiteralError("spoken instant timezone did not produce a typed timezone value".into()));
        };
        index += zone.consumed;
        let value = TemporalLiteralValue::Instant {
            year: to_i32(&fields[0], "year")?,
            month: to_u8(&fields[1], "month")?,
            day: to_u8(&fields[2], "day")?,
            hour: to_u8(&fields[3], "hour")?,
            minute: to_u8(&fields[4], "minute")?,
            second: to_u8(&fields[5], "second")?,
            offset_minutes: *offset_minutes,
        };
        self.temporal_match(value, index - start, LiteralRealization::Spoken).map(Some)
    }

    fn parse_spoken_duration(&self, tokens: &[String], start: usize) -> Result<Option<LiteralMatch>, LiteralError> {
        if tokens.get(start + 1).is_none_or(|token| token != &self.scope_open) { return Ok(None); }
        let mut index = start + 2;
        let Some(number) = parse_spoken_number(tokens, index, &self.config.number, &self.scope_open, &self.scope_close).map_err(LiteralError)? else { return Ok(None) };
        index += number.consumed;
        if tokens.get(index).is_none_or(|token| token != &self.scope_close) { return Ok(None); }
        index += 1;
        let Some(unit) = tokens.get(index).and_then(|surface| self.units.unit_by_spoken(surface)) else { return Ok(None) };
        if unit.dimension != self.duration_dimension { return Ok(None); }
        index += 1;
        let seconds = self.units.convert(
            &QuantityValue { value: number.value, unit_id: unit.id, approximate: false, uncertainty: None },
            self.duration_base_unit,
        ).map_err(LiteralError)?.value;
        if seconds.0.is_negative() {
            return Err(LiteralError("duration cannot be negative".into()));
        }
        let semantic = StructuredLiteral::new(
            StructuredValue::Duration { seconds: seconds.0.clone() },
            self.duration_type.clone(),
        );
        let canonical_written = self.render_written(&semantic)?;
        let canonical_spoken = self.render_spoken(&semantic)?;
        let literal = SurfaceLiteral { semantic, canonical_written, canonical_spoken, realization: LiteralRealization::Spoken };
        Ok(Some(LiteralMatch { literal, consumed: index - start }))
    }

    fn render_duration_spoken_value(&self, seconds: &crate::rational::ExactRational) -> Result<String, LiteralError> {
        let source = QuantityValue {
            value: ExactNumber(seconds.clone()),
            unit_id: self.duration_base_unit,
            approximate: false,
            uncertainty: None,
        };
        let mut candidates = Vec::new();
        for unit in self.units.units().filter(|unit| unit.dimension == self.duration_dimension) {
            let converted = self.units.convert(&source, unit.id).map_err(LiteralError)?;
            let number_tokens = canonical_spoken_number(
                &converted.value,
                &self.config.number,
                &self.scope_open,
                &self.scope_close,
            ).map_err(LiteralError)?;
            candidates.push((number_tokens.len(), unit.scale.clone(), unit.id, number_tokens, unit.spoken.clone()));
        }
        candidates.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| right.1.cmp(&left.1))
                .then_with(|| left.2.cmp(&right.2))
        });
        let Some((_, _, _, number_tokens, unit_spoken)) = candidates.into_iter().next() else {
            return Err(LiteralError("units package has no units in the duration dimension".into()));
        };
        let mut out = vec![self.config.calendar.date_marker.clone(), self.scope_open.clone()];
        out.extend(number_tokens);
        out.push(self.scope_close.clone());
        out.push(unit_spoken);
        Ok(out.join(" "))
    }

    fn render_temporal_spoken(&self, value: &StructuredValue) -> Result<String, LiteralError> {
        let temporal = structured_to_temporal(value)?;
        Ok(time::canonical_spoken_temporal(
            &temporal,
            &self.config.number,
            &self.config.calendar.date_marker,
            &self.config.calendar.timezone_marker,
            &self.scope_open,
            &self.scope_close,
        ).map_err(LiteralError)?.join(" "))
    }

    fn digit_surface(&self, digit: u8) -> Result<&str, LiteralError> {
        self.config.number.digits.iter()
            .find_map(|(surface, value)| (*value == digit).then_some(surface.as_str()))
            .ok_or_else(|| LiteralError(format!("missing digit surface for {digit}")))
    }
}

impl StructuredLiteralCodec for LiteralEngine {
    fn parse_at(&self, tokens: &[String], start: usize) -> Result<Option<LiteralMatch>, LiteralError> {
        LiteralEngine::parse_at(self, tokens, start)
    }

    fn render_written(&self, literal: &StructuredLiteral) -> Result<String, LiteralError> {
        LiteralEngine::render_written(self, literal)
    }

    fn render_spoken(&self, literal: &StructuredLiteral) -> Result<String, LiteralError> {
        LiteralEngine::render_spoken(self, literal)
    }
}


fn temporal_to_structured(value: &TemporalLiteralValue) -> StructuredValue {
    match value {
        TemporalLiteralValue::CalendarDate { year, month, day } => StructuredValue::CalendarDate {
            year: *year,
            month: *month,
            day: *day,
        },
        TemporalLiteralValue::TimeOfDay { hour, minute, second } => StructuredValue::TimeOfDay {
            hour: *hour,
            minute: *minute,
            second: *second,
        },
        TemporalLiteralValue::TimeZone { offset_minutes } => StructuredValue::TimeZone {
            offset_minutes: *offset_minutes,
        },
        TemporalLiteralValue::Instant {
            year, month, day, hour, minute, second, offset_minutes,
        } => StructuredValue::Instant {
            year: *year,
            month: *month,
            day: *day,
            hour: *hour,
            minute: *minute,
            second: *second,
            offset_minutes: *offset_minutes,
        },
        TemporalLiteralValue::DurationSeconds { seconds } => StructuredValue::Duration {
            seconds: seconds.0.clone(),
        },
        TemporalLiteralValue::Interval { start, end } => StructuredValue::Interval {
            start: Box::new(temporal_to_structured(start)),
            end: Box::new(temporal_to_structured(end)),
        },
    }
}

fn structured_to_temporal(value: &StructuredValue) -> Result<TemporalLiteralValue, LiteralError> {
    Ok(match value {
        StructuredValue::CalendarDate { year, month, day } => TemporalLiteralValue::CalendarDate {
            year: *year,
            month: *month,
            day: *day,
        },
        StructuredValue::TimeOfDay { hour, minute, second } => TemporalLiteralValue::TimeOfDay {
            hour: *hour,
            minute: *minute,
            second: *second,
        },
        StructuredValue::TimeZone { offset_minutes } => TemporalLiteralValue::TimeZone {
            offset_minutes: *offset_minutes,
        },
        StructuredValue::Instant {
            year, month, day, hour, minute, second, offset_minutes,
        } => TemporalLiteralValue::Instant {
            year: *year,
            month: *month,
            day: *day,
            hour: *hour,
            minute: *minute,
            second: *second,
            offset_minutes: *offset_minutes,
        },
        StructuredValue::Duration { seconds } => TemporalLiteralValue::DurationSeconds {
            seconds: ExactNumber(seconds.clone()),
        },
        StructuredValue::Interval { start, end } => TemporalLiteralValue::Interval {
            start: Box::new(structured_to_temporal(start)?),
            end: Box::new(structured_to_temporal(end)?),
        },
        other => return Err(LiteralError(format!(
            "structured value family `{}` is not temporal",
            other.family()
        ))),
    })
}

fn temporal_written(value: &StructuredValue) -> Result<String, LiteralError> {
    Ok(structured_to_temporal(value)?.canonical_written())
}

fn parse_config_type(source: &str) -> Result<Type, LiteralError> {
    parse_type(source).map_err(|errors| LiteralError(format!(
        "invalid structured-literal semantic type `{source}`: {}",
        errors.into_iter().map(|error| error.to_string()).collect::<Vec<_>>().join("; ")
    )))
}

fn require(tokens: &[String], index: &mut usize, expected: &str, context: &str) -> Result<(), LiteralError> {
    match tokens.get(*index) {
        Some(token) if token == expected => { *index += 1; Ok(()) }
        Some(token) => Err(LiteralError(format!("{context} expected `{expected}`, found `{token}`"))),
        None => Err(LiteralError(format!("{context} expected `{expected}`, found end of input"))),
    }
}

fn take_scope_group(
    tokens: &[String],
    start: usize,
    open: &str,
    close: &str,
) -> Result<Option<(Vec<String>, usize)>, LiteralError> {
    if tokens.get(start).is_none_or(|token| token != open) {
        return Ok(None);
    }
    let mut depth = 1usize;
    let mut index = start + 1;
    let mut body = Vec::new();
    while let Some(token) = tokens.get(index) {
        if token == open {
            depth += 1;
            body.push(token.clone());
        } else if token == close {
            depth -= 1;
            if depth == 0 {
                return Ok(Some((body, index + 1)));
            }
            body.push(token.clone());
        } else {
            body.push(token.clone());
        }
        index += 1;
    }
    Err(LiteralError(format!("missing structural closer `{close}` in structured literal")))
}

fn to_i32(value: &BigInt, label: &str) -> Result<i32, LiteralError> {
    value.to_i32().ok_or_else(|| LiteralError(format!("{label} outside supported range")))
}

fn to_u8(value: &BigInt, label: &str) -> Result<u8, LiteralError> {
    value.to_u8().ok_or_else(|| LiteralError(format!("{label} outside supported range")))
}

impl fmt::Display for LiteralError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) }
}
impl std::error::Error for LiteralError {}
