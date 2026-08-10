mod checker;
mod environment;
mod normalize;
mod signature;
mod term;
mod types;

pub use checker::{CheckError, Checker};
pub use environment::{Environment, EnvironmentError, LiteralKind, TypeDefinition};
pub use normalize::canonicalize;
pub use signature::{Parameter, Signature};
pub use term::{Literal, Term};
pub use types::{FunctionParameter, Type};
