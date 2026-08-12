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
fn playground_runs_a_small_named_dialogue_with_opaque_text() {
    let (stdout, stderr, success) = run_script(
        "context speaker na(payload = \"artemi\")\n\
         context addressee na(payload = \"mari\")\n\
         analyze mi gov sit sal tis\n\
         analyze ke tu viv\n\
         analyze me tu gov sit sal tis\n\
         quit\n",
    );

    assert!(success);
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("context speaker: na(payload = \"artemi\")"));
    assert!(stdout.contains("context addressee: na(payload = \"mari\")"));
    assert!(stdout.contains(
        "canonical semantics: gov(content = \"sal\", speaker = na(payload = \"artemi\"))"
    ));
    assert!(stdout.contains("canonical semantics: ke(content = viv(entity = na(payload = \"mari\")))"));
    assert!(stdout.contains("canonical semantics: me(content = gov(content = \"sal\", speaker = na(payload = \"mari\")))"));
}

#[test]
fn playground_exposes_named_ambiguity_and_exact_alias_repair() {
    let (stdout, stderr, success) = run_script(
        "context speaker na(payload = \"artemi\")\n\
         intro na alek\n\
         intro na boris\n\
         analyze mi vid ref\n\
         bind zen r0\n\
         analyze mi vid zen\n\
         quit\n",
    );

    assert!(success);
    assert!(stdout.contains("introduced: r0:Entity=na(payload = \"alek\")"));
    assert!(stdout.contains("introduced: r1:Entity=na(payload = \"boris\")"));
    assert!(stdout.contains("alias: zen -> r0"));
    assert!(stdout.contains(
        "canonical semantics: vid(observed = na(payload = \"alek\"), observer = na(payload = \"artemi\"))"
    ));
    assert!(stderr.contains("ambiguous"), "{stderr}");
    assert!(stderr.contains("r0:Entity=na(payload = \"alek\")"), "{stderr}");
    assert!(stderr.contains("r1:Entity=na(payload = \"boris\")"), "{stderr}");
}
