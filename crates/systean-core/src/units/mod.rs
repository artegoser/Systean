mod config;

use std::collections::BTreeMap;

use crate::rational::ExactRational;

pub use config::{DimensionConfig, UnitConfig, UnitsConfig, UnitsConfigError};

use crate::literals::ExactNumber;
use crate::semantics::Type;
use crate::spec::parse_type;

#[derive(Clone, Debug)]
pub struct UnitRegistry {
    config: UnitsConfig,
    by_id: BTreeMap<String, usize>,
    by_symbol: BTreeMap<String, usize>,
    by_spoken: BTreeMap<String, usize>,
    dimension_types: BTreeMap<String, Type>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuantityValue {
    pub value: ExactNumber,
    pub unit_id: String,
    pub approximate: bool,
    pub uncertainty: Option<ExactNumber>,
}

impl UnitRegistry {
    pub fn new(config: UnitsConfig) -> Result<Self, UnitsConfigError> {
        let mut dimension_types = BTreeMap::new();
        for dimension in &config.dimensions {
            let ty = parse_type(&dimension.semantic_type).map_err(|errors| {
                UnitsConfigError::Invalid(format!(
                    "dimension `{}` has invalid semantic type `{}`: {}",
                    dimension.id,
                    dimension.semantic_type,
                    errors.into_iter().map(|error| error.to_string()).collect::<Vec<_>>().join("; ")
                ))
            })?;
            dimension_types.insert(dimension.id.clone(), ty);
        }
        let mut by_id = BTreeMap::new();
        let mut by_symbol = BTreeMap::new();
        let mut by_spoken = BTreeMap::new();
        for (index, unit) in config.units.iter().enumerate() {
            by_id.insert(unit.id.clone(), index);
            by_symbol.insert(unit.symbol.to_lowercase(), index);
            by_spoken.insert(unit.spoken.to_lowercase(), index);
        }
        Ok(Self { config, by_id, by_symbol, by_spoken, dimension_types })
    }

    pub fn config(&self) -> &UnitsConfig { &self.config }

    pub fn unit_by_id(&self, id: &str) -> Option<&UnitConfig> {
        self.by_id.get(id).map(|index| &self.config.units[*index])
    }

    pub fn unit_by_surface(&self, surface: &str) -> Option<&UnitConfig> {
        self.unit_by_symbol(surface).or_else(|| self.unit_by_spoken(surface))
    }

    pub fn unit_by_symbol(&self, surface: &str) -> Option<&UnitConfig> {
        self.by_symbol
            .get(&surface.to_lowercase())
            .map(|index| &self.config.units[*index])
    }

    pub fn unit_by_spoken(&self, surface: &str) -> Option<&UnitConfig> {
        self.by_spoken
            .get(&surface.to_lowercase())
            .map(|index| &self.config.units[*index])
    }

    pub fn is_spoken_surface(&self, surface: &str) -> bool {
        self.by_spoken.contains_key(&surface.to_lowercase())
    }

    pub fn dimension_type(&self, dimension: &str) -> Option<&Type> {
        self.dimension_types.get(dimension)
    }

    pub fn unit_type(&self, unit: &UnitConfig) -> Type {
        Type::Generic {
            name: "Unit".into(),
            arguments: vec![self.dimension_type(&unit.dimension).cloned().expect("validated dimension")],
        }
    }

    pub fn quantity_type(&self, unit: &UnitConfig, approximate: bool) -> Type {
        let quantity = Type::Generic {
            name: "Quantity".into(),
            arguments: vec![self.dimension_type(&unit.dimension).cloned().expect("validated dimension")],
        };
        if approximate {
            Type::Generic { name: "Approximate".into(), arguments: vec![quantity] }
        } else {
            quantity
        }
    }

    pub fn convert(&self, quantity: &QuantityValue, target_unit: &str) -> Result<QuantityValue, String> {
        let source = self.unit_by_id(&quantity.unit_id)
            .ok_or_else(|| format!("unknown source unit `{}`", quantity.unit_id))?;
        let target = self.unit_by_id(target_unit)
            .ok_or_else(|| format!("unknown target unit `{target_unit}`"))?;
        if source.dimension != target.dimension {
            return Err(format!(
                "cannot convert unit `{}` ({}) to `{}` ({})",
                source.id, source.dimension, target.id, target.dimension
            ));
        }
        let source_scale = source.scale().map_err(|error| error.to_string())?;
        let target_scale = target.scale().map_err(|error| error.to_string())?;
        let ratio: ExactRational = source_scale / target_scale;
        let value = ExactNumber(quantity.value.0.clone() * &ratio);
        let uncertainty = quantity.uncertainty.as_ref()
            .map(|value| ExactNumber(value.0.clone() * &ratio));
        Ok(QuantityValue {
            value,
            unit_id: target.id.clone(),
            approximate: quantity.approximate,
            uncertainty,
        })
    }
}
