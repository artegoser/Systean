use std::fs;
use std::path::{Path, PathBuf};

use systean_core::language::{Dictionary, LanguageError, LanguagePackage};
use systean_core::semantics::{ContextSlotId, Type};
use systean_core::syntax::{LexemeConfig, SyntaxConfig};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn typed_sources(repo: &Path) -> Vec<(String, String)> {
    ["core.semsys", "lexicon.semsys", "units.semsys"]
        .into_iter()
        .map(|name| {
            (
                format!("language/typed/{name}"),
                fs::read_to_string(repo.join("language/typed").join(name)).unwrap(),
            )
        })
        .collect()
}

fn package_with_extra(
    extra_dictionary: &str,
    extra_typed: &str,
) -> Result<LanguagePackage, LanguageError> {
    let repo = repository();
    let alphabet = fs::read_to_string(repo.join("language/alphabet.toml")).unwrap();
    let phonology = fs::read_to_string(repo.join("language/phonology.toml")).unwrap();
    let morphology = fs::read_to_string(repo.join("language/morphology.toml")).unwrap();
    let syntax = fs::read_to_string(repo.join("language/syntax.toml")).unwrap();
    let mut dictionary = fs::read_to_string(repo.join("language/dictionary.toml")).unwrap();
    dictionary.push('\n');
    dictionary.push_str(extra_dictionary);
    let mut typed = typed_sources(&repo);
    if !extra_typed.trim().is_empty() {
        typed.push(("tests/extra.semsys".into(), extra_typed.into()));
    }
    let refs = typed
        .iter()
        .map(|(name, source)| (name.as_str(), source.as_str()))
        .collect::<Vec<_>>();

    LanguagePackage::from_sources(
        &alphabet,
        &phonology,
        &morphology,
        &syntax,
        &dictionary,
        &refs,
    )
}

#[test]
fn canonical_language_package_loads_as_one_validated_unit() {
    let language = LanguagePackage::load(repository().join("language")).unwrap();
    assert_eq!(language.phonology().alphabet.letters().len(), 22);
    assert_eq!(language.phonology().alphabet.pronounce("Systean").unwrap(), "sjstean");
    assert_eq!(language.roots().roots().len(), 175);
    assert_eq!(language.dictionary().entries().len(), 175);
    for root in [
        "sol", "ref", "mi", "tu", "na", "ke", "ne", "va", "zo", "ra", "mu", "da", "me",
        "per", "vid", "mov", "viv", "gov", "skrib", "sta", "stop", "unk", "vak", "hid",
        "rov", "mini", "maks", "tip", "stat", "prob", "frek", "imp", "hip", "emo", "fok",
        "top", "ret", "kor", "klar", "felis", "trist", "eros", "eruz", "dolor", "superb",
        "avari", "lusta", "gula", "leni",
    ] {
        assert!(language.roots().roots().iter().any(|known| known == root));
        assert!(language.typed_semantics().symbol_id(root).is_some());
    }
    assert_eq!(language.generate_word("sol").unwrap(), "sol");
    assert!(language.semantics().operator("stop").is_some());
    assert_eq!(language.semantics().constant_type("sol"), Some(&Type::named("Entity")));
    assert_eq!(language.syntax().config().scope.open, "ki");
    assert_eq!(language.syntax().config().scope.close, "ku");
    assert_eq!(language.syntax().config().quotation.open, "sit");
    assert_eq!(language.syntax().config().quotation.close, "tis");
    assert_eq!(language.syntax().config().text.utterance_spoken, "du");
    assert_eq!(language.syntax().config().text.utterance_written, ".");
    assert_eq!(language.syntax().config().discourse.alias, "ali");
    assert_eq!(language.syntax().config().discourse.definition, "def");
    assert_eq!(language.syntax().config().discourse.relative, "rel");
    assert_eq!(language.syntax().config().discourse.frame, "fra");
    assert_eq!(language.syntax().config().pragmatics.default_assertion_operator, "assert");
    assert_eq!(language.syntax().config().pragmatics.expressive_operator, "emo");
    assert!(language.semantics().operator("emo").is_some());
    assert!(language
        .semantics()
        .is_assignable(&Type::named("SubjectiveEvent"), &Type::named("Event")));
    let va = language.typed_semantics().symbol_id("va").unwrap();
    let zo = language.typed_semantics().symbol_id("zo").unwrap();
    assert!(
        language.typed_semantics().surface_rule(va).unwrap().precedence
            > language.typed_semantics().surface_rule(zo).unwrap().precedence
    );
    assert!(language.typed_semantics().surface_rule(va).unwrap().associative);
    assert!(language.typed_semantics().surface_rule(zo).unwrap().associative);
    assert_eq!(language.syntax().lexicon().len(), language.roots().roots().len());
}

#[test]
fn syntax_policy_contains_no_duplicate_lexical_root_table() {
    let source = fs::read_to_string(repository().join("language/syntax.toml")).unwrap();
    let value: toml::Value = toml::from_str(&source).unwrap();
    assert!(value.get("lexemes").is_none());
}

#[test]
fn syntax_config_rejects_legacy_duplicate_lexeme_tables() {
    let mut source = fs::read_to_string(repository().join("language/syntax.toml")).unwrap();
    source.push_str(
        r#"

[lexemes.ne]
kind = "prefix"
semantic = "not"
role = "value"
"#,
    );
    let error = SyntaxConfig::from_toml(&source).unwrap_err();
    assert!(error.to_string().contains("unknown field `lexemes`"), "{error}");
}

#[test]
fn language_package_can_be_built_from_embedded_sources() {
    let repo = repository();
    let alphabet = fs::read_to_string(repo.join("language/alphabet.toml")).unwrap();
    let phonology = fs::read_to_string(repo.join("language/phonology.toml")).unwrap();
    let morphology = fs::read_to_string(repo.join("language/morphology.toml")).unwrap();
    let syntax = fs::read_to_string(repo.join("language/syntax.toml")).unwrap();
    let dictionary = fs::read_to_string(repo.join("language/dictionary.toml")).unwrap();
    let typed = typed_sources(&repo);
    let refs = typed.iter().map(|(name, source)| (name.as_str(), source.as_str())).collect::<Vec<_>>();
    let language = LanguagePackage::from_sources(
        &alphabet,
        &phonology,
        &morphology,
        &syntax,
        &dictionary,
        &refs,
    )
    .unwrap();
    assert!(language.dictionary().entries().iter().any(|entry| entry.root == "sol"));
    assert_eq!(language.generate_word("sol").unwrap(), "sol");
    assert_eq!(language.analyze_surface("sol").unwrap().inferred_type, "Entity");
}

#[test]
fn language_package_can_be_built_from_embedded_full_sources() {
    let repo = repository();
    let alphabet = fs::read_to_string(repo.join("language/alphabet.toml")).unwrap();
    let phonology = fs::read_to_string(repo.join("language/phonology.toml")).unwrap();
    let morphology = fs::read_to_string(repo.join("language/morphology.toml")).unwrap();
    let syntax = fs::read_to_string(repo.join("language/syntax.toml")).unwrap();
    let dictionary = fs::read_to_string(repo.join("language/dictionary.toml")).unwrap();
    let literals = fs::read_to_string(repo.join("language/literals.toml")).unwrap();
    let units = fs::read_to_string(repo.join("language/units.toml")).unwrap();
    let typed = typed_sources(&repo);
    let refs = typed.iter().map(|(name, source)| (name.as_str(), source.as_str())).collect::<Vec<_>>();
    let language = LanguagePackage::from_sources_full(
        &alphabet,
        &phonology,
        &morphology,
        &syntax,
        &dictionary,
        &literals,
        &units,
        &refs,
    )
    .unwrap();
    assert!(language.literals().is_some());
    assert_eq!(
        language.literals().unwrap().parse_complete("mega uno pent").unwrap().canonical_written,
        "1000005"
    );
}

#[test]
fn dictionary_is_metadata_only_and_typed_words_own_semantics() {
    let language = LanguagePackage::load(repository().join("language")).unwrap();
    let sol = language.dictionary().entries().iter().find(|entry| entry.root == "sol").unwrap();
    assert!(!sol.definition.is_empty());
    let sol_id = language.typed_semantics().symbol_id("sol").unwrap();
    let sol_symbol = language.typed_semantics().symbol(sol_id).unwrap();
    assert!(sol_symbol.signature.parameters.is_empty());
    assert_eq!(language.semantics().constant_type("sol"), Some(&Type::named("Entity")));
}

#[test]
fn dictionary_supports_reference_and_context_through_typed_definitions() {
    let language = LanguagePackage::load(repository().join("language")).unwrap();
    assert!(matches!(language.syntax().lexicon().get("ref"), Some(LexemeConfig::Reference)));
    let Some(LexemeConfig::Context { slot, ty, .. }) = language.syntax().lexicon().get("mi") else {
        panic!("mi must compile to a context lexeme")
    };
    assert_eq!(*slot, ContextSlotId::from_source("context", "speaker"));
    assert_eq!(ty, &Type::named("Entity"));
}

#[test]
fn dictionary_rejects_legacy_semantic_binding_fields() {
    let error = Dictionary::from_toml(
        r#"
[sol]
definition = "star"
semantic = { kind = "constant", type = "Entity" }
"#,
    )
    .unwrap_err();
    assert!(error.to_string().contains("semantic identity belongs to language/typed"), "{error}");
}

#[test]
fn dictionary_and_typed_words_must_cover_each_other_exactly() {
    let error = package_with_extra(
        r#"
[lu]
definition = "extra metadata without semantics"
"#,
        "",
    )
    .unwrap_err();
    assert!(error.to_string().contains("ownership mismatch"), "{error}");

    let error = package_with_extra("", "word lu : Entity;").unwrap_err();
    assert!(error.to_string().contains("ownership mismatch"), "{error}");
}

#[test]
fn language_package_rejects_phonologically_invalid_dictionary_roots() {
    let error = package_with_extra(
        r#"
[str]
definition = "invalid root without a vowel"
"#,
        "word str : Entity;",
    )
    .unwrap_err();
    assert!(error.to_string().contains("root `str` is invalid"), "{error}");
}

#[test]
fn language_package_reserves_scope_markers_from_lexical_roots() {
    let error = package_with_extra(
        r#"
[ki]
definition = "must collide with the reserved scope opener"
"#,
        "word ki : Entity;",
    )
    .unwrap_err();
    assert!(error.to_string().contains("scope marker `ki` collides"), "{error}");
}

#[test]
fn language_package_reserves_spoken_utterance_boundary_from_lexical_roots() {
    let error = package_with_extra(
        r#"
[du]
definition = "must collide with the reserved spoken utterance boundary"
"#,
        "word du : Entity;",
    )
    .unwrap_err();
    assert!(error.to_string().contains("spoken utterance boundary `du` collides"), "{error}");
}

#[test]
fn language_package_reserves_discourse_markers_from_lexical_roots() {
    let error = package_with_extra(
        r#"
[fra]
definition = "must collide with the reserved discourse frame marker"
"#,
        "word fra : Entity;",
    )
    .unwrap_err();
    assert!(error.to_string().contains("discourse frame marker `fra` collides"), "{error}");
}

#[test]
fn language_package_reserves_quotation_markers_from_lexical_roots() {
    let error = package_with_extra(
        r#"
[sit]
definition = "must collide with the reserved quotation opener"
"#,
        "word sit : Entity;",
    )
    .unwrap_err();
    assert!(error.to_string().contains("quotation open marker `sit` collides"), "{error}");
}

#[test]
fn typed_semantics_reject_unknown_types_before_surface_compilation() {
    let error = package_with_extra(
        r#"
[zed]
definition = "typed word with a missing type"
"#,
        "word zed : DefinitelyNotAType;",
    )
    .unwrap_err();
    assert!(error.to_string().contains("unknown type `DefinitelyNotAType`"), "{error}");
}
