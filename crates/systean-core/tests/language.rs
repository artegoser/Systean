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
    assert_eq!(language.roots().roots(), &["sol".to_owned()]);
    assert_eq!(language.generate_word("sol").unwrap(), "sol");
    assert!(language.semantics().operator("cease").is_some());
}

#[test]
fn language_package_can_be_built_from_embedded_sources() {
    let repo = repository();
    let alphabet = std::fs::read_to_string(repo.join("language/alphabet.toml")).unwrap();
    let phonology = std::fs::read_to_string(repo.join("language/phonology.toml")).unwrap();
    let morphology = std::fs::read_to_string(repo.join("language/morphology.toml")).unwrap();
    let dictionary = std::fs::read_to_string(repo.join("language/dictionary.toml")).unwrap();
    let semantics = std::fs::read_to_string(repo.join("language/semantics/core.semsys")).unwrap();
    let language = LanguagePackage::from_sources(
        &alphabet,
        &phonology,
        &morphology,
        &dictionary,
        &[("language/semantics/core.semsys", &semantics)],
    )
    .unwrap();
    assert_eq!(language.dictionary().entries()[0].root, "sol");
    assert_eq!(language.generate_word("sol").unwrap(), "sol");
}

#[test]
fn language_package_rejects_phonologically_invalid_dictionary_roots() {
    let repo = repository();
    let alphabet = std::fs::read_to_string(repo.join("language/alphabet.toml")).unwrap();
    let phonology = std::fs::read_to_string(repo.join("language/phonology.toml")).unwrap();
    let morphology = std::fs::read_to_string(repo.join("language/morphology.toml")).unwrap();
    let semantics = std::fs::read_to_string(repo.join("language/semantics/core.semsys")).unwrap();
    let dictionary = "[str]\ndefinition = \"invalid root without a vowel\"\n";

    let error = LanguagePackage::from_sources(
        &alphabet,
        &phonology,
        &morphology,
        dictionary,
        &[("language/semantics/core.semsys", &semantics)],
    )
    .unwrap_err();

    assert!(error.to_string().contains("root `str` is invalid"));
}
