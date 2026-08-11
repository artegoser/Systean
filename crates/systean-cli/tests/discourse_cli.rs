use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_script(script: &str) -> (String, String, bool) {
    let binary = env!("CARGO_BIN_EXE_systean");
    let language = repository().join("language");
    let mut child = Command::new(binary)
        .arg("--language")
        .arg(language)
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
        .write_all(script.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
        output.status.success(),
    )
}

#[test]
fn playground_exposes_reference_alias_frame_and_scope_behavior() {
    let (stdout, stderr, success) = run_script(
        "intro sol\n\
         resolve Entity\n\
         ali zen sol\n\
         fra\n\
         resolve Entity\n\
         analyze zen\n\
         scope enter\n\
         def val sol\n\
         analyze val\n\
         scope leave\n\
         analyze val\n\
         state\n\
         quit\n",
    );

    assert!(success);
    assert!(stdout.contains("introduced: r0:Entity=sol"));
    assert!(stdout.contains("resolved: r0:Entity=sol"));
    assert!(stdout.contains("alias: zen -> r0"));
    assert!(stdout.contains("frame: f1"));
    assert!(stdout.contains("canonical semantics: sol"));
    assert!(stdout.contains("definition: val -> r1"));
    assert!(stdout.contains("scope: s1"));
    assert!(stdout.contains("scope: s0"));
    assert!(stderr.contains("has no accessible candidate"));
    assert!(stderr.contains("unknown lexical root `val`"));
}

#[test]
fn playground_relative_binding_is_lexical_and_temporary() {
    let (stdout, stderr, success) = run_script(
        "rel zen ki sol ku ki zen ku\n\
         analyze zen\n\
         quit\n",
    );

    assert!(success);
    assert!(stdout.contains("canonical surface: zen"));
    assert!(stdout.contains("canonical semantics: sol"));
    assert!(stderr.contains("unknown lexical root `zen`"));
}

#[test]
fn playground_reports_ambiguous_generic_reference_without_ranking() {
    let (stdout, stderr, success) = run_script(
        "intro sol\n\
         intro sol\n\
         resolve Entity\n\
         bind zen r1\n\
         analyze zen\n\
         quit\n",
    );

    assert!(success);
    assert!(stdout.contains("introduced: r0:Entity=sol"));
    assert!(stdout.contains("introduced: r1:Entity=sol"));
    assert!(stdout.contains("alias: zen -> r1"));
    assert!(stdout.contains("canonical semantics: sol"));
    assert!(stderr.contains("is ambiguous"));
    assert!(stderr.contains("r0:Entity=sol"));
    assert!(stderr.contains("r1:Entity=sol"));
}
