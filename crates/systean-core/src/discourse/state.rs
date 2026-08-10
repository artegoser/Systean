use std::collections::BTreeMap;
use std::fmt;

use crate::semantics::{Checker, Environment, Term, Type};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReferentId(u64);

impl ReferentId {
    pub fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ReferentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "r{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccessibilityScopeId(u64);

impl AccessibilityScopeId {
    pub const ROOT: Self = Self(0);

    pub fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for AccessibilityScopeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "s{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IntroductionOrigin {
    Surface {
        source: String,
    },
    Context {
        key: String,
    },
    External {
        label: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Referent {
    pub id: ReferentId,
    pub value: Term,
    pub ty: Type,
    pub origin: IntroductionOrigin,
    pub scope: AccessibilityScopeId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextValue {
    pub value: Term,
    pub ty: Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceCandidate {
    pub id: ReferentId,
    pub value: Term,
    pub ty: Type,
    pub origin: IntroductionOrigin,
}

impl From<&Referent> for ReferenceCandidate {
    fn from(referent: &Referent) -> Self {
        Self {
            id: referent.id,
            value: referent.value.clone(),
            ty: referent.ty.clone(),
            origin: referent.origin.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedReference {
    pub id: ReferentId,
    pub value: Term,
    pub ty: Type,
    pub origin: IntroductionOrigin,
}

impl From<&Referent> for ResolvedReference {
    fn from(referent: &Referent) -> Self {
        Self {
            id: referent.id,
            value: referent.value.clone(),
            ty: referent.ty.clone(),
            origin: referent.origin.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscourseError {
    CannotLeaveRootScope,
    InvalidType {
        context: String,
        ty: Type,
    },
    InvalidSemanticValue {
        message: String,
    },
    ContextMissing {
        key: String,
    },
    ContextTypeMismatch {
        key: String,
        expected: Type,
        actual: Type,
    },
    UnresolvedReference {
        role: String,
        expected: Type,
    },
    AmbiguousReference {
        role: String,
        expected: Type,
        candidates: Vec<ReferenceCandidate>,
    },
}

#[derive(Clone, Debug)]
pub struct DiscourseState {
    next_referent_id: u64,
    next_scope_id: u64,
    current_scope: AccessibilityScopeId,
    scope_parents: BTreeMap<AccessibilityScopeId, Option<AccessibilityScopeId>>,
    referents: BTreeMap<ReferentId, Referent>,
    context: BTreeMap<String, ContextValue>,
}

impl Default for DiscourseState {
    fn default() -> Self {
        let mut scope_parents = BTreeMap::new();
        scope_parents.insert(AccessibilityScopeId::ROOT, None);
        Self {
            next_referent_id: 0,
            next_scope_id: 1,
            current_scope: AccessibilityScopeId::ROOT,
            scope_parents,
            referents: BTreeMap::new(),
            context: BTreeMap::new(),
        }
    }
}

impl DiscourseState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current_scope(&self) -> AccessibilityScopeId {
        self.current_scope
    }

    pub fn enter_scope(&mut self) -> AccessibilityScopeId {
        let scope = AccessibilityScopeId(self.next_scope_id);
        self.next_scope_id += 1;
        self.scope_parents.insert(scope, Some(self.current_scope));
        self.current_scope = scope;
        scope
    }

    pub fn leave_scope(&mut self) -> Result<AccessibilityScopeId, DiscourseError> {
        let Some(Some(parent)) = self.scope_parents.get(&self.current_scope).copied() else {
            return Err(DiscourseError::CannotLeaveRootScope);
        };
        self.current_scope = parent;
        Ok(parent)
    }

    pub fn introduce(
        &mut self,
        value: Term,
        origin: IntroductionOrigin,
        environment: &Environment,
    ) -> Result<ReferentId, DiscourseError> {
        let ty = Checker::new(environment)
            .infer(&value)
            .map_err(|error| DiscourseError::InvalidSemanticValue {
                message: error.to_string(),
            })?;
        let id = ReferentId(self.next_referent_id);
        self.next_referent_id += 1;
        self.referents.insert(
            id,
            Referent {
                id,
                value,
                ty,
                origin,
                scope: self.current_scope,
            },
        );
        Ok(id)
    }

    pub fn referent(&self, id: ReferentId) -> Option<&Referent> {
        self.referents.get(&id)
    }

    pub fn referents(&self) -> impl Iterator<Item = &Referent> {
        self.referents.values()
    }

    pub fn set_context_value(
        &mut self,
        key: impl Into<String>,
        value: Term,
        environment: &Environment,
    ) -> Result<Option<ContextValue>, DiscourseError> {
        let ty = Checker::new(environment)
            .infer(&value)
            .map_err(|error| DiscourseError::InvalidSemanticValue {
                message: error.to_string(),
            })?;
        Ok(self.context.insert(key.into(), ContextValue { value, ty }))
    }

    pub fn context_value(&self, key: &str) -> Option<&ContextValue> {
        self.context.get(key)
    }

    pub fn resolve_context(
        &self,
        key: &str,
        declared_type: &Type,
        expected_type: &Type,
        environment: &Environment,
    ) -> Result<ContextValue, DiscourseError> {
        if !environment.is_well_formed_type(declared_type) {
            return Err(DiscourseError::InvalidType {
                context: format!("context `{key}` declared type"),
                ty: declared_type.clone(),
            });
        }
        if !environment.is_well_formed_type(expected_type) {
            return Err(DiscourseError::InvalidType {
                context: format!("context `{key}` expected type"),
                ty: expected_type.clone(),
            });
        }
        let value = self
            .context
            .get(key)
            .ok_or_else(|| DiscourseError::ContextMissing {
                key: key.to_owned(),
            })?;
        if !environment.is_assignable(&value.ty, declared_type)
            || !environment.is_assignable(&value.ty, expected_type)
        {
            return Err(DiscourseError::ContextTypeMismatch {
                key: key.to_owned(),
                expected: expected_type.clone(),
                actual: value.ty.clone(),
            });
        }
        Ok(value.clone())
    }

    pub fn resolve_reference(
        &self,
        role: &str,
        expected_type: &Type,
        environment: &Environment,
    ) -> Result<ResolvedReference, DiscourseError> {
        if !environment.is_well_formed_type(expected_type) {
            return Err(DiscourseError::InvalidType {
                context: format!("reference role `{role}`"),
                ty: expected_type.clone(),
            });
        }
        let candidates = self
            .referents
            .values()
            .filter(|referent| self.is_scope_accessible(referent.scope))
            .filter(|referent| environment.is_assignable(&referent.ty, expected_type))
            .collect::<Vec<_>>();
        match candidates.as_slice() {
            [] => Err(DiscourseError::UnresolvedReference {
                role: role.to_owned(),
                expected: expected_type.clone(),
            }),
            [referent] => Ok(ResolvedReference::from(*referent)),
            _ => Err(DiscourseError::AmbiguousReference {
                role: role.to_owned(),
                expected: expected_type.clone(),
                candidates: candidates
                    .into_iter()
                    .map(ReferenceCandidate::from)
                    .collect(),
            }),
        }
    }

    fn is_scope_accessible(&self, scope: AccessibilityScopeId) -> bool {
        let mut current = Some(self.current_scope);
        while let Some(candidate) = current {
            if candidate == scope {
                return true;
            }
            current = self.scope_parents.get(&candidate).copied().flatten();
        }
        false
    }
}

impl fmt::Display for DiscourseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CannotLeaveRootScope => write!(f, "cannot leave the root discourse scope"),
            Self::InvalidType { context, ty } => {
                write!(f, "{context} uses unknown or invalid type `{ty}`")
            }
            Self::InvalidSemanticValue { message } => {
                write!(f, "invalid discourse semantic value: {message}")
            }
            Self::ContextMissing { key } => {
                write!(f, "required discourse context value `{key}` is not provided")
            }
            Self::ContextTypeMismatch {
                key,
                expected,
                actual,
            } => write!(
                f,
                "discourse context `{key}` expects `{expected}` but received `{actual}`"
            ),
            Self::UnresolvedReference { role, expected } => write!(
                f,
                "reference for role `{role}` has no accessible candidate assignable to `{expected}`"
            ),
            Self::AmbiguousReference {
                role,
                expected,
                candidates,
            } => {
                write!(
                    f,
                    "reference for role `{role}` is ambiguous for `{expected}`; candidates: "
                )?;
                for (index, candidate) in candidates.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}:{}={}", candidate.id, candidate.ty, candidate.value)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for DiscourseError {}
