use crate::semantics::{ConstructorId, ContextSlotId, InformationStatus, SymbolId, Type};

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompiledSurfaceBinding {
    Atom { semantic: SymbolId },
    Reference,
    Information {
        mode: ConstructorId,
        status: InformationStatus,
        knower_type: Option<Type>,
    },
    Alias { name: String, ty: String },
    Context {
        slot: ContextSlotId,
        key: String,
        ty: Type,
    },
    Class {
        semantic: SymbolId,
        parameter: u32,
    },
    Predicate {
        semantic: SymbolId,
        primary_parameter: Option<u32>,
        rest_parameters: Vec<u32>,
    },
    Prefix {
        semantic: SymbolId,
        parameter: u32,
        outer_only: bool,
    },
    Infix {
        semantic: SymbolId,
        left_parameter: u32,
        right_parameter: u32,
        precedence: u16,
        associative: bool,
        outer_only: bool,
    },
    Binder {
        semantic: SymbolId,
        binder_parameter: u32,
        direct_parameters: Vec<u32>,
        variable_type: Type,
        combiner_semantic: SymbolId,
        combiner_left_parameter: u32,
        combiner_right_parameter: u32,
    },
    Capture {
        semantic: SymbolId,
        parameter: u32,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompiledSurfaceLexicon {
    entries: BTreeMap<String, CompiledSurfaceBinding>,
    semantic_names: BTreeMap<SymbolId, String>,
}

impl CompiledSurfaceLexicon {
    pub fn new(
        entries: BTreeMap<String, CompiledSurfaceBinding>,
        semantic_names: BTreeMap<SymbolId, String>,
    ) -> Self {
        Self { entries, semantic_names }
    }
    pub fn get(&self, surface: &str) -> Option<&CompiledSurfaceBinding> { self.entries.get(surface) }
    pub fn contains_key(&self, surface: &str) -> bool { self.entries.contains_key(surface) }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &CompiledSurfaceBinding)> { self.entries.iter() }
    pub fn semantic_name(&self, id: SymbolId) -> Option<&str> {
        self.semantic_names.get(&id).map(String::as_str)
    }

    pub fn with_aliases<I>(&self, aliases: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = (String, String)>,
    {
        let mut entries = self.entries.clone();
        for (surface, ty) in aliases {
            if entries.contains_key(&surface) {
                return Err(format!("discourse alias `{surface}` collides with an existing lexical root"));
            }
            entries.insert(surface.clone(), CompiledSurfaceBinding::Alias { name: surface, ty });
        }
        Ok(Self { entries, semantic_names: self.semantic_names.clone() })
    }
}
