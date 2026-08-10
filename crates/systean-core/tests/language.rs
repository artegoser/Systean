use std::fs;
use std::path::{Path, PathBuf};

use systean_core::language::{Dictionary, LanguagePackage, LexicalSemantic};
use systean_core::semantics::Type;
use systean_core::syntax::SyntaxConfig;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn package_from_dictionary(dictionary: &str) -> Result<LanguagePackage, systean_core::language::LanguageError> {
    let repo = repository();
    let alphabet = fs::read_to_string(repo.join("language/alphabet.toml")).unwrap();
    let phonology = fs::read_to_string(repo.join("language/phonology.toml")).unwrap();
    let morphology = fs::read_to_string(repo.join("language/morphology.toml")).unwrap();
    let syntax = fs::read_to_string(repo.join("language/syntax.toml")).unwrap();
    let semantics = fs::read_to_string(repo.join("language/semantics/core.semsys")).unwrap();

    LanguagePackage::from_sources(
        &alphabet,
        &phonology,
        &morphology,
        &syntax,
        dictionary,
        &[("language/semantics/core.semsys", &semantics)],
    )
}

#[test]
fn canonical_language_package_loads_as_one_validated_unit() {
    let language = LanguagePackage::load(repository().join("language")).unwrap();
    assert_eq!(language.phonology().alphabet.letters().len(), 22);
    assert_eq!(
        language.phonology().alphabet.pronounce("Systean").unwrap(),
        "sjstean"
    );
    assert_eq!(
        language.roots().roots(),
        &["da", "ke", "me", "mu", "ne", "ra", "sol", "va", "zo"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    );
    assert_eq!(language.generate_word("sol").unwrap(), "sol");
    assert!(language.semantics().operator("cease").is_some());
    assert_eq!(language.semantics().constant_type("sol"), Some(&Type::named("Entity")));
    assert_eq!(language.syntax().config().scope.open, "ki");
    assert_eq!(language.syntax().config().scope.close, "ku");
    assert!(
        language.syntax().config().logic.precedence["and"]
            > language.syntax().config().logic.precedence["or"]
    );
    assert_eq!(language.syntax().lexicon().len(), language.roots().roots().len());
    for surface in ["sol", "ne", "va", "zo", "ra", "mu", "ke", "da", "me"] {
        assert!(language.syntax().lexicon().contains_key(surface));
    }
}

#[test]
fn syntax_policy_contains_no_duplicate_lexical_root_table() {
    let source = fs::read_to_string(repository().join("language/syntax.toml")).unwrap();
    let value: toml::Value = toml::from_str(&source).unwrap();
    assert!(value.get("lexemes").is_none());
}


#[test]
fn syntax_config_rejects_legacy_duplicate_lexeme_tables() {
    let source = r#"
[order]
frame = "primary_predicate_rest"
free_order = false

[scope]
open = "ki"
close = "ku"
explicit = "when_ambiguous"
quantifier_order = "appearance"

[logic]
flatten_same_operator = true

[logic.precedence]
and = 20
or = 10

[roles]
realization = "frame_order"

[arguments]
omission = "unique_reference_only"

[questions]
realization = "explicit_operator"

[commands]
realization = "explicit_operator"

[focus]
reorders = false

[grammar]
traditional_pos = false

[lexemes.ne]
kind = "prefix"
semantic = "not"
role = "value"
"#;

    let error = SyntaxConfig::from_toml(source).unwrap_err();
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
    let semantics = fs::read_to_string(repo.join("language/semantics/core.semsys")).unwrap();
    let language = LanguagePackage::from_sources(
        &alphabet,
        &phonology,
        &morphology,
        &syntax,
        &dictionary,
        &[("language/semantics/core.semsys", &semantics)],
    )
    .unwrap();
    assert!(
        language
            .dictionary()
            .entries()
            .iter()
            .any(|entry| entry.root == "sol")
    );
    assert_eq!(language.generate_word("sol").unwrap(), "sol");
    assert_eq!(language.analyze_surface("sol").unwrap().inferred_type, "Entity");
}

#[test]
fn dictionary_constant_semantics_are_installed_into_the_environment() {
    let language = LanguagePackage::load(repository().join("language")).unwrap();
    let sol = language
        .dictionary()
        .entries()
        .iter()
        .find(|entry| entry.root == "sol")
        .unwrap();
    assert_eq!(
        sol.semantic,
        LexicalSemantic::Constant {
            name: None,
            ty: "Entity".into(),
        }
    );
    assert_eq!(language.semantics().constant_type("sol"), Some(&Type::named("Entity")));
}

#[test]
fn language_package_rejects_phonologically_invalid_dictionary_roots() {
    let dictionary = r#"
[str]
definition = "invalid root without a vowel"
semantic = { kind = "constant", type = "Entity" }
"#;

    let error = package_from_dictionary(dictionary).unwrap_err();
    assert!(error.to_string().contains("root `str` is invalid"));
}

#[test]
fn language_package_reserves_scope_markers_from_lexical_roots() {
    let dictionary = r#"
[ki]
definition = "must collide with the reserved scope opener"
semantic = { kind = "constant", type = "Entity" }
"#;

    let error = package_from_dictionary(dictionary).unwrap_err();
    assert!(error.to_string().contains("scope marker `ki` collides"));
}

#[test]
fn dictionary_operator_root_requires_surface_realization() {
    let error = Dictionary::from_toml(
        r#"
[ne]
definition = "negation"
semantic = { kind = "operator", name = "not" }
"#,
    )
    .unwrap_err();

    assert!(error.to_string().contains("requires one explicit `syntax` realization"));
}

#[test]
fn dictionary_constant_root_rejects_redundant_surface_atom_configuration() {
    let error = Dictionary::from_toml(
        r#"
[sol]
definition = "star"
semantic = { kind = "constant", type = "Entity" }
syntax = { kind = "prefix", role = "value" }
"#,
    )
    .unwrap_err();

    assert!(error.to_string().contains("automatically a surface atom"));
}

#[test]
fn dictionary_rejects_unknown_semantic_types_at_package_compile_time() {
    let dictionary = r#"
[sol]
definition = "star"
semantic = { kind = "constant", type = "DefinitelyNotAType" }
"#;

    let error = package_from_dictionary(dictionary).unwrap_err();
    assert!(error.to_string().contains("unknown type `DefinitelyNotAType`"), "{error}");
}
