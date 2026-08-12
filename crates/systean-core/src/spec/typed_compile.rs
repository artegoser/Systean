use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::semantics::{
    ConstructorId, ContextSlotId, DimensionId, FieldId, IntrinsicId, LiteralKind, SymbolId, Type,
    TypeId, UnitId,
};

use super::typed_ast::{
    CompiledActKind, CompiledBinderRule, CompiledCaptureKind, CompiledCaptureRule,
    CompiledConstructor, CompiledContextSlot, CompiledEffectInstruction, CompiledEffectProgram,
    CompiledRepairKind, CompiledScalar, CompiledSignature, CompiledSymbol, CompiledSurfaceItem,
    CompiledSurfaceRule, CompiledSymbolKind, CompiledTerm, CompiledType, CompiledUnit,
    DeclarationProvenance, DefaultSurfaceFrame, DefaultSurfaceItem, IntrinsicBinding, SourceActKind,
    SourceCaptureKind, SourceEffectDirective, SourceExpr, SourceParameter, SourceRepairKind, SourceSpan,
    SourceSurfaceItem, SourceSurfaceRule, SymbolDebugInfo, TypedDeclaration,
    TypedSpecification,
};
use super::typed_parser::{TypedParseError, parse_typed_specification};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedSourceSpecification {
    pub source: String,
    pub specification: TypedSpecification,
    pub declaration_spans: Vec<Option<SourceSpan>>,
}

impl TypedSourceSpecification {
    pub fn new(source: impl Into<String>, specification: TypedSpecification) -> Self {
        let declaration_spans = vec![None; specification.declarations.len()];
        Self { source: source.into(), specification, declaration_spans }
    }

    pub fn with_spans(
        source: impl Into<String>,
        specification: TypedSpecification,
        declaration_spans: Vec<Option<SourceSpan>>,
    ) -> Self {
        Self { source: source.into(), specification, declaration_spans }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedSemanticPackage {
    types: BTreeMap<TypeId, u32>,
    type_provenance: BTreeMap<TypeId, DeclarationProvenance>,
    subtypes: BTreeSet<(TypeId, TypeId)>,
    subtype_provenance: BTreeMap<(TypeId, TypeId), DeclarationProvenance>,
    literal_types: BTreeMap<LiteralKind, CompiledType>,
    literal_provenance: BTreeMap<LiteralKind, DeclarationProvenance>,
    symbols: BTreeMap<SymbolId, CompiledSymbol>,
    constructors: BTreeMap<ConstructorId, CompiledConstructor>,
    contexts: BTreeMap<ContextSlotId, CompiledContextSlot>,
    dimensions: BTreeSet<DimensionId>,
    dimension_types: BTreeMap<DimensionId, CompiledType>,
    dimension_provenance: BTreeMap<DimensionId, DeclarationProvenance>,
    units: BTreeMap<UnitId, CompiledUnit>,
    intrinsics: BTreeMap<SymbolId, IntrinsicBinding>,
    type_names: BTreeMap<String, TypeId>,
    symbol_names: BTreeMap<String, SymbolId>,
    constructor_names: BTreeMap<String, ConstructorId>,
    context_names: BTreeMap<String, ContextSlotId>,
    dimension_names: BTreeMap<String, DimensionId>,
    unit_names: BTreeMap<String, UnitId>,
    debug_symbols: BTreeMap<SymbolId, SymbolDebugInfo>,
    surface_rules: BTreeMap<SymbolId, CompiledSurfaceRule>,
    effect_programs: BTreeMap<SymbolId, CompiledEffectProgram>,
    default_effect: Option<SymbolId>,
    semantic_fingerprint: String,
    surface_fingerprint: String,
}

impl TypedSemanticPackage {
    pub fn types(&self) -> impl Iterator<Item = (TypeId, u32)> + '_ {
        self.types.iter().map(|(id, arity)| (*id, *arity))
    }

    pub fn subtypes(&self) -> impl Iterator<Item = (TypeId, TypeId)> + '_ {
        self.subtypes.iter().copied()
    }

    pub fn literal_types(&self) -> impl Iterator<Item = (LiteralKind, &CompiledType)> + '_ {
        self.literal_types.iter().map(|(kind, ty)| (*kind, ty))
    }

    pub fn symbols(&self) -> impl Iterator<Item = &CompiledSymbol> + '_ {
        self.symbols.values()
    }

    pub fn units(&self) -> impl Iterator<Item = &CompiledUnit> + '_ {
        self.units.values()
    }

    pub fn dimensions(&self) -> impl Iterator<Item = DimensionId> + '_ {
        self.dimensions.iter().copied()
    }

    pub fn dimension_type(&self, id: DimensionId) -> Option<&CompiledType> {
        self.dimension_types.get(&id)
    }

    pub fn source_name_for_type(&self, id: TypeId) -> Option<&str> {
        self.type_names.iter().find_map(|(name, candidate)| (*candidate == id).then_some(name.as_str()))
    }

    pub fn source_name_for_symbol(&self, id: SymbolId) -> Option<&str> {
        self.debug_symbols.get(&id).map(|debug| debug.source_name.as_str())
    }

    pub fn source_name_for_context(&self, id: ContextSlotId) -> Option<&str> {
        self.context_names.iter().find_map(|(name, candidate)| (*candidate == id).then_some(name.as_str()))
    }

    pub fn source_name_for_dimension(&self, id: DimensionId) -> Option<&str> {
        self.dimension_names.iter().find_map(|(name, candidate)| (*candidate == id).then_some(name.as_str()))
    }

    pub fn source_name_for_unit(&self, id: UnitId) -> Option<&str> {
        self.unit_names.iter().find_map(|(name, candidate)| (*candidate == id).then_some(name.as_str()))
    }

    pub fn subtype_provenance(&self, child: TypeId, parent: TypeId) -> Option<&DeclarationProvenance> {
        self.subtype_provenance.get(&(child, parent))
    }

    pub fn literal_provenance(&self, kind: LiteralKind) -> Option<&DeclarationProvenance> {
        self.literal_provenance.get(&kind)
    }

    pub fn symbol_id(&self, source_name: &str) -> Option<SymbolId> {
        self.symbol_names.get(source_name).copied()
    }

    pub fn type_id(&self, source_name: &str) -> Option<TypeId> {
        self.type_names.get(source_name).copied()
    }

    pub fn type_provenance(&self, id: TypeId) -> Option<&DeclarationProvenance> {
        self.type_provenance.get(&id)
    }

    pub fn constructor_id(&self, source_name: &str) -> Option<ConstructorId> {
        self.constructor_names.get(source_name).copied()
    }

    pub fn context_slot_id(&self, source_name: &str) -> Option<ContextSlotId> {
        self.context_names.get(source_name).copied()
    }

    pub fn dimension_id(&self, source_name: &str) -> Option<DimensionId> {
        self.dimension_names.get(source_name).copied()
    }

    pub fn unit_id(&self, source_name: &str) -> Option<UnitId> {
        self.unit_names.get(source_name).copied()
    }

    pub fn has_dimension(&self, id: DimensionId) -> bool {
        self.dimensions.contains(&id)
    }

    pub fn dimension_provenance(&self, id: DimensionId) -> Option<&DeclarationProvenance> {
        self.dimension_provenance.get(&id)
    }

    pub fn intrinsic_id(&self, symbol: SymbolId) -> Option<IntrinsicId> {
        self.intrinsics.get(&symbol).map(|binding| binding.intrinsic)
    }

    pub fn symbol(&self, id: SymbolId) -> Option<&CompiledSymbol> {
        self.symbols.get(&id)
    }

    pub fn constructor(&self, id: ConstructorId) -> Option<&CompiledConstructor> {
        self.constructors.get(&id)
    }

    pub fn context_slot(&self, id: ContextSlotId) -> Option<&CompiledContextSlot> {
        self.contexts.get(&id)
    }

    pub fn unit(&self, id: UnitId) -> Option<&CompiledUnit> {
        self.units.get(&id)
    }

    pub fn debug_symbol(&self, id: SymbolId) -> Option<&SymbolDebugInfo> {
        self.debug_symbols.get(&id)
    }

    pub fn surface_rule(&self, id: SymbolId) -> Option<&CompiledSurfaceRule> {
        self.surface_rules.get(&id)
    }

    pub fn surface_rules(&self) -> impl Iterator<Item = &CompiledSurfaceRule> + '_ {
        self.surface_rules.values()
    }

    pub fn effect_program(&self, id: SymbolId) -> Option<&CompiledEffectProgram> {
        self.effect_programs.get(&id)
    }

    pub fn effect_programs(&self) -> impl Iterator<Item = &CompiledEffectProgram> + '_ {
        self.effect_programs.values()
    }

    pub fn default_effect(&self) -> Option<SymbolId> {
        self.default_effect
    }

    pub fn semantic_fingerprint(&self) -> &str {
        &self.semantic_fingerprint
    }

    pub fn surface_fingerprint(&self) -> &str {
        &self.surface_fingerprint
    }

    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    pub fn default_surface_frame(&self, id: SymbolId) -> Option<DefaultSurfaceFrame> {
        let symbol = self.symbols.get(&id)?;
        if symbol.kind != CompiledSymbolKind::Word {
            return None;
        }
        let debug = self.debug_symbols.get(&id)?;
        let rule = self.surface_rules.get(&id)?;
        let items = rule
            .items
            .iter()
            .map(|item| match item {
                CompiledSurfaceItem::Root => Some(DefaultSurfaceItem::Root),
                CompiledSurfaceItem::Argument(index) => Some(DefaultSurfaceItem::Argument(*index)),
                CompiledSurfaceItem::Restriction => None,
            })
            .collect::<Option<Vec<_>>>()?;
        Some(DefaultSurfaceFrame { root: debug.source_name.clone(), items })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedCompileError {
    Parse { source: String, error: TypedParseError },
    DuplicateDeclaration { namespace: &'static str, name: String },
    IdentityCollision { namespace: &'static str, left: String, right: String },
    DuplicateParameter { declaration: String, parameter: String },
    DuplicateTypeParameter { declaration: String, parameter: String },
    DuplicateLiteralKind(LiteralKind),
    UnknownType(String),
    InvalidSubtype { child: String, parent: String },
    CyclicSubtype(Vec<TypeId>),
    WrongTypeArity { name: String, expected: u32, actual: usize },
    UnknownSymbol(String),
    UnknownDimension(String),
    UnknownUnit(String),
    InvalidUnitScale { name: String },
    UnitDimensionMismatch { unit: String, base: String },
    InvalidCallArity { name: String, expected: usize, actual: usize },
    UnboundLocal(String),
    CyclicDefinition(Vec<SymbolId>),
    CyclicUnitDefinition(Vec<UnitId>),
    UnknownSurfaceParameter { word: String, parameter: String },
    DuplicateSurfaceParameter { word: String, parameter: String },
    MissingSurfaceParameter { word: String, parameter: String },
    InvalidSurfaceRootCount { word: String, count: usize },
    InvalidSurfacePrecedence { word: String },
    InvalidAssociativeSurface { word: String },
    InvalidSurfaceBinder { word: String, message: String },
    InvalidSurfaceCapture { word: String, message: String },
    DuplicateEffect { target: String },
    DuplicateDefaultEffect,
    InvalidEffectParameter { target: String, parameter: String },
    InvalidEffect { target: String, message: String },
}

pub fn compile_typed_sources(
    sources: impl IntoIterator<Item = (String, String)>,
) -> Result<TypedSemanticPackage, Vec<TypedCompileError>> {
    let mut parsed = Vec::new();
    let mut errors = Vec::new();
    for (source, text) in sources {
        match parse_typed_specification(&text) {
            Ok(specification) => {
                let spans = locate_declaration_spans(&text, &specification);
                parsed.push(TypedSourceSpecification::with_spans(source, specification, spans));
            },
            Err(parse_errors) => errors.extend(parse_errors.into_iter().map(|error| {
                TypedCompileError::Parse { source: source.clone(), error }
            })),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    compile_typed_specifications(&parsed)
}

pub fn compile_typed_specifications(
    sources: &[TypedSourceSpecification],
) -> Result<TypedSemanticPackage, Vec<TypedCompileError>> {
    let mut errors = Vec::new();
    let mut type_names = BTreeMap::new();
    let mut symbol_names = BTreeMap::new();
    let mut constructor_names = BTreeMap::new();
    let mut context_names = BTreeMap::new();
    let mut dimension_names = BTreeMap::new();
    let mut unit_names = BTreeMap::new();
    let mut type_arities = BTreeMap::new();

    for source in sources {
        for declaration in &source.specification.declarations {
            match declaration {
                TypedDeclaration::Type { name, type_parameters }
                | TypedDeclaration::Data { name, type_parameters, .. } => {
                    let id = TypeId::from_source("type", name);
                    insert_identity(
                        "type",
                        name,
                        id.0,
                        &mut type_names,
                        &mut errors,
                    );
                    type_arities.insert(name.clone(), type_parameters.len() as u32);
                }
                TypedDeclaration::Subtype { .. }
                | TypedDeclaration::Literal { .. }
                | TypedDeclaration::Effect { .. }
                | TypedDeclaration::DefaultEffect { .. } => {}
                TypedDeclaration::Word { name, .. }
                | TypedDeclaration::Primitive { name, .. }
                | TypedDeclaration::Def { name, .. }
                | TypedDeclaration::Intrinsic { name, .. } => {
                    let id = SymbolId::from_source("symbol", name);
                    insert_identity(
                        "symbol",
                        name,
                        id.0,
                        &mut symbol_names,
                        &mut errors,
                    );
                }
                TypedDeclaration::Context { name, .. } => {
                    let id = ContextSlotId::from_source("context", name);
                    insert_identity(
                        "context",
                        name,
                        id.0,
                        &mut context_names,
                        &mut errors,
                    );
                }
                TypedDeclaration::Dimension { name, .. } => {
                    let id = DimensionId::from_source("dimension", name);
                    insert_identity(
                        "dimension",
                        name,
                        id.0,
                        &mut dimension_names,
                        &mut errors,
                    );
                }
                TypedDeclaration::Unit { name, .. } => {
                    let id = UnitId::from_source("unit", name);
                    insert_identity(
                        "unit",
                        name,
                        id.0,
                        &mut unit_names,
                        &mut errors,
                    );
                }
            }
        }
    }

    for source in sources {
        for declaration in &source.specification.declarations {
            if let TypedDeclaration::Data { name, constructors, .. } = declaration {
                for constructor in constructors {
                    let qualified = format!("{name}.{}", constructor.name);
                    let id = ConstructorId::from_source("constructor", &qualified);
                    insert_identity(
                        "constructor",
                        &qualified,
                        id.0,
                        &mut constructor_names,
                        &mut errors,
                    );
                }
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let mut package = TypedSemanticPackage {
        types: BTreeMap::new(),
        type_provenance: BTreeMap::new(),
        subtypes: BTreeSet::new(),
        subtype_provenance: BTreeMap::new(),
        literal_types: BTreeMap::new(),
        literal_provenance: BTreeMap::new(),
        symbols: BTreeMap::new(),
        constructors: BTreeMap::new(),
        contexts: BTreeMap::new(),
        dimensions: dimension_names.values().copied().collect(),
        dimension_types: BTreeMap::new(),
        dimension_provenance: BTreeMap::new(),
        units: BTreeMap::new(),
        intrinsics: BTreeMap::new(),
        type_names,
        symbol_names,
        constructor_names,
        context_names,
        dimension_names,
        unit_names,
        debug_symbols: BTreeMap::new(),
        surface_rules: BTreeMap::new(),
        effect_programs: BTreeMap::new(),
        default_effect: None,
        semantic_fingerprint: String::new(),
        surface_fingerprint: String::new(),
    };

    for (name, id) in &package.type_names {
        let arity = type_arities.get(name).copied().unwrap_or(0);
        package.types.insert(*id, arity);
    }

    for source in sources {
        for (index, declaration) in source.specification.declarations.iter().enumerate() {
            let provenance = DeclarationProvenance {
                source: source.source.clone(),
                declaration: index + 1,
                span: source.declaration_spans.get(index).copied().flatten(),
            };
            match declaration {
                TypedDeclaration::Type { name, .. } => {
                    let id = package.type_names[name];
                    package.type_provenance.insert(id, provenance);
                }
                TypedDeclaration::Subtype { child, parent } => {
                    let Some(child_id) = package.type_names.get(child).copied() else {
                        errors.push(TypedCompileError::UnknownType(child.clone()));
                        continue;
                    };
                    let Some(parent_id) = package.type_names.get(parent).copied() else {
                        errors.push(TypedCompileError::UnknownType(parent.clone()));
                        continue;
                    };
                    if type_arities.get(child).copied().unwrap_or(0) != 0
                        || type_arities.get(parent).copied().unwrap_or(0) != 0
                    {
                        errors.push(TypedCompileError::InvalidSubtype { child: child.clone(), parent: parent.clone() });
                        continue;
                    }
                    package.subtypes.insert((child_id, parent_id));
                    package.subtype_provenance.insert((child_id, parent_id), provenance);
                }
                TypedDeclaration::Literal { kind, ty } => {
                    match resolve_type(ty, &package, &BTreeMap::new(), &type_arities) {
                        Ok(ty) => {
                            if package.literal_types.insert(*kind, ty).is_some() {
                                errors.push(TypedCompileError::DuplicateLiteralKind(*kind));
                            } else {
                                package.literal_provenance.insert(*kind, provenance);
                            }
                        }
                        Err(error) => errors.push(error),
                    }
                }
                TypedDeclaration::Dimension { name, ty } => {
                    let id = package.dimension_names[name];
                    match resolve_type(ty, &package, &BTreeMap::new(), &type_arities) {
                        Ok(ty) => {
                            package.dimension_types.insert(id, ty);
                            package.dimension_provenance.insert(id, provenance);
                        }
                        Err(error) => errors.push(error),
                    }
                }
                TypedDeclaration::Effect { .. } | TypedDeclaration::DefaultEffect { .. } => {}
                TypedDeclaration::Data { name, type_parameters, constructors } => {
                    let owner = package.type_names[name];
                    package.type_provenance.insert(owner, provenance.clone());
                    validate_unique_names(name, "type parameter", type_parameters, &mut errors);
                    let type_variables = type_variable_map(type_parameters);
                    for constructor in constructors {
                        validate_parameter_names(&constructor.name, &constructor.parameters, &mut errors);
                        let fields = constructor
                            .parameters
                            .iter()
                            .map(|parameter| resolve_type(&parameter.ty, &package, &type_variables, &type_arities))
                            .collect::<Result<Vec<_>, _>>();
                        match fields {
                            Ok(fields) => {
                                let qualified = format!("{name}.{}", constructor.name);
                                let id = package.constructor_names[&qualified];
                                package.constructors.insert(
                                    id,
                                    CompiledConstructor {
                                        id,
                                        owner,
                                        fields,
                                        provenance: provenance.clone(),
                                    },
                                );
                            }
                            Err(error) => errors.push(error),
                        }
                    }
                }
                TypedDeclaration::Context { name, ty } => {
                    match resolve_type(ty, &package, &BTreeMap::new(), &type_arities) {
                        Ok(ty) => {
                            let id = package.context_names[name];
                            package.contexts.insert(
                                id,
                                CompiledContextSlot { id, ty, provenance: provenance.clone() },
                            );
                        }
                        Err(error) => errors.push(error),
                    }
                }
                TypedDeclaration::Unit { name, dimension, definition } => {
                    let Some(dimension_id) = package.dimension_names.get(dimension).copied() else {
                        errors.push(TypedCompileError::UnknownDimension(dimension.clone()));
                        continue;
                    };
                    let (scale_numerator, scale_denominator, base) = match definition {
                        Some(definition) => {
                            if definition.numerator == 0 || definition.denominator == 0 {
                                errors.push(TypedCompileError::InvalidUnitScale { name: name.clone() });
                                continue;
                            }
                            let Some(base) = package.unit_names.get(&definition.base).copied() else {
                                errors.push(TypedCompileError::UnknownUnit(definition.base.clone()));
                                continue;
                            };
                            let divisor = gcd(definition.numerator, definition.denominator);
                            (
                                definition.numerator / divisor,
                                definition.denominator / divisor,
                                Some(base),
                            )
                        }
                        None => (1, 1, None),
                    };
                    let id = package.unit_names[name];
                    package.units.insert(
                        id,
                        CompiledUnit {
                            id,
                            dimension: dimension_id,
                            scale_numerator,
                            scale_denominator,
                            base,
                            provenance,
                        },
                    );
                }
                TypedDeclaration::Word {
                    name,
                    type_parameters,
                    parameters,
                    returns,
                    definition,
                    surface,
                } => compile_symbol(
                    &mut package,
                    name,
                    CompiledSymbolKind::Word,
                    type_parameters,
                    parameters,
                    returns,
                    definition.as_ref(),
                    surface.as_ref(),
                    provenance,
                    &type_arities,
                    &mut errors,
                ),
                TypedDeclaration::Primitive {
                    name,
                    type_parameters,
                    parameters,
                    returns,
                } => compile_symbol(
                    &mut package,
                    name,
                    CompiledSymbolKind::Primitive,
                    type_parameters,
                    parameters,
                    returns,
                    None,
                    None,
                    provenance,
                    &type_arities,
                    &mut errors,
                ),
                TypedDeclaration::Def {
                    name,
                    type_parameters,
                    parameters,
                    returns,
                    definition,
                } => compile_symbol(
                    &mut package,
                    name,
                    CompiledSymbolKind::Definition,
                    type_parameters,
                    parameters,
                    returns,
                    Some(definition),
                    None,
                    provenance,
                    &type_arities,
                    &mut errors,
                ),
                TypedDeclaration::Intrinsic {
                    name,
                    type_parameters,
                    parameters,
                    returns,
                } => {
                    compile_symbol(
                        &mut package,
                        name,
                        CompiledSymbolKind::Intrinsic,
                        type_parameters,
                        parameters,
                        returns,
                        None,
                        None,
                        provenance,
                        &type_arities,
                        &mut errors,
                    );
                    if let Some(symbol) = package.symbol_names.get(name).copied() {
                        package.intrinsics.insert(
                            symbol,
                            IntrinsicBinding {
                                symbol,
                                intrinsic: IntrinsicId::from_source("intrinsic", name),
                            },
                        );
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        validate_surface_binders(&package, &mut errors);
    }
    if errors.is_empty() {
        for source in sources {
            for (index, declaration) in source.specification.declarations.iter().enumerate() {
                let provenance = DeclarationProvenance {
                    source: source.source.clone(),
                    declaration: index + 1,
                    span: source.declaration_spans.get(index).copied().flatten(),
                };
                match declaration {
                    TypedDeclaration::Effect { target, directives } => {
                        compile_effect_program(&mut package, target, directives, provenance, &mut errors);
                    }
                    TypedDeclaration::DefaultEffect { target } => {
                        let Some(symbol) = package.symbol_names.get(target).copied() else {
                            errors.push(TypedCompileError::UnknownSymbol(target.clone()));
                            continue;
                        };
                        if package.default_effect.replace(symbol).is_some() {
                            errors.push(TypedCompileError::DuplicateDefaultEffect);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if errors.is_empty() {
        validate_subtypes(&package, &mut errors);
    }
    if errors.is_empty() {
        validate_units(&package, &mut errors);
    }
    if errors.is_empty() {
        validate_compiled_arities(&package, &mut errors);
    }
    if errors.is_empty() {
        if let Err(error) = validate_definition_cycles(&package) {
            errors.push(error);
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    package.semantic_fingerprint = semantic_fingerprint(&package);
    package.surface_fingerprint = surface_fingerprint(&package);
    Ok(package)
}

fn compile_symbol(
    package: &mut TypedSemanticPackage,
    name: &str,
    kind: CompiledSymbolKind,
    type_parameters: &[String],
    parameters: &[SourceParameter],
    returns: &Type,
    definition: Option<&SourceExpr>,
    surface: Option<&SourceSurfaceRule>,
    provenance: DeclarationProvenance,
    type_arities: &BTreeMap<String, u32>,
    errors: &mut Vec<TypedCompileError>,
) {
    let errors_before = errors.len();
    validate_unique_names(name, "type parameter", type_parameters, errors);
    validate_parameter_names(name, parameters, errors);
    if errors.len() != errors_before {
        return;
    }
    let type_variables = type_variable_map(type_parameters);
    let parameter_types = parameters
        .iter()
        .map(|parameter| resolve_type(&parameter.ty, package, &type_variables, type_arities))
        .collect::<Result<Vec<_>, _>>();
    let returns = resolve_type(returns, package, &type_variables, type_arities);
    let parameter_types = match parameter_types {
        Ok(value) => value,
        Err(error) => {
            errors.push(error);
            return;
        }
    };
    let returns = match returns {
        Ok(value) => value,
        Err(error) => {
            errors.push(error);
            return;
        }
    };

    let definition = definition.map(|expr| {
        let locals = parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect::<Vec<_>>();
        compile_expr(expr, package, &locals, &type_variables, type_arities).map(|body| {
            parameter_types
                .iter()
                .cloned()
                .rev()
                .fold(body, |body, parameter_type| CompiledTerm::Lambda {
                    parameter_type,
                    body: Box::new(body),
                })
        })
    }).transpose();
    let definition = match definition {
        Ok(definition) => definition,
        Err(error) => {
            errors.push(error);
            return;
        }
    };

    let id = package.symbol_names[name];
    package.debug_symbols.insert(
        id,
        SymbolDebugInfo {
            source_name: name.to_owned(),
            parameter_names: parameters.iter().map(|parameter| parameter.name.clone()).collect(),
        },
    );
    let signature = CompiledSignature {
        type_parameter_count: type_parameters.len() as u32,
        parameters: parameter_types,
        returns,
    };
    package.symbols.insert(
        id,
        CompiledSymbol {
            id,
            kind,
            signature: signature.clone(),
            definition,
            provenance: provenance.clone(),
        },
    );
    if kind == CompiledSymbolKind::Word {
        match compile_surface_rule(package, id, name, parameters, &signature, surface, provenance) {
            Ok(rule) => { package.surface_rules.insert(id, rule); }
            Err(error) => errors.push(error),
        }
    }
}

fn compile_surface_rule(
    package: &TypedSemanticPackage,
    symbol: SymbolId,
    word: &str,
    parameters: &[SourceParameter],
    signature: &CompiledSignature,
    source: Option<&SourceSurfaceRule>,
    provenance: DeclarationProvenance,
) -> Result<CompiledSurfaceRule, TypedCompileError> {
    let source_items = match source {
        Some(rule) => rule.items.clone(),
        None => {
            let mut items = Vec::with_capacity(parameters.len() + 1);
            if parameters.is_empty() {
                items.push(SourceSurfaceItem::Root);
            } else {
                items.push(SourceSurfaceItem::Argument(parameters[0].name.clone()));
                items.push(SourceSurfaceItem::Root);
                items.extend(parameters.iter().skip(1).map(|parameter| {
                    SourceSurfaceItem::Argument(parameter.name.clone())
                }));
            }
            items
        }
    };

    let parameter_indices = parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| (parameter.name.as_str(), index as u32))
        .collect::<BTreeMap<_, _>>();

    let binder = source.and_then(|rule| rule.binder.as_ref()).map(|binder| {
        let Some(parameter) = parameter_indices.get(binder.parameter.as_str()).copied() else {
            return Err(TypedCompileError::UnknownSurfaceParameter {
                word: word.to_owned(),
                parameter: binder.parameter.clone(),
            });
        };
        let Some(CompiledType::Function { parameters: function_parameters, returns }) = signature.parameters.get(parameter as usize) else {
            return Err(TypedCompileError::InvalidSurfaceBinder {
                word: word.to_owned(),
                message: format!("binder parameter `${}` must have a function type", binder.parameter),
            });
        };
        if function_parameters.len() != 1 {
            return Err(TypedCompileError::InvalidSurfaceBinder {
                word: word.to_owned(),
                message: format!("binder parameter `${}` must take exactly one bound value", binder.parameter),
            });
        }
        let Some(combiner) = package.symbol_names.get(&binder.combiner).copied() else {
            return Err(TypedCompileError::UnknownSymbol(binder.combiner.clone()));
        };
        let _ = returns;
        Ok(CompiledBinderRule {
            parameter,
            variable_type: function_parameters[0].clone(),
            combiner,
        })
    }).transpose()?;

    let mut captures = Vec::new();
    let mut capture_parameters = BTreeSet::new();
    if let Some(source) = source {
        for capture in &source.captures {
            let Some(parameter) = parameter_indices.get(capture.parameter.as_str()).copied() else {
                return Err(TypedCompileError::UnknownSurfaceParameter {
                    word: word.to_owned(),
                    parameter: capture.parameter.clone(),
                });
            };
            if !capture_parameters.insert(parameter) {
                return Err(TypedCompileError::InvalidSurfaceCapture {
                    word: word.to_owned(),
                    message: format!("surface parameter `${}` is captured more than once", capture.parameter),
                });
            }
            captures.push(CompiledCaptureRule {
                parameter,
                kind: match capture.kind { SourceCaptureKind::BareToken => CompiledCaptureKind::BareToken },
            });
        }
    }

    let binder_parameter = binder.as_ref().map(|binder| binder.parameter);
    let mut seen_parameters = BTreeSet::new();
    let mut root_count = 0usize;
    let mut restriction_count = 0usize;
    let mut items = Vec::with_capacity(source_items.len());
    for item in source_items {
        match item {
            SourceSurfaceItem::Root => {
                root_count += 1;
                items.push(CompiledSurfaceItem::Root);
            }
            SourceSurfaceItem::Restriction => {
                restriction_count += 1;
                items.push(CompiledSurfaceItem::Restriction);
            }
            SourceSurfaceItem::Argument(parameter) => {
                let Some(index) = parameter_indices.get(parameter.as_str()).copied() else {
                    return Err(TypedCompileError::UnknownSurfaceParameter {
                        word: word.to_owned(),
                        parameter,
                    });
                };
                if Some(index) == binder_parameter {
                    return Err(TypedCompileError::InvalidSurfaceBinder {
                        word: word.to_owned(),
                        message: format!("binder parameter `${parameter}` is realized through `@restriction`, not as a direct surface argument"),
                    });
                }
                if !seen_parameters.insert(index) {
                    return Err(TypedCompileError::DuplicateSurfaceParameter {
                        word: word.to_owned(),
                        parameter,
                    });
                }
                items.push(CompiledSurfaceItem::Argument(index));
            }
        }
    }
    if root_count != 1 {
        return Err(TypedCompileError::InvalidSurfaceRootCount {
            word: word.to_owned(),
            count: root_count,
        });
    }
    if binder.is_some() != (restriction_count == 1) {
        return Err(TypedCompileError::InvalidSurfaceBinder {
            word: word.to_owned(),
            message: "a binder surface rule requires exactly one `@restriction`, and `@restriction` requires a binder".into(),
        });
    }
    if restriction_count > 1 {
        return Err(TypedCompileError::InvalidSurfaceBinder {
            word: word.to_owned(),
            message: "a surface rule may contain only one `@restriction`".into(),
        });
    }
    for (index, parameter) in parameters.iter().enumerate() {
        if Some(index as u32) != binder_parameter && !seen_parameters.contains(&(index as u32)) {
            return Err(TypedCompileError::MissingSurfaceParameter {
                word: word.to_owned(),
                parameter: parameter.name.clone(),
            });
        }
    }
    for capture in &captures {
        if !seen_parameters.contains(&capture.parameter) {
            return Err(TypedCompileError::InvalidSurfaceCapture {
                word: word.to_owned(),
                message: "captured parameters must appear in the form".into(),
            });
        }
    }

    let precedence = source.and_then(|rule| rule.precedence);
    let associative = source.is_some_and(|rule| rule.associative);
    if precedence.is_some() {
        let root_index = items.iter().position(|item| matches!(item, CompiledSurfaceItem::Root));
        if items.len() != 3 || root_index != Some(1) || signature.parameters.len() != 2 || binder.is_some() {
            return Err(TypedCompileError::InvalidSurfacePrecedence { word: word.to_owned() });
        }
    }
    if associative {
        let Some(precedence) = precedence else {
            return Err(TypedCompileError::InvalidAssociativeSurface { word: word.to_owned() });
        };
        let _ = precedence;
        if signature.parameters.len() != 2
            || signature.parameters[0] != signature.returns
            || signature.parameters[1] != signature.returns
        {
            return Err(TypedCompileError::InvalidAssociativeSurface { word: word.to_owned() });
        }
    }

    Ok(CompiledSurfaceRule {
        symbol,
        items,
        precedence,
        associative,
        binder,
        captures,
        outer_only: source.is_some_and(|rule| rule.outer_only),
        provenance,
    })
}

fn compile_effect_program(
    package: &mut TypedSemanticPackage,
    target: &str,
    directives: &[SourceEffectDirective],
    provenance: DeclarationProvenance,
    errors: &mut Vec<TypedCompileError>,
) {
    let Some(symbol) = package.symbol_names.get(target).copied() else {
        errors.push(TypedCompileError::UnknownSymbol(target.to_owned()));
        return;
    };
    if package.effect_programs.contains_key(&symbol) {
        errors.push(TypedCompileError::DuplicateEffect { target: target.to_owned() });
        return;
    }
    let Some(debug) = package.debug_symbols.get(&symbol) else {
        errors.push(TypedCompileError::UnknownSymbol(target.to_owned()));
        return;
    };
    let parameter = |name: &str| -> Result<u32, TypedCompileError> {
        debug.parameter_names
            .iter()
            .position(|candidate| candidate == name)
            .map(|index| index as u32)
            .ok_or_else(|| TypedCompileError::InvalidEffectParameter {
                target: target.to_owned(),
                parameter: name.to_owned(),
            })
    };
    let mut instructions = Vec::new();
    for directive in directives {
        let compiled = match directive {
            SourceEffectDirective::Act { kind, arguments } => {
                let expected_arity = match kind {
                    SourceActKind::Assertion
                    | SourceActKind::Question
                    | SourceActKind::Command
                    | SourceActKind::Request
                    | SourceActKind::Expressive
                    | SourceActKind::Retraction => 1,
                    SourceActKind::Focus
                    | SourceActKind::Topic
                    | SourceActKind::Correction
                    | SourceActKind::Clarification => 2,
                };
                if arguments.len() != expected_arity {
                    errors.push(TypedCompileError::InvalidEffect {
                        target: target.to_owned(),
                        message: format!(
                            "act instruction expects {expected_arity} argument(s), found {}",
                            arguments.len(),
                        ),
                    });
                    return;
                }
                let arguments = match arguments.iter().map(|name| parameter(name)).collect::<Result<Vec<_>, _>>() {
                    Ok(arguments) => arguments,
                    Err(error) => { errors.push(error); return; }
                };
                CompiledEffectInstruction::Act {
                    kind: match kind {
                        SourceActKind::Assertion => CompiledActKind::Assertion,
                        SourceActKind::Question => CompiledActKind::Question,
                        SourceActKind::Command => CompiledActKind::Command,
                        SourceActKind::Request => CompiledActKind::Request,
                        SourceActKind::Expressive => CompiledActKind::Expressive,
                        SourceActKind::Focus => CompiledActKind::Focus,
                        SourceActKind::Topic => CompiledActKind::Topic,
                        SourceActKind::Retraction => CompiledActKind::Retraction,
                        SourceActKind::Correction => CompiledActKind::Correction,
                        SourceActKind::Clarification => CompiledActKind::Clarification,
                    },
                    arguments,
                }
            }
            SourceEffectDirective::Commit { argument } => match parameter(argument) {
                Ok(argument) => CompiledEffectInstruction::Commit { argument },
                Err(error) => { errors.push(error); return; }
            },
            SourceEffectDirective::RequireContains { content, target: required_target } => {
                let content = match parameter(content) { Ok(value) => value, Err(error) => { errors.push(error); return; } };
                let required_target = match parameter(required_target) { Ok(value) => value, Err(error) => { errors.push(error); return; } };
                CompiledEffectInstruction::RequireContains { content, target: required_target }
            }
            SourceEffectDirective::Choice { operator } => {
                let Some(operator) = package.symbol_names.get(operator).copied() else {
                    errors.push(TypedCompileError::UnknownSymbol(operator.clone()));
                    return;
                };
                CompiledEffectInstruction::Choice { operator }
            }
            SourceEffectDirective::Repair { kind, target: repair_target, value } => {
                let repair_target = match parameter(repair_target) { Ok(value) => value, Err(error) => { errors.push(error); return; } };
                let value = match value {
                    Some(value) => match parameter(value) { Ok(value) => Some(value), Err(error) => { errors.push(error); return; } },
                    None => None,
                };
                let kind = match kind {
                    SourceRepairKind::Retract => CompiledRepairKind::Retract,
                    SourceRepairKind::Replace => CompiledRepairKind::Replace,
                    SourceRepairKind::Clarify => CompiledRepairKind::Clarify,
                };
                if matches!(kind, CompiledRepairKind::Retract) != value.is_none() {
                    errors.push(TypedCompileError::InvalidEffect {
                        target: target.to_owned(),
                        message: "retract repair takes only a target; replace/clarify repairs require a value".into(),
                    });
                    return;
                }
                CompiledEffectInstruction::Repair { kind, target: repair_target, value }
            }
        };
        instructions.push(compiled);
    }
    let act_count = instructions.iter().filter(|instruction| matches!(instruction, CompiledEffectInstruction::Act { .. })).count();
    if act_count != 1 {
        errors.push(TypedCompileError::InvalidEffect {
            target: target.to_owned(),
            message: format!("effect program requires exactly one `act` instruction, found {act_count}"),
        });
        return;
    }
    package.effect_programs.insert(symbol, CompiledEffectProgram { symbol, instructions, provenance });
}

fn validate_surface_binders(package: &TypedSemanticPackage, errors: &mut Vec<TypedCompileError>) {
    for rule in package.surface_rules.values() {
        let Some(binder) = &rule.binder else { continue; };
        let Some(owner) = package.symbols.get(&rule.symbol) else { continue; };
        let Some(CompiledType::Function { returns, .. }) = owner.signature.parameters.get(binder.parameter as usize) else { continue; };
        let Some(combiner) = package.symbols.get(&binder.combiner) else {
            let word = package.source_name_for_symbol(rule.symbol).unwrap_or("<unknown>").to_owned();
            errors.push(TypedCompileError::InvalidSurfaceBinder {
                word,
                message: "binder combiner was not compiled".into(),
            });
            continue;
        };
        if combiner.signature.parameters.len() != 2
            || combiner.signature.parameters[0] != **returns
            || combiner.signature.parameters[1] != **returns
            || combiner.signature.returns != **returns
        {
            let word = package.source_name_for_symbol(rule.symbol).unwrap_or("<unknown>").to_owned();
            let combiner_name = package.source_name_for_symbol(binder.combiner).unwrap_or("<unknown>");
            errors.push(TypedCompileError::InvalidSurfaceBinder {
                word,
                message: format!("binder combiner `{combiner_name}` must be a binary operator over the predicate result type"),
            });
        }
    }
}

fn resolve_type(
    ty: &Type,
    package: &TypedSemanticPackage,
    type_variables: &BTreeMap<String, u32>,
    type_arities: &BTreeMap<String, u32>,
) -> Result<CompiledType, TypedCompileError> {
    match ty {
        Type::Named(name) => {
            let id = package.type_names.get(name).copied().ok_or_else(|| TypedCompileError::UnknownType(name.clone()))?;
            let expected = type_arities.get(name).copied().unwrap_or(0);
            if expected != 0 {
                return Err(TypedCompileError::WrongTypeArity { name: name.clone(), expected, actual: 0 });
            }
            Ok(CompiledType::Named(id))
        }
        Type::Generic { name, arguments } => {
            let constructor = package.type_names.get(name).copied().ok_or_else(|| TypedCompileError::UnknownType(name.clone()))?;
            let expected = type_arities.get(name).copied().unwrap_or(0);
            if expected as usize != arguments.len() {
                return Err(TypedCompileError::WrongTypeArity {
                    name: name.clone(),
                    expected,
                    actual: arguments.len(),
                });
            }
            Ok(CompiledType::Generic {
                constructor,
                arguments: arguments
                    .iter()
                    .map(|argument| resolve_type(argument, package, type_variables, type_arities))
                    .collect::<Result<Vec<_>, _>>()?,
            })
        }
        Type::Variable(name) => type_variables
            .get(name)
            .copied()
            .map(CompiledType::Variable)
            .ok_or_else(|| TypedCompileError::UnknownType(format!("${name}"))),
        Type::Function { parameters, returns } => Ok(CompiledType::Function {
            parameters: parameters
                .iter()
                .map(|parameter| resolve_type(&parameter.ty, package, type_variables, type_arities))
                .collect::<Result<Vec<_>, _>>()?,
            returns: Box::new(resolve_type(returns, package, type_variables, type_arities)?),
        }),
        Type::Record(fields) => Ok(CompiledType::Record(
            fields
                .iter()
                .map(|(name, ty)| {
                    Ok((
                        FieldId::from_source("record-field", name),
                        resolve_type(ty, package, type_variables, type_arities)?,
                    ))
                })
                .collect::<Result<Vec<_>, TypedCompileError>>()?,
        )),
    }
}

fn compile_expr(
    expr: &SourceExpr,
    package: &TypedSemanticPackage,
    locals: &[String],
    type_variables: &BTreeMap<String, u32>,
    type_arities: &BTreeMap<String, u32>,
) -> Result<CompiledTerm, TypedCompileError> {
    match expr {
        SourceExpr::Local(name) => locals
            .iter()
            .rev()
            .position(|local| local == name)
            .map(|index| CompiledTerm::Bound(index as u32))
            .ok_or_else(|| TypedCompileError::UnboundLocal(name.clone())),
        SourceExpr::Name(name) => resolve_value_name(name, package),
        SourceExpr::Integer(value) => Ok(CompiledTerm::Scalar(CompiledScalar::Integer(*value))),
        SourceExpr::Boolean(value) => Ok(CompiledTerm::Scalar(CompiledScalar::Boolean(*value))),
        SourceExpr::String(value) => Ok(CompiledTerm::Scalar(CompiledScalar::String(value.clone()))),
        SourceExpr::Call { function, arguments } => {
            let compiled = arguments
                .iter()
                .map(|argument| compile_expr(argument, package, locals, type_variables, type_arities))
                .collect::<Result<Vec<_>, _>>()?;
            if let Some(symbol) = package.symbol_names.get(function).copied() {
                return Ok(CompiledTerm::Apply { function: symbol, arguments: compiled });
            }
            if let Some(constructor) = package.constructor_names.get(function).copied() {
                return Ok(CompiledTerm::Constructor { constructor, fields: compiled });
            }
            Err(TypedCompileError::UnknownSymbol(function.clone()))
        }
        SourceExpr::Invoke { function, arguments } => {
            let function = compile_expr(function, package, locals, type_variables, type_arities)?;
            let arguments = arguments
                .iter()
                .map(|argument| compile_expr(argument, package, locals, type_variables, type_arities))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CompiledTerm::Invoke { function: Box::new(function), arguments })
        }
        SourceExpr::Lambda { parameter, body } => {
            let parameter_type = resolve_type(&parameter.ty, package, type_variables, type_arities)?;
            let mut nested_locals = locals.to_vec();
            nested_locals.push(parameter.name.clone());
            Ok(CompiledTerm::Lambda {
                parameter_type,
                body: Box::new(compile_expr(
                    body,
                    package,
                    &nested_locals,
                    type_variables,
                    type_arities,
                )?),
            })
        }
    }
}

fn resolve_value_name(
    name: &str,
    package: &TypedSemanticPackage,
) -> Result<CompiledTerm, TypedCompileError> {
    if let Some(id) = package.symbol_names.get(name).copied() {
        return Ok(CompiledTerm::Symbol(id));
    }
    if let Some(id) = package.constructor_names.get(name).copied() {
        return Ok(CompiledTerm::Constructor { constructor: id, fields: Vec::new() });
    }
    if let Some(id) = package.context_names.get(name).copied() {
        return Ok(CompiledTerm::Context(id));
    }
    if let Some(id) = package.unit_names.get(name).copied() {
        return Ok(CompiledTerm::Unit(id));
    }
    Err(TypedCompileError::UnknownSymbol(name.to_owned()))
}

fn validate_subtypes(package: &TypedSemanticPackage, errors: &mut Vec<TypedCompileError>) {
    fn visit(
        current: TypeId,
        package: &TypedSemanticPackage,
        visiting: &mut BTreeSet<TypeId>,
        visited: &mut BTreeSet<TypeId>,
        path: &mut Vec<TypeId>,
    ) -> Option<Vec<TypeId>> {
        if visited.contains(&current) {
            return None;
        }
        if !visiting.insert(current) {
            let start = path.iter().position(|id| *id == current).unwrap_or(0);
            let mut cycle = path[start..].to_vec();
            cycle.push(current);
            return Some(cycle);
        }
        path.push(current);
        for (_, parent) in package.subtypes.iter().filter(|(child, _)| *child == current) {
            if let Some(cycle) = visit(*parent, package, visiting, visited, path) {
                return Some(cycle);
            }
        }
        path.pop();
        visiting.remove(&current);
        visited.insert(current);
        None
    }

    let mut visited = BTreeSet::new();
    for start in package.types.keys().copied() {
        let mut visiting = BTreeSet::new();
        let mut path = Vec::new();
        if let Some(cycle) = visit(start, package, &mut visiting, &mut visited, &mut path) {
            errors.push(TypedCompileError::CyclicSubtype(cycle));
            return;
        }
    }
}

fn validate_units(package: &TypedSemanticPackage, errors: &mut Vec<TypedCompileError>) {
    for unit in package.units.values() {
        let Some(base_id) = unit.base else { continue; };
        let Some(base) = package.units.get(&base_id) else { continue; };
        if base.dimension != unit.dimension {
            let unit_name = source_name_for_unit(package, unit.id);
            let base_name = source_name_for_unit(package, base_id);
            errors.push(TypedCompileError::UnitDimensionMismatch {
                unit: unit_name,
                base: base_name,
            });
        }
    }
    if !errors.is_empty() {
        return;
    }

    let mut visited = BTreeSet::new();
    for id in package.units.keys().copied() {
        let mut path = Vec::new();
        let mut position = BTreeMap::new();
        let mut current = Some(id);
        while let Some(unit_id) = current {
            if visited.contains(&unit_id) {
                break;
            }
            if let Some(start) = position.get(&unit_id).copied() {
                let mut cycle = path[start..].to_vec();
                cycle.push(unit_id);
                errors.push(TypedCompileError::CyclicUnitDefinition(cycle));
                return;
            }
            position.insert(unit_id, path.len());
            path.push(unit_id);
            current = package.units.get(&unit_id).and_then(|unit| unit.base);
        }
        visited.extend(path);
    }
}

fn source_name_for_unit(package: &TypedSemanticPackage, id: UnitId) -> String {
    package
        .unit_names
        .iter()
        .find_map(|(name, candidate)| (*candidate == id).then(|| name.clone()))
        .unwrap_or_else(|| id.to_string())
}

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left.max(1)
}

fn validate_compiled_arities(
    package: &TypedSemanticPackage,
    errors: &mut Vec<TypedCompileError>,
) {
    for symbol in package.symbols.values() {
        if let Some(definition) = &symbol.definition {
            validate_term_arities(definition, package, errors);
        }
    }
}

fn validate_term_arities(
    term: &CompiledTerm,
    package: &TypedSemanticPackage,
    errors: &mut Vec<TypedCompileError>,
) {
    match term {
        CompiledTerm::Apply { function, arguments } => {
            if let Some(symbol) = package.symbols.get(function) {
                let expected = symbol.signature.parameters.len();
                if expected != arguments.len() {
                    let name = package
                        .debug_symbols
                        .get(function)
                        .map(|debug| debug.source_name.clone())
                        .unwrap_or_else(|| function.to_string());
                    errors.push(TypedCompileError::InvalidCallArity {
                        name,
                        expected,
                        actual: arguments.len(),
                    });
                }
            }
            for argument in arguments {
                validate_term_arities(argument, package, errors);
            }
        }
        CompiledTerm::Constructor { constructor, fields } => {
            if let Some(declaration) = package.constructors.get(constructor) {
                let expected = declaration.fields.len();
                if expected != fields.len() {
                    let name = package
                        .constructor_names
                        .iter()
                        .find_map(|(name, id)| (*id == *constructor).then(|| name.clone()))
                        .unwrap_or_else(|| constructor.to_string());
                    errors.push(TypedCompileError::InvalidCallArity {
                        name,
                        expected,
                        actual: fields.len(),
                    });
                }
            }
            for field in fields {
                validate_term_arities(field, package, errors);
            }
        }
        CompiledTerm::Invoke { function, arguments } => {
            validate_term_arities(function, package, errors);
            for argument in arguments {
                validate_term_arities(argument, package, errors);
            }
        }
        CompiledTerm::Lambda { body, .. } => validate_term_arities(body, package, errors),
        CompiledTerm::Symbol(_)
        | CompiledTerm::Context(_)
        | CompiledTerm::Unit(_)
        | CompiledTerm::Bound(_)
        | CompiledTerm::Scalar(_) => {}
    }
}

fn validate_definition_cycles(package: &TypedSemanticPackage) -> Result<(), TypedCompileError> {
    let defined = package
        .symbols
        .iter()
        .filter_map(|(id, symbol)| symbol.definition.as_ref().map(|_| *id))
        .collect::<BTreeSet<_>>();
    let mut dependencies = BTreeMap::<SymbolId, BTreeSet<SymbolId>>::new();
    for id in &defined {
        let symbol = &package.symbols[id];
        let mut deps = BTreeSet::new();
        collect_symbol_dependencies(symbol.definition.as_ref().expect("defined symbol"), &mut deps);
        deps.retain(|dependency| defined.contains(dependency));
        dependencies.insert(*id, deps);
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut stack = Vec::new();
    for id in defined {
        if let Some(cycle) = visit_definition(id, &dependencies, &mut visiting, &mut visited, &mut stack) {
            return Err(TypedCompileError::CyclicDefinition(cycle));
        }
    }
    Ok(())
}

fn visit_definition(
    id: SymbolId,
    dependencies: &BTreeMap<SymbolId, BTreeSet<SymbolId>>,
    visiting: &mut BTreeSet<SymbolId>,
    visited: &mut BTreeSet<SymbolId>,
    stack: &mut Vec<SymbolId>,
) -> Option<Vec<SymbolId>> {
    if visited.contains(&id) {
        return None;
    }
    if !visiting.insert(id) {
        let start = stack.iter().position(|candidate| *candidate == id).unwrap_or(0);
        let mut cycle = stack[start..].to_vec();
        cycle.push(id);
        return Some(cycle);
    }
    stack.push(id);
    if let Some(next) = dependencies.get(&id) {
        for dependency in next {
            if let Some(cycle) = visit_definition(*dependency, dependencies, visiting, visited, stack) {
                return Some(cycle);
            }
        }
    }
    stack.pop();
    visiting.remove(&id);
    visited.insert(id);
    None
}

fn collect_symbol_dependencies(term: &CompiledTerm, output: &mut BTreeSet<SymbolId>) {
    match term {
        CompiledTerm::Symbol(id) => {
            output.insert(*id);
        }
        CompiledTerm::Apply { function, arguments } => {
            output.insert(*function);
            for argument in arguments {
                collect_symbol_dependencies(argument, output);
            }
        }
        CompiledTerm::Constructor { fields, .. } => {
            for field in fields {
                collect_symbol_dependencies(field, output);
            }
        }
        CompiledTerm::Invoke { function, arguments } => {
            collect_symbol_dependencies(function, output);
            for argument in arguments {
                collect_symbol_dependencies(argument, output);
            }
        }
        CompiledTerm::Lambda { body, .. } => collect_symbol_dependencies(body, output),
        CompiledTerm::Context(_)
        | CompiledTerm::Unit(_)
        | CompiledTerm::Bound(_)
        | CompiledTerm::Scalar(_) => {}
    }
}

fn insert_identity<I: Copy>(
    namespace: &'static str,
    name: &str,
    raw_id: u64,
    by_name: &mut BTreeMap<String, I>,
    errors: &mut Vec<TypedCompileError>,
) where
    I: FromRawId,
{
    if by_name.contains_key(name) {
        errors.push(TypedCompileError::DuplicateDeclaration {
            namespace,
            name: name.to_owned(),
        });
        return;
    }
    if let Some((other, _)) = by_name
        .iter()
        .find(|(_, id)| (**id).raw_id() == raw_id)
    {
        errors.push(TypedCompileError::IdentityCollision {
            namespace,
            left: other.clone(),
            right: name.to_owned(),
        });
        return;
    }
    by_name.insert(name.to_owned(), I::from_raw_id(raw_id));
}

trait FromRawId: Copy {
    fn from_raw_id(raw: u64) -> Self;
    fn raw_id(self) -> u64;
}

macro_rules! raw_id_impl {
    ($ty:ty) => {
        impl FromRawId for $ty {
            fn from_raw_id(raw: u64) -> Self { Self(raw) }
            fn raw_id(self) -> u64 { self.0 }
        }
    };
}

raw_id_impl!(TypeId);
raw_id_impl!(SymbolId);
raw_id_impl!(ConstructorId);
raw_id_impl!(ContextSlotId);
raw_id_impl!(DimensionId);
raw_id_impl!(UnitId);

fn validate_parameter_names(
    declaration: &str,
    parameters: &[SourceParameter],
    errors: &mut Vec<TypedCompileError>,
) {
    let names = parameters.iter().map(|parameter| parameter.name.clone()).collect::<Vec<_>>();
    validate_unique_names(declaration, "parameter", &names, errors);
}

fn validate_unique_names(
    declaration: &str,
    kind: &'static str,
    names: &[String],
    errors: &mut Vec<TypedCompileError>,
) {
    let mut seen = BTreeSet::new();
    for name in names {
        if !seen.insert(name.clone()) {
            errors.push(match kind {
                "parameter" => TypedCompileError::DuplicateParameter {
                    declaration: declaration.to_owned(),
                    parameter: name.clone(),
                },
                _ => TypedCompileError::DuplicateTypeParameter {
                    declaration: declaration.to_owned(),
                    parameter: name.clone(),
                },
            });
        }
    }
}

fn type_variable_map(parameters: &[String]) -> BTreeMap<String, u32> {
    parameters
        .iter()
        .enumerate()
        .map(|(index, name)| (name.clone(), index as u32))
        .collect()
}

fn semantic_fingerprint(package: &TypedSemanticPackage) -> String {
    let mut bytes = Vec::new();
    for (id, arity) in &package.types {
        bytes.extend_from_slice(&id.0.to_le_bytes());
        bytes.extend_from_slice(&arity.to_le_bytes());
    }
    for (child, parent) in &package.subtypes {
        bytes.push(0x51);
        bytes.extend_from_slice(&child.0.to_le_bytes());
        bytes.extend_from_slice(&parent.0.to_le_bytes());
    }
    for (kind, ty) in &package.literal_types {
        bytes.push(0x52);
        bytes.push(match kind { LiteralKind::Integer => 1, LiteralKind::Boolean => 2, LiteralKind::String => 3 });
        encode_type(ty, &mut bytes);
    }
    for (id, constructor) in &package.constructors {
        bytes.extend_from_slice(&id.0.to_le_bytes());
        bytes.extend_from_slice(&constructor.owner.0.to_le_bytes());
        for field in &constructor.fields {
            encode_type(field, &mut bytes);
        }
    }
    for id in &package.dimensions {
        bytes.extend_from_slice(&id.0.to_le_bytes());
        if let Some(ty) = package.dimension_types.get(id) {
            encode_type(ty, &mut bytes);
        }
    }
    for (id, context) in &package.contexts {
        bytes.extend_from_slice(&id.0.to_le_bytes());
        encode_type(&context.ty, &mut bytes);
    }
    for (id, symbol) in &package.symbols {
        bytes.extend_from_slice(&id.0.to_le_bytes());
        bytes.push(match symbol.kind {
            CompiledSymbolKind::Word => 1,
            CompiledSymbolKind::Primitive => 2,
            CompiledSymbolKind::Definition => 3,
            CompiledSymbolKind::Intrinsic => 4,
        });
        bytes.extend_from_slice(&symbol.signature.type_parameter_count.to_le_bytes());
        for parameter in &symbol.signature.parameters {
            encode_type(parameter, &mut bytes);
        }
        encode_type(&symbol.signature.returns, &mut bytes);
        if let Some(definition) = &symbol.definition {
            bytes.push(1);
            encode_term(definition, &mut bytes);
        } else {
            bytes.push(0);
        }
    }
    for (id, program) in &package.effect_programs {
        bytes.push(0x60);
        bytes.extend_from_slice(&id.0.to_le_bytes());
        for instruction in &program.instructions {
            match instruction {
                CompiledEffectInstruction::Act { kind, arguments } => {
                    bytes.push(1);
                    bytes.push(match kind {
                        CompiledActKind::Assertion => 1,
                        CompiledActKind::Question => 2,
                        CompiledActKind::Command => 3,
                        CompiledActKind::Request => 4,
                        CompiledActKind::Expressive => 5,
                        CompiledActKind::Focus => 6,
                        CompiledActKind::Topic => 7,
                        CompiledActKind::Retraction => 8,
                        CompiledActKind::Correction => 9,
                        CompiledActKind::Clarification => 10,
                    });
                    for argument in arguments { bytes.extend_from_slice(&argument.to_le_bytes()); }
                    bytes.push(0xff);
                }
                CompiledEffectInstruction::Commit { argument } => {
                    bytes.push(2); bytes.extend_from_slice(&argument.to_le_bytes());
                }
                CompiledEffectInstruction::RequireContains { content, target } => {
                    bytes.push(3); bytes.extend_from_slice(&content.to_le_bytes()); bytes.extend_from_slice(&target.to_le_bytes());
                }
                CompiledEffectInstruction::Choice { operator } => {
                    bytes.push(4); bytes.extend_from_slice(&operator.0.to_le_bytes());
                }
                CompiledEffectInstruction::Repair { kind, target, value } => {
                    bytes.push(5);
                    bytes.push(match kind { CompiledRepairKind::Retract => 1, CompiledRepairKind::Replace => 2, CompiledRepairKind::Clarify => 3 });
                    bytes.extend_from_slice(&target.to_le_bytes());
                    match value { Some(value) => { bytes.push(1); bytes.extend_from_slice(&value.to_le_bytes()); }, None => bytes.push(0) }
                }
            }
        }
    }
    match package.default_effect {
        Some(symbol) => { bytes.push(0x61); bytes.extend_from_slice(&symbol.0.to_le_bytes()); }
        None => bytes.push(0x62),
    }
    for (id, unit) in &package.units {
        bytes.extend_from_slice(&id.0.to_le_bytes());
        bytes.extend_from_slice(&unit.dimension.0.to_le_bytes());
        bytes.extend_from_slice(&unit.scale_numerator.to_le_bytes());
        bytes.extend_from_slice(&unit.scale_denominator.to_le_bytes());
        match unit.base {
            Some(base) => {
                bytes.push(1);
                bytes.extend_from_slice(&base.0.to_le_bytes());
            }
            None => bytes.push(0),
        }
    }
    stable_digest(&bytes)
}

fn surface_fingerprint(package: &TypedSemanticPackage) -> String {
    let mut bytes = Vec::new();
    for (id, rule) in &package.surface_rules {
        bytes.extend_from_slice(&id.0.to_le_bytes());
        let root = package.source_name_for_symbol(*id).unwrap_or("");
        bytes.extend_from_slice(root.as_bytes());
        bytes.push(0);
        for item in &rule.items {
            match item {
                CompiledSurfaceItem::Root => bytes.push(1),
                CompiledSurfaceItem::Argument(index) => {
                    bytes.push(2);
                    bytes.extend_from_slice(&index.to_le_bytes());
                }
                CompiledSurfaceItem::Restriction => bytes.push(5),
            }
        }
        match rule.precedence {
            Some(precedence) => {
                bytes.push(3);
                bytes.extend_from_slice(&precedence.to_le_bytes());
            }
            None => bytes.push(4),
        }
        bytes.push(u8::from(rule.associative));
        bytes.push(u8::from(rule.outer_only));
        if let Some(binder) = &rule.binder {
            bytes.push(6);
            bytes.extend_from_slice(&binder.parameter.to_le_bytes());
            encode_type(&binder.variable_type, &mut bytes);
            bytes.extend_from_slice(&binder.combiner.0.to_le_bytes());
        } else {
            bytes.push(7);
        }
        for capture in &rule.captures {
            bytes.push(8);
            bytes.extend_from_slice(&capture.parameter.to_le_bytes());
            bytes.push(match capture.kind { CompiledCaptureKind::BareToken => 1 });
        }
    }
    stable_digest(&bytes)
}

fn encode_type(ty: &CompiledType, output: &mut Vec<u8>) {
    match ty {
        CompiledType::Named(id) => {
            output.push(1);
            output.extend_from_slice(&id.0.to_le_bytes());
        }
        CompiledType::Generic { constructor, arguments } => {
            output.push(2);
            output.extend_from_slice(&constructor.0.to_le_bytes());
            output.extend_from_slice(&(arguments.len() as u64).to_le_bytes());
            for argument in arguments { encode_type(argument, output); }
        }
        CompiledType::Variable(index) => {
            output.push(3);
            output.extend_from_slice(&index.to_le_bytes());
        }
        CompiledType::Function { parameters, returns } => {
            output.push(4);
            output.extend_from_slice(&(parameters.len() as u64).to_le_bytes());
            for parameter in parameters { encode_type(parameter, output); }
            encode_type(returns, output);
        }
        CompiledType::Record(fields) => {
            output.push(5);
            output.extend_from_slice(&(fields.len() as u64).to_le_bytes());
            for (field, ty) in fields {
                output.extend_from_slice(&field.0.to_le_bytes());
                encode_type(ty, output);
            }
        }
    }
}

fn encode_term(term: &CompiledTerm, output: &mut Vec<u8>) {
    match term {
        CompiledTerm::Symbol(id) => {
            output.push(1);
            output.extend_from_slice(&id.0.to_le_bytes());
        }
        CompiledTerm::Apply { function, arguments } => {
            output.push(2);
            output.extend_from_slice(&function.0.to_le_bytes());
            output.extend_from_slice(&(arguments.len() as u64).to_le_bytes());
            for argument in arguments { encode_term(argument, output); }
        }
        CompiledTerm::Constructor { constructor, fields } => {
            output.push(3);
            output.extend_from_slice(&constructor.0.to_le_bytes());
            output.extend_from_slice(&(fields.len() as u64).to_le_bytes());
            for field in fields { encode_term(field, output); }
        }
        CompiledTerm::Invoke { function, arguments } => {
            output.push(4);
            encode_term(function, output);
            output.extend_from_slice(&(arguments.len() as u64).to_le_bytes());
            for argument in arguments { encode_term(argument, output); }
        }
        CompiledTerm::Context(id) => {
            output.push(5);
            output.extend_from_slice(&id.0.to_le_bytes());
        }
        CompiledTerm::Unit(id) => {
            output.push(6);
            output.extend_from_slice(&id.0.to_le_bytes());
        }
        CompiledTerm::Bound(index) => {
            output.push(7);
            output.extend_from_slice(&index.to_le_bytes());
        }
        CompiledTerm::Lambda { parameter_type, body } => {
            output.push(8);
            encode_type(parameter_type, output);
            encode_term(body, output);
        }
        CompiledTerm::Scalar(CompiledScalar::Integer(value)) => {
            output.push(9);
            output.extend_from_slice(&value.to_le_bytes());
        }
        CompiledTerm::Scalar(CompiledScalar::Boolean(value)) => {
            output.push(10);
            output.push(u8::from(*value));
        }
        CompiledTerm::Scalar(CompiledScalar::String(value)) => {
            output.push(11);
            output.extend_from_slice(value.as_bytes());
            output.push(0);
        }
    }
}

fn stable_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}


fn locate_declaration_spans(
    source: &str,
    specification: &TypedSpecification,
) -> Vec<Option<SourceSpan>> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut start_line = 0usize;
    specification
        .declarations
        .iter()
        .map(|declaration| {
            let name = declaration.source_name();
            let keyword = match declaration {
                TypedDeclaration::Type { .. } => "type",
                TypedDeclaration::Subtype { .. } => "subtype",
                TypedDeclaration::Literal { .. } => "literal",
                TypedDeclaration::Word { .. } => "word",
                TypedDeclaration::Primitive { .. } => "primitive",
                TypedDeclaration::Def { .. } => "def",
                TypedDeclaration::Intrinsic { .. } => "intrinsic",
                TypedDeclaration::Data { .. } => "data",
                TypedDeclaration::Context { .. } => "context",
                TypedDeclaration::Dimension { .. } => "dimension",
                TypedDeclaration::Effect { .. } => "effect",
                TypedDeclaration::DefaultEffect { .. } => "default",
                TypedDeclaration::Unit { .. } => "unit",
            };
            let mut found = None;
            for (index, line) in lines.iter().enumerate().skip(start_line) {
                let trimmed = line.trim_start();
                let Some(after_keyword) = trimmed.strip_prefix(keyword) else { continue; };
                if !after_keyword.chars().next().is_some_and(char::is_whitespace) {
                    continue;
                }
                let after_keyword = after_keyword.trim_start();
                let after_keyword = if matches!(declaration, TypedDeclaration::DefaultEffect { .. }) {
                    let Some(rest) = after_keyword.strip_prefix("effect") else { continue; };
                    rest.trim_start()
                } else {
                    after_keyword
                };
                let matches_name = after_keyword.strip_prefix(name).is_some_and(|rest| {
                    rest.is_empty()
                        || rest.chars().next().is_some_and(|ch| {
                            ch.is_whitespace() || matches!(ch, ';' | '<' | '(' | ':' | '{')
                        })
                });
                if matches_name {
                    let column = line.len() - trimmed.len() + 1;
                    found = Some(SourceSpan { line: index + 1, column });
                    start_line = index;
                    break;
                }
            }
            found
        })
        .collect()
}

impl fmt::Display for TypedCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse { source, error } => write!(f, "{source}: {error}"),
            Self::DuplicateDeclaration { namespace, name } => {
                write!(f, "duplicate {namespace} declaration `{name}`")
            }
            Self::IdentityCollision { namespace, left, right } => write!(
                f,
                "{namespace} identity collision between `{left}` and `{right}`",
            ),
            Self::DuplicateParameter { declaration, parameter } => write!(
                f,
                "`{declaration}` declares parameter `${parameter}` more than once",
            ),
            Self::DuplicateTypeParameter { declaration, parameter } => write!(
                f,
                "`{declaration}` declares type parameter `{parameter}` more than once",
            ),
            Self::DuplicateLiteralKind(kind) => write!(f, "literal kind `{kind:?}` is already typed"),
            Self::UnknownType(name) => write!(f, "unknown type `{name}`"),
            Self::InvalidSubtype { child, parent } => write!(f, "subtype `{child}: {parent}` requires non-generic named types"),
            Self::CyclicSubtype(cycle) => write!(f, "cyclic subtype relation: {cycle:?}"),
            Self::WrongTypeArity { name, expected, actual } => write!(
                f,
                "type `{name}` expects {expected} type argument(s) but received {actual}",
            ),
            Self::UnknownSymbol(name) => write!(f, "unknown semantic symbol `{name}`"),
            Self::UnknownDimension(name) => write!(f, "unknown dimension `{name}`"),
            Self::UnknownUnit(name) => write!(f, "unknown unit `{name}`"),
            Self::InvalidUnitScale { name } => write!(f, "unit `{name}` must use a positive exact rational scale"),
            Self::UnitDimensionMismatch { unit, base } => write!(
                f,
                "unit `{unit}` derives from `{base}` in a different dimension",
            ),
            Self::InvalidCallArity { name, expected, actual } => write!(
                f,
                "`{name}` expects {expected} argument(s) but received {actual}",
            ),
            Self::UnboundLocal(name) => write!(f, "unbound local `${name}`"),
            Self::CyclicDefinition(cycle) => write!(f, "cyclic semantic definition: {cycle:?}"),
            Self::CyclicUnitDefinition(cycle) => write!(f, "cyclic unit definition: {cycle:?}"),
            Self::UnknownSurfaceParameter { word, parameter } => write!(
                f,
                "surface form for `{word}` references unknown parameter `${parameter}`",
            ),
            Self::DuplicateSurfaceParameter { word, parameter } => write!(
                f,
                "surface form for `{word}` uses parameter `${parameter}` more than once",
            ),
            Self::MissingSurfaceParameter { word, parameter } => write!(
                f,
                "surface form for `{word}` does not realize parameter `${parameter}`",
            ),
            Self::InvalidSurfaceRootCount { word, count } => write!(
                f,
                "surface form for `{word}` must contain exactly one `_` root marker, found {count}",
            ),
            Self::InvalidSurfacePrecedence { word } => write!(
                f,
                "surface precedence for `{word}` requires a binary `$left _ $right` form",
            ),
            Self::InvalidAssociativeSurface { word } => write!(
                f,
                "associative surface form `{word}` must be a precedence-bearing binary endomorphism",
            ),
            Self::InvalidSurfaceBinder { word, message } => write!(f, "surface binder for `{word}`: {message}"),
            Self::InvalidSurfaceCapture { word, message } => write!(f, "surface capture for `{word}`: {message}"),
            Self::DuplicateEffect { target } => write!(f, "duplicate discourse effect declaration for `{target}`"),
            Self::DuplicateDefaultEffect => f.write_str("more than one default discourse effect is declared"),
            Self::InvalidEffectParameter { target, parameter } => write!(
                f,
                "discourse effect for `{target}` references unknown parameter `${parameter}`",
            ),
            Self::InvalidEffect { target, message } => write!(f, "discourse effect for `{target}`: {message}"),
        }
    }
}

impl std::error::Error for TypedCompileError {}
