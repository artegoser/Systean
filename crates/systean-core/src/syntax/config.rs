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
    pub text: TextConfig,
    pub pragmatics: PragmaticsConfig,
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
    #[serde(default)]
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TextConfig {
    pub utterance_spoken: String,
    pub utterance_written: String,
    #[serde(default)]
    pub readability_punctuation: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PragmaticsConfig {
    pub default_assertion_operator: String,
    pub default_assertion_role: String,
    pub question_operator: String,
    pub question_content_role: String,
    pub command_operator: String,
    pub command_content_role: String,
    pub request_operator: String,
    pub request_content_role: String,
    pub expressive_operator: String,
    pub expressive_state_role: String,
    pub focus_operator: String,
    pub focus_target_role: String,
    pub focus_content_role: String,
    pub topic_operator: String,
    pub topic_target_role: String,
    pub topic_content_role: String,
    pub retract_operator: String,
    pub repair_target_role: String,
    pub correction_operator: String,
    pub correction_replacement_role: String,
    pub clarification_operator: String,
    pub clarification_content_role: String,
    pub disjunction_operator: String,
    pub disjunction_left_role: String,
    pub disjunction_right_role: String,
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
    CountedQuantifier {
        binder_role: String,
        count_role: String,
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
        if self.text.utterance_spoken.trim().is_empty() {
            return Err(SyntaxConfigError::UnsupportedPolicy(
                "empty spoken utterance boundary".into(),
            ));
        }
        let mut written_boundary = self.text.utterance_written.chars();
        let Some(written_boundary_character) = written_boundary.next() else {
            return Err(SyntaxConfigError::UnsupportedPolicy(
                "empty written utterance boundary".into(),
            ));
        };
        if written_boundary.next().is_some() || written_boundary_character.is_alphanumeric() {
            return Err(SyntaxConfigError::UnsupportedPolicy(format!(
                "written utterance boundary `{}` must be one non-alphanumeric character",
                self.text.utterance_written
            )));
        }
        for punctuation in &self.text.readability_punctuation {
            let mut characters = punctuation.chars();
            let Some(character) = characters.next() else {
                return Err(SyntaxConfigError::UnsupportedPolicy(
                    "empty readability punctuation form".into(),
                ));
            };
            if characters.next().is_some() || character.is_alphanumeric() {
                return Err(SyntaxConfigError::UnsupportedPolicy(format!(
                    "readability punctuation `{punctuation}` must be one non-alphanumeric character"
                )));
            }
            if punctuation == &self.text.utterance_written {
                return Err(SyntaxConfigError::UnsupportedPolicy(format!(
                    "written utterance boundary `{}` must not also be readability-only punctuation",
                    self.text.utterance_written
                )));
            }
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
            self.text.utterance_spoken.as_str(),
        ];
        all_markers.sort_unstable();
        if let Some(pair) = all_markers.windows(2).find(|pair| pair[0] == pair[1]) {
            return Err(SyntaxConfigError::UnsupportedPolicy(format!(
                "structural marker `{}` is assigned more than once",
                pair[0]
            )));
        }
        for (name, value) in [
            ("default_assertion_operator", self.pragmatics.default_assertion_operator.as_str()),
            ("default_assertion_role", self.pragmatics.default_assertion_role.as_str()),
            ("question_operator", self.pragmatics.question_operator.as_str()),
            ("question_content_role", self.pragmatics.question_content_role.as_str()),
            ("command_operator", self.pragmatics.command_operator.as_str()),
            ("command_content_role", self.pragmatics.command_content_role.as_str()),
            ("request_operator", self.pragmatics.request_operator.as_str()),
            ("request_content_role", self.pragmatics.request_content_role.as_str()),
            ("expressive_operator", self.pragmatics.expressive_operator.as_str()),
            ("expressive_state_role", self.pragmatics.expressive_state_role.as_str()),
            ("focus_operator", self.pragmatics.focus_operator.as_str()),
            ("focus_target_role", self.pragmatics.focus_target_role.as_str()),
            ("focus_content_role", self.pragmatics.focus_content_role.as_str()),
            ("topic_operator", self.pragmatics.topic_operator.as_str()),
            ("topic_target_role", self.pragmatics.topic_target_role.as_str()),
            ("topic_content_role", self.pragmatics.topic_content_role.as_str()),
            ("retract_operator", self.pragmatics.retract_operator.as_str()),
            ("repair_target_role", self.pragmatics.repair_target_role.as_str()),
            ("correction_operator", self.pragmatics.correction_operator.as_str()),
            ("correction_replacement_role", self.pragmatics.correction_replacement_role.as_str()),
            ("clarification_operator", self.pragmatics.clarification_operator.as_str()),
            ("clarification_content_role", self.pragmatics.clarification_content_role.as_str()),
            ("disjunction_operator", self.pragmatics.disjunction_operator.as_str()),
            ("disjunction_left_role", self.pragmatics.disjunction_left_role.as_str()),
            ("disjunction_right_role", self.pragmatics.disjunction_right_role.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(SyntaxConfigError::UnsupportedPolicy(format!(
                    "empty pragmatics field `{name}`"
                )));
            }
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
