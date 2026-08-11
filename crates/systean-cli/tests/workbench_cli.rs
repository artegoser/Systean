use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run(args: &[&str]) -> (String, String, bool) {
    let output = Command::new(env!("CARGO_BIN_EXE_systean"))
        .arg("--language")
        .arg(repository().join("language"))
        .arg("workbench")
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
        output.status.success(),
    )
}

#[test]
fn workbench_cli_exposes_package_and_unified_word_report_as_json() {
    let (package, stderr, success) = run(&["package"]);
    assert!(success, "{stderr}");
    let package: serde_json::Value = serde_json::from_str(&package).unwrap();
    assert_eq!(package["manifest"]["package"]["version"], "0.17.0");
    assert!(package["provenance"]["fingerprint"].as_str().unwrap().starts_with("fnv1a64:"));

    let (word, stderr, success) = run(&["word", "vid"]);
    assert!(success, "{stderr}");
    let word: serde_json::Value = serde_json::from_str(&word).unwrap();
    assert_eq!(word["root"], "vid");
    assert_eq!(word["dictionary_entry"]["semantic"]["kind"], "operator");
    assert!(word["package"]["provenance"]["sources"].as_array().unwrap().len() > 7);
}

#[test]
fn workbench_cli_generates_canonical_surface_from_semantic_ir() {
    let (stdout, stderr, success) = run(&[
        "generate",
        "see(observed = proper_name(payload = \"mari\"), observer = proper_name(payload = \"artemi\"))",
    ]);
    assert!(success, "{stderr}");
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["canonical_surface"], "na artemi vid na mari");
    assert_eq!(report["roundtrip_verified"], true);
}

#[test]
fn workbench_cli_groups_failures_by_language_layer() {
    let (_stdout, stderr, success) = run(&["surface", "not-a-systean-root"]);
    assert!(!success);
    let diagnostic: serde_json::Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(diagnostic["layer"], "syntax");
    assert!(!diagnostic["message"].as_str().unwrap().is_empty());
}
