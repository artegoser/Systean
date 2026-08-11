use std::fs;
use std::path::{Path, PathBuf};

use systean_core::language::{Dictionary, LanguagePackage, LexicalSemantic};
use systean_core::semantics::Type;
use systean_core::syntax::SyntaxConfig;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn package_from_dictionary(dictionary: &str) -> Result<LanguagePackage, systean_core::language::LanguageError> {
    let repo = repository();
    let alphabet = fs::read_to_string(repo.join("language/alphabet.toml")).unwrap();
    let phonology = fs::read_to_string(repo.join("language/phonology.toml")).unwrap();
    let morphology = fs::read_to_string(repo.join("language/morphology.toml")).unwrap();
    let syntax = fs::read_to_string(repo.join("language/syntax.toml")).unwrap();
    let semantics = fs::read_to_string(repo.join("language/semantics/core.semsys")).unwrap();
    let pragmatics = fs::read_to_string(repo.join("language/semantics/pragmatics.semsys")).unwrap();
    let subjective = fs::read_to_string(repo.join("language/semantics/subjective.semsys")).unwrap();

    LanguagePackage::from_sources(
        &alphabet,
        &phonology,
        &morphology,
        &syntax,
        dictionary,
        &[
            ("language/semantics/core.semsys", &semantics),
            ("language/semantics/pragmatics.semsys", &pragmatics),
            ("language/semantics/subjective.semsys", &subjective),
        ],
    )
}

#[test]
fn canonical_language_package_loads_as_one_validated_unit() {
    let language = LanguagePackage::load(repository().join("language")).unwrap();
    assert_eq!(language.phonology().alphabet.letters().len(), 22);
    assert_eq!(
        language.phonology().alphabet.pronounce("Systean").unwrap(),
        "sjstean"
    );
    assert_eq!(language.roots().roots().len(), 175);
    for root in [
        "sol", "ref", "mi", "tu", "na", "ke", "ne", "va", "zo", "ra", "mu", "da", "me",
        "per", "vid", "mov", "viv", "gov", "skrib", "sta", "stop", "unk", "vak",
        "hid", "rov", "mini", "maks", "tip", "stat", "prob", "frek", "imp", "hip",
        "emo", "fok", "top", "ret", "kor", "klar", "felis", "trist", "eros", "eruz",
        "dolor", "superb", "avari", "lusta", "gula", "leni",
    ] {
        assert!(language.roots().roots().iter().any(|known| known == root));
    }
    assert_eq!(language.generate_word("sol").unwrap(), "sol");
    assert!(language.semantics().operator("cease").is_some());
    assert_eq!(language.semantics().constant_type("sol"), Some(&Type::named("Entity")));
    assert_eq!(language.syntax().config().scope.open, "ki");
    assert_eq!(language.syntax().config().scope.close, "ku");
    assert_eq!(language.syntax().config().quotation.open, "sit");
    assert_eq!(language.syntax().config().quotation.close, "tis");
    assert_eq!(language.syntax().config().discourse.alias, "ali");
    assert_eq!(language.syntax().config().discourse.definition, "def");
    assert_eq!(language.syntax().config().discourse.relative, "rel");
    assert_eq!(language.syntax().config().discourse.frame, "fra");
    assert_eq!(language.syntax().config().pragmatics.default_assertion_operator, "assert");
    assert_eq!(language.syntax().config().pragmatics.expressive_operator, "express_affect");
    assert!(language.semantics().operator("express_affect").is_some());
    assert!(language
        .semantics()
        .is_assignable(&Type::named("SubjectiveEvent"), &Type::named("Event")));
    assert!(
        language.syntax().config().logic.precedence["and"]
            > language.syntax().config().logic.precedence["or"]
    );
    assert_eq!(language.syntax().lexicon().len(), language.roots().roots().len());
    for surface in [
        "sol", "ref", "mi", "tu", "na", "ne", "va", "zo", "ra", "mu", "ke", "da", "me",
        "emo", "fok", "top", "ret", "kor", "klar", "felis", "dolor",
    ] {
        assert!(language.syntax().lexicon().contains_key(surface));
    }
}

#[test]
fn syntax_policy_contains_no_duplicate_lexical_root_table() {
    let source = fs::read_to_string(repository().join("language/syntax.toml")).unwrap();
    let value: toml::Value = toml::from_str(&source).unwrap();
    assert!(value.get("lexemes").is_none());
}

#[test]
fn syntax_config_rejects_legacy_duplicate_lexeme_tables() {
    let source = r#"
[order]
frame = "primary_predicate_rest"
free_order = false

[scope]
open = "ki"
close = "ku"
explicit = "when_ambiguous"
quantifier_order = "appearance"

[logic]
flatten_same_operator = true

[logic.precedence]
and = 20
or = 10

[roles]
realization = "frame_order"

[arguments]
omission = "unique_reference_only"

[questions]
realization = "explicit_operator"

[commands]
realization = "explicit_operator"

[focus]
reorders = false

[grammar]
traditional_pos = false

[discourse]
alias = "ali"
definition = "def"
relative = "rel"
frame = "fra"

[quotation]
open = "sit"
close = "tis"

[pragmatics]
default_assertion_operator = "assert"
default_assertion_role = "content"
question_operator = "ask_truth"
question_content_role = "content"
command_operator = "command"
command_content_role = "content"
request_operator = "request"
request_content_role = "content"
expressive_operator = "express_affect"
expressive_state_role = "state"
focus_operator = "focus"
focus_target_role = "target"
focus_content_role = "content"
topic_operator = "topic"
topic_target_role = "target"
topic_content_role = "content"
retract_operator = "retract"
repair_target_role = "target"
correction_operator = "correct"
correction_replacement_role = "replacement"
clarification_operator = "clarify"
clarification_content_role = "content"
disjunction_operator = "or"
disjunction_left_role = "left"
disjunction_right_role = "right"
information_family = "information"
unknown_status = "unknown"

[lexemes.ne]
kind = "prefix"
semantic = "not"
role = "value"
"#;

    let error = SyntaxConfig::from_toml(source).unwrap_err();
    assert!(error.to_string().contains("unknown field `lexemes`"), "{error}");
}

#[test]
fn language_package_can_be_built_from_embedded_sources() {
    let repo = repository();
    let alphabet = fs::read_to_string(repo.join("language/alphabet.toml")).unwrap();
    let phonology = fs::read_to_string(repo.join("language/phonology.toml")).unwrap();
    let morphology = fs::read_to_string(repo.join("language/morphology.toml")).unwrap();
    let syntax = fs::read_to_string(repo.join("language/syntax.toml")).unwrap();
    let dictionary = fs::read_to_string(repo.join("language/dictionary.toml")).unwrap();
    let semantics = fs::read_to_string(repo.join("language/semantics/core.semsys")).unwrap();
    let pragmatics = fs::read_to_string(repo.join("language/semantics/pragmatics.semsys")).unwrap();
    let subjective = fs::read_to_string(repo.join("language/semantics/subjective.semsys")).unwrap();
    let language = LanguagePackage::from_sources(
        &alphabet,
        &phonology,
        &morphology,
        &syntax,
        &dictionary,
        &[
            ("language/semantics/core.semsys", &semantics),
            ("language/semantics/pragmatics.semsys", &pragmatics),
            ("language/semantics/subjective.semsys", &subjective),
        ],
    )
    .unwrap();
    assert!(
        language
            .dictionary()
            .entries()
            .iter()
            .any(|entry| entry.root == "sol")
    );
    assert_eq!(language.generate_word("sol").unwrap(), "sol");
    assert_eq!(language.analyze_surface("sol").unwrap().inferred_type, "Entity");
}

#[test]
fn language_package_can_be_built_from_embedded_full_sources() {
    let repo = repository();
    let alphabet = fs::read_to_string(repo.join("language/alphabet.toml")).unwrap();
    let phonology = fs::read_to_string(repo.join("language/phonology.toml")).unwrap();
    let morphology = fs::read_to_string(repo.join("language/morphology.toml")).unwrap();
    let syntax = fs::read_to_string(repo.join("language/syntax.toml")).unwrap();
    let dictionary = fs::read_to_string(repo.join("language/dictionary.toml")).unwrap();
    let literals = fs::read_to_string(repo.join("language/literals.toml")).unwrap();
    let units = fs::read_to_string(repo.join("language/units.toml")).unwrap();
    let semantics = fs::read_to_string(repo.join("language/semantics/core.semsys")).unwrap();
    let pragmatics = fs::read_to_string(repo.join("language/semantics/pragmatics.semsys")).unwrap();
    let subjective = fs::read_to_string(repo.join("language/semantics/subjective.semsys")).unwrap();
    let language = LanguagePackage::from_sources_full(
        &alphabet,
        &phonology,
        &morphology,
        &syntax,
        &dictionary,
        &literals,
        &units,
        &[
            ("language/semantics/core.semsys", &semantics),
            ("language/semantics/pragmatics.semsys", &pragmatics),
            ("language/semantics/subjective.semsys", &subjective),
        ],
    )
    .unwrap();

    assert!(language.literals().is_some());
    assert_eq!(
        language.literals().unwrap().parse_complete("mega uno pent").unwrap().canonical_written,
        "1000005"
    );
}

#[test]
fn dictionary_constant_semantics_are_installed_into_the_environment() {
    let language = LanguagePackage::load(repository().join("language")).unwrap();
    let sol = language
        .dictionary()
        .entries()
        .iter()
        .find(|entry| entry.root == "sol")
        .unwrap();
    assert_eq!(
        sol.semantic,
        LexicalSemantic::Constant {
            name: None,
            ty: "Entity".into(),
        }
    );
    assert_eq!(language.semantics().constant_type("sol"), Some(&Type::named("Entity")));
}

#[test]
fn dictionary_supports_runtime_reference_and_context_lexemes() {
    let language = LanguagePackage::load(repository().join("language")).unwrap();
    let reference = language
        .dictionary()
        .entries()
        .iter()
        .find(|entry| entry.root == "ref")
        .unwrap();
    assert_eq!(reference.semantic, LexicalSemantic::Reference);

    let speaker = language
        .dictionary()
        .entries()
        .iter()
        .find(|entry| entry.root == "mi")
        .unwrap();
    assert_eq!(
        speaker.semantic,
        LexicalSemantic::Context {
            key: "speaker".into(),
            ty: "Entity".into(),
        }
    );
}

#[test]
fn language_package_rejects_phonologically_invalid_dictionary_roots() {
    let dictionary = r#"
[str]
definition = "invalid root without a vowel"
semantic = { kind = "constant", type = "Entity" }
"#;

    let error = package_from_dictionary(dictionary).unwrap_err();
    assert!(error.to_string().contains("root `str` is invalid"));
}

#[test]
fn language_package_reserves_scope_markers_from_lexical_roots() {
    let dictionary = r#"
[ki]
definition = "must collide with the reserved scope opener"
semantic = { kind = "constant", type = "Entity" }
"#;

    let error = package_from_dictionary(dictionary).unwrap_err();
    assert!(error.to_string().contains("scope marker `ki` collides"));
}

#[test]
fn language_package_reserves_discourse_markers_from_lexical_roots() {
    let dictionary = r#"
[fra]
definition = "must collide with the reserved discourse frame marker"
semantic = { kind = "constant", type = "Entity" }
"#;

    let error = package_from_dictionary(dictionary).unwrap_err();
    assert!(error.to_string().contains("discourse frame marker `fra` collides"));
}

#[test]
fn language_package_reserves_quotation_markers_from_lexical_roots() {
    let dictionary = r#"
[sit]
definition = "must collide with the reserved quotation opener"
semantic = { kind = "constant", type = "Entity" }
"#;

    let error = package_from_dictionary(dictionary).unwrap_err();
    assert!(error
        .to_string()
        .contains("quotation open marker `sit` collides"));
}

#[test]
fn dictionary_operator_root_requires_surface_realization() {
    let error = Dictionary::from_toml(
        r#"
[ne]
definition = "negation"
semantic = { kind = "operator", name = "not" }
"#,
    )
    .unwrap_err();

    assert!(error.to_string().contains("requires one explicit `syntax` realization"));
}

#[test]
fn dictionary_constant_root_rejects_redundant_surface_atom_configuration() {
    let error = Dictionary::from_toml(
        r#"
[sol]
definition = "star"
semantic = { kind = "constant", type = "Entity" }
syntax = { kind = "prefix", role = "value" }
"#,
    )
    .unwrap_err();

    assert!(error.to_string().contains("automatically a surface atom"));
}

#[test]
fn dictionary_rejects_unknown_semantic_types_at_package_compile_time() {
    let dictionary = r#"
[sol]
definition = "star"
semantic = { kind = "constant", type = "DefinitelyNotAType" }
"#;

    let error = package_from_dictionary(dictionary).unwrap_err();
    assert!(error.to_string().contains("unknown type `DefinitelyNotAType`"), "{error}");
}
