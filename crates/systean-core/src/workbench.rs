use std::collections::BTreeMap;

use serde::Serialize;

use crate::compiler::{PackageManifest, PackageProvenance, PackageValidationReport};
use crate::discourse::{
    ConversationState, DiscourseError, DiscourseResolutionError, DiscourseState, IntroductionOrigin,
    TextDocument, TextRealization, TextSessionState, TextTurn,
};
use crate::language::{DictionaryEntry, LanguageError, LanguagePackage};
use crate::literals::{LiteralRealization, SurfaceLiteral};
use crate::pragmatics::{DiscourseEffect, PragmaticAnalysis};
use crate::semantics::{
    Checker, Explanation, InformationKnowerValue, InformationStatus, Literal, Origin,
    StructuredValue, Term, canonicalize,
};
use crate::spec::{lower_term, parse_term};
use crate::syntax::{Argument, Clause, InformationKnower, CompiledSurfaceBinding, SurfaceExpr, TypedSurfaceAst};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLayer {
    Package,
    Phonology,
    Morphology,
    Lexical,
    Literal,
    Syntax,
    Discourse,
    Semantics,
    Pragmatics,
    TextStructure,
    Generation,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct AmbiguityCandidateView {
    pub id: String,
    pub ty: String,
    pub value: String,
    pub origin: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct WorkbenchDiagnostic {
    pub layer: DiagnosticLayer,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<AmbiguityCandidateView>,
}

impl WorkbenchDiagnostic {
    pub fn generation(message: impl Into<String>) -> Self {
        Self {
            layer: DiagnosticLayer::Generation,
            message: message.into(),
            candidates: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct PackageWorkbenchInfo {
    pub manifest: PackageManifest,
    pub provenance: PackageProvenance,
    pub semantic_fingerprint: String,
    pub surface_fingerprint: String,
    pub validation: PackageValidationReport,
}

impl PackageWorkbenchInfo {
    fn from_language(language: &LanguagePackage) -> Self {
        Self {
            manifest: language.manifest().clone(),
            provenance: language.provenance().clone(),
            semantic_fingerprint: language.semantic_fingerprint().to_owned(),
            surface_fingerprint: language.surface_fingerprint().to_owned(),
            validation: language.validation_report().clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct MorphemeWorkbenchView {
    pub kind: String,
    pub spelling: String,
    pub grapheme_start: usize,
    pub grapheme_end: usize,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct SyllableWorkbenchView {
    pub spelling: String,
    pub pronunciation: String,
    pub stressed: bool,
    pub grapheme_start: usize,
    pub grapheme_end: usize,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct WordWorkbenchAnalysis {
    pub source: String,
    pub root: String,
    pub definition: String,
    pub dictionary_entry: DictionaryEntry,
    pub semantic_origin: Option<String>,
    pub pronunciation: String,
    pub stressed_pronunciation: String,
    pub morphemes: Vec<MorphemeWorkbenchView>,
    pub syllables: Vec<SyllableWorkbenchView>,
    pub package: PackageWorkbenchInfo,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct AstNodeView {
    pub kind: String,
    pub label: String,
    pub children: Vec<AstNodeView>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ScopeView {
    pub path: String,
    pub kind: String,
    pub operator: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReferenceSlotView {
    pub placeholder: String,
    pub role: String,
    pub expected_type: String,
    pub source: String,
    pub resolution: Option<ResolvedBindingView>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ContextSlotView {
    pub placeholder: String,
    pub surface: String,
    pub key: String,
    pub declared_type: String,
    pub expected_type: String,
    pub resolution: Option<ResolvedBindingView>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct AliasSlotView {
    pub placeholder: String,
    pub surface: String,
    pub alias: String,
    pub declared_type: String,
    pub expected_type: String,
    pub resolution: Option<ResolvedBindingView>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ResolvedBindingView {
    pub id: Option<String>,
    pub ty: String,
    pub value: String,
    pub origin: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct SemanticNodeView {
    pub label: String,
    pub ty: String,
    pub origin: Option<String>,
    pub children: Vec<SemanticEdgeView>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct SemanticEdgeView {
    pub relation: String,
    pub node: SemanticNodeView,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct SurfaceWorkbenchAnalysis {
    pub source: String,
    pub canonical_surface: String,
    pub canonical_resolved_surface: String,
    pub inferred_type: String,
    pub surface_ast: AstNodeView,
    pub typed_template: String,
    pub references: Vec<ReferenceSlotView>,
    pub contexts: Vec<ContextSlotView>,
    pub aliases: Vec<AliasSlotView>,
    pub scopes: Vec<ScopeView>,
    pub semantic_ir: String,
    pub canonical_semantic_ir: String,
    pub semantic_explanation: SemanticNodeView,
    pub package_fingerprint: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct PragmaticWorkbenchView {
    pub act: String,
    pub effects: Vec<String>,
    pub utterance: String,
    pub inferred_type: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct UtteranceWorkbenchAnalysis {
    pub surface: SurfaceWorkbenchAnalysis,
    pub pragmatics: PragmaticWorkbenchView,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReferentWorkbenchView {
    pub id: String,
    pub ty: String,
    pub value: String,
    pub origin: String,
    pub scope: String,
    pub frame: String,
    pub shorthand: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct AliasWorkbenchView {
    pub surface: String,
    pub referent: String,
    pub ty: String,
    pub scope: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct HistoryWorkbenchView {
    pub id: String,
    pub source: String,
    pub canonical_surface: String,
    pub act: String,
    pub semantics: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CommitmentWorkbenchView {
    pub entry: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DiscourseStateView {
    pub scope: String,
    pub frame: String,
    pub referents: Vec<ReferentWorkbenchView>,
    pub aliases: Vec<AliasWorkbenchView>,
    pub history: Vec<HistoryWorkbenchView>,
    pub active_commitments: Vec<CommitmentWorkbenchView>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DiscourseTurnWorkbenchAnalysis {
    pub key: String,
    pub realization: String,
    pub source: String,
    pub before: DiscourseStateView,
    pub after: DiscourseStateView,
    pub events: Vec<DiscourseEventWorkbenchView>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DiscourseEventWorkbenchView {
    pub kind: String,
    pub id: Option<String>,
    pub section: String,
    pub frame: Option<String>,
    pub act: Option<String>,
    pub source: Option<String>,
    pub canonical_surface: Option<String>,
    pub canonical_spoken: Option<String>,
    pub canonical_written: Option<String>,
    pub semantics: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DiscourseWorkbenchAnalysis {
    pub turns: Vec<DiscourseTurnWorkbenchAnalysis>,
    pub final_state: DiscourseStateView,
    pub package: PackageWorkbenchInfo,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct GenerationWorkbenchAnalysis {
    pub input_semantics: String,
    pub canonical_semantics: String,
    pub inferred_type: String,
    pub surface_ast: AstNodeView,
    pub canonical_surface: String,
    pub roundtrip_semantics: String,
    pub roundtrip_verified: bool,
    pub package_fingerprint: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct LiteralWorkbenchAnalysis {
    pub source: String,
    pub family: String,
    pub ty: String,
    pub semantic_canonical: String,
    pub canonical_written: String,
    pub canonical_spoken: String,
    pub realization: String,
    pub package_fingerprint: String,
}

pub fn package_info(language: &LanguagePackage) -> PackageWorkbenchInfo {
    PackageWorkbenchInfo::from_language(language)
}

pub fn analyze_word(
    language: &LanguagePackage,
    source: &str,
) -> Result<WordWorkbenchAnalysis, WorkbenchDiagnostic> {
    let analysis = language
        .analyze_word(source)
        .map_err(diagnostic_from_language_error)?;
    let entry = language
        .dictionary()
        .entries()
        .iter()
        .find(|entry| entry.root == analysis.morphology.root)
        .cloned()
        .ok_or_else(|| WorkbenchDiagnostic {
            layer: DiagnosticLayer::Lexical,
            message: format!(
                "analyzed root `{}` has no dictionary entry",
                analysis.morphology.root
            ),
            candidates: Vec::new(),
        })?;
    let semantic_origin = lexical_origin(language, &entry).map(|origin| origin.to_string());
    Ok(WordWorkbenchAnalysis {
        source: source.to_owned(),
        root: analysis.morphology.root.clone(),
        definition: entry.definition.clone(),
        dictionary_entry: entry,
        semantic_origin,
        pronunciation: analysis.phonology.pronunciation,
        stressed_pronunciation: analysis.phonology.stressed_pronunciation,
        morphemes: analysis
            .morphology
            .morphemes
            .into_iter()
            .map(|morpheme| MorphemeWorkbenchView {
                kind: format!("{:?}", morpheme.kind).to_lowercase(),
                spelling: morpheme.spelling,
                grapheme_start: morpheme.grapheme_range.start,
                grapheme_end: morpheme.grapheme_range.end,
            })
            .collect(),
        syllables: analysis
            .phonology
            .syllables
            .into_iter()
            .map(|syllable| SyllableWorkbenchView {
                spelling: syllable.spelling,
                pronunciation: syllable.pronunciation,
                stressed: syllable.stressed,
                grapheme_start: syllable.grapheme_range.start,
                grapheme_end: syllable.grapheme_range.end,
            })
            .collect(),
        package: PackageWorkbenchInfo::from_language(language),
    })
}

pub fn analyze_surface(
    language: &LanguagePackage,
    source: &str,
    discourse: &DiscourseState,
) -> Result<SurfaceWorkbenchAnalysis, WorkbenchDiagnostic> {
    let analysis = language
        .analyze_surface_with_discourse(source, discourse)
        .map_err(diagnostic_from_language_error)?;
    let explanation = crate::semantics::Explainer::new(language.semantics())
        .explain(&analysis.resolved.term)
        .map_err(|error| WorkbenchDiagnostic {
            layer: DiagnosticLayer::Semantics,
            message: error.to_string(),
            candidates: Vec::new(),
        })?;
    Ok(SurfaceWorkbenchAnalysis {
        source: source.to_owned(),
        canonical_surface: analysis.canonical_surface,
        canonical_resolved_surface: analysis.canonical_resolved_surface,
        inferred_type: analysis.inferred_type,
        surface_ast: surface_node(&analysis.syntax),
        typed_template: analysis.typed.template.to_string(),
        references: reference_views(&analysis.typed, &analysis.resolved),
        contexts: context_views(&analysis.typed, &analysis.resolved),
        aliases: alias_views(&analysis.typed, &analysis.resolved),
        scopes: scope_views(&analysis.syntax),
        semantic_ir: analysis.resolved.term.to_string(),
        canonical_semantic_ir: analysis.canonical_semantics,
        semantic_explanation: explanation_view(&explanation),
        package_fingerprint: language.package_fingerprint().to_owned(),
    })
}

pub fn analyze_utterance(
    language: &LanguagePackage,
    source: &str,
    discourse: &DiscourseState,
) -> Result<UtteranceWorkbenchAnalysis, WorkbenchDiagnostic> {
    let analysis = language
        .analyze_utterance_with_discourse(source, discourse)
        .map_err(diagnostic_from_language_error)?;
    let surface = surface_workbench_from_analysis(language, source, &analysis.surface)?;
    Ok(UtteranceWorkbenchAnalysis {
        surface,
        pragmatics: pragmatic_view(&analysis.pragmatics),
    })
}

pub fn analyze_document(
    language: &LanguagePackage,
    document: &TextDocument,
) -> Result<DiscourseWorkbenchAnalysis, WorkbenchDiagnostic> {
    let mut session = TextSessionState::new();
    let mut discourse = DiscourseState::new();
    let mut conversation = ConversationState::new();
    let mut turns = Vec::with_capacity(document.turns.len());
    for turn in &document.turns {
        let before = state_view(&discourse, &conversation);
        let analysis = language
            .apply_text_turn(turn, &mut session, &mut discourse, &mut conversation)
            .map_err(diagnostic_from_language_error)?;
        let after = state_view(&discourse, &conversation);
        let events = analysis
            .events
            .into_iter()
            .map(|event| match event {
                crate::language::TextTurnEvent::FrameBoundary { section, frame } => {
                    DiscourseEventWorkbenchView {
                        kind: "frame_boundary".into(),
                        id: None,
                        section: section.to_string(),
                        frame: Some(frame.to_string()),
                        act: None,
                        source: None,
                        canonical_surface: None,
                        canonical_spoken: None,
                        canonical_written: None,
                        semantics: None,
                    }
                }
                crate::language::TextTurnEvent::Utterance(utterance) => {
                    DiscourseEventWorkbenchView {
                        kind: "utterance".into(),
                        id: Some(utterance.id.to_string()),
                        section: utterance.section.to_string(),
                        frame: None,
                        act: Some(utterance.pragmatics.act.label().into()),
                        source: Some(utterance.source),
                        canonical_surface: Some(utterance.canonical_surface),
                        canonical_spoken: Some(utterance.canonical_spoken),
                        canonical_written: Some(utterance.canonical_written),
                        semantics: Some(utterance.pragmatics.utterance.to_string()),
                    }
                }
            })
            .collect();
        turns.push(DiscourseTurnWorkbenchAnalysis {
            key: turn.key.clone(),
            realization: realization_label(turn.realization).into(),
            source: turn.source.clone(),
            before,
            after,
            events,
        });
    }
    Ok(DiscourseWorkbenchAnalysis {
        turns,
        final_state: state_view(&discourse, &conversation),
        package: PackageWorkbenchInfo::from_language(language),
    })
}

pub fn analyze_text_stream(
    language: &LanguagePackage,
    source: &str,
    realization: TextRealization,
) -> Result<DiscourseWorkbenchAnalysis, WorkbenchDiagnostic> {
    analyze_document(
        language,
        &TextDocument {
            turns: vec![TextTurn {
                key: "workbench".into(),
                realization,
                source: source.into(),
            }],
        },
    )
}

pub fn analyze_literal(
    language: &LanguagePackage,
    source: &str,
) -> Result<LiteralWorkbenchAnalysis, WorkbenchDiagnostic> {
    let engine = language.literals().ok_or_else(|| WorkbenchDiagnostic {
        layer: DiagnosticLayer::Literal,
        message: "language package has no structured-literal codecs".into(),
        candidates: Vec::new(),
    })?;
    let literal = engine.parse_complete(source).map_err(|error| WorkbenchDiagnostic {
        layer: DiagnosticLayer::Literal,
        message: error.to_string(),
        candidates: Vec::new(),
    })?;
    Ok(LiteralWorkbenchAnalysis {
        source: source.into(),
        family: literal.semantic.family().into(),
        ty: literal.semantic.ty.to_string(),
        semantic_canonical: literal.semantic.value.to_string(),
        canonical_written: literal.canonical_written,
        canonical_spoken: literal.canonical_spoken,
        realization: match literal.realization {
            LiteralRealization::Written => "written",
            LiteralRealization::Spoken => "spoken",
        }
        .into(),
        package_fingerprint: language.package_fingerprint().into(),
    })
}

pub fn generate_surface(
    language: &LanguagePackage,
    semantic_source: &str,
) -> Result<GenerationWorkbenchAnalysis, WorkbenchDiagnostic> {
    let parsed = parse_term(semantic_source).map_err(|errors| WorkbenchDiagnostic {
        layer: DiagnosticLayer::Semantics,
        message: errors
            .into_iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("; "),
        candidates: Vec::new(),
    })?;
    generate_term(language, &lower_term(parsed))
}

pub fn generate_term(
    language: &LanguagePackage,
    term: &Term,
) -> Result<GenerationWorkbenchAnalysis, WorkbenchDiagnostic> {
    let inferred = Checker::new(language.semantics())
        .infer(term)
        .map_err(|error| WorkbenchDiagnostic {
            layer: DiagnosticLayer::Semantics,
            message: error.to_string(),
            candidates: Vec::new(),
        })?;
    let canonical = canonicalize(term);
    let ast = SemanticSurfaceGenerator::new(language).plan(&canonical)?;
    let surface = language
        .syntax()
        .linearize(&ast)
        .map_err(|error| WorkbenchDiagnostic::generation(error.to_string()))?;

    let surface_roundtrip = language
        .analyze_surface(&surface)
        .ok()
        .map(|analysis| analysis.canonical_semantics);
    let utterance_roundtrip = language
        .analyze_utterance(&surface)
        .ok()
        .map(|analysis| canonicalize(&analysis.pragmatics.utterance).to_string());
    let canonical_text = canonical.to_string();
    let (roundtrip_semantics, roundtrip_verified) = if surface_roundtrip.as_deref() == Some(canonical_text.as_str()) {
        (surface_roundtrip.unwrap_or_default(), true)
    } else if utterance_roundtrip.as_deref() == Some(canonical_text.as_str()) {
        (utterance_roundtrip.unwrap_or_default(), true)
    } else {
        (
            utterance_roundtrip
                .or(surface_roundtrip)
                .unwrap_or_else(|| "<unavailable>".into()),
            false,
        )
    };
    if !roundtrip_verified {
        return Err(WorkbenchDiagnostic::generation(format!(
            "generated surface `{surface}` did not round-trip to `{canonical_text}`; obtained `{roundtrip_semantics}`"
        )));
    }

    Ok(GenerationWorkbenchAnalysis {
        input_semantics: term.to_string(),
        canonical_semantics: canonical_text,
        inferred_type: inferred.to_string(),
        surface_ast: surface_node(&ast),
        canonical_surface: surface,
        roundtrip_semantics,
        roundtrip_verified,
        package_fingerprint: language.package_fingerprint().into(),
    })
}

pub fn diagnostic_from_language_error(error: LanguageError) -> WorkbenchDiagnostic {
    let mut candidates = Vec::new();
    let layer = match &error {
        LanguageError::Io { .. }
        | LanguageError::PackageManifest(_)
        | LanguageError::Package(_)
        | LanguageError::TypedSemantics(_) => DiagnosticLayer::Package,
        LanguageError::Phonology(_) | LanguageError::RootInventory(_) => DiagnosticLayer::Phonology,
        LanguageError::MorphologyConfig(_) | LanguageError::Morphology(_) => DiagnosticLayer::Morphology,
        LanguageError::Dictionary(_) => DiagnosticLayer::Lexical,
        LanguageError::LiteralConfig(_) | LanguageError::UnitsConfig(_) | LanguageError::Literal(_) => {
            DiagnosticLayer::Literal
        }
        LanguageError::SyntaxConfig(_) | LanguageError::Syntax(_) => DiagnosticLayer::Syntax,
        LanguageError::Discourse(resolution) => {
            if let DiscourseResolutionError::Discourse(DiscourseError::AmbiguousReference {
                candidates: values,
                ..
            }) = resolution
            {
                candidates = values
                    .iter()
                    .map(|candidate| AmbiguityCandidateView {
                        id: candidate.id.to_string(),
                        ty: candidate.ty.to_string(),
                        value: canonicalize(&candidate.value).to_string(),
                        origin: introduction_origin_label(&candidate.origin),
                    })
                    .collect();
            }
            DiagnosticLayer::Discourse
        }
        LanguageError::DiscourseGenerate(_) => DiagnosticLayer::Generation,
        LanguageError::Pragmatics(_) | LanguageError::Conversation(_) => DiagnosticLayer::Pragmatics,
        LanguageError::TextStructure(_) => DiagnosticLayer::TextStructure,
        LanguageError::SemanticExpression(_) => DiagnosticLayer::Semantics,
    };
    WorkbenchDiagnostic {
        layer,
        message: error.to_string(),
        candidates,
    }
}

fn surface_workbench_from_analysis(
    language: &LanguagePackage,
    source: &str,
    analysis: &crate::language::DiscourseSurfaceAnalysis,
) -> Result<SurfaceWorkbenchAnalysis, WorkbenchDiagnostic> {
    let explanation = crate::semantics::Explainer::new(language.semantics())
        .explain(&analysis.resolved.term)
        .map_err(|error| WorkbenchDiagnostic {
            layer: DiagnosticLayer::Semantics,
            message: error.to_string(),
            candidates: Vec::new(),
        })?;
    Ok(SurfaceWorkbenchAnalysis {
        source: source.into(),
        canonical_surface: analysis.canonical_surface.clone(),
        canonical_resolved_surface: analysis.canonical_resolved_surface.clone(),
        inferred_type: analysis.inferred_type.clone(),
        surface_ast: surface_node(&analysis.syntax),
        typed_template: analysis.typed.template.to_string(),
        references: reference_views(&analysis.typed, &analysis.resolved),
        contexts: context_views(&analysis.typed, &analysis.resolved),
        aliases: alias_views(&analysis.typed, &analysis.resolved),
        scopes: scope_views(&analysis.syntax),
        semantic_ir: analysis.resolved.term.to_string(),
        canonical_semantic_ir: analysis.canonical_semantics.clone(),
        semantic_explanation: explanation_view(&explanation),
        package_fingerprint: language.package_fingerprint().into(),
    })
}

fn lexical_origin(language: &LanguagePackage, entry: &DictionaryEntry) -> Option<Origin> {
    let id = language.typed_semantics().symbol_id(&entry.root)?;
    let symbol = language.typed_semantics().symbol(id)?;
    Some(Origin::new(
        symbol.provenance.source.clone(),
        symbol.provenance.declaration,
    ))
}

fn reference_views(
    typed: &TypedSurfaceAst,
    resolved: &crate::discourse::ResolvedSurfaceAst,
) -> Vec<ReferenceSlotView> {
    typed
        .references
        .iter()
        .enumerate()
        .map(|(index, slot)| ReferenceSlotView {
            placeholder: slot.placeholder.clone(),
            role: slot.role.clone(),
            expected_type: slot.expected_type.to_string(),
            source: format!("{:?}", slot.source).to_lowercase(),
            resolution: resolved.references.get(index).map(|binding| ResolvedBindingView {
                id: Some(binding.referent.id.to_string()),
                ty: binding.referent.ty.to_string(),
                value: canonicalize(&binding.referent.value).to_string(),
                origin: introduction_origin_label(&binding.referent.origin),
            }),
        })
        .collect()
}

fn context_views(
    typed: &TypedSurfaceAst,
    resolved: &crate::discourse::ResolvedSurfaceAst,
) -> Vec<ContextSlotView> {
    typed
        .contexts
        .iter()
        .enumerate()
        .map(|(index, slot)| ContextSlotView {
            placeholder: slot.placeholder.clone(),
            surface: slot.surface.clone(),
            key: slot.key.clone(),
            declared_type: slot.declared_type.to_string(),
            expected_type: slot.expected_type.to_string(),
            resolution: resolved.contexts.get(index).map(|binding| ResolvedBindingView {
                id: None,
                ty: binding.value.ty.to_string(),
                value: canonicalize(&binding.value.value).to_string(),
                origin: format!("context:{}", binding.slot.key),
            }),
        })
        .collect()
}

fn alias_views(
    typed: &TypedSurfaceAst,
    resolved: &crate::discourse::ResolvedSurfaceAst,
) -> Vec<AliasSlotView> {
    typed
        .aliases
        .iter()
        .enumerate()
        .map(|(index, slot)| AliasSlotView {
            placeholder: slot.placeholder.clone(),
            surface: slot.surface.clone(),
            alias: slot.alias.clone(),
            declared_type: slot.declared_type.to_string(),
            expected_type: slot.expected_type.to_string(),
            resolution: resolved.aliases.get(index).map(|binding| ResolvedBindingView {
                id: Some(binding.referent.id.to_string()),
                ty: binding.referent.ty.to_string(),
                value: canonicalize(&binding.referent.value).to_string(),
                origin: introduction_origin_label(&binding.referent.origin),
            }),
        })
        .collect()
}

fn explanation_view(explanation: &Explanation) -> SemanticNodeView {
    SemanticNodeView {
        label: explanation.label.clone(),
        ty: explanation.ty.to_string(),
        origin: explanation.origin.as_ref().map(ToString::to_string),
        children: explanation
            .children
            .iter()
            .map(|edge| SemanticEdgeView {
                relation: edge.relation.clone(),
                node: explanation_view(&edge.node),
            })
            .collect(),
    }
}

fn pragmatic_view(analysis: &PragmaticAnalysis) -> PragmaticWorkbenchView {
    PragmaticWorkbenchView {
        act: analysis.act.label().into(),
        effects: analysis.effects.iter().map(|effect| match effect {
            DiscourseEffect::Commit { content } => format!("commit {}", canonicalize(content)),
            DiscourseEffect::Retract { target } => format!("retract u{target}"),
            DiscourseEffect::Replace { target, replacement } => format!("replace u{target} {}", canonicalize(replacement)),
            DiscourseEffect::Clarify { target, content } => format!("clarify u{target} {}", canonicalize(content)),
        }).collect(),
        utterance: analysis.utterance.to_string(),
        inferred_type: analysis.inferred_type.to_string(),
    }
}

fn state_view(discourse: &DiscourseState, conversation: &ConversationState) -> DiscourseStateView {
    DiscourseStateView {
        scope: discourse.current_scope().to_string(),
        frame: discourse.current_frame().to_string(),
        referents: discourse
            .referents()
            .map(|referent| ReferentWorkbenchView {
                id: referent.id.to_string(),
                ty: referent.ty.to_string(),
                value: canonicalize(&referent.value).to_string(),
                origin: introduction_origin_label(&referent.origin),
                scope: referent.scope.to_string(),
                frame: referent.frame.to_string(),
                shorthand: referent.shorthand,
            })
            .collect(),
        aliases: discourse
            .active_aliases()
            .into_iter()
            .map(|alias| AliasWorkbenchView {
                surface: alias.surface,
                referent: alias.referent.to_string(),
                ty: alias.ty.to_string(),
                scope: alias.scope.to_string(),
            })
            .collect(),
        history: conversation
            .history()
            .map(|entry| HistoryWorkbenchView {
                id: entry.id.to_string(),
                source: entry.source.clone(),
                canonical_surface: entry.canonical_surface.clone(),
                act: entry.analysis.act.label().into(),
                semantics: entry.analysis.utterance.to_string(),
            })
            .collect(),
        active_commitments: conversation
            .active_commitments()
            .into_iter()
            .map(|commitment| CommitmentWorkbenchView {
                entry: commitment.entry.to_string(),
                content: canonicalize(&commitment.content).to_string(),
            })
            .collect(),
    }
}

fn realization_label(realization: TextRealization) -> &'static str {
    match realization {
        TextRealization::Spoken => "spoken",
        TextRealization::Written => "written",
    }
}

fn introduction_origin_label(origin: &IntroductionOrigin) -> String {
    match origin {
        IntroductionOrigin::Surface { source } => format!("surface:{source}"),
        IntroductionOrigin::Context { key } => format!("context:{key}"),
        IntroductionOrigin::External { label } => format!("external:{label}"),
        IntroductionOrigin::Definition { alias } => format!("definition:{alias}"),
    }
}

fn surface_node(expression: &SurfaceExpr) -> AstNodeView {
    match expression {
        SurfaceExpr::Atom(surface) => leaf("atom", surface),
        SurfaceExpr::Context(surface) => leaf("context", surface),
        SurfaceExpr::Alias(surface) => leaf("alias", surface),
        SurfaceExpr::Captured { marker, payload } => leaf("name", &format!("{marker} {payload}")),
        SurfaceExpr::Quote(payload) => leaf("quote", payload),
        SurfaceExpr::Literal(literal) => leaf("literal", literal.canonical_surface()),
        SurfaceExpr::Clause(clause) => {
            let mut children = Vec::new();
            if let Some(primary) = &clause.primary {
                children.push(argument_node("primary", primary));
            }
            children.extend(
                clause
                    .inner_prefixes
                    .iter()
                    .map(|prefix| leaf("inner_prefix", prefix)),
            );
            children.extend(
                clause
                    .rest
                    .iter()
                    .enumerate()
                    .map(|(index, argument)| argument_node(&format!("rest[{index}]"), argument)),
            );
            AstNodeView {
                kind: "clause".into(),
                label: clause.predicate.clone(),
                children,
            }
        }
        SurfaceExpr::Prefix { operator, operand } => AstNodeView {
            kind: "prefix".into(),
            label: operator.clone(),
            children: vec![surface_node(operand)],
        },
        SurfaceExpr::Infix { operator, operands } => AstNodeView {
            kind: "infix".into(),
            label: operator.clone(),
            children: operands.iter().map(surface_node).collect(),
        },
        SurfaceExpr::Outer { operator, content } => AstNodeView {
            kind: "outer".into(),
            label: operator.clone(),
            children: vec![surface_node(content)],
        },
    }
}

fn argument_node(role: &str, argument: &Argument) -> AstNodeView {
    let (kind, label, children) = match argument {
        Argument::Atom(surface) => ("atom", surface.clone(), Vec::new()),
        Argument::Context(surface) => ("context", surface.clone(), Vec::new()),
        Argument::Reference(surface) => ("reference", surface.clone(), Vec::new()),
        Argument::Alias(surface) => ("alias", surface.clone(), Vec::new()),
        Argument::Captured { marker, payload } => ("name", format!("{marker} {payload}"), Vec::new()),
        Argument::Quote(payload) => ("quote", payload.clone(), Vec::new()),
        Argument::Literal(literal) => ("literal", literal.canonical_surface().into(), Vec::new()),
        Argument::Information { marker, knower } => (
            "information",
            marker.clone(),
            knower
                .iter()
                .map(|knower| match knower {
                    InformationKnower::Context(surface) => leaf("knower_context", surface),
                    InformationKnower::Captured { marker, payload } => {
                        leaf("knower_name", &format!("{marker} {payload}"))
                    }
                })
                .collect(),
        ),
        Argument::Omitted => ("omitted", "∅".into(), Vec::new()),
        Argument::Scoped { binder, direct, restriction } => (
            "scoped",
            binder.clone(),
            direct
                .iter()
                .enumerate()
                .map(|(index, argument)| argument_node(&format!("direct[{index}]"), argument))
                .chain(std::iter::once(leaf("restriction", restriction)))
                .collect(),
        ),
    };
    AstNodeView {
        kind: format!("argument:{kind}"),
        label: role.into(),
        children: std::iter::once(leaf(kind, &label)).chain(children).collect(),
    }
}

fn leaf(kind: &str, label: &str) -> AstNodeView {
    AstNodeView {
        kind: kind.into(),
        label: label.into(),
        children: Vec::new(),
    }
}

fn scope_views(expression: &SurfaceExpr) -> Vec<ScopeView> {
    let mut scopes = Vec::new();
    collect_scopes(expression, "root", &mut scopes);
    scopes
}

fn collect_scopes(expression: &SurfaceExpr, path: &str, output: &mut Vec<ScopeView>) {
    match expression {
        SurfaceExpr::Prefix { operator, operand } => {
            output.push(ScopeView {
                path: path.into(),
                kind: "prefix".into(),
                operator: operator.clone(),
            });
            collect_scopes(operand, &format!("{path}.operand"), output);
        }
        SurfaceExpr::Infix { operator, operands } => {
            output.push(ScopeView {
                path: path.into(),
                kind: "infix".into(),
                operator: operator.clone(),
            });
            for (index, operand) in operands.iter().enumerate() {
                collect_scopes(operand, &format!("{path}.operand[{index}]"), output);
            }
        }
        SurfaceExpr::Outer { operator, content } => {
            output.push(ScopeView {
                path: path.into(),
                kind: "outer".into(),
                operator: operator.clone(),
            });
            collect_scopes(content, &format!("{path}.content"), output);
        }
        SurfaceExpr::Clause(clause) => {
            for (index, prefix) in clause.inner_prefixes.iter().enumerate() {
                output.push(ScopeView {
                    path: format!("{path}.inner_prefix[{index}]"),
                    kind: "inner_prefix".into(),
                    operator: prefix.clone(),
                });
            }
            for (index, argument) in clause
                .primary
                .iter()
                .chain(clause.rest.iter())
                .enumerate()
            {
                match argument {
                    Argument::Scoped { binder, .. } => output.push(ScopeView {
                        path: format!("{path}.argument[{index}]"),
                        kind: "binder".into(),
                        operator: binder.clone(),
                    }),
                    _ => {}
                }
            }
        }
        SurfaceExpr::Atom(_)
        | SurfaceExpr::Context(_)
        | SurfaceExpr::Alias(_)
        | SurfaceExpr::Captured { .. }
        | SurfaceExpr::Quote(_)
        | SurfaceExpr::Literal(_) => {}
    }
}

#[derive(Clone)]
struct VariableOverride {
    variable: String,
    argument: Argument,
}

struct SemanticSurfaceGenerator<'a> {
    language: &'a LanguagePackage,
}

impl<'a> SemanticSurfaceGenerator<'a> {
    fn new(language: &'a LanguagePackage) -> Self {
        Self { language }
    }

    fn plan(&self, term: &Term) -> Result<SurfaceExpr, WorkbenchDiagnostic> {
        self.plan_expr(term, &[])
    }

    fn plan_expr(
        &self,
        term: &Term,
        overrides: &[VariableOverride],
    ) -> Result<SurfaceExpr, WorkbenchDiagnostic> {
        if let Some((body, override_value)) = self.quantifier_body(term)? {
            let mut next = overrides.to_vec();
            next.push(override_value);
            return self.plan_expr(body, &next);
        }
        match term {
            Term::Const(name) => {
                let symbol = self.symbol_id(name)?;
                self.find_lexeme(|lexeme| matches!(lexeme, CompiledSurfaceBinding::Atom { semantic } if *semantic == symbol))
                    .map(SurfaceExpr::Atom)
                    .ok_or_else(|| WorkbenchDiagnostic::generation(format!(
                        "semantic constant `{name}` has no canonical surface atom"
                    )))
            },
            Term::Literal(literal) => self.literal_expr(literal),
            Term::Call {
                function,
                arguments,
            } => {
                if let Some(default) = self.language.typed_semantics().default_effect() {
                    if self.language.typed_semantics().source_name_for_symbol(default).is_some_and(|name| name == function) {
                        let debug = self.language.typed_semantics().debug_symbol(default).ok_or_else(|| {
                            WorkbenchDiagnostic::generation("default discourse effect has no debug parameter table")
                        })?;
                        let role = debug.parameter_names.first().ok_or_else(|| {
                            WorkbenchDiagnostic::generation("default discourse effect has no proposition argument")
                        })?;
                        let content = arguments.get(role).ok_or_else(|| {
                            WorkbenchDiagnostic::generation(format!("default assertion `{function}` is missing role `{role}`"))
                        })?;
                        return self.plan_expr(content, overrides);
                    }
                }
                let function_id = self.symbol_id(function)?;
                if let Some(surface) = self.find_lexeme(|lexeme| {
                    matches!(lexeme, CompiledSurfaceBinding::Capture { semantic, .. } if *semantic == function_id)
                }) {
                    let CompiledSurfaceBinding::Capture { parameter, .. } = self
                        .language
                        .syntax()
                        .lexicon()
                        .get(&surface)
                        .expect("surface came from lexicon")
                    else {
                        unreachable!();
                    };
                    let parameter_name = self.parameter_name(function, *parameter)?;
                    let Some(Term::Literal(Literal::String(payload))) = arguments.get(&parameter_name) else {
                        return Err(WorkbenchDiagnostic::generation(format!(
                            "capture operator `{function}` requires one string payload"
                        )));
                    };
                    return Ok(SurfaceExpr::Captured {
                        marker: surface,
                        payload: payload.clone(),
                    });
                }
                if let Some(surface) = self.find_lexeme(|lexeme| {
                    matches!(lexeme, CompiledSurfaceBinding::Prefix { semantic, outer_only: false, .. } if *semantic == function_id)
                }) {
                    let CompiledSurfaceBinding::Prefix { parameter, .. } = self
                        .language
                        .syntax()
                        .lexicon()
                        .get(&surface)
                        .expect("surface came from lexicon")
                    else {
                        unreachable!();
                    };
                    let parameter_name = self.parameter_name(function, *parameter)?;
                    let operand = arguments.get(&parameter_name).ok_or_else(|| {
                        WorkbenchDiagnostic::generation(format!(
                            "prefix operator `{function}` is missing parameter {parameter}"
                        ))
                    })?;
                    return Ok(SurfaceExpr::Prefix {
                        operator: surface,
                        operand: Box::new(self.plan_expr(operand, overrides)?),
                    });
                }
                if let Some(surface) = self.find_lexeme(|lexeme| {
                    matches!(lexeme, CompiledSurfaceBinding::Prefix { semantic, outer_only: true, .. } if *semantic == function_id)
                }) {
                    let CompiledSurfaceBinding::Prefix { parameter, outer_only: true, .. } = self
                        .language
                        .syntax()
                        .lexicon()
                        .get(&surface)
                        .expect("surface came from lexicon")
                    else {
                        unreachable!();
                    };
                    let parameter_name = self.parameter_name(function, *parameter)?;
                    let content = arguments.get(&parameter_name).ok_or_else(|| {
                        WorkbenchDiagnostic::generation(format!(
                            "outer operator `{function}` is missing parameter {parameter}"
                        ))
                    })?;
                    return Ok(SurfaceExpr::Outer {
                        operator: surface,
                        content: Box::new(self.plan_expr(content, overrides)?),
                    });
                }
                if let Some(surface) = self.find_lexeme(|lexeme| {
                    matches!(lexeme, CompiledSurfaceBinding::Infix { semantic, .. } if *semantic == function_id)
                }) {
                    return self.plan_infix(function, &surface, arguments, overrides);
                }
                if let Some(surface) = self.find_lexeme(|lexeme| {
                    matches!(lexeme, CompiledSurfaceBinding::Class { semantic, .. } if *semantic == function_id)
                }) {
                    let CompiledSurfaceBinding::Class { parameter, .. } = self
                        .language
                        .syntax()
                        .lexicon()
                        .get(&surface)
                        .expect("surface came from lexicon")
                    else {
                        unreachable!();
                    };
                    let parameter_name = self.parameter_name(function, *parameter)?;
                    let value = arguments.get(&parameter_name).ok_or_else(|| {
                        WorkbenchDiagnostic::generation(format!(
                            "class operator `{function}` is missing parameter {parameter}"
                        ))
                    })?;
                    return Ok(SurfaceExpr::Clause(Clause {
                        primary: Some(self.plan_argument(value, overrides)?),
                        inner_prefixes: Vec::new(),
                        predicate: surface,
                        rest: Vec::new(),
                    }));
                }
                if let Some(surface) = self.find_lexeme(|lexeme| {
                    matches!(lexeme, CompiledSurfaceBinding::Predicate { semantic, .. } if *semantic == function_id)
                }) {
                    let CompiledSurfaceBinding::Predicate {
                        primary_parameter,
                        rest_parameters,
                        ..
                    } = self
                        .language
                        .syntax()
                        .lexicon()
                        .get(&surface)
                        .expect("surface came from lexicon")
                    else {
                        unreachable!();
                    };
                    let primary = primary_parameter
                        .as_ref()
                        .map(|parameter| {
                            let parameter_name = self.parameter_name(function, *parameter)?;
                            arguments
                                .get(&parameter_name)
                                .ok_or_else(|| WorkbenchDiagnostic::generation(format!(
                                    "predicate `{function}` is missing parameter {parameter}"
                                )))
                                .and_then(|term| self.plan_argument(term, overrides))
                        })
                        .transpose()?;
                    let rest = rest_parameters
                        .iter()
                        .map(|parameter| {
                            let parameter_name = self.parameter_name(function, *parameter)?;
                            arguments
                                .get(&parameter_name)
                                .ok_or_else(|| WorkbenchDiagnostic::generation(format!(
                                    "predicate `{function}` is missing parameter {parameter}"
                                )))
                                .and_then(|term| self.plan_argument(term, overrides))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    return Ok(SurfaceExpr::Clause(Clause {
                        primary,
                        inner_prefixes: Vec::new(),
                        predicate: surface,
                        rest,
                    }));
                }
                Err(WorkbenchDiagnostic::generation(format!(
                    "semantic operator `{function}` has no generatable canonical surface realization"
                )))
            }
            Term::Var(variable) => Err(WorkbenchDiagnostic::generation(format!(
                "free semantic variable `{variable}` cannot be realized as canonical surface"
            ))),
            Term::Bind { .. } => Err(WorkbenchDiagnostic::generation(
                "standalone semantic binder has no canonical surface realization",
            )),
            Term::Record(_) | Term::Field { .. } => Err(WorkbenchDiagnostic::generation(
                "record-valued semantic structure has no direct canonical surface realization",
            )),
        }
    }

    fn plan_infix(
        &self,
        function: &str,
        surface: &str,
        arguments: &BTreeMap<String, Term>,
        overrides: &[VariableOverride],
    ) -> Result<SurfaceExpr, WorkbenchDiagnostic> {
        let CompiledSurfaceBinding::Infix {
            left_parameter,
            right_parameter,
            associative,
            ..
        } = self
            .language
            .syntax()
            .lexicon()
            .get(surface)
            .expect("surface came from lexicon")
        else {
            unreachable!();
        };
        let left_name = self.parameter_name(function, *left_parameter)?;
        let right_name = self.parameter_name(function, *right_parameter)?;
        let left = arguments.get(&left_name).ok_or_else(|| {
            WorkbenchDiagnostic::generation(format!(
                "infix operator `{function}` is missing parameter {left_parameter}"
            ))
        })?;
        let right = arguments.get(&right_name).ok_or_else(|| {
            WorkbenchDiagnostic::generation(format!(
                "infix operator `{function}` is missing parameter {right_parameter}"
            ))
        })?;
        let mut terms = Vec::new();
        if *associative {
            self.collect_same_infix(function, &left_name, &right_name, left, &mut terms);
            self.collect_same_infix(function, &left_name, &right_name, right, &mut terms);
        } else {
            terms.push(left);
            terms.push(right);
        }
        Ok(SurfaceExpr::Infix {
            operator: surface.into(),
            operands: terms
                .into_iter()
                .map(|term| self.plan_expr(term, overrides))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    fn collect_same_infix<'b>(
        &self,
        function: &str,
        left_name: &str,
        right_name: &str,
        term: &'b Term,
        output: &mut Vec<&'b Term>,
    ) {
        if let Term::Call {
            function: nested,
            arguments,
        } = term
        {
            if nested == function {
                if let (Some(left), Some(right)) =
                    (arguments.get(left_name), arguments.get(right_name))
                {
                    self.collect_same_infix(function, left_name, right_name, left, output);
                    self.collect_same_infix(function, left_name, right_name, right, output);
                    return;
                }
            }
        }
        output.push(term);
    }

    fn plan_argument(
        &self,
        term: &Term,
        overrides: &[VariableOverride],
    ) -> Result<Argument, WorkbenchDiagnostic> {
        if let Term::Var(variable) = term {
            if let Some(value) = overrides.iter().rev().find(|value| value.variable == *variable) {
                return Ok(value.argument.clone());
            }
            return Err(WorkbenchDiagnostic::generation(format!(
                "free semantic variable `{variable}` cannot fill a surface argument"
            )));
        }
        match term {
            Term::Const(name) => {
                let symbol = self.symbol_id(name)?;
                self.find_lexeme(|lexeme| matches!(lexeme, CompiledSurfaceBinding::Atom { semantic } if *semantic == symbol))
                    .map(Argument::Atom)
                    .ok_or_else(|| WorkbenchDiagnostic::generation(format!(
                        "semantic constant `{name}` has no argument surface atom"
                    )))
            },
            Term::Literal(Literal::String(payload)) => Ok(Argument::Quote(payload.clone())),
            Term::Literal(Literal::Structured(value)) if matches!(&value.value, StructuredValue::Information { .. }) => {
                self.information_argument(&value.value)
            }
            Term::Literal(literal) => Ok(Argument::Literal(self.literal_surface(literal)?)),
            Term::Call {
                function,
                arguments,
            } => {
                let function_id = self.symbol_id(function)?;
                if let Some(surface) = self.find_lexeme(|lexeme| {
                    matches!(lexeme, CompiledSurfaceBinding::Capture { semantic, .. } if *semantic == function_id)
                }) {
                    let CompiledSurfaceBinding::Capture { parameter, .. } = self
                        .language
                        .syntax()
                        .lexicon()
                        .get(&surface)
                        .expect("surface came from lexicon")
                    else {
                        unreachable!();
                    };
                    let parameter_name = self.parameter_name(function, *parameter)?;
                    let Some(Term::Literal(Literal::String(payload))) = arguments.get(&parameter_name) else {
                        return Err(WorkbenchDiagnostic::generation(format!(
                            "capture operator `{function}` requires a string payload"
                        )));
                    };
                    Ok(Argument::Captured {
                        marker: surface,
                        payload: payload.clone(),
                    })
                } else {
                    Err(WorkbenchDiagnostic::generation(format!(
                        "nested semantic call `{function}` cannot fill a simple surface argument"
                    )))
                }
            }
            Term::Var(_) => unreachable!("handled above"),
            Term::Bind { .. } | Term::Record(_) | Term::Field { .. } => Err(
                WorkbenchDiagnostic::generation(
                    "semantic structure cannot fill a simple canonical surface argument",
                ),
            ),
        }
    }

    fn information_argument(
        &self,
        value: &StructuredValue,
    ) -> Result<Argument, WorkbenchDiagnostic> {
        let StructuredValue::Information { status, knower, .. } = value else {
            return Err(WorkbenchDiagnostic::generation("expected a typed information value"));
        };
        let status_name = match status {
            InformationStatus::Unknown => "unknown",
            InformationStatus::Unspecified => "unspecified",
            InformationStatus::Withheld => "withheld",
        };
        let marker = self
            .find_lexeme(|lexeme| {
                matches!(lexeme, CompiledSurfaceBinding::Information { status: candidate, .. } if candidate == status)
            })
            .ok_or_else(|| {
                WorkbenchDiagnostic::generation(format!(
                    "information status `{status_name}` has no canonical surface marker"
                ))
            })?;
        let CompiledSurfaceBinding::Information { knower_type, .. } = self
            .language
            .syntax()
            .lexicon()
            .get(&marker)
            .expect("surface came from lexicon")
        else {
            unreachable!();
        };
        let knower = match (knower_type, knower) {
            (None, None) => None,
            (Some(_), Some(InformationKnowerValue::Context(slot))) => {
                let surface = self
                    .find_lexeme(|lexeme| {
                        matches!(lexeme, CompiledSurfaceBinding::Context { slot: candidate, .. } if candidate == slot)
                    })
                    .ok_or_else(|| {
                        WorkbenchDiagnostic::generation(format!(
                            "information knower context `{slot}` has no canonical surface marker"
                        ))
                    })?;
                Some(InformationKnower::Context(surface))
            }
            (Some(_), Some(InformationKnowerValue::Value(value))) => {
                let Term::Call { function, arguments } = value.as_ref() else {
                    return Err(WorkbenchDiagnostic::generation(
                        "typed information knower value has no canonical captured-value realization"
                    ));
                };
                let function_id = self.symbol_id(function)?;
                let surface = self
                    .find_lexeme(|lexeme| {
                        matches!(lexeme, CompiledSurfaceBinding::Capture { semantic: candidate, .. } if *candidate == function_id)
                    })
                    .ok_or_else(|| {
                        WorkbenchDiagnostic::generation(format!(
                            "information knower operator `{function}` has no canonical capture marker"
                        ))
                    })?;
                let CompiledSurfaceBinding::Capture { parameter, .. } = self
                    .language
                    .syntax()
                    .lexicon()
                    .get(&surface)
                    .expect("surface came from lexicon")
                else {
                    unreachable!();
                };
                let parameter_name = self.parameter_name(function, *parameter)?;
                let Some(Term::Literal(Literal::String(payload))) = arguments.get(&parameter_name) else {
                    return Err(WorkbenchDiagnostic::generation(
                        "information knower captured value lacks its text payload"
                    ));
                };
                Some(InformationKnower::Captured { marker: surface, payload: payload.clone() })
            }
            _ => {
                return Err(WorkbenchDiagnostic::generation(format!(
                    "typed information value does not match the declared knower shape for `{marker}`"
                )));
            }
        };
        Ok(Argument::Information { marker, knower })
    }

    fn quantifier_body<'b>(
        &self,
        term: &'b Term,
    ) -> Result<Option<(&'b Term, VariableOverride)>, WorkbenchDiagnostic> {
        let Term::Call { function, arguments } = term else { return Ok(None); };
        let function_id = self.symbol_id(function)?;
        for (surface, lexeme) in self.language.syntax().lexicon().iter() {
            let CompiledSurfaceBinding::Binder {
                semantic,
                binder_parameter,
                direct_parameters,
                variable_type,
                combiner_semantic,
                combiner_left_parameter,
                combiner_right_parameter,
            } = lexeme else { continue; };
            if *semantic != function_id { continue; }
            let binder_name = self.parameter_name(function, *binder_parameter)?;
            let Some(Term::Bind { variable, variable_type: actual_variable_type, body: combined }) = arguments.get(&binder_name) else { continue; };
            if variable_type != actual_variable_type { continue; }
            let Term::Call { function: combined_function, arguments: combined_arguments } = combined.as_ref() else { continue; };
            let combiner_name = self.language.syntax().lexicon().semantic_name(*combiner_semantic).ok_or_else(|| {
                WorkbenchDiagnostic::generation(format!("unknown binder combiner `{combiner_semantic}`"))
            })?;
            if combined_function != combiner_name { continue; }
            let left_name = self.parameter_name(combiner_name, *combiner_left_parameter)?;
            let right_name = self.parameter_name(combiner_name, *combiner_right_parameter)?;
            let Some(restriction) = combined_arguments.get(&left_name) else { continue; };
            let Some(body) = combined_arguments.get(&right_name) else { continue; };
            let Some(restriction_surface) = self.restriction_surface(restriction, variable)? else { continue; };
            let direct = direct_parameters
                .iter()
                .map(|parameter| {
                    let parameter_name = self.parameter_name(function, *parameter)?;
                    arguments.get(&parameter_name).ok_or_else(|| WorkbenchDiagnostic::generation(format!(
                        "scoped binder `{function}` is missing direct parameter {parameter}"
                    ))).and_then(|term| self.plan_argument(term, &[]))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let argument = Argument::Scoped {
                binder: surface.clone(),
                direct,
                restriction: restriction_surface,
            };
            return Ok(Some((body, VariableOverride { variable: variable.clone(), argument })));
        }
        Ok(None)
    }

    fn restriction_surface(&self, term: &Term, variable: &str) -> Result<Option<String>, WorkbenchDiagnostic> {
        let Term::Call {
            function,
            arguments,
        } = term
        else {
            return Ok(None);
        };
        let function_id = self.symbol_id(function)?;
        for (surface, lexeme) in self.language.syntax().lexicon().iter() {
            let parameter = match lexeme {
                CompiledSurfaceBinding::Class { semantic, parameter } if *semantic == function_id => Some(*parameter),
                CompiledSurfaceBinding::Predicate { semantic, primary_parameter: Some(parameter), rest_parameters }
                    if *semantic == function_id && rest_parameters.is_empty() => Some(*parameter),
                _ => None,
            };
            let Some(parameter) = parameter else { continue; };
            let parameter_name = self.parameter_name(function, parameter)?;
            if matches!(arguments.get(&parameter_name), Some(Term::Var(value)) if value == variable) {
                return Ok(Some(surface.clone()));
            }
        }
        Ok(None)
    }

    fn literal_expr(&self, literal: &Literal) -> Result<SurfaceExpr, WorkbenchDiagnostic> {
        if let Literal::String(payload) = literal {
            return Ok(SurfaceExpr::Quote(payload.clone()));
        }
        Ok(SurfaceExpr::Literal(self.literal_surface(literal)?))
    }

    fn literal_surface(&self, literal: &Literal) -> Result<SurfaceLiteral, WorkbenchDiagnostic> {
        let engine = self.language.literals().ok_or_else(|| {
            WorkbenchDiagnostic::generation("language package has no structured-literal generator")
        })?;
        match literal {
            Literal::Structured(value) => {
                let written = engine
                    .render_written(value)
                    .map_err(|error| WorkbenchDiagnostic::generation(error.to_string()))?;
                engine
                    .parse_complete(&written)
                    .map_err(|error| WorkbenchDiagnostic::generation(error.to_string()))
            }
            Literal::Integer(value) => engine
                .parse_complete(&value.to_string())
                .map_err(|error| WorkbenchDiagnostic::generation(error.to_string())),
            Literal::Boolean(value) => Err(WorkbenchDiagnostic::generation(format!(
                "boolean literal `{value}` has no declared Systean surface codec"
            ))),
            Literal::String(_) => Err(WorkbenchDiagnostic::generation(
                "string literal is realized as an opaque quote, not a structured literal",
            )),
        }
    }

    fn symbol_id(&self, function: &str) -> Result<crate::semantics::SymbolId, WorkbenchDiagnostic> {
        self.language.typed_semantics().symbol_id(function).ok_or_else(|| {
            WorkbenchDiagnostic::generation(format!("unknown semantic symbol `{function}`"))
        })
    }

    fn parameter_name(&self, function: &str, parameter: u32) -> Result<String, WorkbenchDiagnostic> {
        let signature = self.language.semantics().operator(function).ok_or_else(|| {
            WorkbenchDiagnostic::generation(format!("unknown semantic operator `{function}`"))
        })?;
        signature
            .parameters
            .get(parameter as usize)
            .map(|parameter| parameter.name.clone())
            .ok_or_else(|| WorkbenchDiagnostic::generation(format!(
                "semantic operator `{function}` has no parameter {parameter}"
            )))
    }

    fn find_lexeme(
        &self,
        predicate: impl Fn(&CompiledSurfaceBinding) -> bool,
    ) -> Option<String> {
        self.language
            .syntax()
            .lexicon()
            .iter()
            .find_map(|(surface, lexeme)| predicate(lexeme).then(|| surface.clone()))
    }
}
