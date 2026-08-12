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
fn playground_accepts_spoken_and_written_turn_streams() {
    let (stdout, stderr, success) = run_script(
        "turn spoken alice na alfa viv du ke na alfa viv du\n\
         turn written bob na beta viv. ke na beta viv.\n\
         history\n\
         quit\n",
    );

    assert!(success);
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("turn: alice (spoken)"), "{stdout}");
    assert!(stdout.contains("turn: bob (written)"), "{stdout}");
    assert!(stdout.contains("spoken: na alfa viv du"), "{stdout}");
    assert!(stdout.contains("written: na alfa viv."), "{stdout}");
    assert!(stdout.contains("u4 act=truth_question"), "{stdout}");
}

#[test]
fn playground_frame_boundary_changes_section_without_erasing_history() {
    let (stdout, stderr, success) = run_script(
        "turn spoken alice na alfa viv du\n\
         turn spoken bob fra uno kor na alfa mor du\n\
         history\n\
         commitments\n\
         quit\n",
    );

    assert!(success);
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("section: b1 frame: f1"), "{stdout}");
    assert!(stdout.contains("u1 act=assertion"), "{stdout}");
    assert!(stdout.contains("u2 act=correction"), "{stdout}");
    assert!(stdout.contains("u2 active=mor(entity = na(payload = \"alfa\"))"), "{stdout}");
}
