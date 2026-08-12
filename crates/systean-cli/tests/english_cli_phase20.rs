use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn command(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_systean"))
        .arg("--language")
        .arg(repository().join("language"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn english_command_prints_the_deterministic_renderer_output() {
    let output = command(&["english", "na", "artemi", "vid", "na", "mari"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "(proper name (\"artemi\")) see (proper name (\"mari\"))"
    );
}

#[test]
fn check_reports_documentation_coverage_and_fingerprint() {
    let output = command(&["check"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("English documentation: 175 entries (fnv1a64:"), "{stdout}");
}
