use std::path::{Path, PathBuf};
use systean_core::language::LanguagePackage;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn canonical_language_package_loads_as_one_validated_unit() {
    let language = LanguagePackage::load(repository().join("language")).unwrap();
    assert_eq!(language.phonology().alphabet.letters().len(), 22);
    assert_eq!(language.phonology().alphabet.pronounce("Systean").unwrap(), "sjstean");
    assert_eq!(
        language.roots().roots(),
        &["da", "ke", "me", "mu", "ne", "ra", "sol", "va", "zo"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    );
    assert_eq!(language.generate_word("sol").unwrap(), "sol");
    assert!(language.semantics().operator("cease").is_some());
    assert_eq!(language.syntax().config().scope.open, "ki");
    assert_eq!(language.syntax().config().scope.close, "ku");
    assert!(language.syntax().config().logic.precedence["and"] > language.syntax().config().logic.precedence["or"]);
    assert_eq!(language.syntax().config().lexemes.len(), 8);
    for surface in ["ne", "va", "zo", "ra", "mu", "ke", "da", "me"] {
        assert!(language.syntax().config().lexemes.contains_key(surface));
    }
}

#[test]
fn language_package_can_be_built_from_embedded_sources() {
    let repo = repository();
    let alphabet = std::fs::read_to_string(repo.join("language/alphabet.toml")).unwrap();
    let phonology = std::fs::read_to_string(repo.join("language/phonology.toml")).unwrap();
    let morphology = std::fs::read_to_string(repo.join("language/morphology.toml")).unwrap();
    let syntax = std::fs::read_to_string(repo.join("language/syntax.toml")).unwrap();
    let dictionary = std::fs::read_to_string(repo.join("language/dictionary.toml")).unwrap();
    let semantics = std::fs::read_to_string(repo.join("language/semantics/core.semsys")).unwrap();
    let language = LanguagePackage::from_sources(
        &alphabet,
        &phonology,
        &morphology,
        &syntax,
        &dictionary,
        &[("language/semantics/core.semsys", &semantics)],
    )
    .unwrap();
    assert!(language.dictionary().entries().iter().any(|entry| entry.root == "sol"));
    assert_eq!(language.generate_word("sol").unwrap(), "sol");
}

#[test]
fn language_package_rejects_phonologically_invalid_dictionary_roots() {
    let repo = repository();
    let alphabet = std::fs::read_to_string(repo.join("language/alphabet.toml")).unwrap();
    let phonology = std::fs::read_to_string(repo.join("language/phonology.toml")).unwrap();
    let morphology = std::fs::read_to_string(repo.join("language/morphology.toml")).unwrap();
    let syntax = std::fs::read_to_string(repo.join("language/syntax.toml")).unwrap();
    let semantics = std::fs::read_to_string(repo.join("language/semantics/core.semsys")).unwrap();
    let dictionary = "[str]\ndefinition = \"invalid root without a vowel\"\n";

    let error = LanguagePackage::from_sources(
        &alphabet,
        &phonology,
        &morphology,
        &syntax,
        dictionary,
        &[("language/semantics/core.semsys", &semantics)],
    )
    .unwrap_err();

    assert!(error.to_string().contains("root `str` is invalid"));
}

#[test]
fn language_package_reserves_scope_markers_from_lexical_roots() {
    let repo = repository();
    let alphabet = std::fs::read_to_string(repo.join("language/alphabet.toml")).unwrap();
    let phonology = std::fs::read_to_string(repo.join("language/phonology.toml")).unwrap();
    let morphology = std::fs::read_to_string(repo.join("language/morphology.toml")).unwrap();
    let syntax = std::fs::read_to_string(repo.join("language/syntax.toml")).unwrap();
    let semantics = std::fs::read_to_string(repo.join("language/semantics/core.semsys")).unwrap();
    let dictionary = "[ki]\ndefinition = \"must collide with the reserved scope opener\"\n";

    let error = LanguagePackage::from_sources(
        &alphabet,
        &phonology,
        &morphology,
        &syntax,
        dictionary,
        &[("language/semantics/core.semsys", &semantics)],
    )
    .unwrap_err();

    assert!(error.to_string().contains("scope marker `ki` collides"));
}
