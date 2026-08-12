use std::collections::BTreeMap;
use std::fmt;

use crate::pragmatics::{DiscourseEffect, PragmaticAnalysis};
use crate::semantics::Term;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtteranceId(u64);

impl UtteranceId {
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for UtteranceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "u{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryEntry {
    pub id: UtteranceId,
    pub source: String,
    pub canonical_surface: String,
    pub analysis: PragmaticAnalysis,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commitment {
    pub entry: UtteranceId,
    pub content: Term,
    pub active: bool,
    pub superseded_by: Option<UtteranceId>,
    pub retracted_by: Option<UtteranceId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConversationError {
    MissingRepairTarget(UtteranceId),
    TargetHasNoCommitment(UtteranceId),
    TargetHasNoActiveCommitment(UtteranceId),
}

#[derive(Clone, Debug)]
pub struct ConversationState {
    next_utterance_id: u64,
    history: BTreeMap<UtteranceId, HistoryEntry>,
    commitments: BTreeMap<UtteranceId, Commitment>,
}

impl Default for ConversationState {
    fn default() -> Self {
        Self {
            next_utterance_id: 1,
            history: BTreeMap::new(),
            commitments: BTreeMap::new(),
        }
    }
}

impl ConversationState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn history(&self) -> impl Iterator<Item = &HistoryEntry> {
        self.history.values()
    }

    pub fn entry(&self, id: UtteranceId) -> Option<&HistoryEntry> {
        self.history.get(&id)
    }

    pub fn commitment(&self, id: UtteranceId) -> Option<&Commitment> {
        self.commitments.get(&id)
    }

    pub fn active_commitments(&self) -> Vec<&Commitment> {
        self.commitments
            .values()
            .filter(|commitment| commitment.active)
            .collect()
    }

    pub fn current_commitment_for(
        &self,
        target: UtteranceId,
    ) -> Result<&Commitment, ConversationError> {
        if !self.history.contains_key(&target) {
            return Err(ConversationError::MissingRepairTarget(target));
        }
        let Some(mut current) = self.commitments.get(&target) else {
            return Err(ConversationError::TargetHasNoCommitment(target));
        };
        loop {
            if current.active {
                return Ok(current);
            }
            if let Some(next) = current.superseded_by {
                current = self
                    .commitments
                    .get(&next)
                    .ok_or(ConversationError::TargetHasNoActiveCommitment(target))?;
                continue;
            }
            return Err(ConversationError::TargetHasNoActiveCommitment(target));
        }
    }

    pub fn apply(
        &mut self,
        source: impl Into<String>,
        canonical_surface: impl Into<String>,
        analysis: PragmaticAnalysis,
    ) -> Result<UtteranceId, ConversationError> {
        let source = source.into();
        let canonical_surface = canonical_surface.into();

        let mut resolved_repairs = Vec::with_capacity(analysis.effects.len());
        for effect in &analysis.effects {
            let resolved = match effect {
                DiscourseEffect::Retract { target } | DiscourseEffect::Replace { target, .. } => {
                    Some(self.current_commitment_for(UtteranceId::from_raw(*target))?.entry)
                }
                DiscourseEffect::Clarify { target, .. } => {
                    let target = UtteranceId::from_raw(*target);
                    if !self.history.contains_key(&target) {
                        return Err(ConversationError::MissingRepairTarget(target));
                    }
                    None
                }
                DiscourseEffect::Commit { .. } => None,
            };
            resolved_repairs.push(resolved);
        }

        let id = UtteranceId(self.next_utterance_id);
        self.next_utterance_id += 1;
        self.history.insert(
            id,
            HistoryEntry {
                id,
                source,
                canonical_surface,
                analysis: analysis.clone(),
            },
        );

        for (effect, resolved) in analysis.effects.iter().zip(resolved_repairs) {
            match effect {
                DiscourseEffect::Commit { content } => self.insert_commitment(id, content.clone()),
                DiscourseEffect::Replace { replacement, .. } => {
                    let current_id = resolved.expect("replacement target resolved before insert");
                    let current = self.commitments.get_mut(&current_id).expect("resolved commitment exists");
                    current.active = false;
                    current.superseded_by = Some(id);
                    self.insert_commitment(id, replacement.clone());
                }
                DiscourseEffect::Retract { .. } => {
                    let current_id = resolved.expect("retraction target resolved before insert");
                    let current = self.commitments.get_mut(&current_id).expect("resolved commitment exists");
                    current.active = false;
                    current.retracted_by = Some(id);
                }
                DiscourseEffect::Clarify { content, .. } => self.insert_commitment(id, content.clone()),
            }
        }

        Ok(id)
    }

    fn insert_commitment(&mut self, id: UtteranceId, content: Term) {
        self.commitments.insert(
            id,
            Commitment {
                entry: id,
                content,
                active: true,
                superseded_by: None,
                retracted_by: None,
            },
        );
    }
}

impl fmt::Display for ConversationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRepairTarget(target) => {
                write!(f, "repair target `{target}` does not exist in conversation history")
            }
            Self::TargetHasNoCommitment(target) => {
                write!(f, "repair target `{target}` does not denote a discourse commitment")
            }
            Self::TargetHasNoActiveCommitment(target) => {
                write!(f, "repair target `{target}` has no active commitment to modify")
            }
        }
    }
}

impl std::error::Error for ConversationError {}
