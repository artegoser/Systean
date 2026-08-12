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
    child.stdin.as_mut().unwrap().write_all(script.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
        output.status.success(),
    )
}

#[test]
fn playground_classifies_phase12_question_types_without_intonation() {
    let (stdout, stderr, success) = run_script(
        "say na alfa viv\n\
         say ke na alfa viv\n\
         say ke na artemi vid unk na mari\n\
         say ke ki na alfa viv zo na beta viv ku\n\
         say da na alfa viv\n\
         say me na alfa viv\n\
         quit\n",
    );

    assert!(success);
    assert!(stderr.is_empty(), "{stderr}");
    for act in [
        "act: assertion",
        "act: truth_question",
        "act: value_question",
        "act: choice_question",
        "act: command",
        "act: request",
    ] {
        assert!(stdout.contains(act), "missing {act}:\n{stdout}");
    }
}

#[test]
fn playground_exposes_phase13_expressive_focus_and_topic_acts() {
    let (stdout, stderr, success) = run_script(
        "context speaker na(payload = \"artemi\")\n\
         say mi felis 0.8\n\
         say emo mi felis 0.8\n\
         say na beta fok na alfa vid na beta\n\
         say na alfa top ne na alfa viv\n\
         quit\n",
    );

    assert!(success);
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("act: assertion"));
    assert!(stdout.contains("act: expressive"));
    assert!(stdout.contains("act: focus"));
    assert!(stdout.contains("act: topic"));
    assert!(stdout.contains("felis"));
}

#[test]
fn playground_preserves_history_and_updates_commitments_for_repairs() {
    let (stdout, stderr, success) = run_script(
        "say na alfa viv\n\
         say uno kor na alfa mor\n\
         say uno klar na alfa per\n\
         history\n\
         commitments\n\
         say ret uno\n\
         commitments\n\
         quit\n",
    );

    assert!(success);
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("u1 act=assertion surface=na alfa viv"));
    assert!(stdout.contains("u2 act=correction surface=uno kor na alfa mor"));
    assert!(stdout.contains("u3 act=clarification surface=uno klar na alfa per"));
    assert!(stdout.contains("act: retraction"));
    assert!(stdout.contains("mor(entity = na(payload = \"alfa\"))"));
}
