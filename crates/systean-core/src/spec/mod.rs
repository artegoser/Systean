mod ast;
mod compile;
mod package;
mod parser;
mod typed_ast;
mod typed_compile;
mod typed_parser;

pub use ast::{Declaration, ParsedTerm, Specification};
pub use compile::{CompileError, SourceSpecification, compile_specification, compile_specifications, lower_term};
pub use parser::{ParseError, parse_specification, parse_term, parse_type};

pub use package::{PackageError, compile_path, compile_sources, load_specifications};


pub use typed_ast::{
    CompiledConstructor, CompiledContextSlot, CompiledScalar, CompiledSignature, CompiledSymbol,
    CompiledSymbolKind, CompiledTerm, CompiledType, CompiledUnit, DataConstructorDeclaration,
    DeclarationProvenance, DefaultSurfaceFrame, DefaultSurfaceItem, SourceExpr, SourceParameter,
    SourceSpan, SourceUnitDefinition, SymbolDebugInfo, TypedDeclaration,
    TypedSpecification,
};
pub use typed_compile::{
    TypedCompileError, TypedSemanticPackage, TypedSourceSpecification, compile_typed_sources,
    compile_typed_specifications,
};
pub use typed_parser::{TypedParseError, parse_typed_specification};
