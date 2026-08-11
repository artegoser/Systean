use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_systean"));
    command
        .arg("--language")
        .arg(repository().join("language"));
    command
}

#[test]
fn cli_exposes_canonical_numeric_and_quantity_literals() {
    let number = command()
        .args(["literals", "analyze", "1000005"])
        .output()
        .unwrap();
    assert!(number.status.success());
    let stdout = String::from_utf8(number.stdout).unwrap();
    assert!(stdout.contains("family: number"));
    assert!(stdout.contains("type: Number"));
    assert!(stdout.contains("canonical spoken: mega uno pent"));

    let quantity = command()
        .args(["literals", "analyze", "5", "m"])
        .output()
        .unwrap();
    assert!(quantity.status.success());
    let stdout = String::from_utf8(quantity.stdout).unwrap();
    assert!(stdout.contains("type: Quantity<LengthDimension>"));
    assert!(stdout.contains("canonical spoken: pent metr"));
}

#[test]
fn cli_unit_conversion_is_explicit_and_exact() {
    let output = command()
        .args(["literals", "convert", "1", "km", "--to", "meter"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("canonical written: 1000 m"), "{stdout}");
}

#[test]
fn discourse_playground_accepts_surface_now_context() {
    let mut child = command()
        .arg("discourse")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            b"context-surface now 2026-08-11T12:00:00Z\n\
              analyze nau ante 2026-08-12T00:00:00Z\n\
              quit\n",
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success());
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("context now: instant<\"2026-08-11T12:00:00Z\">"), "{stdout}");
    assert!(stdout.contains("canonical semantics: before("), "{stdout}");
}
