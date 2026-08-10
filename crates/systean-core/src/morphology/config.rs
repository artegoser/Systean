use serde::Deserialize;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MorphologyStrategy {
    BareRoots,
}

impl fmt::Display for MorphologyStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BareRoots => f.write_str("bare_roots"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct MorphologyConfig {
    pub strategy: MorphologyStrategy,
}

#[derive(Clone, Debug, Deserialize)]
struct MorphologyFile {
    morphology: MorphologyConfig,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MorphologyConfigError {
    Io(String),
    Toml(String),
}

impl MorphologyConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, MorphologyConfigError> {
        let source = fs::read_to_string(path.as_ref())
            .map_err(|error| MorphologyConfigError::Io(error.to_string()))?;
        Self::from_toml(&source)
    }

    pub fn from_toml(source: &str) -> Result<Self, MorphologyConfigError> {
        let parsed: MorphologyFile = toml::from_str(source)
            .map_err(|error| MorphologyConfigError::Toml(error.to_string()))?;
        Ok(parsed.morphology)
    }
}

impl fmt::Display for MorphologyConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Toml(error) => write!(f, "TOML error: {error}"),
        }
    }
}

impl std::error::Error for MorphologyConfigError {}
