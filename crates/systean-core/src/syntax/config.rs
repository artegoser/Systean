use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SyntaxConfig {
    pub meta: Option<MetaConfig>,
    pub order: OrderConfig,
    pub scope: ScopeConfig,
    pub logic: LogicConfig,
    pub roles: RolesConfig,
    pub arguments: ArgumentsConfig,
    pub questions: ExplicitOperatorConfig,
    pub commands: ExplicitOperatorConfig,
    pub focus: FocusConfig,
    pub grammar: GrammarConfig,
    pub discourse: DiscourseConfig,
    pub quotation: QuotationConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MetaConfig {
    pub description: Option<String>,
    pub version: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct OrderConfig {
    pub frame: FrameOrder,
    pub free_order: bool,
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
    pub explicit: ExplicitScopePolicy,
    pub quantifier_order: QuantifierScopePolicy,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExplicitScopePolicy {
    WhenAmbiguous,
    Always,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuantifierScopePolicy {
    Appearance,
    Explicit,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct LogicConfig {
    pub flatten_same_operator: bool,
    pub precedence: BTreeMap<String, u16>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RolesConfig {
    pub realization: RoleRealization,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoleRealization {
    FrameOrder,
    ExplicitMarkers,
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
    Contextual,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExplicitOperatorConfig {
    pub realization: ExplicitOperatorRealization,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExplicitOperatorRealization {
    ExplicitOperator,
    WordOrder,
    PredicateMorphology,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct FocusConfig {
    pub reorders: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GrammarConfig {
    pub traditional_pos: bool,
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

/// Surface realization attached to one lexical root in `dictionary.toml`.
///
/// Semantic identity is deliberately not repeated here. Constant roots need no
/// explicit surface realization and become atoms automatically. Operator roots
/// select one of these structural realizations and obtain their semantic symbol
/// from the same dictionary entry.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SurfaceFormConfig {
    Class {
        role: String,
    },
    Predicate {
        primary_role: Option<String>,
        #[serde(default)]
        rest_roles: Vec<String>,
    },
    Prefix {
        role: String,
    },
    Infix {
        left_role: String,
        right_role: String,
    },
    Quantifier {
        binder_role: String,
        variable_type: String,
        restriction_operator: String,
        restriction_role: String,
        body_role: String,
    },
    SpeechAct {
        role: String,
    },
    Name {
        role: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyntaxConfigError {
    Toml(String),
    EmptyScopeMarker(&'static str),
    EqualScopeMarkers(String),
    UnsupportedPolicy(String),
}

impl SyntaxConfig {
    pub fn from_toml(source: &str) -> Result<Self, SyntaxConfigError> {
        let config: Self = toml::from_str(source)
            .map_err(|error| SyntaxConfigError::Toml(error.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), SyntaxConfigError> {
        if self.order.free_order {
            return Err(SyntaxConfigError::UnsupportedPolicy(
                "free unmarked word order".into(),
            ));
        }
        if self.scope.explicit != ExplicitScopePolicy::WhenAmbiguous {
            return Err(SyntaxConfigError::UnsupportedPolicy(
                "mandatory scope delimiters".into(),
            ));
        }
        if self.scope.quantifier_order != QuantifierScopePolicy::Appearance {
            return Err(SyntaxConfigError::UnsupportedPolicy(
                "explicit-only quantifier scope".into(),
            ));
        }
        if self.roles.realization != RoleRealization::FrameOrder {
            return Err(SyntaxConfigError::UnsupportedPolicy(
                "mandatory role markers".into(),
            ));
        }
        if self.arguments.omission == ArgumentOmission::Contextual {
            return Err(SyntaxConfigError::UnsupportedPolicy(
                "pragmatic/contextual argument omission".into(),
            ));
        }
        if self.questions.realization != ExplicitOperatorRealization::ExplicitOperator
            || self.commands.realization != ExplicitOperatorRealization::ExplicitOperator
        {
            return Err(SyntaxConfigError::UnsupportedPolicy(
                "non-explicit question/command realization".into(),
            ));
        }
        if self.focus.reorders {
            return Err(SyntaxConfigError::UnsupportedPolicy(
                "focus by word-order rearrangement".into(),
            ));
        }
        if self.grammar.traditional_pos {
            return Err(SyntaxConfigError::UnsupportedPolicy(
                "traditional POS-driven grammar".into(),
            ));
        }
        if self.scope.open.trim().is_empty() {
            return Err(SyntaxConfigError::EmptyScopeMarker("open"));
        }
        if self.scope.close.trim().is_empty() {
            return Err(SyntaxConfigError::EmptyScopeMarker("close"));
        }
        if self.scope.open == self.scope.close {
            return Err(SyntaxConfigError::EqualScopeMarkers(
                self.scope.open.clone(),
            ));
        }
        let discourse_markers = [
            ("alias", self.discourse.alias.as_str()),
            ("definition", self.discourse.definition.as_str()),
            ("relative", self.discourse.relative.as_str()),
            ("frame", self.discourse.frame.as_str()),
        ];
        for (name, marker) in discourse_markers {
            if marker.trim().is_empty() {
                return Err(SyntaxConfigError::UnsupportedPolicy(format!(
                    "empty discourse {name} marker"
                )));
            }
        }
        if self.quotation.open.trim().is_empty() || self.quotation.close.trim().is_empty() {
            return Err(SyntaxConfigError::UnsupportedPolicy(
                "empty quotation boundary marker".into(),
            ));
        }
        if self.quotation.open == self.quotation.close {
            return Err(SyntaxConfigError::UnsupportedPolicy(format!(
                "quotation open/close marker `{}` must be distinct",
                self.quotation.open
            )));
        }
        let mut all_markers = vec![
            self.scope.open.as_str(),
            self.scope.close.as_str(),
            self.discourse.alias.as_str(),
            self.discourse.definition.as_str(),
            self.discourse.relative.as_str(),
            self.discourse.frame.as_str(),
            self.quotation.open.as_str(),
            self.quotation.close.as_str(),
        ];
        all_markers.sort_unstable();
        if let Some(pair) = all_markers.windows(2).find(|pair| pair[0] == pair[1]) {
            return Err(SyntaxConfigError::UnsupportedPolicy(format!(
                "structural marker `{}` is assigned more than once",
                pair[0]
            )));
        }
        Ok(())
    }

    pub fn precedence(&self, semantic: &str) -> Option<u16> {
        self.logic.precedence.get(semantic).copied()
    }
}

impl fmt::Display for SyntaxConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Toml(error) => write!(f, "syntax TOML: {error}"),
            Self::EmptyScopeMarker(which) => {
                write!(f, "syntax scope {which} marker must not be empty")
            }
            Self::EqualScopeMarkers(marker) => {
                write!(f, "syntax scope markers must differ; both are `{marker}`")
            }
            Self::UnsupportedPolicy(policy) => write!(f, "unsupported syntax policy `{policy}`"),
        }
    }
}

impl std::error::Error for SyntaxConfigError {}
