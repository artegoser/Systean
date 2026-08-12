mod ast;
mod config;
mod elaborate;
mod engine;
mod generate;
mod lexicon;
mod lower;
mod parser;

pub use ast::{Argument, Clause, InformationKnower, SurfaceExpr};
pub use config::{
    ArgumentOmission, ArgumentsConfig, DiscourseConfig, FrameOrder, MetaConfig, OrderConfig,
    QuotationConfig, ScopeConfig, SyntaxConfig, SyntaxConfigError, TextConfig,
};
pub use elaborate::{
    AliasSlot, ContextSlot, ReferenceSlot, ReferenceSource, SurfaceElaborationError, TypedSurfaceAst,
    elaborate_surface,
};
pub use engine::{SurfaceAnalysis, SurfaceError, SyntaxEngine};
pub use generate::{SurfaceGenerationError, linearize_surface};
pub use lexicon::{CompiledSurfaceBinding, CompiledSurfaceLexicon};
pub use lower::{LoweredSurface, SurfaceLowerError, lower_surface};
pub use parser::{SurfaceParseError, parse_surface, parse_surface_with_literals, tokenize};
