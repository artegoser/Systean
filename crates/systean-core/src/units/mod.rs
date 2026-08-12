mod config;

use std::collections::{BTreeMap, BTreeSet};

use num_bigint::BigInt;

use crate::literals::ExactNumber;
use crate::rational::ExactRational;
use crate::semantics::{DimensionId, Type, UnitId};
use crate::spec::{TypedSemanticPackage, legacy_typed_type};

pub use config::{UnitConfig, UnitsConfig, UnitsConfigError};

#[derive(Clone, Debug)]
pub struct UnitRegistry {
    config: UnitsConfig,
    units: BTreeMap<UnitId, ResolvedUnit>,
    by_name: BTreeMap<String, UnitId>,
    by_symbol: BTreeMap<String, UnitId>,
    by_spoken: BTreeMap<String, UnitId>,
    dimension_types: BTreeMap<DimensionId, Type>,
}

/// Runtime unit data after the package boundary has resolved all textual names.
/// `legacy_name` is diagnostics/compatibility metadata only; semantic identity is `id`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedUnit {
    pub id: UnitId,
    pub dimension: DimensionId,
    pub legacy_name: String,
    pub symbol: String,
    pub spoken: String,
    pub scale: ExactRational,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuantityValue {
    pub value: ExactNumber,
    pub unit_id: UnitId,
    pub approximate: bool,
    pub uncertainty: Option<ExactNumber>,
}


fn absolute_scale(
    typed: &TypedSemanticPackage,
    id: UnitId,
) -> Result<ExactRational, UnitsConfigError> {
    let mut current = id;
    let mut scale = ExactRational::from_integer(BigInt::from(1u8));
    let mut visited = BTreeSet::new();
    loop {
        if !visited.insert(current) {
            return Err(UnitsConfigError::Invalid(format!(
                "typed unit conversion cycle reached `{current}`"
            )));
        }
        let unit = typed.unit(current).ok_or_else(|| {
            UnitsConfigError::Invalid(format!("unknown typed unit `{current}`"))
        })?;
        scale = scale * ExactRational::new(
            BigInt::from(unit.scale_numerator),
            BigInt::from(unit.scale_denominator),
        );
        match unit.base {
            Some(base) => current = base,
            None => return Ok(scale),
        }
    }
}

impl UnitRegistry {
    /// Phase 19 production boundary: semantic unit identity, dimension, dimension value type,
    /// and exact conversion scale come from the typed package. TOML contributes only human/API
    /// aliases and written/spoken surface metadata.
    pub fn new(
        config: UnitsConfig,
        typed: &TypedSemanticPackage,
    ) -> Result<Self, UnitsConfigError> {
        let mut dimension_types = BTreeMap::new();
        for id in typed.dimensions() {
            let compiled = typed.dimension_type(id).ok_or_else(|| {
                UnitsConfigError::Invalid(format!("typed dimension `{id}` has no declared value type"))
            })?;
            dimension_types.insert(id, legacy_typed_type(typed, compiled));
        }

        let mut units = BTreeMap::new();
        let mut by_name = BTreeMap::new();
        let mut by_symbol = BTreeMap::new();
        let mut by_spoken = BTreeMap::new();
        let mut configured_units = BTreeSet::new();
        for unit in &config.units {
            let id = typed.unit_id(&unit.spoken).ok_or_else(|| {
                UnitsConfigError::Invalid(format!(
                    "unit metadata `{}` / `{}` has no typed unit declaration",
                    unit.id, unit.spoken
                ))
            })?;
            configured_units.insert(id);
            let compiled = typed.unit(id).expect("typed unit ID resolves to compiled unit");
            if !dimension_types.contains_key(&compiled.dimension) {
                return Err(UnitsConfigError::Invalid(format!(
                    "unit `{}` uses a typed dimension with no semantic-type adapter",
                    unit.id
                )));
            }
            let resolved = ResolvedUnit {
                id,
                dimension: compiled.dimension,
                legacy_name: unit.id.clone(),
                symbol: unit.symbol.clone(),
                spoken: unit.spoken.clone(),
                scale: absolute_scale(typed, id)?,
            };
            if units.insert(id, resolved).is_some() {
                return Err(UnitsConfigError::Invalid(format!(
                    "unit `{}` collides after typed ID resolution",
                    unit.id
                )));
            }
            by_name.insert(unit.id.to_lowercase(), id);
            by_name.insert(unit.spoken.to_lowercase(), id);
            by_symbol.insert(unit.symbol.to_lowercase(), id);
            by_spoken.insert(unit.spoken.to_lowercase(), id);
        }
        let typed_units = typed.units().map(|unit| unit.id).collect::<BTreeSet<_>>();
        if configured_units != typed_units {
            return Err(UnitsConfigError::Invalid(
                "units.toml surface metadata must cover exactly the typed unit declarations".into(),
            ));
        }
        Ok(Self { config, units, by_name, by_symbol, by_spoken, dimension_types })
    }

    pub fn config(&self) -> &UnitsConfig { &self.config }
    pub fn units(&self) -> impl Iterator<Item = &ResolvedUnit> { self.units.values() }

    pub fn unit_id_by_name(&self, name: &str) -> Option<UnitId> {
        self.by_name.get(&name.to_lowercase()).copied()
    }

    pub fn unit_by_id(&self, id: UnitId) -> Option<&ResolvedUnit> {
        self.units.get(&id)
    }

    pub fn unit_by_name(&self, name: &str) -> Option<&ResolvedUnit> {
        self.unit_id_by_name(name).and_then(|id| self.unit_by_id(id))
    }

    pub fn unit_by_surface(&self, surface: &str) -> Option<&ResolvedUnit> {
        self.unit_by_symbol(surface).or_else(|| self.unit_by_spoken(surface))
    }

    pub fn unit_by_symbol(&self, surface: &str) -> Option<&ResolvedUnit> {
        self.by_symbol
            .get(&surface.to_lowercase())
            .and_then(|id| self.units.get(id))
    }

    pub fn unit_by_spoken(&self, surface: &str) -> Option<&ResolvedUnit> {
        self.by_spoken
            .get(&surface.to_lowercase())
            .and_then(|id| self.units.get(id))
    }

    pub fn is_spoken_surface(&self, surface: &str) -> bool {
        self.by_spoken.contains_key(&surface.to_lowercase())
    }

    pub fn dimension_type(&self, dimension: DimensionId) -> Option<&Type> {
        self.dimension_types.get(&dimension)
    }

    pub fn unit_type(&self, unit: &ResolvedUnit) -> Type {
        Type::Generic {
            name: "Unit".into(),
            arguments: vec![self.dimension_type(unit.dimension).cloned().expect("validated dimension")],
        }
    }

    pub fn quantity_type(&self, unit: &ResolvedUnit, approximate: bool) -> Type {
        let quantity = Type::Generic {
            name: "Quantity".into(),
            arguments: vec![self.dimension_type(unit.dimension).cloned().expect("validated dimension")],
        };
        if approximate {
            Type::Generic { name: "Approximate".into(), arguments: vec![quantity] }
        } else {
            quantity
        }
    }

    pub fn convert(&self, quantity: &QuantityValue, target_unit: UnitId) -> Result<QuantityValue, String> {
        let source = self.unit_by_id(quantity.unit_id)
            .ok_or_else(|| format!("unknown source unit `{}`", quantity.unit_id))?;
        let target = self.unit_by_id(target_unit)
            .ok_or_else(|| format!("unknown target unit `{target_unit}`"))?;
        if source.dimension != target.dimension {
            return Err(format!(
                "cannot convert unit `{}` to `{}` across dimensions",
                source.legacy_name, target.legacy_name
            ));
        }
        let ratio: ExactRational = source.scale.clone() / target.scale.clone();
        let value = ExactNumber(quantity.value.0.clone() * &ratio);
        let uncertainty = quantity.uncertainty.as_ref()
            .map(|value| ExactNumber(value.0.clone() * &ratio));
        Ok(QuantityValue {
            value,
            unit_id: target.id,
            approximate: quantity.approximate,
            uncertainty,
        })
    }

    pub fn convert_to_name(&self, quantity: &QuantityValue, target: &str) -> Result<QuantityValue, String> {
        let id = self.unit_id_by_name(target)
            .ok_or_else(|| format!("unknown target unit `{target}`"))?;
        self.convert(quantity, id)
    }
}
