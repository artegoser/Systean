use std::fs;
use std::path::{Path, PathBuf};

use systean_core::discourse::{DiscourseState, IntroductionOrigin};
use systean_core::language::LanguagePackage;
use systean_core::semantics::Type;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    LanguagePackage::load(repository().join("language")).unwrap()
}

#[test]
fn proper_name_round_trips_as_one_marked_entity() {
    let language = language();
    let analysis = language.analyze_surface("na artemi").unwrap();
    assert_eq!(analysis.canonical_surface, "na artemi");
    assert_eq!(analysis.inferred_type, "Entity");
    assert_eq!(
        analysis.canonical_semantics,
        "na(payload = \"artemi\")"
    );

    let pronunciation = language
        .phonology()
        .alphabet
        .pronounce(&analysis.canonical_surface)
        .unwrap();
    assert_eq!(
        language.phonology().alphabet.spell(&pronunciation).unwrap(),
        analysis.canonical_surface
    );
}

#[test]
fn proper_name_payload_is_manual_systean_pronunciation_not_source_language_guessing() {
    let language = language();
    let error = language.analyze_surface("na chris").unwrap_err().to_string();
    assert!(error.contains("proper-name payload `chris`"), "{error}");
    assert!(error.contains("unsupported grapheme"), "{error}");
}

#[test]
fn same_name_referents_remain_distinct_in_discourse() {
    let language = language();
    let empty = DiscourseState::new();
    let named = language
        .analyze_surface_with_discourse("na alek", &empty)
        .unwrap()
        .resolved
        .term;

    let mut discourse = DiscourseState::new();
    let first = discourse
        .introduce(
            named.clone(),
            IntroductionOrigin::External {
                label: "first Alek".into(),
            },
            language.semantics(),
        )
        .unwrap();
    let second = discourse
        .introduce(
            named,
            IntroductionOrigin::External {
                label: "second Alek".into(),
            },
            language.semantics(),
        )
        .unwrap();
    assert_ne!(first, second);

    let error = discourse
        .resolve_reference("named person", &Type::named("Entity"), language.semantics())
        .unwrap_err()
        .to_string();
    assert!(error.contains("ambiguous"), "{error}");

    language.bind_alias(&mut discourse, "alekuno", first).unwrap();
    language.bind_alias(&mut discourse, "alekdva", second).unwrap();
    let first_analysis = language
        .analyze_surface_with_discourse("alekuno", &discourse)
        .unwrap();
    let second_analysis = language
        .analyze_surface_with_discourse("alekdva", &discourse)
        .unwrap();
    assert_eq!(first_analysis.canonical_semantics, second_analysis.canonical_semantics);
    assert_eq!(first_analysis.resolved.aliases[0].referent.id, first);
    assert_eq!(second_analysis.resolved.aliases[0].referent.id, second);
}

#[test]
fn opaque_quote_preserves_foreign_text_without_lexing_it_as_systean() {
    let language = language();
    let source = "sit Hello, мир! sol ne ki <external> 123 tis";
    let analysis = language.analyze_surface(source).unwrap();
    assert_eq!(analysis.canonical_surface, source);
    assert_eq!(analysis.inferred_type, "Text");
    assert_eq!(
        analysis.canonical_semantics,
        "\"Hello, мир! sol ne ki <external> 123\""
    );
}

#[test]
fn opaque_quote_can_fill_a_typed_text_slot_without_parsing_payload_roots() {
    let repo = repository();
    let alphabet = fs::read_to_string(repo.join("language/alphabet.toml")).unwrap();
    let phonology = fs::read_to_string(repo.join("language/phonology.toml")).unwrap();
    let morphology = fs::read_to_string(repo.join("language/morphology.toml")).unwrap();
    let syntax = fs::read_to_string(repo.join("language/syntax.toml")).unwrap();
    let mut dictionary = fs::read_to_string(repo.join("language/dictionary.toml")).unwrap();
    dictionary.push_str(
        r#"
[govtest]
definition = "Fixture text-taking predicate."
"#,
    );
    let core = fs::read_to_string(repo.join("language/typed/core.semsys")).unwrap();
    let mut lexicon = fs::read_to_string(repo.join("language/typed/lexicon.semsys")).unwrap();
    lexicon.push_str("\nword govtest($speaker: Entity, $content: Text) -> Proposition;\n");
    let effects = fs::read_to_string(repo.join("language/typed/effects.semsys")).unwrap();
    let units = fs::read_to_string(repo.join("language/typed/units.semsys")).unwrap();
    let language = LanguagePackage::from_sources(
        &alphabet,
        &phonology,
        &morphology,
        &syntax,
        &dictionary,
        &[
            ("language/typed/core.semsys", &core),
            ("language/typed/lexicon.semsys", &lexicon),
            ("language/typed/effects.semsys", &effects),
            ("language/typed/units.semsys", &units),
        ],
    )
    .unwrap();

    let analysis = language
        .analyze_surface("na artemi govtest sit not valid Systean!!! tis")
        .unwrap();
    assert_eq!(analysis.inferred_type, "Proposition");
    assert!(analysis.canonical_semantics.contains("\"not valid Systean!!!\""));
}

#[test]
fn nested_opaque_quotes_preserve_exact_structural_boundaries() {
    let language = language();
    let source = "sit outer sit inner text tis tail tis";
    let analysis = language.analyze_surface(source).unwrap();
    assert_eq!(analysis.canonical_surface, source);
    assert_eq!(
        analysis.canonical_semantics,
        "\"outer sit inner text tis tail\""
    );

    let reparsed = language.syntax().parse(&analysis.canonical_surface).unwrap();
    assert_eq!(analysis.syntax, reparsed);
}

#[test]
fn malformed_quote_boundaries_fail_structurally() {
    let language = language();
    let missing = language.syntax().parse("sit hello").unwrap_err().to_string();
    assert!(missing.contains("missing close marker `tis`"), "{missing}");

    let stray = language.syntax().parse("tis").unwrap_err().to_string();
    assert!(stray.contains("unexpected quotation close marker `tis`"), "{stray}");
}
