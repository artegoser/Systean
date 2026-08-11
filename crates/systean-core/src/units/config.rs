use std::collections::{BTreeMap, BTreeSet};

use num_bigint::BigInt;
use crate::rational::ExactRational;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UnitsConfig {
    pub meta: Option<MetaConfig>,
    #[serde(default)]
    pub dimensions: Vec<DimensionConfig>,
    #[serde(default)]
    pub units: Vec<UnitConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MetaConfig {
    pub description: Option<String>,
    pub version: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DimensionConfig {
    pub id: String,
    pub semantic_type: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UnitConfig {
    pub id: String,
    pub dimension: String,
    pub symbol: String,
    pub spoken: String,
    pub scale_numerator: String,
    pub scale_denominator: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnitsConfigError {
    Toml(String),
    Invalid(String),
}

impl UnitsConfig {
    pub fn from_toml(source: &str) -> Result<Self, UnitsConfigError> {
        let config: Self = toml::from_str(source)
            .map_err(|error| UnitsConfigError::Toml(error.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), UnitsConfigError> {
        let mut dimensions = BTreeSet::new();
        for dimension in &self.dimensions {
            if dimension.id.trim().is_empty() || dimension.semantic_type.trim().is_empty() {
                return Err(UnitsConfigError::Invalid(
                    "unit dimensions require non-empty id and semantic_type".into(),
                ));
            }
            if !dimensions.insert(dimension.id.clone()) {
                return Err(UnitsConfigError::Invalid(format!(
                    "duplicate unit dimension `{}`",
                    dimension.id
                )));
            }
        }
        let mut ids = BTreeSet::new();
        let mut symbols = BTreeSet::new();
        let mut spoken = BTreeSet::new();
        for unit in &self.units {
            if !dimensions.contains(&unit.dimension) {
                return Err(UnitsConfigError::Invalid(format!(
                    "unit `{}` references unknown dimension `{}`",
                    unit.id, unit.dimension
                )));
            }
            if !ids.insert(unit.id.clone()) {
                return Err(UnitsConfigError::Invalid(format!("duplicate unit id `{}`", unit.id)));
            }
            if !symbols.insert(unit.symbol.to_lowercase()) {
                return Err(UnitsConfigError::Invalid(format!(
                    "duplicate unit symbol `{}`",
                    unit.symbol
                )));
            }
            if !spoken.insert(unit.spoken.to_lowercase()) {
                return Err(UnitsConfigError::Invalid(format!(
                    "duplicate spoken unit form `{}`",
                    unit.spoken
                )));
            }
            let numerator = parse_bigint(&unit.scale_numerator)?;
            let denominator = parse_bigint(&unit.scale_denominator)?;
            if numerator <= BigInt::from(0u8) || denominator <= BigInt::from(0u8) {
                return Err(UnitsConfigError::Invalid(format!(
                    "unit `{}` conversion scale must be positive",
                    unit.id
                )));
            }
        }
        Ok(())
    }

    pub fn dimensions_by_id(&self) -> BTreeMap<&str, &DimensionConfig> {
        self.dimensions.iter().map(|value| (value.id.as_str(), value)).collect()
    }
}

impl UnitConfig {
    pub fn scale(&self) -> Result<ExactRational, UnitsConfigError> {
        Ok(ExactRational::new(
            parse_bigint(&self.scale_numerator)?,
            parse_bigint(&self.scale_denominator)?,
        ))
    }
}

fn parse_bigint(source: &str) -> Result<BigInt, UnitsConfigError> {
    BigInt::parse_bytes(source.as_bytes(), 10).ok_or_else(|| {
        UnitsConfigError::Invalid(format!("invalid exact integer `{source}` in unit conversion"))
    })
}

impl std::fmt::Display for UnitsConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Toml(message) => write!(f, "units config TOML: {message}"),
            Self::Invalid(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for UnitsConfigError {}
