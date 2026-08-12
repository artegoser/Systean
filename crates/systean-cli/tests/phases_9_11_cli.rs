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
fn playground_exposes_explicit_aspect_targets_and_repetition() {
    let (stdout, stderr, success) = run_script(
        "intro-sem process_of(content = move(mover = sol))\n\
         bind prun r0\n\
         analyze sta prun\n\
         analyze stop reg prun\n\
         analyze prun rep tri\n\
         quit\n",
    );

    assert!(success);
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("introduced: r0:Process=process_of(content = move(mover = sol))"));
    assert!(stdout.contains("canonical semantics: start(target = process_of(content = move(mover = sol)))"));
    assert!(stdout.contains("canonical semantics: cease(target = habitual(activity = process_of(content = move(mover = sol))))"));
    assert!(stdout.contains("canonical semantics: repeat(count = number<\"3\">, target = process_of(content = move(mover = sol)))"));
}

#[test]
fn playground_exposes_information_statuses_without_guessing() {
    let (stdout, stderr, success) = run_script(
        "context addressee proper_name(payload = \"mari\")\n\
         analyze na artemi vid unk tu\n\
         analyze na artemi vid vak\n\
         analyze na artemi vid hid\n\
         quit\n",
    );

    assert!(success);
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("information<\"unknown:context:@"));
    assert!(stdout.contains("information<\"unspecified\">"));
    assert!(stdout.contains("information<\"withheld\">"));
}

#[test]
fn playground_exposes_counted_quantifiers_and_distinct_generalizations() {
    let (stdout, stderr, success) = run_script(
        "analyze rov tri per viv\n\
         analyze na alfa tip na beta 0.8\n\
         analyze na alfa stat na beta 0.8\n\
         analyze na alfa viv imp na beta viv\n\
         analyze na alfa viv hip na beta viv\n\
         quit\n",
    );

    assert!(success);
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("canonical semantics: exactly(count = number<\"3\">, predicate = bind"));
    assert!(stdout.contains("canonical semantics: typical(domain = proper_name(payload = \"alfa\")"));
    assert!(stdout.contains("canonical semantics: statistical(domain = proper_name(payload = \"alfa\")"));
    assert!(stdout.contains("canonical semantics: implies("));
    assert!(stdout.contains("canonical semantics: counterfactual("));
}
