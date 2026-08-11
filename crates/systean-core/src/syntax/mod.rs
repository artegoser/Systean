mod ast;
mod config;
mod elaborate;
mod engine;
mod generate;
mod lexicon;
mod lower;
mod parser;

pub use ast::{Argument, Clause, SurfaceExpr};
pub use config::{
    ArgumentOmission, DiscourseConfig, ExplicitOperatorRealization, ExplicitScopePolicy, FrameOrder, GrammarConfig,
    QuantifierScopePolicy, RoleRealization, SurfaceFormConfig, SyntaxConfig, SyntaxConfigError,
};
pub use elaborate::{
    AliasSlot, ContextSlot, ReferenceSlot, ReferenceSource, SurfaceElaborationError, TypedSurfaceAst,
    elaborate_surface,
};
pub use engine::{SurfaceAnalysis, SurfaceError, SyntaxEngine};
pub use generate::{SurfaceGenerationError, linearize_surface};
pub use lexicon::{LexemeConfig, SurfaceLexicon};
pub use lower::{LoweredSurface, SurfaceLowerError, lower_surface};
pub use parser::{SurfaceParseError, parse_surface, tokenize};
