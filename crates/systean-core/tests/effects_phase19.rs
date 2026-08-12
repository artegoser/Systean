use std::path::{Path, PathBuf};

use systean_core::language::LanguagePackage;
use systean_core::pragmatics::{CommunicativeAct, DiscourseEffect};
use systean_core::spec::{CompiledActKind, CompiledEffectInstruction, TypedCompileError, compile_typed_sources};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    LanguagePackage::load(repository().join("language")).expect("Phase 19 language package")
}

#[test]
fn production_discourse_behavior_is_owned_by_typed_effect_programs() {
    let language = language();
    let typed = language.typed_semantics();
    let default = typed.default_effect().expect("default assertion effect");
    assert_eq!(typed.source_name_for_symbol(default), Some("assert"));
    assert_eq!(typed.effect_programs().count(), 10);

    let ke = typed.symbol_id("ke").unwrap();
    let program = typed.effect_program(ke).expect("question effect");
    assert!(program.instructions.iter().any(|instruction| {
        matches!(instruction, CompiledEffectInstruction::Act { kind: CompiledActKind::Question, .. })
    }));
    assert!(program.instructions.iter().any(|instruction| {
        matches!(instruction, CompiledEffectInstruction::Choice { .. })
    }));

    let syntax = std::fs::read_to_string(repository().join("language/syntax.toml")).unwrap();
    assert!(!syntax.contains("[pragmatics]"));
    assert!(std::fs::read_to_string(repository().join("language/typed/effects.semsys")).unwrap().contains("effect ke"));
}

#[test]
fn generic_effect_vm_preserves_assertion_question_and_focus_behavior() {
    let language = language();

    let assertion = language.analyze_utterance("na alfa viv").unwrap();
    assert!(matches!(assertion.pragmatics.act, CommunicativeAct::Assertion { .. }));
    assert!(matches!(assertion.pragmatics.effects.as_slice(), [DiscourseEffect::Commit { .. }]));

    let question = language.analyze_utterance("ke na alfa viv").unwrap();
    assert!(matches!(question.pragmatics.act, CommunicativeAct::Question { .. }));
    assert!(question.pragmatics.effects.is_empty());

    let focus = language.analyze_utterance("na beta fok na alfa vid na beta").unwrap();
    assert!(matches!(focus.pragmatics.act, CommunicativeAct::Focus { .. }));
    assert!(matches!(focus.pragmatics.effects.as_slice(), [DiscourseEffect::Commit { .. }]));
}

#[test]
fn adding_a_question_root_uses_only_package_declarations() {
    let root = repository().join("language");
    let alphabet = std::fs::read_to_string(root.join("alphabet.toml")).unwrap();
    let phonology = std::fs::read_to_string(root.join("phonology.toml")).unwrap();
    let morphology = std::fs::read_to_string(root.join("morphology.toml")).unwrap();
    let syntax = std::fs::read_to_string(root.join("syntax.toml")).unwrap();
    let mut dictionary = std::fs::read_to_string(root.join("dictionary.toml")).unwrap();
    dictionary.push_str("\n[keb]\ndefinition = \"Test question marker declared entirely by the package.\"\n");
    let literals = std::fs::read_to_string(root.join("literals.toml")).unwrap();
    let units = std::fs::read_to_string(root.join("units.toml")).unwrap();

    let mut sources = Vec::<(String, String)>::new();
    for entry in std::fs::read_dir(root.join("typed")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|value| value.to_str()) == Some("semsys") {
            let name = format!("typed/{}", path.file_name().unwrap().to_string_lossy());
            let mut source = std::fs::read_to_string(&path).unwrap();
            if path.file_name().unwrap().to_string_lossy() == "lexicon.semsys" {
                source.push_str("\nword keb($content: Proposition) -> Utterance { form _ $content; outer; }\n");
            }
            if path.file_name().unwrap().to_string_lossy() == "effects.semsys" {
                source.push_str("\neffect keb { act question($content); }\n");
            }
            sources.push((name, source));
        }
    }
    sources.sort_by(|left, right| left.0.cmp(&right.0));
    let borrowed = sources.iter().map(|(name, source)| (name.as_str(), source.as_str())).collect::<Vec<_>>();
    let package = LanguagePackage::from_sources_full(
        &alphabet, &phonology, &morphology, &syntax, &dictionary, &literals, &units, &borrowed,
    ).expect("package-only question extension");

    let analysis = package.analyze_utterance("keb na alfa viv").unwrap();
    assert!(matches!(analysis.pragmatics.act, CommunicativeAct::Question { .. }));
}


#[test]
fn adding_a_repair_root_uses_only_package_declarations() {
    let root = repository().join("language");
    let alphabet = std::fs::read_to_string(root.join("alphabet.toml")).unwrap();
    let phonology = std::fs::read_to_string(root.join("phonology.toml")).unwrap();
    let morphology = std::fs::read_to_string(root.join("morphology.toml")).unwrap();
    let syntax = std::fs::read_to_string(root.join("syntax.toml")).unwrap();
    let mut dictionary = std::fs::read_to_string(root.join("dictionary.toml")).unwrap();
    dictionary.push_str("\n[ker]\ndefinition = \"Test correction marker declared entirely by the package.\"\n");
    let literals = std::fs::read_to_string(root.join("literals.toml")).unwrap();
    let units = std::fs::read_to_string(root.join("units.toml")).unwrap();

    let mut sources = Vec::<(String, String)>::new();
    for entry in std::fs::read_dir(root.join("typed")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|value| value.to_str()) == Some("semsys") {
            let file_name = path.file_name().unwrap().to_string_lossy();
            let name = format!("typed/{file_name}");
            let mut source = std::fs::read_to_string(&path).unwrap();
            if file_name == "lexicon.semsys" {
                source.push_str("\nword ker($target: Number, $replacement: Proposition) -> Utterance { form $target _ $replacement; precedence 1; outer; }\n");
            }
            if file_name == "effects.semsys" {
                source.push_str("\neffect ker { act correction($target, $replacement); repair replace $target $replacement; }\n");
            }
            sources.push((name, source));
        }
    }
    sources.sort_by(|left, right| left.0.cmp(&right.0));
    let borrowed = sources.iter().map(|(name, source)| (name.as_str(), source.as_str())).collect::<Vec<_>>();
    let package = LanguagePackage::from_sources_full(
        &alphabet, &phonology, &morphology, &syntax, &dictionary, &literals, &units, &borrowed,
    ).expect("package-only repair extension");

    let analysis = package.analyze_utterance("uno ker na alfa mor").unwrap();
    assert!(matches!(analysis.pragmatics.act, CommunicativeAct::Correction { target: 1, .. }));
    assert!(matches!(
        analysis.pragmatics.effects.as_slice(),
        [DiscourseEffect::Replace { target: 1, .. }]
    ));
}

#[test]
fn malformed_effect_programs_fail_structurally() {
    let errors = compile_typed_sources([(
        "bad-effect.semsys".into(),
        r#"
            type Proposition;
            type Utterance;
            word ask($content: Proposition) -> Utterance { form _ $content; outer; }
            effect ask { commit $missing; }
        "#.into(),
    )]).expect_err("bad effect must fail");
    assert!(errors.iter().any(|error| matches!(error, TypedCompileError::InvalidEffectParameter { .. })));

    let errors = compile_typed_sources([(
        "bad-act.semsys".into(),
        r#"
            type Proposition;
            type Utterance;
            word ask($content: Proposition) -> Utterance { form _ $content; outer; }
            effect ask { act correction($content); }
        "#.into(),
    )]).expect_err("wrong act arity must fail during package compilation");
    assert!(errors.iter().any(|error| matches!(error, TypedCompileError::InvalidEffect { .. })));
}
