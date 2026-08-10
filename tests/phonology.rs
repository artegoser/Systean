use std::path::PathBuf;
use systean::phonology::{
    Alphabet, ConfigError, Letter, LetterKind, PhonologyConfig, PhonologyError, RootInventory,
    RootIssue, RootWarning, Segmentation, SpokenForm, segment_spoken_stream,
};

fn repo_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn phonology() -> PhonologyConfig {
    PhonologyConfig::load(
        repo_path("lib/config/alphabet.toml"),
        repo_path("lib/config/phonology.toml"),
    )
    .unwrap()
}

#[test]
fn canonical_alphabet_matches_existing_pronunciation_contract() {
    let phonology = phonology();
    assert_eq!(phonology.alphabet.letters().len(), 22);
    assert_eq!(phonology.alphabet.pronounce("Systean").unwrap(), "sjstean");
    assert_eq!(phonology.alphabet.spell("sjstean").unwrap(), "systean");
    assert_eq!(phonology.alphabet.pronounce("hjy").unwrap(), "xʐj");
    assert_eq!(phonology.alphabet.spell("xʐj").unwrap(), "hjy");
}

#[test]
fn every_two_letter_written_form_round_trips_through_pronunciation() {
    let phonology = phonology();
    for left in phonology.alphabet.letters() {
        for right in phonology.alphabet.letters() {
            let spelling = format!("{}{}", left.symbol, right.symbol);
            let pronunciation = phonology.alphabet.pronounce(&spelling).unwrap();
            assert_eq!(phonology.alphabet.spell(&pronunciation).unwrap(), spelling);
        }
    }
}

#[test]
fn unsupported_spelling_is_rejected_instead_of_guessed() {
    let phonology = phonology();
    assert!(matches!(
        phonology.alphabet.pronounce("sol!"),
        Err(PhonologyError::UnsupportedGrapheme { .. })
    ));
}

#[test]
fn alphabet_rejects_prefix_ambiguous_graphemes() {
    let error = Alphabet::new(vec![
        Letter {
            symbol: "a".into(),
            pronunciation: "x".into(),
            kind: LetterKind::Vowel,
        },
        Letter {
            symbol: "ab".into(),
            pronunciation: "y".into(),
            kind: LetterKind::Consonant,
        },
    ])
    .unwrap_err();
    assert!(matches!(error, ConfigError::SymbolPrefixCollision { .. }));
}

#[test]
fn alphabet_rejects_prefix_ambiguous_reverse_transcription() {
    let error = Alphabet::new(vec![
        Letter {
            symbol: "a".into(),
            pronunciation: "x".into(),
            kind: LetterKind::Vowel,
        },
        Letter {
            symbol: "b".into(),
            pronunciation: "xy".into(),
            kind: LetterKind::Consonant,
        },
    ])
    .unwrap_err();
    assert!(matches!(
        error,
        ConfigError::PronunciationPrefixCollision { .. }
    ));
}

#[test]
fn syllabification_is_deterministic_and_stress_is_on_first_root_syllable() {
    let phonology = phonology();
    let analysis = phonology.analyze_root("artemi").unwrap();
    assert_eq!(
        analysis
            .syllables
            .iter()
            .map(|syllable| syllable.spelling.as_str())
            .collect::<Vec<_>>(),
        vec!["ar", "te", "mi"]
    );
    assert_eq!(analysis.stressed_syllable, 0);
    assert_eq!(analysis.stressed_pronunciation, "ˈartemi");
}

#[test]
fn prefixes_do_not_move_stress_away_from_first_root_syllable() {
    let phonology = phonology();
    let analysis = phonology.analyze_word_with_root_text("nasol", "sol").unwrap();
    assert_eq!(
        analysis
            .syllables
            .iter()
            .map(|syllable| syllable.spelling.as_str())
            .collect::<Vec<_>>(),
        vec!["na", "sol"]
    );
    assert_eq!(analysis.stressed_syllable, 1);
    assert_eq!(analysis.stressed_pronunciation, "naˈsol");
}

#[test]
fn root_span_must_contain_a_vowel_for_lexical_stress() {
    let phonology = phonology();
    let graphemes = phonology.alphabet.tokenize_spelling("nsol").unwrap();
    assert_eq!(
        phonology.analyze_graphemes(graphemes, 0..1),
        Err(PhonologyError::RootHasNoVowel)
    );
}

#[test]
fn manual_root_checker_rejects_collisions_and_only_warns_on_similarity() {
    let phonology = phonology();
    let inventory = RootInventory::load_dictionary(repo_path("lib/config/dictionary.toml")).unwrap();

    let existing = phonology.check_root("sol", &inventory);
    assert!(existing
        .issues
        .contains(&RootIssue::ExistingSpelling("sol".into())));

    let similar = phonology.check_root("sal", &inventory);
    assert!(similar.is_valid());
    assert!(similar.warnings.contains(&RootWarning::SimilarSpelling {
        root: "sol".into(),
        distance: 1,
    }));

    let vowelless = phonology.check_root("str", &inventory);
    assert!(!vowelless.is_valid());
    assert!(vowelless
        .issues
        .contains(&RootIssue::Invalid(PhonologyError::NoVowel)));
}

#[test]
fn spoken_segmentation_reports_unique_ambiguous_and_impossible_streams() {
    let unique_forms = vec![
        SpokenForm {
            label: "mi".into(),
            pronunciation: "mi".into(),
        },
        SpokenForm {
            label: "sol".into(),
            pronunciation: "sol".into(),
        },
    ];
    assert_eq!(
        segment_spoken_stream("misol", &unique_forms),
        Segmentation::Unique(vec!["mi".into(), "sol".into()])
    );

    let mut ambiguous_forms = unique_forms.clone();
    ambiguous_forms.push(SpokenForm {
        label: "misol".into(),
        pronunciation: "misol".into(),
    });
    assert!(matches!(
        segment_spoken_stream("misol", &ambiguous_forms),
        Segmentation::Ambiguous { .. }
    ));
    assert_eq!(
        segment_spoken_stream("zzz", &unique_forms),
        Segmentation::Impossible
    );
}
