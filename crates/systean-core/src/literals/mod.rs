mod config;
mod number;
mod time;

use std::collections::BTreeSet;
use std::fmt;

use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive};

pub use config::{CalendarConfig, LiteralConfig, LiteralConfigError, NumberConfig};
pub use number::{ExactNumber, NumberParse, canonical_spoken as canonical_spoken_number, integer_to_spoken, parse_spoken_integer, parse_spoken_number};
pub use time::TemporalLiteralValue;

use crate::semantics::{StructuredLiteral, Type};
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
    digit_type: Type,
    digit_sequence_type: Type,
    date_type: Type,
    time_of_day_type: Type,
    instant_type: Type,
    interval_type: Type,
    duration_type: Type,
    timezone_type: Type,
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
        let digit_type = parse_config_type(&config.number.digit_type)?;
        let digit_sequence_type = parse_config_type(&config.number.digit_sequence_type)?;
        let date_type = parse_config_type(&config.calendar.date_type)?;
        let time_of_day_type = parse_config_type(&config.calendar.time_of_day_type)?;
        let instant_type = parse_config_type(&config.calendar.instant_type)?;
        let interval_type = parse_config_type(&config.calendar.interval_type)?;
        let duration_type = parse_config_type(&config.calendar.duration_type)?;
        let timezone_type = parse_config_type(&config.calendar.timezone_type)?;
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
            digit_type,
            digit_sequence_type,
            date_type,
            time_of_day_type,
            instant_type,
            interval_type,
            duration_type,
            timezone_type,
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
        Ok(StructuredLiteral::new("digit", digit.to_string(), self.digit_type.clone()))
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
        if !matches!(literal.family.as_str(), "quantity" | "approximate_quantity") {
            return Err(LiteralError(format!(
                "cannot convert structured literal family `{}` as a quantity",
                literal.family
            )));
        }
        let source_tokens = literal
            .canonical
            .split_whitespace()
            .map(str::to_lowercase)
            .collect::<Vec<_>>();
        let source = self
            .parse_written_quantity(&source_tokens, 0)?
            .ok_or_else(|| LiteralError(format!("invalid canonical quantity `{}`", literal.canonical)))?;
        let quantity = self.quantity_value_from_canonical(&source.literal.semantic.canonical)?;
        let converted = self.units.convert(&quantity, target_unit).map_err(LiteralError)?;
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

        if let Some(value) = time::parse_written_temporal(&tokens[start]).map_err(LiteralError)? {
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
        match literal.family.as_str() {
            "number" => Ok(literal.canonical.clone()),
            "digit_sequence" => Ok(format!("{} {}", self.config.number.digit_sequence_marker, literal.canonical)),
            "unit" => self.units.unit_by_id(&literal.canonical)
                .map(|unit| unit.symbol.clone())
                .ok_or_else(|| LiteralError(format!("unknown unit `{}`", literal.canonical))),
            "quantity" | "approximate_quantity" => Ok(literal.canonical.clone()),
            "calendar_date" | "time_of_day" | "timezone" | "instant" | "duration" | "interval" => Ok(literal.canonical.clone()),
            family => Err(LiteralError(format!("no structured-literal renderer for family `{family}`"))),
        }
    }

    pub fn render_spoken(&self, literal: &StructuredLiteral) -> Result<String, LiteralError> {
        match literal.family.as_str() {
            "number" => {
                let value = ExactNumber::parse_written(&literal.canonical, self.config.number.max_explicit_exponent).map_err(LiteralError)?;
                Ok(canonical_spoken_number(&value, &self.config.number, &self.scope_open, &self.scope_close).map_err(LiteralError)?.join(" "))
            }
            "digit_sequence" => {
                let mut tokens = vec![self.config.number.digit_sequence_marker.clone()];
                for digit in literal.canonical.chars() {
                    let value = digit.to_digit(10).ok_or_else(|| LiteralError("invalid canonical digit sequence".into()))? as u8;
                    tokens.push(self.digit_surface(value)?.to_owned());
                }
                Ok(tokens.join(" "))
            }
            "unit" => self.units.unit_by_id(&literal.canonical)
                .map(|unit| unit.spoken.clone())
                .ok_or_else(|| LiteralError(format!("unknown unit `{}`", literal.canonical))),
            "quantity" | "approximate_quantity" => self.render_quantity_spoken(&literal.canonical),
            "calendar_date" | "time_of_day" | "timezone" | "instant" | "interval" => {
                let value = time::parse_written_temporal(&literal.canonical).map_err(LiteralError)?
                    .ok_or_else(|| LiteralError(format!("invalid canonical temporal literal `{}`", literal.canonical)))?;
                Ok(time::canonical_spoken_temporal(
                    &value,
                    &self.config.number,
                    &self.config.calendar.date_marker,
                    &self.config.calendar.timezone_marker,
                    &self.scope_open,
                    &self.scope_close,
                ).map_err(LiteralError)?.join(" "))
            }
            "duration" => self.render_duration_spoken(&literal.canonical),
            family => Err(LiteralError(format!("no structured-literal spoken renderer for family `{family}`"))),
        }
    }

    fn number_match(&self, value: ExactNumber, consumed: usize, realization: LiteralRealization) -> Result<LiteralMatch, LiteralError> {
        let canonical_written = value.canonical_written();
        let canonical_spoken = canonical_spoken_number(&value, &self.config.number, &self.scope_open, &self.scope_close).map_err(LiteralError)?.join(" ");
        Ok(LiteralMatch {
            literal: SurfaceLiteral {
                semantic: StructuredLiteral::new("number", canonical_written.clone(), self.number_type.clone()),
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
                        "digit_sequence",
                        first.clone(),
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
                semantic: StructuredLiteral::new("digit_sequence", digits.clone(), self.digit_sequence_type.clone()),
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
                semantic: StructuredLiteral::new("unit", unit.id.clone(), self.units.unit_type(unit)),
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
            return self.parse_spoken_uncertain_quantity(tokens, start).map(Some);
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

    fn parse_spoken_uncertain_quantity(&self, tokens: &[String], start: usize) -> Result<LiteralMatch, LiteralError> {
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
            return Err(LiteralError("approximate quantity requires a spoken unit form".into()));
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
    }

    fn quantity_value_from_canonical(&self, canonical: &str) -> Result<QuantityValue, LiteralError> {
        let tokens = canonical
            .split_whitespace()
            .map(str::to_lowercase)
            .collect::<Vec<_>>();
        let matched = self
            .parse_written_quantity(&tokens, 0)?
            .ok_or_else(|| LiteralError(format!("invalid canonical quantity `{canonical}`")))?;
        let value_token = tokens.first().expect("parsed quantity has value");
        let unit_token = tokens.get(1).expect("parsed quantity has unit");
        let unit = self.units.unit_by_symbol(unit_token)
            .ok_or_else(|| LiteralError(format!("unknown written quantity unit `{unit_token}`")))?;
        let (value_source, approximate, uncertainty_source) = if let Some(value) = value_token.strip_prefix('~') {
            (value, true, None)
        } else if let Some((value, uncertainty)) = value_token.split_once('±') {
            (value, true, Some(uncertainty))
        } else {
            (value_token.as_str(), false, None)
        };
        let value = ExactNumber::parse_written(value_source, self.config.number.max_explicit_exponent)
            .map_err(LiteralError)?;
        let uncertainty = uncertainty_source
            .map(|source| ExactNumber::parse_written(source, self.config.number.max_explicit_exponent).map_err(LiteralError))
            .transpose()?;
        debug_assert_eq!(matched.literal.semantic.family == "approximate_quantity", approximate);
        Ok(QuantityValue {
            value,
            unit_id: unit.id.clone(),
            approximate,
            uncertainty,
        })
    }

    fn quantity_match(&self, quantity: QuantityValue, consumed: usize, realization: LiteralRealization) -> Result<LiteralMatch, LiteralError> {
        if quantity.uncertainty.as_ref().is_some_and(|value| value.0.is_negative()) {
            return Err(LiteralError("measurement uncertainty cannot be negative".into()));
        }
        let unit = self.units.unit_by_id(&quantity.unit_id)
            .ok_or_else(|| LiteralError(format!("unknown unit `{}`", quantity.unit_id)))?;
        let canonical_written = if let Some(uncertainty) = &quantity.uncertainty {
            format!("{}±{} {}", quantity.value.canonical_written(), uncertainty.canonical_written(), unit.symbol)
        } else if quantity.approximate {
            format!("~{} {}", quantity.value.canonical_written(), unit.symbol)
        } else {
            format!("{} {}", quantity.value.canonical_written(), unit.symbol)
        };
        let canonical_spoken = self.quantity_to_spoken(&quantity)?;
        let family = if quantity.approximate { "approximate_quantity" } else { "quantity" };
        Ok(LiteralMatch {
            literal: SurfaceLiteral {
                semantic: StructuredLiteral::new(family, canonical_written.clone(), self.units.quantity_type(unit, quantity.approximate)),
                canonical_written,
                canonical_spoken,
                realization,
            },
            consumed,
        })
    }

    fn quantity_to_spoken(&self, quantity: &QuantityValue) -> Result<String, LiteralError> {
        let unit = self.units.unit_by_id(&quantity.unit_id)
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

    fn render_quantity_spoken(&self, canonical: &str) -> Result<String, LiteralError> {
        let tokens = canonical.split_whitespace().map(str::to_lowercase).collect::<Vec<_>>();
        self.parse_written_quantity(&tokens, 0)?
            .map(|matched| matched.literal.canonical_spoken)
            .ok_or_else(|| LiteralError(format!("invalid canonical quantity `{canonical}`")))
    }

    fn temporal_match(&self, value: TemporalLiteralValue, consumed: usize, realization: LiteralRealization) -> Result<LiteralMatch, LiteralError> {
        let value = match value {
            TemporalLiteralValue::DurationSeconds { seconds_canonical } => {
                let seconds = ExactNumber::parse_written(
                    &seconds_canonical,
                    self.config.number.max_explicit_exponent,
                )
                .map_err(LiteralError)?;
                if seconds.0.is_negative() {
                    return Err(LiteralError("duration cannot be negative".into()));
                }
                TemporalLiteralValue::DurationSeconds {
                    seconds_canonical: seconds.canonical_written(),
                }
            }
            other => other,
        };
        let (family, ty) = match value {
            TemporalLiteralValue::CalendarDate { .. } => ("calendar_date", self.date_type.clone()),
            TemporalLiteralValue::TimeOfDay { .. } => ("time_of_day", self.time_of_day_type.clone()),
            TemporalLiteralValue::TimeZone { .. } => ("timezone", self.timezone_type.clone()),
            TemporalLiteralValue::Instant { .. } => ("instant", self.instant_type.clone()),
            TemporalLiteralValue::DurationSeconds { .. } => ("duration", self.duration_type.clone()),
            TemporalLiteralValue::Interval { .. } => ("interval", self.interval_type.clone()),
        };
        let canonical_written = value.canonical_written();
        let canonical_spoken = match &value {
            TemporalLiteralValue::DurationSeconds { .. } => self.render_duration_spoken(&canonical_written)?,
            _ => time::canonical_spoken_temporal(
                &value, &self.config.number, &self.config.calendar.date_marker, &self.config.calendar.timezone_marker,
                &self.scope_open, &self.scope_close
            ).map_err(LiteralError)?.join(" "),
        };
        Ok(LiteralMatch {
            literal: SurfaceLiteral {
                semantic: StructuredLiteral::new(family, canonical_written.clone(), ty),
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
        let value = TemporalLiteralValue::Interval {
            start: left.literal.canonical_written,
            end: right.literal.canonical_written,
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
        let zone_written = zone.literal.canonical_written;
        index += zone.consumed;
        let written = format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}",
            to_i32(&fields[0], "year")?,
            to_u8(&fields[1], "month")?,
            to_u8(&fields[2], "day")?,
            to_u8(&fields[3], "hour")?,
            to_u8(&fields[4], "minute")?,
            to_u8(&fields[5], "second")?,
            zone_written,
        );
        let value = time::parse_written_temporal(&written).map_err(LiteralError)?
            .ok_or_else(|| LiteralError("constructed instant is invalid".into()))?;
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
        if unit.dimension != "time" { return Ok(None); }
        index += 1;
        let seconds = self.units.convert(
            &QuantityValue { value: number.value, unit_id: unit.id.clone(), approximate: false, uncertainty: None },
            "second",
        ).map_err(LiteralError)?.value;
        if seconds.0.is_negative() {
            return Err(LiteralError("duration cannot be negative".into()));
        }
        let canonical = format!("PT{}S", seconds.canonical_written());
        let literal = SurfaceLiteral {
            semantic: StructuredLiteral::new("duration", canonical.clone(), self.duration_type.clone()),
            canonical_written: canonical.clone(),
            canonical_spoken: self.render_duration_spoken(&canonical)?,
            realization: LiteralRealization::Spoken,
        };
        Ok(Some(LiteralMatch { literal, consumed: index - start }))
    }

    fn render_duration_spoken(&self, canonical: &str) -> Result<String, LiteralError> {
        if !canonical.starts_with("PT") || !canonical.ends_with('S') {
            return Err(LiteralError(format!("invalid canonical duration `{canonical}`")));
        }
        let seconds = ExactNumber::parse_written(
            &canonical[2..canonical.len() - 1],
            self.config.number.max_explicit_exponent,
        ).map_err(LiteralError)?;
        let source = QuantityValue {
            value: seconds,
            unit_id: "second".into(),
            approximate: false,
            uncertainty: None,
        };
        if self.units.unit_by_id("second").is_none() {
            return Err(LiteralError("units package lacks canonical `second` unit".into()));
        }

        let mut candidates = Vec::new();
        for unit in self.units.config().units.iter().filter(|unit| unit.dimension == "time") {
            let converted = self.units.convert(&source, &unit.id).map_err(LiteralError)?;
            let number_tokens = canonical_spoken_number(
                &converted.value,
                &self.config.number,
                &self.scope_open,
                &self.scope_close,
            ).map_err(LiteralError)?;
            let scale = unit.scale().map_err(|error| LiteralError(error.to_string()))?;
            candidates.push((number_tokens.len(), scale, unit.id.clone(), number_tokens, unit.spoken.clone()));
        }
        candidates.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| right.1.cmp(&left.1))
                .then_with(|| left.2.cmp(&right.2))
        });
        let Some((_, _, _, number_tokens, unit_spoken)) = candidates.into_iter().next() else {
            return Err(LiteralError("units package has no time units".into()));
        };
        let mut out = vec![self.config.calendar.date_marker.clone(), self.scope_open.clone()];
        out.extend(number_tokens);
        out.push(self.scope_close.clone());
        out.push(unit_spoken);
        Ok(out.join(" "))
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
