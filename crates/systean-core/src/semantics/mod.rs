mod checker;
mod environment;
mod explain;
mod normalize;
mod origin;
mod signature;
mod term;
mod types;

pub use checker::{CheckError, Checker};
pub use environment::{Environment, EnvironmentError, LiteralKind, TypeDefinition};
pub use explain::{Explainer, Explanation, ExplanationEdge};
pub use normalize::canonicalize;
pub use origin::Origin;
pub use signature::{Parameter, Signature};
pub use term::{Literal, StructuredLiteral, Term};
pub use types::{FunctionParameter, Type};
