mod resolve;
mod state;

pub use resolve::{
    DiscourseResolutionError, ResolvedContextBinding, ResolvedReferenceBinding,
    ResolvedSurfaceAst, resolve_surface,
};
pub use state::{
    AccessibilityScopeId, ContextValue, DiscourseError, DiscourseState, IntroductionOrigin,
    ReferenceCandidate, Referent, ReferentId, ResolvedReference,
};
