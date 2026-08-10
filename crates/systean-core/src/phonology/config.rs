use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LetterKind {
    Vowel,
    Consonant,
}

impl fmt::Display for LetterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vowel => f.write_str("vowel"),
            Self::Consonant => f.write_str("consonant"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Letter {
    pub symbol: String,
    pub pronunciation: String,
    #[serde(rename = "_type")]
    pub kind: LetterKind,
}

#[derive(Clone, Debug, Deserialize)]
struct AlphabetFile {
    letters: Vec<Letter>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct SyllableRules {
    pub intervocalic_onset_consonants: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StressStrategy {
    FirstRootSyllable,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct StressRules {
    pub strategy: StressStrategy,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct RootRules {
    pub similarity_warning_distance: usize,
}

#[derive(Clone, Debug, Deserialize)]
struct RulesFile {
    syllables: SyllableRules,
    stress: StressRules,
    roots: RootRules,
}

#[derive(Clone, Debug)]
pub struct Alphabet {
    letters: Vec<Letter>,
    by_symbol: BTreeMap<String, usize>,
    by_pronunciation: BTreeMap<String, usize>,
}

#[derive(Clone, Debug)]
pub struct PhonologyConfig {
    pub alphabet: Alphabet,
    pub syllables: SyllableRules,
    pub stress: StressRules,
    pub roots: RootRules,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigError {
    Io(String),
    Toml(String),
    EmptyAlphabet,
    EmptySymbol,
    EmptyPronunciation(String),
    DuplicateSymbol(String),
    DuplicatePronunciation(String),
    SymbolPrefixCollision { shorter: String, longer: String },
    PronunciationPrefixCollision { shorter: String, longer: String },
    InvalidIntervocalicOnset,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Toml(error) => write!(f, "TOML error: {error}"),
            Self::EmptyAlphabet => f.write_str("alphabet must contain at least one letter"),
            Self::EmptySymbol => f.write_str("alphabet contains an empty grapheme symbol"),
            Self::EmptyPronunciation(symbol) => {
                write!(f, "alphabet symbol `{symbol}` has an empty pronunciation")
            }
            Self::DuplicateSymbol(symbol) => write!(f, "duplicate alphabet symbol `{symbol}`"),
            Self::DuplicatePronunciation(pronunciation) => {
                write!(f, "duplicate pronunciation `{pronunciation}`")
            }
            Self::SymbolPrefixCollision { shorter, longer } => write!(
                f,
                "grapheme `{shorter}` is a prefix of `{longer}`; written tokenization would be ambiguous"
            ),
            Self::PronunciationPrefixCollision { shorter, longer } => write!(
                f,
                "pronunciation `{shorter}` is a prefix of `{longer}`; reverse transcription would be ambiguous"
            ),
            Self::InvalidIntervocalicOnset => f.write_str(
                "syllables.intervocalic_onset_consonants must be at least 1",
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Alphabet {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let source = fs::read_to_string(path.as_ref()).map_err(|error| ConfigError::Io(error.to_string()))?;
        Self::from_toml(&source)
    }

    pub fn from_toml(source: &str) -> Result<Self, ConfigError> {
        let parsed: AlphabetFile = toml::from_str(source).map_err(|error| ConfigError::Toml(error.to_string()))?;
        Self::new(parsed.letters)
    }

    pub fn new(letters: Vec<Letter>) -> Result<Self, ConfigError> {
        if letters.is_empty() {
            return Err(ConfigError::EmptyAlphabet);
        }

        let mut by_symbol = BTreeMap::new();
        let mut by_pronunciation = BTreeMap::new();
        for (index, letter) in letters.iter().enumerate() {
            let symbol = canonical_symbol(&letter.symbol);
            if symbol.is_empty() {
                return Err(ConfigError::EmptySymbol);
            }
            if letter.pronunciation.is_empty() {
                return Err(ConfigError::EmptyPronunciation(letter.symbol.clone()));
            }
            if by_symbol.insert(symbol.clone(), index).is_some() {
                return Err(ConfigError::DuplicateSymbol(symbol));
            }
            if by_pronunciation
                .insert(letter.pronunciation.clone(), index)
                .is_some()
            {
                return Err(ConfigError::DuplicatePronunciation(
                    letter.pronunciation.clone(),
                ));
            }
        }

        validate_prefix_free(by_symbol.keys(), true)?;
        validate_prefix_free(by_pronunciation.keys(), false)?;

        Ok(Self {
            letters,
            by_symbol,
            by_pronunciation,
        })
    }

    pub fn letters(&self) -> &[Letter] {
        &self.letters
    }

    pub fn letter_by_symbol(&self, symbol: &str) -> Option<&Letter> {
        self.by_symbol
            .get(&canonical_symbol(symbol))
            .map(|index| &self.letters[*index])
    }

    pub fn letter_by_pronunciation(&self, pronunciation: &str) -> Option<&Letter> {
        self.by_pronunciation
            .get(pronunciation)
            .map(|index| &self.letters[*index])
    }

    pub(crate) fn symbols(&self) -> impl Iterator<Item = (&str, &Letter)> {
        self.by_symbol
            .iter()
            .map(|(symbol, index)| (symbol.as_str(), &self.letters[*index]))
    }

    pub(crate) fn pronunciations(&self) -> impl Iterator<Item = (&str, &Letter)> {
        self.by_pronunciation
            .iter()
            .map(|(pronunciation, index)| (pronunciation.as_str(), &self.letters[*index]))
    }
}

impl PhonologyConfig {
    pub fn load(
        alphabet_path: impl AsRef<Path>,
        rules_path: impl AsRef<Path>,
    ) -> Result<Self, ConfigError> {
        let alphabet_source = fs::read_to_string(alphabet_path.as_ref())
            .map_err(|error| ConfigError::Io(error.to_string()))?;
        let rules_source = fs::read_to_string(rules_path.as_ref())
            .map_err(|error| ConfigError::Io(error.to_string()))?;
        Self::from_toml(&alphabet_source, &rules_source)
    }

    pub fn from_toml(alphabet_source: &str, rules_source: &str) -> Result<Self, ConfigError> {
        let alphabet = Alphabet::from_toml(alphabet_source)?;
        let rules: RulesFile =
            toml::from_str(rules_source).map_err(|error| ConfigError::Toml(error.to_string()))?;
        if rules.syllables.intervocalic_onset_consonants == 0 {
            return Err(ConfigError::InvalidIntervocalicOnset);
        }
        Ok(Self {
            alphabet,
            syllables: rules.syllables,
            stress: rules.stress,
            roots: rules.roots,
        })
    }
}

pub fn canonical_symbol(symbol: &str) -> String {
    symbol.to_lowercase()
}

fn validate_prefix_free<'a>(values: impl Iterator<Item = &'a String>, symbols: bool) -> Result<(), ConfigError> {
    let values = values.cloned().collect::<BTreeSet<_>>();
    for shorter in &values {
        for longer in values.range(shorter.clone()..) {
            if shorter == longer {
                continue;
            }
            if longer.starts_with(shorter) {
                return Err(if symbols {
                    ConfigError::SymbolPrefixCollision {
                        shorter: shorter.clone(),
                        longer: longer.clone(),
                    }
                } else {
                    ConfigError::PronunciationPrefixCollision {
                        shorter: shorter.clone(),
                        longer: longer.clone(),
                    }
                });
            }
        }
    }
    Ok(())
}
