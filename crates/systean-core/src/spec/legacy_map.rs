use std::collections::BTreeSet;
use std::fmt;

use crate::semantics::SymbolId;

use super::{CompiledSymbolKind, TypedSemanticPackage};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyLexicalMapping {
    pub root: String,
    pub legacy_semantic: String,
    pub symbol: SymbolId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LegacyLexicalMapError {
    MalformedLine { line: usize, text: String },
    DuplicateRoot(String),
    UnknownTypedWord(String),
    Incomplete { missing: Vec<String>, extra: Vec<String> },
}

/// Resolves the archived Phase 17 lexical aliases to Phase 18 typed symbol IDs.
///
/// The TSV is migration/diagnostic data only: the returned `SymbolId` comes exclusively from the
/// typed semantic package and the legacy label never participates in semantic compilation.
pub fn resolve_legacy_lexical_map(
    package: &TypedSemanticPackage,
    source: &str,
) -> Result<Vec<LegacyLexicalMapping>, LegacyLexicalMapError> {
    let mut mappings = Vec::new();
    let mut roots = BTreeSet::new();

    for (index, raw) in source.lines().enumerate() {
        let line = index + 1;
        let text = raw.trim();
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        if line == 1 && text == "root\tlegacy_semantic" {
            continue;
        }
        let Some((root, legacy_semantic)) = text.split_once('\t') else {
            return Err(LegacyLexicalMapError::MalformedLine {
                line,
                text: raw.to_owned(),
            });
        };
        if root.is_empty() || legacy_semantic.is_empty() || legacy_semantic.contains('\t') {
            return Err(LegacyLexicalMapError::MalformedLine {
                line,
                text: raw.to_owned(),
            });
        }
        if !roots.insert(root.to_owned()) {
            return Err(LegacyLexicalMapError::DuplicateRoot(root.to_owned()));
        }
        let Some(symbol) = package.symbol_id(root) else {
            return Err(LegacyLexicalMapError::UnknownTypedWord(root.to_owned()));
        };
        if package.symbol(symbol).is_none_or(|symbol| symbol.kind != CompiledSymbolKind::Word) {
            return Err(LegacyLexicalMapError::UnknownTypedWord(root.to_owned()));
        }
        mappings.push(LegacyLexicalMapping {
            root: root.to_owned(),
            legacy_semantic: legacy_semantic.to_owned(),
            symbol,
        });
    }

    let typed_roots = package
        .symbols()
        .filter(|symbol| symbol.kind == CompiledSymbolKind::Word)
        .filter_map(|symbol| package.source_name_for_symbol(symbol.id))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let missing = typed_roots.difference(&roots).cloned().collect::<Vec<_>>();
    let extra = roots.difference(&typed_roots).cloned().collect::<Vec<_>>();
    if !missing.is_empty() || !extra.is_empty() {
        return Err(LegacyLexicalMapError::Incomplete { missing, extra });
    }

    mappings.sort_by(|left, right| left.root.cmp(&right.root));
    Ok(mappings)
}

impl fmt::Display for LegacyLexicalMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedLine { line, text } => {
                write!(f, "legacy lexical map line {line} is malformed: `{text}`")
            }
            Self::DuplicateRoot(root) => write!(f, "legacy lexical map repeats root `{root}`"),
            Self::UnknownTypedWord(root) => {
                write!(f, "legacy lexical map root `{root}` has no typed word declaration")
            }
            Self::Incomplete { missing, extra } => write!(
                f,
                "legacy lexical map does not cover typed words exactly; missing: {missing:?}; extra: {extra:?}",
            ),
        }
    }
}

impl std::error::Error for LegacyLexicalMapError {}
