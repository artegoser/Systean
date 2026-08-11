use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use systean_core::language::LanguagePackage;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    LanguagePackage::load(repository().join("language")).unwrap()
}

#[test]
fn whole_language_package_exposes_manifest_provenance_and_validation_report() {
    let language = language();
    assert_eq!(language.manifest().package.name, "systean");
    assert_eq!(language.manifest().package.version, "0.16.0");
    assert_eq!(language.manifest().package.revision, 16);
    assert_eq!(language.manifest().compatibility.epoch, 1);
    assert!(language.package_fingerprint().starts_with("fnv1a64:"));
    assert!(
        language
            .provenance()
            .sources
            .iter()
            .any(|source| source.path == "package.toml")
    );

    let report = language.validation_report();
    assert!(report.owned_forms > language.roots().roots().len());
    assert!(report.generated_asts > 0);
    assert!(report.generated_surfaces > 0);
    assert!(report.semantic_roundtrips > 0);
    assert!(report.exhaustive_candidates > 0);
    assert!(report.exhaustive_valid_surfaces > 0);
    assert_eq!(report.compatibility_entries, 8);
    assert_eq!(report.adversarial_entries, 8);
}

#[test]
fn package_fingerprint_is_deterministic_for_identical_sources() {
    let first = language();
    let second = language();
    assert_eq!(first.provenance(), second.provenance());
    assert_eq!(first.package_fingerprint(), second.package_fingerprint());
}

#[test]
fn manifest_module_version_mismatch_fails_package_compilation() {
    let fixture = copy_language_fixture("module-version");
    let manifest = fixture.join("package.toml");
    let source = fs::read_to_string(&manifest).unwrap();
    fs::write(&manifest, source.replace("syntax = \"1.6\"", "syntax = \"999.0\"")).unwrap();

    let error = LanguagePackage::load(&fixture).unwrap_err().to_string();
    assert!(error.contains("module `syntax` declares version `1.6`"), "{error}");
    assert!(error.contains("manifest requires `999.0`"), "{error}");
    fs::remove_dir_all(fixture).unwrap();
}

#[test]
fn normative_cross_layer_collision_fails_package_compilation() {
    let fixture = copy_language_fixture("surface-collision");
    let units = fixture.join("units.toml");
    let source = fs::read_to_string(&units).unwrap();
    fs::write(
        &units,
        source.replacen("spoken = \"metr\"", "spoken = \"sol\"", 1),
    )
    .unwrap();

    let error = LanguagePackage::load(&fixture).unwrap_err().to_string();
    assert!(error.contains("sol"), "{error}");
    assert!(error.contains("collides") || error.contains("owners"), "{error}");
    fs::remove_dir_all(fixture).unwrap();
}

#[test]
fn compatibility_snapshot_diff_detects_parse_or_meaning_change() {
    let language = language();
    let snapshot = language
        .compatibility_snapshot([
            ("entity".to_owned(), "sol".to_owned()),
            ("predicate".to_owned(), "mi vi tu".to_owned()),
        ])
        .unwrap();
    assert!(language.diff_compatibility(&snapshot).unwrap().is_empty());

    let mut changed = snapshot;
    changed.entries[0].canonical_semantics = "different".into();
    let diff = language.diff_compatibility(&changed).unwrap();
    assert_eq!(diff.len(), 1);
    assert_eq!(diff[0].label, changed.entries[0].label);
}

#[test]
fn compatibility_epoch_change_is_explicitly_incompatible() {
    let language = language();
    let mut snapshot = language
        .compatibility_snapshot([("entity".to_owned(), "sol".to_owned())])
        .unwrap();
    snapshot.compatibility_epoch += 1;
    let error = language.diff_compatibility(&snapshot).unwrap_err();
    assert!(error.contains("compatibility epoch changed"), "{error}");
}

#[test]
fn frozen_corpora_are_compile_time_inputs_not_optional_test_data() {
    let fixture = copy_language_fixture("corpus");
    let corpus = fixture.join("corpus/compatibility.tsv");
    let source = fs::read_to_string(&corpus).unwrap();
    fs::write(
        &corpus,
        source.replace("bare entity\tsol\tsol\tEntity\tsol", "bare entity\tsol\tsol\tEntity\twrong"),
    )
    .unwrap();

    let error = LanguagePackage::load(&fixture).unwrap_err().to_string();
    assert!(error.contains("bare entity"), "{error}");
    assert!(error.contains("semantics changed"), "{error}");
    fs::remove_dir_all(fixture).unwrap();
}

fn copy_language_fixture(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let destination = std::env::temp_dir().join(format!(
        "systean-phase16-{label}-{}-{nonce}",
        std::process::id()
    ));
    copy_directory(&repository().join("language"), &destination);
    destination
}

fn copy_directory(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_directory(&source_path, &destination_path);
        } else {
            fs::copy(source_path, destination_path).unwrap();
        }
    }
}
