use std::path::{Path, PathBuf};

use systean_core::language::{Dictionary, LanguagePackage};
use systean_core::morphology::MorphemeKind;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn canonical_morphology_is_bare_root_identity() {
    let language = LanguagePackage::load(repository().join("language")).unwrap();
    let analysis = language.analyze_word("sol").unwrap();

    assert_eq!(analysis.morphology.root, "sol");
    assert_eq!(analysis.morphology.spelling, "sol");
    assert_eq!(analysis.morphology.root_grapheme_range, 0..3);
    assert_eq!(analysis.morphology.morphemes.len(), 1);
    assert_eq!(analysis.morphology.morphemes[0].kind, MorphemeKind::Root);
    assert_eq!(analysis.phonology.stressed_pronunciation, "ˈsol");
    assert_eq!(language.generate_word("sol").unwrap(), "sol");
}

#[test]
fn morphology_round_trips_every_declared_root() {
    let language = LanguagePackage::load(repository().join("language")).unwrap();

    for root in language.roots().roots() {
        let word = language.generate_word(root).unwrap();
        let analysis = language.analyze_word(&word).unwrap();
        assert_eq!(&analysis.morphology.root, root);
        assert_eq!(&word, root);
    }
}

#[test]
fn old_pos_endings_are_not_implicitly_accepted() {
    let language = LanguagePackage::load(repository().join("language")).unwrap();

    for old_surface_form in ["solo", "sola", "soli", "sole", "solu", "soly"] {
        let error = language.analyze_word(old_surface_form).unwrap_err();
        assert!(error.to_string().contains("no normative morphological analysis"));
    }
}

#[test]
fn old_prefix_stack_is_not_implicitly_accepted() {
    let language = LanguagePackage::load(repository().join("language")).unwrap();
    let error = language.analyze_word("ntspsol").unwrap_err();
    assert!(error.to_string().contains("no normative morphological analysis"));
}

#[test]
fn dictionary_rejects_pos_specific_lexical_meanings() {
    let error = Dictionary::from_toml(
        r#"
        [sol]
        definition = "one lexical identity"
        semantic = { kind = "constant", type = "Entity" }
        adj = "old contextual adjective meaning"
        "#,
    )
    .unwrap_err();

    assert!(error.to_string().contains("unsupported field `adj`"));
}

#[test]
fn dictionary_requires_one_definition_per_root() {
    let error = Dictionary::from_toml("[sol]\nsemantic = { kind = \"constant\", type = \"Entity\" }\n").unwrap_err();
    assert!(error.to_string().contains("non-empty `definition`"));
}
