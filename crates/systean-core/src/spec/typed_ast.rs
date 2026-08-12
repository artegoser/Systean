use crate::semantics::{
    ConstructorId, ContextSlotId, DimensionId, IntrinsicId, SymbolId, Type, TypeId, UnitId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedSpecification {
    pub declarations: Vec<TypedDeclaration>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceParameter {
    pub name: String,
    pub ty: Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataConstructorDeclaration {
    pub name: String,
    pub parameters: Vec<SourceParameter>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceUnitDefinition {
    pub numerator: u64,
    pub denominator: u64,
    pub base: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedDeclaration {
    Type {
        name: String,
        type_parameters: Vec<String>,
    },
    Word {
        name: String,
        type_parameters: Vec<String>,
        parameters: Vec<SourceParameter>,
        returns: Type,
        definition: Option<SourceExpr>,
    },
    Primitive {
        name: String,
        type_parameters: Vec<String>,
        parameters: Vec<SourceParameter>,
        returns: Type,
    },
    Def {
        name: String,
        type_parameters: Vec<String>,
        parameters: Vec<SourceParameter>,
        returns: Type,
        definition: SourceExpr,
    },
    Intrinsic {
        name: String,
        type_parameters: Vec<String>,
        parameters: Vec<SourceParameter>,
        returns: Type,
    },
    Data {
        name: String,
        type_parameters: Vec<String>,
        constructors: Vec<DataConstructorDeclaration>,
    },
    Context {
        name: String,
        ty: Type,
    },
    Dimension {
        name: String,
    },
    Unit {
        name: String,
        dimension: String,
        definition: Option<SourceUnitDefinition>,
    },
}

impl TypedDeclaration {
    pub fn source_name(&self) -> &str {
        match self {
            Self::Type { name, .. }
            | Self::Word { name, .. }
            | Self::Primitive { name, .. }
            | Self::Def { name, .. }
            | Self::Intrinsic { name, .. }
            | Self::Data { name, .. }
            | Self::Context { name, .. }
            | Self::Dimension { name }
            | Self::Unit { name, .. } => name,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceExpr {
    Name(String),
    Local(String),
    Integer(i64),
    Boolean(bool),
    String(String),
    Call {
        function: String,
        arguments: Vec<SourceExpr>,
    },
    Invoke {
        function: Box<SourceExpr>,
        arguments: Vec<SourceExpr>,
    },
    Lambda {
        parameter: SourceParameter,
        body: Box<SourceExpr>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompiledType {
    Named(TypeId),
    Generic {
        constructor: TypeId,
        arguments: Vec<CompiledType>,
    },
    Variable(u32),
    Function {
        parameters: Vec<CompiledType>,
        returns: Box<CompiledType>,
    },
    Record(Vec<(crate::semantics::FieldId, CompiledType)>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledSignature {
    pub type_parameter_count: u32,
    pub parameters: Vec<CompiledType>,
    pub returns: CompiledType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompiledScalar {
    Integer(i64),
    Boolean(bool),
    String(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompiledTerm {
    Symbol(SymbolId),
    Apply {
        function: SymbolId,
        arguments: Vec<CompiledTerm>,
    },
    Invoke {
        function: Box<CompiledTerm>,
        arguments: Vec<CompiledTerm>,
    },
    Constructor {
        constructor: ConstructorId,
        fields: Vec<CompiledTerm>,
    },
    Context(ContextSlotId),
    Unit(UnitId),
    Bound(u32),
    Lambda {
        parameter_type: CompiledType,
        body: Box<CompiledTerm>,
    },
    Scalar(CompiledScalar),
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DefaultSurfaceItem {
    Root,
    Argument(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefaultSurfaceFrame {
    pub root: String,
    pub items: Vec<DefaultSurfaceItem>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompiledSymbolKind {
    Word,
    Primitive,
    Definition,
    Intrinsic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledSymbol {
    pub id: SymbolId,
    pub kind: CompiledSymbolKind,
    pub signature: CompiledSignature,
    pub definition: Option<CompiledTerm>,
    pub provenance: DeclarationProvenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledConstructor {
    pub id: ConstructorId,
    pub owner: TypeId,
    pub fields: Vec<CompiledType>,
    pub provenance: DeclarationProvenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledContextSlot {
    pub id: ContextSlotId,
    pub ty: CompiledType,
    pub provenance: DeclarationProvenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledUnit {
    pub id: UnitId,
    pub dimension: DimensionId,
    pub scale_numerator: u64,
    pub scale_denominator: u64,
    pub base: Option<UnitId>,
    pub provenance: DeclarationProvenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceSpan {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclarationProvenance {
    pub source: String,
    pub declaration: usize,
    pub span: Option<SourceSpan>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolDebugInfo {
    pub source_name: String,
    pub parameter_names: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntrinsicBinding {
    pub symbol: SymbolId,
    pub intrinsic: IntrinsicId,
}
