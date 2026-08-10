use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::phonology::{ConfigError, PhonologyConfig, RootInventory};
use crate::semantics::{Checker, Environment, Explainer, Type, canonicalize};
use crate::spec::{PackageError, compile_path, compile_sources, lower_term, parse_term};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DictionaryEntry {
    pub root: String,
    pub fields: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub struct Dictionary {
    entries: Vec<DictionaryEntry>,
}

impl Dictionary {
    pub fn from_toml(source: &str) -> Result<Self, LanguageError> {
        let value: toml::Value = toml::from_str(source)
            .map_err(|error| LanguageError::Dictionary(error.to_string()))?;
        let table = value
            .as_table()
            .ok_or_else(|| LanguageError::Dictionary("dictionary root must be a TOML table".into()))?;
        let mut entries = table
            .iter()
            .filter(|(key, _)| key.as_str() != "meta")
            .map(|(root, value)| {
                let fields = value
                    .as_table()
                    .ok_or_else(|| {
                        LanguageError::Dictionary(format!(
                            "dictionary entry `{root}` must be a TOML table"
                        ))
                    })?
                    .iter()
                    .filter_map(|(name, value)| {
                        value
                            .as_str()
                            .map(|text| (name.clone(), text.trim().to_owned()))
                    })
                    .collect();
                Ok(DictionaryEntry {
                    root: root.to_lowercase(),
                    fields,
                })
            })
            .collect::<Result<Vec<_>, LanguageError>>()?;
        entries.sort_by(|left, right| left.root.cmp(&right.root));
        Ok(Self { entries })
    }

    pub fn entries(&self) -> &[DictionaryEntry] {
        &self.entries
    }
}

#[derive(Clone, Debug)]
pub struct LanguagePackage {
    phonology: PhonologyConfig,
    dictionary: Dictionary,
    roots: RootInventory,
    semantics: Environment,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct SemanticAnalysis {
    pub inferred_type: String,
    pub canonical: String,
    pub explanation: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LanguageError {
    Io { path: PathBuf, message: String },
    Phonology(ConfigError),
    Dictionary(String),
    RootInventory(String),
    Semantics(Vec<PackageError>),
    SemanticExpression(String),
}

impl LanguagePackage {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LanguageError> {
        let path = path.as_ref();
        let alphabet = read(path.join("alphabet.toml"))?;
        let phonology = read(path.join("phonology.toml"))?;
        let dictionary = read(path.join("dictionary.toml"))?;
        let semantics = compile_path(path.join("semantics")).map_err(LanguageError::Semantics)?;
        Self::from_parts(&alphabet, &phonology, &dictionary, semantics)
    }

    pub fn from_sources(
        alphabet: &str,
        phonology: &str,
        dictionary: &str,
        semantic_sources: &[(&str, &str)],
    ) -> Result<Self, LanguageError> {
        let semantics = compile_sources(
            semantic_sources
                .iter()
                .map(|(name, source)| ((*name).to_owned(), (*source).to_owned())),
        )
        .map_err(LanguageError::Semantics)?;
        Self::from_parts(alphabet, phonology, dictionary, semantics)
    }

    fn from_parts(
        alphabet: &str,
        phonology: &str,
        dictionary: &str,
        semantics: Environment,
    ) -> Result<Self, LanguageError> {
        let phonology = PhonologyConfig::from_toml(alphabet, phonology)
            .map_err(LanguageError::Phonology)?;
        let dictionary_parsed = Dictionary::from_toml(dictionary)?;
        let roots = RootInventory::from_dictionary_toml(dictionary)
            .map_err(|error| LanguageError::Dictionary(error.to_string()))?;
        validate_roots(&phonology, &roots)?;
        Ok(Self {
            phonology,
            dictionary: dictionary_parsed,
            roots,
            semantics,
        })
    }

    pub fn phonology(&self) -> &PhonologyConfig {
        &self.phonology
    }

    pub fn dictionary(&self) -> &Dictionary {
        &self.dictionary
    }

    pub fn roots(&self) -> &RootInventory {
        &self.roots
    }

    pub fn semantics(&self) -> &Environment {
        &self.semantics
    }

    pub fn explain(&self, expression: &str) -> Result<SemanticAnalysis, LanguageError> {
        let parsed = parse_term(expression).map_err(|errors| {
            LanguageError::SemanticExpression(
                errors
                    .into_iter()
                    .map(|error| error.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        })?;
        let term = lower_term(parsed);
        let ty: Type = Checker::new(&self.semantics)
            .infer(&term)
            .map_err(|error| LanguageError::SemanticExpression(error.to_string()))?;
        let explanation = Explainer::new(&self.semantics)
            .explain(&term)
            .map_err(|error| LanguageError::SemanticExpression(error.to_string()))?
            .render();
        Ok(SemanticAnalysis {
            inferred_type: ty.to_string(),
            canonical: canonicalize(&term).to_string(),
            explanation,
        })
    }
}

fn validate_roots(
    phonology: &PhonologyConfig,
    roots: &RootInventory,
) -> Result<(), LanguageError> {
    let mut by_pronunciation = BTreeMap::<String, Vec<&str>>::new();
    for root in roots.roots() {
        let analysis = phonology.analyze_root(root).map_err(|error| {
            LanguageError::RootInventory(format!("root `{root}` is invalid: {error}"))
        })?;
        by_pronunciation
            .entry(analysis.pronunciation)
            .or_default()
            .push(root);
    }
    if let Some((pronunciation, roots)) = by_pronunciation
        .into_iter()
        .find(|(_, roots)| roots.len() > 1)
    {
        return Err(LanguageError::RootInventory(format!(
            "roots {} share pronunciation /{pronunciation}/",
            roots.join(", ")
        )));
    }
    Ok(())
}

fn read(path: PathBuf) -> Result<String, LanguageError> {
    fs::read_to_string(&path).map_err(|error| LanguageError::Io {
        path,
        message: error.to_string(),
    })
}

impl fmt::Display for LanguageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, message } => write!(f, "{}: {message}", path.display()),
            Self::Phonology(error) => write!(f, "phonology: {error}"),
            Self::Dictionary(error) => write!(f, "dictionary: {error}"),
            Self::RootInventory(error) => write!(f, "root inventory: {error}"),
            Self::Semantics(errors) => {
                for (index, error) in errors.iter().enumerate() {
                    if index > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "semantics: {error}")?;
                }
                Ok(())
            }
            Self::SemanticExpression(error) => write!(f, "semantic expression: {error}"),
        }
    }
}

impl std::error::Error for LanguageError {}
