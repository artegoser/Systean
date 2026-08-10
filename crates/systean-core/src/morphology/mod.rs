mod config;
mod engine;

pub use config::{MorphologyConfig, MorphologyConfigError, MorphologyStrategy};
pub use engine::{MorphemeAnalysis, MorphemeKind, MorphologyAnalysis, MorphologyError, MorphologyEngine};
