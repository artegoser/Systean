mod ast;
mod compile;
mod parser;

pub use ast::{Declaration, ParsedTerm, Specification};
pub use compile::{CompileError, compile_specification, lower_term};
pub use parser::{ParseError, parse_specification, parse_term, parse_type};
