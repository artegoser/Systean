use std::path::{Path, PathBuf};

use systean_core::discourse::{DiscourseState, IntroductionOrigin, TextRealization};
use systean_core::language::LanguagePackage;
use systean_core::workbench::{self, DiagnosticLayer};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    LanguagePackage::load(repository().join("language")).unwrap()
}

#[test]
fn workbench_exposes_package_version_fingerprint_and_full_provenance() {
    let language = language();
    let info = workbench::package_info(&language);

    assert_eq!(info.manifest.package.version, "0.18.0");
    assert_eq!(info.manifest.package.revision, 18);
    assert_eq!(info.provenance.fingerprint, language.package_fingerprint());
    assert_eq!(info.semantic_fingerprint, language.semantic_fingerprint());
    assert_eq!(info.surface_fingerprint, language.surface_fingerprint());
    assert!(info.provenance.sources.iter().any(|source| source.path == "package.toml"));
    assert!(info.validation.generated_surfaces > 0);
}

#[test]
fn word_workbench_unifies_dictionary_phonology_morphology_semantics_and_provenance() {
    let language = language();
    let report = workbench::analyze_word(&language, "vid").unwrap();

    assert_eq!(report.root, "vid");
    assert_eq!(report.dictionary_entry.root, "vid");
    assert_eq!(report.pronunciation, language.analyze_word("vid").unwrap().phonology.pronunciation);
    assert_eq!(report.morphemes.len(), 1);
    assert!(!report.syllables.is_empty());
    assert!(report.semantic_origin.is_some());
    assert_eq!(report.package.provenance.fingerprint, language.package_fingerprint());
}

#[test]
fn surface_workbench_shows_typed_ast_scope_semantic_ir_and_resolved_reference() {
    let language = language();
    let mut discourse = DiscourseState::new();
    let named = language
        .analyze_surface_with_discourse("na artemi", &DiscourseState::new())
        .unwrap()
        .resolved
        .term;
    let id = discourse
        .introduce(
            named,
            IntroductionOrigin::External {
                label: "phase17-test".into(),
            },
            language.semantics(),
        )
        .unwrap();

    let report = workbench::analyze_surface(&language, "ref viv", &discourse).unwrap();
    assert_eq!(report.inferred_type, "Proposition");
    assert_eq!(report.references.len(), 1);
    let id_text = id.to_string();
    assert_eq!(
        report.references[0]
            .resolution
            .as_ref()
            .unwrap()
            .id
            .as_deref(),
        Some(id_text.as_str())
    );
    assert!(report.surface_ast.kind.contains("clause"));
    assert!(report.semantic_ir.contains("viv"));
    assert!(!report.semantic_explanation.label.is_empty());
}

#[test]
fn workbench_diagnostic_preserves_ambiguity_candidates_in_the_discourse_layer() {
    let language = language();
    let mut discourse = DiscourseState::new();
    for name in ["artemi", "mari"] {
        let named = language
            .analyze_surface_with_discourse(&format!("na {name}"), &DiscourseState::new())
            .unwrap()
            .resolved
            .term;
        discourse
            .introduce(
                named,
                IntroductionOrigin::External {
                    label: name.into(),
                },
                language.semantics(),
            )
            .unwrap();
    }

    let error = workbench::analyze_surface(&language, "ref viv", &discourse).unwrap_err();
    assert_eq!(error.layer, DiagnosticLayer::Discourse);
    assert_eq!(error.candidates.len(), 2);
    assert!(error.candidates.iter().all(|candidate| candidate.ty == "Entity"));
}

#[test]
fn semantic_generator_round_trips_through_the_same_language_pipeline() {
    let language = language();
    let source = "na artemi vid na mari";
    let semantic = language.analyze_surface(source).unwrap().canonical_semantics;
    let generated = workbench::generate_surface(&language, &semantic).unwrap();

    assert_eq!(generated.canonical_surface, source);
    assert!(generated.roundtrip_verified);
    assert_eq!(generated.roundtrip_semantics, semantic);
}

#[test]
fn semantic_generator_recovers_quantifier_surface_without_parser_branch_ranking() {
    let language = language();
    let source = "ra per viv";
    let semantic = language.analyze_surface(source).unwrap().canonical_semantics;
    let generated = workbench::generate_surface(&language, &semantic).unwrap();

    assert_eq!(generated.canonical_surface, source);
    assert!(generated.roundtrip_verified);
}

#[test]
fn discourse_workbench_reports_state_transitions_and_history() {
    let language = language();
    let report = workbench::analyze_text_stream(
        &language,
        "na artemi viv. ke na artemi viv.",
        TextRealization::Written,
    )
    .unwrap();

    assert_eq!(report.turns.len(), 1);
    assert!(report.turns[0].before.history.is_empty());
    assert_eq!(report.turns[0].after.history.len(), 2);
    assert_eq!(report.final_state.history.len(), 2);
    assert!(!report.final_state.active_commitments.is_empty());
}

#[test]
fn literal_inspector_exposes_written_spoken_and_semantic_forms() {
    let language = language();
    let report = workbench::analyze_literal(&language, "2000-12-31").unwrap();

    assert_eq!(report.ty, "CalendarDate");
    assert_eq!(report.canonical_written, "2000-12-31");
    assert!(!report.canonical_spoken.is_empty());
    assert!(!report.semantic_canonical.is_empty());
}

#[test]
fn embedded_versioned_package_has_the_same_fingerprint_as_native_package() {
    let native = language();
    let root = repository().join("language");
    let read = |path: &str| std::fs::read_to_string(root.join(path)).unwrap();
    let manifest = read("package.toml");
    let alphabet = read("alphabet.toml");
    let phonology = read("phonology.toml");
    let morphology = read("morphology.toml");
    let syntax = read("syntax.toml");
    let dictionary = read("dictionary.toml");
    let literals = read("literals.toml");
    let units = read("units.toml");
    let core = read("typed/core.semsys");
    let lexicon = read("typed/lexicon.semsys");
    let typed_units = read("typed/units.semsys");
    let compatibility = read("corpus/compatibility.tsv");
    let adversarial = read("corpus/adversarial.tsv");

    let embedded = LanguagePackage::from_versioned_sources_full(
        &manifest,
        &alphabet,
        &phonology,
        &morphology,
        &syntax,
        &dictionary,
        &literals,
        &units,
        &[
            ("typed/core.semsys", &core),
            ("typed/lexicon.semsys", &lexicon),
            ("typed/units.semsys", &typed_units),
        ],
        &compatibility,
        &adversarial,
    )
    .unwrap();

    assert_eq!(embedded.manifest(), native.manifest());
    assert_eq!(embedded.provenance(), native.provenance());
    assert_eq!(embedded.validation_report(), native.validation_report());
    assert_eq!(embedded.package_fingerprint(), native.package_fingerprint());
}

#[test]
fn term_generator_handles_structured_literals_without_reparsing_display_text() {
    let language = language();
    let syntax = language.syntax().parse("2000-12-31").unwrap();
    let lowered = language.syntax().lower(&syntax, language.semantics()).unwrap();
    let generated = workbench::generate_term(&language, &lowered.term).unwrap();

    assert_eq!(generated.canonical_surface, "2000-12-31");
    assert!(generated.roundtrip_verified);
}

#[test]
fn term_generator_recovers_information_status_arguments() {
    let language = language();
    let source = "na artemi vid unk na mari";
    let syntax = language.syntax().parse(source).unwrap();
    let lowered = language.syntax().lower(&syntax, language.semantics()).unwrap();
    let generated = workbench::generate_term(&language, &lowered.term).unwrap();

    assert_eq!(generated.canonical_surface, source);
    assert!(generated.roundtrip_verified);
}
