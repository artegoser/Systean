use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LiteralConfig {
    pub meta: Option<MetaConfig>,
    pub number: NumberConfig,
    pub calendar: CalendarConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MetaConfig {
    pub description: Option<String>,
    pub version: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NumberConfig {
    pub semantic_type: String,
    pub digit_type: String,
    pub digit_sequence_type: String,
    pub digit_sequence_marker: String,
    pub minus: String,
    pub plus: String,
    pub decimal: String,
    pub rational: String,
    pub exponent: String,
    pub approximation: String,
    pub ordinal: String,
    pub max_explicit_exponent: u32,
    pub max_inline_fraction_digits: usize,
    pub max_inline_leading_fraction_zeros: usize,
    pub digits: BTreeMap<String, u8>,
    pub coefficient_magnitudes: BTreeMap<String, u32>,
    pub outer_magnitudes: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CalendarConfig {
    pub date_type: String,
    pub time_of_day_type: String,
    pub instant_type: String,
    pub interval_type: String,
    pub duration_type: String,
    pub timezone_type: String,
    pub date_marker: String,
    pub timezone_marker: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiteralConfigError {
    Toml(String),
    Invalid(String),
}

impl LiteralConfig {
    pub fn from_toml(source: &str) -> Result<Self, LiteralConfigError> {
        let config: Self = toml::from_str(source)
            .map_err(|error| LiteralConfigError::Toml(error.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), LiteralConfigError> {
        if self.number.max_inline_fraction_digits == 0 {
            return Err(LiteralConfigError::Invalid(
                "max_inline_fraction_digits must be greater than zero".into(),
            ));
        }
        if self.number.digits.len() != 10 {
            return Err(LiteralConfigError::Invalid(
                "number.digits must declare exactly ten spoken digit forms".into(),
            ));
        }
        let mut values = self.number.digits.values().copied().collect::<Vec<_>>();
        values.sort_unstable();
        if values != (0u8..=9).collect::<Vec<_>>() {
            return Err(LiteralConfigError::Invalid(
                "number.digits must map bijectively to 0..9".into(),
            ));
        }
        let coefficient_exponents = self
            .number
            .coefficient_magnitudes
            .values()
            .copied()
            .collect::<BTreeSet<_>>();
        if coefficient_exponents != BTreeSet::from([1, 2])
            || self.number.coefficient_magnitudes.len() != 2
        {
            return Err(LiteralConfigError::Invalid(
                "coefficient magnitudes must bijectively denote decimal exponents 1 and 2".into(),
            ));
        }
        if self.number.outer_magnitudes.is_empty()
            || self.number.outer_magnitudes.values().any(|value| *value < 3 || value % 3 != 0)
        {
            return Err(LiteralConfigError::Invalid(
                "outer magnitudes must be non-empty positive multiples of three >= 3".into(),
            ));
        }
        let mut exponents = self.number.outer_magnitudes.values().copied().collect::<Vec<_>>();
        exponents.sort_unstable();
        exponents.dedup();
        if exponents.len() != self.number.outer_magnitudes.len() {
            return Err(LiteralConfigError::Invalid(
                "outer magnitude exponents must be unique".into(),
            ));
        }
        let raw_forms = self.raw_reserved_forms();
        let unique_forms = raw_forms.iter().copied().collect::<BTreeSet<_>>();
        if unique_forms.len() != raw_forms.len() {
            return Err(LiteralConfigError::Invalid(
                "structured-literal spoken forms must be globally unique".into(),
            ));
        }
        for form in self.reserved_forms() {
            if form.trim().is_empty() || form.chars().any(char::is_whitespace) {
                return Err(LiteralConfigError::Invalid(format!(
                    "structured-literal form `{form}` must be one non-empty token"
                )));
            }
        }
        Ok(())
    }

    fn raw_reserved_forms(&self) -> Vec<&str> {
        let mut forms = vec![
            self.number.digit_sequence_marker.as_str(),
            self.number.minus.as_str(),
            self.number.plus.as_str(),
            self.number.decimal.as_str(),
            self.number.rational.as_str(),
            self.number.exponent.as_str(),
            self.number.approximation.as_str(),
            self.number.ordinal.as_str(),
            self.calendar.date_marker.as_str(),
            self.calendar.timezone_marker.as_str(),
        ];
        forms.extend(self.number.digits.keys().map(String::as_str));
        forms.extend(self.number.coefficient_magnitudes.keys().map(String::as_str));
        forms.extend(self.number.outer_magnitudes.keys().map(String::as_str));
        forms
    }

    pub fn reserved_forms(&self) -> Vec<&str> {
        let mut forms = self.raw_reserved_forms();
        forms.sort_unstable();
        forms.dedup();
        forms
    }
}

impl std::fmt::Display for LiteralConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Toml(message) => write!(f, "literal config TOML: {message}"),
            Self::Invalid(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for LiteralConfigError {}
