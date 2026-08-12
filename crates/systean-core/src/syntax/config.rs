use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SyntaxConfig {
    pub meta: Option<MetaConfig>,
    pub order: OrderConfig,
    pub scope: ScopeConfig,
    pub arguments: ArgumentsConfig,
    pub discourse: DiscourseConfig,
    pub quotation: QuotationConfig,
    pub text: TextConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MetaConfig {
    pub description: Option<String>,
    pub version: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct OrderConfig {
    pub frame: FrameOrder,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FrameOrder {
    PrimaryPredicateRest,
    PredicateArguments,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ScopeConfig {
    pub open: String,
    pub close: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ArgumentsConfig {
    pub omission: ArgumentOmission,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArgumentOmission {
    UniqueReferenceOnly,
    Never,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DiscourseConfig {
    pub alias: String,
    pub definition: String,
    pub relative: String,
    pub frame: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct QuotationConfig {
    pub open: String,
    pub close: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TextConfig {
    pub utterance_spoken: String,
    pub utterance_written: String,
    #[serde(default)]
    pub readability_punctuation: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyntaxConfigError {
    Toml(String),
    EmptyScopeMarker(&'static str),
    EqualScopeMarkers(String),
    InvalidPolicy(String),
}

impl SyntaxConfig {
    pub fn from_toml(source: &str) -> Result<Self, SyntaxConfigError> {
        let config: Self = toml::from_str(source)
            .map_err(|error| SyntaxConfigError::Toml(error.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), SyntaxConfigError> {
        if self.scope.open.trim().is_empty() {
            return Err(SyntaxConfigError::EmptyScopeMarker("open"));
        }
        if self.scope.close.trim().is_empty() {
            return Err(SyntaxConfigError::EmptyScopeMarker("close"));
        }
        if self.scope.open == self.scope.close {
            return Err(SyntaxConfigError::EqualScopeMarkers(self.scope.open.clone()));
        }
        for (name, marker) in [
            ("alias", self.discourse.alias.as_str()),
            ("definition", self.discourse.definition.as_str()),
            ("relative", self.discourse.relative.as_str()),
            ("frame", self.discourse.frame.as_str()),
        ] {
            if marker.trim().is_empty() {
                return Err(SyntaxConfigError::InvalidPolicy(format!("empty discourse {name} marker")));
            }
        }
        if self.quotation.open.trim().is_empty() || self.quotation.close.trim().is_empty() {
            return Err(SyntaxConfigError::InvalidPolicy("empty quotation boundary marker".into()));
        }
        if self.quotation.open == self.quotation.close {
            return Err(SyntaxConfigError::InvalidPolicy(format!(
                "quotation open/close marker `{}` must be distinct",
                self.quotation.open
            )));
        }
        if self.text.utterance_spoken.trim().is_empty() {
            return Err(SyntaxConfigError::InvalidPolicy("empty spoken utterance boundary".into()));
        }
        let mut written = self.text.utterance_written.chars();
        let Some(character) = written.next() else {
            return Err(SyntaxConfigError::InvalidPolicy("empty written utterance boundary".into()));
        };
        if written.next().is_some() || character.is_alphanumeric() {
            return Err(SyntaxConfigError::InvalidPolicy(format!(
                "written utterance boundary `{}` must be one non-alphanumeric character",
                self.text.utterance_written
            )));
        }
        for punctuation in &self.text.readability_punctuation {
            let mut chars = punctuation.chars();
            let Some(character) = chars.next() else {
                return Err(SyntaxConfigError::InvalidPolicy("empty readability punctuation form".into()));
            };
            if chars.next().is_some() || character.is_alphanumeric() {
                return Err(SyntaxConfigError::InvalidPolicy(format!(
                    "readability punctuation `{punctuation}` must be one non-alphanumeric character"
                )));
            }
            if punctuation == &self.text.utterance_written {
                return Err(SyntaxConfigError::InvalidPolicy(format!(
                    "written utterance boundary `{}` must not also be readability-only punctuation",
                    self.text.utterance_written
                )));
            }
        }
        let mut all_markers = vec![
            self.scope.open.as_str(), self.scope.close.as_str(),
            self.discourse.alias.as_str(), self.discourse.definition.as_str(),
            self.discourse.relative.as_str(), self.discourse.frame.as_str(),
            self.quotation.open.as_str(), self.quotation.close.as_str(),
            self.text.utterance_spoken.as_str(),
        ];
        all_markers.sort_unstable();
        if let Some(pair) = all_markers.windows(2).find(|pair| pair[0] == pair[1]) {
            return Err(SyntaxConfigError::InvalidPolicy(format!(
                "structural marker `{}` is assigned more than once", pair[0]
            )));
        }
        Ok(())
    }
}

impl fmt::Display for SyntaxConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Toml(error) => write!(f, "syntax TOML: {error}"),
            Self::EmptyScopeMarker(which) => write!(f, "syntax scope {which} marker must not be empty"),
            Self::EqualScopeMarkers(marker) => write!(f, "syntax scope markers must differ; both are `{marker}`"),
            Self::InvalidPolicy(policy) => write!(f, "invalid syntax policy `{policy}`"),
        }
    }
}

impl std::error::Error for SyntaxConfigError {}
