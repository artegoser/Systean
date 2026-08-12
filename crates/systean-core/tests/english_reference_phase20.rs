use std::path::{Path, PathBuf};

use systean_core::discourse::DiscourseState;
use systean_core::documentation::DocumentationDeclarationKind;
use systean_core::language::LanguagePackage;
use systean_core::semantics::{Literal, Term, canonicalize};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn language() -> LanguagePackage {
    LanguagePackage::load(repository().join("language")).unwrap()
}

#[test]
fn every_public_word_has_compiled_english_documentation_and_an_executable_example() {
    let language = language();
    let documentation = language.documentation();

    assert_eq!(documentation.len(), language.dictionary().entries().len());
    assert_eq!(documentation.len(), 175);
    for entry in documentation.entries() {
        assert!(!entry.gloss.trim().is_empty(), "{} has no gloss", entry.root);
        assert!(!entry.explain.trim().is_empty(), "{} has no explanation", entry.root);
        assert!(!entry.signature.trim().is_empty(), "{} has no signature", entry.root);
        assert!(!entry.pronunciation.trim().is_empty(), "{} has no pronunciation", entry.root);
        assert!(!entry.stressed_pronunciation.trim().is_empty(), "{} has no stressed pronunciation", entry.root);
        assert!(!entry.examples.is_empty(), "{} has no examples", entry.root);
        for example in &entry.examples {
            assert_eq!(example.source, example.canonical_surface, "{} example is not canonical", entry.root);
            assert!(!example.canonical_semantics.is_empty(), "{} example has no checked semantics", entry.root);
            assert!(!example.english.is_empty(), "{} example has no English rendering", entry.root);
        }
    }

    assert_eq!(
        documentation.entry("vid").unwrap().declaration_kind,
        DocumentationDeclarationKind::Primitive
    );
    assert_eq!(
        documentation.entry("ref").unwrap().declaration_kind,
        DocumentationDeclarationKind::IntrinsicBacked
    );
    assert_eq!(
        documentation.entry("ne").unwrap().declaration_kind,
        DocumentationDeclarationKind::Defined
    );
}

#[test]
fn documentation_search_index_is_compiled_from_semantic_and_documentation_metadata() {
    let language = language();
    let index = language.documentation().search_index();
    assert_eq!(index.len(), 175);

    let see = index.iter().find(|entry| entry.root == "vid").unwrap();
    assert_eq!(see.gloss, "see");
    assert_eq!(see.result_type, "Proposition");
    assert_eq!(see.argument_count, 2);
    assert_eq!(see.argument_types, ["Entity", "Entity"]);
    assert!(see.examples.iter().any(|example| example == "sol vid sol"));
    assert_eq!(see.source, "docs/en.sydoc");
    assert!(see.source_line > 0);
}

#[test]
fn documentation_is_fingerprinted_independently_from_normative_semantics_and_surface() {
    let language = language();
    let source = std::fs::read_to_string(repository().join("language/docs/en.sydoc")).unwrap();
    let edited = source.replacen("gloss \"Sun\"", "gloss \"the Sun\"", 1);
    assert_ne!(source, edited);

    let modified = language
        .clone()
        .with_documentation_sources(&[("docs/en.sydoc", edited.as_str())])
        .unwrap();

    assert_ne!(language.documentation_fingerprint(), modified.documentation_fingerprint());
    assert_eq!(language.semantic_fingerprint(), modified.semantic_fingerprint());
    assert_eq!(language.surface_fingerprint(), modified.surface_fingerprint());
    assert_eq!(language.package_fingerprint(), modified.package_fingerprint());
}

#[test]
fn documentation_compiler_rejects_incomplete_public_coverage() {
    let language = language();
    let source = std::fs::read_to_string(repository().join("language/docs/en.sydoc")).unwrap();
    let start = source.find("word sol {").unwrap();
    let end = source.find("word ref {").unwrap();
    let missing_sol = format!("{}{}", &source[..start], &source[end..]);

    let error = language
        .with_documentation_sources(&[("docs/en.sydoc", missing_sol.as_str())])
        .unwrap_err()
        .to_string();
    assert!(error.contains("coverage mismatch"), "{error}");
    assert!(error.contains("sol"), "{error}");
}

#[test]
fn documentation_compiler_rejects_example_semantic_drift() {
    let language = language();
    let source = std::fs::read_to_string(repository().join("language/docs/en.sydoc")).unwrap();
    let edited = source.replacen(
        "expect_semantics \"\"\"\n    sol\n  \"\"\";",
        "expect_semantics \"\"\"\n    na(payload = \"wrong\")\n  \"\"\";",
        1,
    );
    assert_ne!(source, edited);

    let error = language
        .with_documentation_sources(&[("docs/en.sydoc", edited.as_str())])
        .unwrap_err()
        .to_string();
    assert!(error.contains("changed canonical semantics"), "{error}");
    assert!(error.contains("expected `na(payload = \"wrong\")`"), "{error}");
}

#[test]
fn controlled_english_has_stable_goldens_across_major_semantic_subsystems() {
    let language = language();
    let cases = [
        (
            "na artemi vid na mari",
            "(proper name (\"artemi\")) see (proper name (\"mari\"))",
        ),
        ("ne sol viv", "not (alive [entity: Sun])"),
        (
            "sol viv va sol mor",
            "(alive [entity: Sun]) and (die [entity: Sun])",
        ),
        (
            "sol viv zo sol mor",
            "(alive [entity: Sun]) or (die [entity: Sun])",
        ),
        (
            "ra per viv",
            "every value v0 such that (person [entity: v0]) => (alive [entity: v0])",
        ),
        (
            "mu per viv",
            "some value v0 such that (person [entity: v0]) => (alive [entity: v0])",
        ),
        (
            "mini tri per viv",
            "at least 3 value v0 such that (person [entity: v0]) => (alive [entity: v0])",
        ),
        (
            "sol vid vak",
            "(Sun) see (unspecified information)",
        ),
        (
            "sol vid hid",
            "(Sun) see (withheld information)",
        ),
        (
            "sol vid unk na artemi",
            "(Sun) see (unknown information [knower: proper name (\"artemi\")])",
        ),
        (
            "2026-08-11 ante 2026-08-12",
            "(2026-08-11) before (2026-08-12)",
        ),
        ("ke sol viv", "question: (alive [entity: Sun])"),
        ("sol fok sol viv", "focus [target: Sun]: (alive [entity: Sun])"),
        ("uno kor sol viv", "correct [target: 1]: (alive [entity: Sun])"),
    ];

    for (surface, expected) in cases {
        let first = language.render_english(surface).unwrap();
        let second = language.render_english(surface).unwrap();
        assert_eq!(first, second, "renderer is not deterministic for `{surface}`");
        assert_eq!(first.text, expected, "unexpected English for `{surface}`");
    }
}

#[test]
fn controlled_english_keeps_counted_typical_statistical_probability_and_frequency_claims_explicit() {
    let language = language();
    let cases = [
        ("rov tri per viv", "exactly 3 value v0 such that (person [entity: v0]) => (alive [entity: v0])"),
        (
            "na alfa tip na beta 0.8",
            "typical [domain: proper name (\"alfa\"); measure: proper name (\"beta\"); standard: 4/5]",
        ),
        (
            "na alfa stat na beta 0.8",
            "statistical [domain: proper name (\"alfa\"); measure: proper name (\"beta\"); value: 4/5]",
        ),
        ("vak prob 0.7", "(unspecified information) probability (7/10)"),
        (
            "vak frek na kal 3",
            "frequency [activity: unspecified information; measure: proper name (\"kal\"); value: 3]",
        ),
    ];

    for (surface, expected) in cases {
        assert_eq!(language.render_english(surface).unwrap().text, expected, "{surface}");
    }
}

#[test]
fn structured_quantity_rendering_comes_from_typed_values() {
    let language = language();
    let literal = language.literals().unwrap().parse_complete("1±0.1 km").unwrap();
    let term = Term::Literal(Literal::Structured(literal.semantic));
    let rendering = language.render_english_term(&term).unwrap();
    assert_eq!(rendering.text, "approximately 1 kilometer ± 1/10");
}

#[test]
fn resolved_context_is_used_by_controlled_english_without_reading_host_state() {
    let language = language();
    let mut discourse = DiscourseState::new();
    discourse
        .set_context_value("speaker", Term::Const("sol".into()), language.semantics())
        .unwrap();

    let rendering = language
        .render_english_with_discourse("mi viv", &discourse)
        .unwrap();
    assert_eq!(rendering.text, "alive [entity: Sun]");
    assert!(language.render_english("mi viv").is_err());
}

#[test]
fn renderer_preserves_semantic_terms_and_emits_valid_semantic_path_alignment() {
    let language = language();
    let analysis = language.analyze_surface("na artemi vid na mari").unwrap();
    let lowered = language
        .syntax()
        .lower(&analysis.syntax, language.semantics())
        .unwrap();
    let before = canonicalize(&lowered.term);
    let rendering = language.render_english_term(&lowered.term).unwrap();
    let after = canonicalize(&lowered.term);

    assert_eq!(before, after);
    assert!(!rendering.alignments.is_empty());
    assert!(rendering.alignments.iter().any(|alignment| alignment.semantic_path == "$"));
    for alignment in &rendering.alignments {
        assert!(alignment.start <= alignment.end);
        assert!(alignment.end <= rendering.text.len());
        assert!(rendering.text.is_char_boundary(alignment.start));
        assert!(rendering.text.is_char_boundary(alignment.end));
    }
}

#[test]
fn controlled_english_keeps_systean_scope_visibly_distinct() {
    let language = language();
    let not_every = language.render_english("ne ra per viv").unwrap().text;
    let every_not = language.render_english("ra per ne viv").unwrap().text;

    assert_eq!(
        not_every,
        "not (every value v0 such that (person [entity: v0]) => (alive [entity: v0]))"
    );
    assert_eq!(
        every_not,
        "every value v0 such that (person [entity: v0]) => (not (alive [entity: v0]))"
    );
    assert_ne!(not_every, every_not);
}
