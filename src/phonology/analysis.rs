use super::config::{Alphabet, Letter, LetterKind, PhonologyConfig, StressStrategy};
use std::fmt;
use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grapheme {
    pub symbol: String,
    pub pronunciation: String,
    pub kind: LetterKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Syllable {
    pub grapheme_range: Range<usize>,
    pub spelling: String,
    pub pronunciation: String,
    pub stressed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WordAnalysis {
    pub canonical_spelling: String,
    pub pronunciation: String,
    pub stressed_pronunciation: String,
    pub graphemes: Vec<Grapheme>,
    pub syllables: Vec<Syllable>,
    pub root_grapheme_range: Range<usize>,
    pub stressed_syllable: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PhonologyError {
    UnsupportedGrapheme { offset: usize, remainder: String },
    UnsupportedPronunciation { offset: usize, remainder: String },
    EmptyWord,
    WhitespaceInsideWord,
    NoVowel,
    InvalidRootRange { start: usize, end: usize, len: usize },
    RootHasNoVowel,
    RootTextNotFound(String),
    RootTextOccursMultipleTimes(String),
}

impl fmt::Display for PhonologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedGrapheme { offset, remainder } => write!(
                f,
                "unsupported grapheme at byte {offset}: `{remainder}`"
            ),
            Self::UnsupportedPronunciation { offset, remainder } => write!(
                f,
                "unsupported pronunciation token at byte {offset}: `{remainder}`"
            ),
            Self::EmptyWord => f.write_str("word must not be empty"),
            Self::WhitespaceInsideWord => f.write_str("word analysis does not accept whitespace"),
            Self::NoVowel => f.write_str("word has no configured vowel nucleus"),
            Self::InvalidRootRange { start, end, len } => write!(
                f,
                "invalid root grapheme range {start}..{end} for word of {len} graphemes"
            ),
            Self::RootHasNoVowel => f.write_str("root span has no configured vowel nucleus"),
            Self::RootTextNotFound(root) => write!(f, "root `{root}` was not found in the word"),
            Self::RootTextOccursMultipleTimes(root) => {
                write!(f, "root `{root}` occurs more than once in the word")
            }
        }
    }
}

impl std::error::Error for PhonologyError {}

impl Alphabet {
    pub fn tokenize_spelling(&self, input: &str) -> Result<Vec<Grapheme>, PhonologyError> {
        if input.chars().any(char::is_whitespace) {
            return Err(PhonologyError::WhitespaceInsideWord);
        }
        let canonical = input.to_lowercase();
        tokenize_by(&canonical, self.symbols(), true)
    }

    pub fn pronounce(&self, input: &str) -> Result<String, PhonologyError> {
        transform_text(input, |word| {
            let graphemes = self.tokenize_spelling(word)?;
            Ok(graphemes
                .into_iter()
                .map(|grapheme| grapheme.pronunciation)
                .collect())
        })
    }

    pub fn spell(&self, pronunciation: &str) -> Result<String, PhonologyError> {
        transform_text(pronunciation, |word| {
            let graphemes = tokenize_by(word, self.pronunciations(), false)?;
            Ok(graphemes.into_iter().map(|grapheme| grapheme.symbol).collect())
        })
    }
}

impl PhonologyConfig {
    pub fn analyze_root(&self, root: &str) -> Result<WordAnalysis, PhonologyError> {
        let graphemes = self.alphabet.tokenize_spelling(root)?;
        let len = graphemes.len();
        self.analyze_graphemes(graphemes, 0..len)
    }

    pub fn analyze_word_with_root_text(
        &self,
        word: &str,
        root: &str,
    ) -> Result<WordAnalysis, PhonologyError> {
        let word_graphemes = self.alphabet.tokenize_spelling(word)?;
        let root_graphemes = self.alphabet.tokenize_spelling(root)?;
        let needle = root_graphemes
            .iter()
            .map(|grapheme| grapheme.symbol.as_str())
            .collect::<Vec<_>>();
        if needle.is_empty() {
            return Err(PhonologyError::EmptyWord);
        }
        let mut matches = Vec::new();
        for start in 0..=word_graphemes.len().saturating_sub(needle.len()) {
            let matches_here = needle.iter().enumerate().all(|(offset, expected)| {
                word_graphemes[start + offset].symbol == *expected
            });
            if matches_here {
                matches.push(start);
            }
        }
        match matches.as_slice() {
            [] => Err(PhonologyError::RootTextNotFound(root.to_owned())),
            [start] => self.analyze_graphemes(
                word_graphemes,
                *start..(*start + needle.len()),
            ),
            _ => Err(PhonologyError::RootTextOccursMultipleTimes(root.to_owned())),
        }
    }

    pub fn analyze_graphemes(
        &self,
        graphemes: Vec<Grapheme>,
        root_grapheme_range: Range<usize>,
    ) -> Result<WordAnalysis, PhonologyError> {
        if graphemes.is_empty() {
            return Err(PhonologyError::EmptyWord);
        }
        if root_grapheme_range.start >= root_grapheme_range.end
            || root_grapheme_range.end > graphemes.len()
        {
            return Err(PhonologyError::InvalidRootRange {
                start: root_grapheme_range.start,
                end: root_grapheme_range.end,
                len: graphemes.len(),
            });
        }

        let nuclei = graphemes
            .iter()
            .enumerate()
            .filter_map(|(index, grapheme)| (grapheme.kind == LetterKind::Vowel).then_some(index))
            .collect::<Vec<_>>();
        if nuclei.is_empty() {
            return Err(PhonologyError::NoVowel);
        }
        let root_first_nucleus = nuclei
            .iter()
            .copied()
            .find(|index| root_grapheme_range.contains(index))
            .ok_or(PhonologyError::RootHasNoVowel)?;

        let mut boundaries = vec![0usize];
        for pair in nuclei.windows(2) {
            let left = pair[0];
            let right = pair[1];
            let consonants_between = right - left - 1;
            let onset = self
                .syllables
                .intervocalic_onset_consonants
                .min(consonants_between);
            boundaries.push(right - onset);
        }
        boundaries.push(graphemes.len());

        let stressed_syllable = boundaries
            .windows(2)
            .position(|range| range[0] <= root_first_nucleus && root_first_nucleus < range[1])
            .expect("root nucleus must belong to one generated syllable");

        match self.stress.strategy {
            StressStrategy::FirstRootSyllable => {}
        }

        let syllables = boundaries
            .windows(2)
            .enumerate()
            .map(|(index, range)| {
                let slice = &graphemes[range[0]..range[1]];
                Syllable {
                    grapheme_range: range[0]..range[1],
                    spelling: slice.iter().map(|item| item.symbol.as_str()).collect(),
                    pronunciation: slice
                        .iter()
                        .map(|item| item.pronunciation.as_str())
                        .collect(),
                    stressed: index == stressed_syllable,
                }
            })
            .collect::<Vec<_>>();

        let canonical_spelling = graphemes
            .iter()
            .map(|grapheme| grapheme.symbol.as_str())
            .collect::<String>();
        let pronunciation = graphemes
            .iter()
            .map(|grapheme| grapheme.pronunciation.as_str())
            .collect::<String>();
        let stressed_pronunciation = syllables
            .iter()
            .map(|syllable| {
                if syllable.stressed {
                    format!("ˈ{}", syllable.pronunciation)
                } else {
                    syllable.pronunciation.clone()
                }
            })
            .collect::<String>();

        Ok(WordAnalysis {
            canonical_spelling,
            pronunciation,
            stressed_pronunciation,
            graphemes,
            syllables,
            root_grapheme_range,
            stressed_syllable,
        })
    }
}

fn transform_text(
    input: &str,
    mut transform_word: impl FnMut(&str) -> Result<String, PhonologyError>,
) -> Result<String, PhonologyError> {
    let mut output = String::new();
    let mut word_start = None;
    for (index, character) in input.char_indices() {
        if character.is_whitespace() {
            if let Some(start) = word_start.take() {
                output.push_str(&transform_word(&input[start..index])?);
            }
            output.push(character);
        } else if word_start.is_none() {
            word_start = Some(index);
        }
    }
    if let Some(start) = word_start {
        output.push_str(&transform_word(&input[start..])?);
    }
    Ok(output)
}

fn tokenize_by<'a>(
    input: &str,
    vocabulary: impl Iterator<Item = (&'a str, &'a Letter)>,
    spelling: bool,
) -> Result<Vec<Grapheme>, PhonologyError> {
    let vocabulary = vocabulary.collect::<Vec<_>>();
    let mut output = Vec::new();
    let mut offset = 0usize;
    while offset < input.len() {
        let remainder = &input[offset..];
        let Some((token, letter)) = vocabulary
            .iter()
            .find(|(token, _)| remainder.starts_with(*token))
        else {
            return Err(if spelling {
                PhonologyError::UnsupportedGrapheme {
                    offset,
                    remainder: remainder.to_owned(),
                }
            } else {
                PhonologyError::UnsupportedPronunciation {
                    offset,
                    remainder: remainder.to_owned(),
                }
            });
        };
        output.push(Grapheme {
            symbol: letter.symbol.to_lowercase(),
            pronunciation: letter.pronunciation.clone(),
            kind: letter.kind,
        });
        offset += token.len();
    }
    Ok(output)
}
