use std::fmt;

use crate::syntax::SyntaxConfig;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextRealization {
    Spoken,
    Written,
}

impl fmt::Display for TextRealization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spoken => write!(f, "spoken"),
            Self::Written => write!(f, "written"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextTurn {
    pub key: String,
    pub realization: TextRealization,
    pub source: String,
}

impl TextTurn {
    pub fn spoken(key: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            realization: TextRealization::Spoken,
            source: source.into(),
        }
    }

    pub fn written(key: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            realization: TextRealization::Written,
            source: source.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextDocument {
    pub turns: Vec<TextTurn>,
}

impl TextDocument {
    pub fn new(turns: Vec<TextTurn>) -> Self {
        Self { turns }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SectionId(u64);

impl SectionId {
    pub const ROOT: Self = Self(0);

    pub fn get(self) -> u64 {
        self.0
    }

    fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl fmt::Display for SectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "b{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextSessionState {
    current_section: SectionId,
}

impl Default for TextSessionState {
    fn default() -> Self {
        Self {
            current_section: SectionId::ROOT,
        }
    }
}

impl TextSessionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current_section(&self) -> SectionId {
        self.current_section
    }

    pub fn advance_section(&mut self) -> SectionId {
        self.current_section = self.current_section.next();
        self.current_section
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextStreamItem {
    Utterance(String),
    FrameBoundary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextStructureError {
    EmptyUtteranceBoundary {
        realization: TextRealization,
    },
    MissingUtteranceBoundary {
        realization: TextRealization,
        tail: String,
    },
    FrameBoundaryInsideUtterance {
        marker: String,
    },
    UnexpectedQuotationClose {
        marker: String,
    },
    MissingQuotationClose {
        marker: String,
    },
}

pub fn parse_text_turn(
    source: &str,
    realization: TextRealization,
    config: &SyntaxConfig,
) -> Result<Vec<TextStreamItem>, TextStructureError> {
    match realization {
        TextRealization::Spoken => parse_spoken_turn(source, config),
        TextRealization::Written => parse_written_turn(source, config),
    }
}

fn parse_spoken_turn(
    source: &str,
    config: &SyntaxConfig,
) -> Result<Vec<TextStreamItem>, TextStructureError> {
    let mut items = Vec::new();
    let mut current = Vec::new();
    let mut quote_depth = 0usize;

    for token in source.split_whitespace() {
        if quote_depth > 0 {
            current.push(token.to_owned());
            if token == config.quotation.open.as_str() {
                quote_depth += 1;
            } else if token == config.quotation.close.as_str() {
                quote_depth -= 1;
            }
            continue;
        }

        if token == config.quotation.open.as_str() {
            quote_depth = 1;
            current.push(token.to_owned());
        } else if token == config.quotation.close.as_str() {
            return Err(TextStructureError::UnexpectedQuotationClose {
                marker: config.quotation.close.clone(),
            });
        } else if token == config.text.utterance_spoken.as_str() {
            finish_utterance(&mut items, &mut current, TextRealization::Spoken)?;
        } else if token == config.discourse.frame.as_str() {
            if !current.is_empty() {
                return Err(TextStructureError::FrameBoundaryInsideUtterance {
                    marker: config.discourse.frame.clone(),
                });
            }
            items.push(TextStreamItem::FrameBoundary);
        } else {
            current.push(token.to_owned());
        }
    }

    finish_turn(items, current, quote_depth, TextRealization::Spoken, config)
}

fn parse_written_turn(
    source: &str,
    config: &SyntaxConfig,
) -> Result<Vec<TextStreamItem>, TextStructureError> {
    let boundary = config
        .text
        .utterance_written
        .chars()
        .next()
        .expect("text config validation guarantees one written boundary character");
    let readability = config
        .text
        .readability_punctuation
        .iter()
        .filter_map(|form| form.chars().next())
        .collect::<Vec<_>>();

    let mut items = Vec::new();
    let mut current = Vec::new();
    let mut quote_depth = 0usize;

    for raw in source.split_whitespace() {
        if quote_depth > 0 {
            if raw == config.quotation.open.as_str() {
                quote_depth += 1;
                current.push(raw.to_owned());
                continue;
            }
            if raw == config.quotation.close.as_str() {
                quote_depth -= 1;
                current.push(raw.to_owned());
                continue;
            }
            if quote_depth == 1 {
                let close = config.quotation.close.as_str();
                if let Some(suffix) = raw.strip_prefix(close) {
                    if !suffix.is_empty()
                        && suffix
                            .chars()
                            .all(|character| character == boundary || readability.contains(&character))
                    {
                        current.push(close.to_owned());
                        quote_depth = 0;
                        process_written_suffix(
                            suffix,
                            boundary,
                            &mut items,
                            &mut current,
                        )?;
                        continue;
                    }
                }
            }
            current.push(raw.to_owned());
            continue;
        }

        for piece in split_written_boundaries(raw, boundary) {
            match piece {
                WrittenPiece::Boundary => {
                    finish_utterance(&mut items, &mut current, TextRealization::Written)?;
                }
                WrittenPiece::Text(text) => {
                    let normalized = text.trim_matches(|character| readability.contains(&character));
                    if normalized.is_empty() {
                        continue;
                    }
                    if normalized == config.quotation.open.as_str() {
                        quote_depth = 1;
                        current.push(normalized.to_owned());
                    } else if normalized == config.quotation.close.as_str() {
                        return Err(TextStructureError::UnexpectedQuotationClose {
                            marker: config.quotation.close.clone(),
                        });
                    } else if normalized == config.discourse.frame.as_str() {
                        if !current.is_empty() {
                            return Err(TextStructureError::FrameBoundaryInsideUtterance {
                                marker: config.discourse.frame.clone(),
                            });
                        }
                        items.push(TextStreamItem::FrameBoundary);
                    } else {
                        current.push(normalized.to_owned());
                    }
                }
            }
        }
    }

    finish_turn(items, current, quote_depth, TextRealization::Written, config)
}

fn process_written_suffix(
    suffix: &str,
    boundary: char,
    items: &mut Vec<TextStreamItem>,
    current: &mut Vec<String>,
) -> Result<(), TextStructureError> {
    for character in suffix.chars() {
        if character == boundary {
            finish_utterance(items, current, TextRealization::Written)?;
        }
    }
    Ok(())
}

fn finish_turn(
    items: Vec<TextStreamItem>,
    current: Vec<String>,
    quote_depth: usize,
    realization: TextRealization,
    config: &SyntaxConfig,
) -> Result<Vec<TextStreamItem>, TextStructureError> {
    if quote_depth > 0 {
        return Err(TextStructureError::MissingQuotationClose {
            marker: config.quotation.close.clone(),
        });
    }
    if !current.is_empty() {
        return Err(TextStructureError::MissingUtteranceBoundary {
            realization,
            tail: current.join(" "),
        });
    }
    Ok(items)
}

fn finish_utterance(
    items: &mut Vec<TextStreamItem>,
    current: &mut Vec<String>,
    realization: TextRealization,
) -> Result<(), TextStructureError> {
    if current.is_empty() {
        return Err(TextStructureError::EmptyUtteranceBoundary { realization });
    }
    items.push(TextStreamItem::Utterance(current.join(" ")));
    current.clear();
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum WrittenPiece {
    Text(String),
    Boundary,
}

fn split_written_boundaries(source: &str, boundary: char) -> Vec<WrittenPiece> {
    let characters = source.chars().collect::<Vec<_>>();
    let mut pieces = Vec::new();
    let mut current = String::new();

    for (index, character) in characters.iter().copied().enumerate() {
        let decimal_point = character == boundary
            && boundary == '.'
            && index > 0
            && index + 1 < characters.len()
            && characters[index - 1].is_ascii_digit()
            && characters[index + 1].is_ascii_digit();
        if character == boundary && !decimal_point {
            if !current.is_empty() {
                pieces.push(WrittenPiece::Text(std::mem::take(&mut current)));
            }
            pieces.push(WrittenPiece::Boundary);
        } else {
            current.push(character);
        }
    }

    if !current.is_empty() {
        pieces.push(WrittenPiece::Text(current));
    }
    pieces
}

impl fmt::Display for TextStructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyUtteranceBoundary { realization } => write!(
                f,
                "{realization} utterance boundary occurs without an utterance"
            ),
            Self::MissingUtteranceBoundary { realization, tail } => write!(
                f,
                "{realization} turn ends before an explicit utterance boundary; unterminated tail: `{tail}`"
            ),
            Self::FrameBoundaryInsideUtterance { marker } => write!(
                f,
                "discourse-frame boundary `{marker}` occurs before the current utterance is terminated"
            ),
            Self::UnexpectedQuotationClose { marker } => {
                write!(f, "unexpected quotation close marker `{marker}` in text stream")
            }
            Self::MissingQuotationClose { marker } => {
                write!(f, "text stream is missing quotation close marker `{marker}`")
            }
        }
    }
}

impl std::error::Error for TextStructureError {}
