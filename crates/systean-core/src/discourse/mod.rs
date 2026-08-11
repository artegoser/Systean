mod generate;
mod resolve;
mod state;

pub use generate::{DiscourseGenerationError, materialize_resolved_surface};
pub use resolve::{
    DiscourseResolutionError, ResolvedAliasBinding, ResolvedContextBinding, ResolvedReferenceBinding,
    ResolvedSurfaceAst, resolve_surface,
};
pub use state::{
    AccessibilityScopeId, AliasBinding, ContextValue, DiscourseError, DiscourseFrameId, DiscourseState, IntroductionOrigin,
    ReferenceCandidate, Referent, ReferentId, ResolvedReference,
};
