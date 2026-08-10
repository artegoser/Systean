use serde::Serialize;
use std::fmt;
use std::ops::Range;

use crate::phonology::{PhonologyConfig, RootInventory};

use super::{MorphologyConfig, MorphologyStrategy};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MorphemeKind {
    Root,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MorphemeAnalysis {
    pub kind: MorphemeKind,
    pub spelling: String,
    pub grapheme_range: Range<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MorphologyAnalysis {
    pub spelling: String,
    pub root: String,
    pub root_grapheme_range: Range<usize>,
    pub morphemes: Vec<MorphemeAnalysis>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MorphologyError {
    UnknownWord(String),
    UnknownRoot(String),
    InvalidWord(String),
}

#[derive(Clone, Debug)]
pub struct MorphologyEngine {
    config: MorphologyConfig,
}

impl MorphologyEngine {
    pub fn new(config: MorphologyConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &MorphologyConfig {
        &self.config
    }

    pub fn analyze(
        &self,
        word: &str,
        phonology: &PhonologyConfig,
        roots: &RootInventory,
    ) -> Result<MorphologyAnalysis, MorphologyError> {
        match self.config.strategy {
            MorphologyStrategy::BareRoots => self.analyze_bare_root(word, phonology, roots),
        }
    }

    pub fn generate(
        &self,
        root: &str,
        phonology: &PhonologyConfig,
        roots: &RootInventory,
    ) -> Result<String, MorphologyError> {
        match self.config.strategy {
            MorphologyStrategy::BareRoots => {
                let analysis = phonology
                    .analyze_root(root)
                    .map_err(|error| MorphologyError::InvalidWord(error.to_string()))?;
                if roots.roots().iter().any(|candidate| candidate == &analysis.canonical_spelling) {
                    Ok(analysis.canonical_spelling)
                } else {
                    Err(MorphologyError::UnknownRoot(analysis.canonical_spelling))
                }
            }
        }
    }

    fn analyze_bare_root(
        &self,
        word: &str,
        phonology: &PhonologyConfig,
        roots: &RootInventory,
    ) -> Result<MorphologyAnalysis, MorphologyError> {
        let analysis = phonology
            .analyze_root(word)
            .map_err(|error| MorphologyError::InvalidWord(error.to_string()))?;
        let spelling = analysis.canonical_spelling;
        if !roots.roots().iter().any(|root| root == &spelling) {
            return Err(MorphologyError::UnknownWord(spelling));
        }
        let range = 0..analysis.graphemes.len();
        Ok(MorphologyAnalysis {
            spelling: spelling.clone(),
            root: spelling.clone(),
            root_grapheme_range: range.clone(),
            morphemes: vec![MorphemeAnalysis {
                kind: MorphemeKind::Root,
                spelling,
                grapheme_range: range,
            }],
        })
    }
}

impl fmt::Display for MorphologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownWord(word) => write!(
                f,
                "`{word}` has no normative morphological analysis; current Systean morphology accepts declared bare roots only"
            ),
            Self::UnknownRoot(root) => write!(f, "unknown lexical root `{root}`"),
            Self::InvalidWord(error) => write!(f, "invalid word form: {error}"),
        }
    }
}

impl std::error::Error for MorphologyError {}
