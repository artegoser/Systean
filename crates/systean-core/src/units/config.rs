use std::collections::{BTreeMap, BTreeSet};

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

/// Surface/type-adapter metadata for a typed semantic dimension.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DimensionConfig {
    pub id: String,
    pub semantic_type: String,
}

/// Human/API aliases and surface spellings only. Dimension and exact scale are owned by
/// `language/typed/units.semsys` as of Phase 18.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UnitConfig {
    pub id: String,
    pub symbol: String,
    pub spoken: String,
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
        }
        Ok(())
    }

    pub fn dimensions_by_id(&self) -> BTreeMap<&str, &DimensionConfig> {
        self.dimensions.iter().map(|value| (value.id.as_str(), value)).collect()
    }
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
