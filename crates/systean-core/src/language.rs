use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::morphology::{
    MorphologyAnalysis, MorphologyConfig, MorphologyConfigError, MorphologyEngine, MorphologyError,
};
use crate::phonology::{ConfigError, PhonologyConfig, RootInventory, WordAnalysis};
use crate::semantics::{Checker, Environment, Explainer, Type, canonicalize};
use crate::spec::{PackageError, compile_path, compile_sources, lower_term, parse_term};
use crate::syntax::{SurfaceAnalysis, SurfaceError, SyntaxConfig, SyntaxConfigError, SyntaxEngine};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DictionaryEntry {
    pub root: String,
    pub definition: String,
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
                let fields = value.as_table().ok_or_else(|| {
                    LanguageError::Dictionary(format!(
                        "dictionary entry `{root}` must be a TOML table"
                    ))
                })?;
                if let Some(unknown) = fields.keys().find(|name| name.as_str() != "definition") {
                    return Err(LanguageError::Dictionary(format!(
                        "dictionary entry `{root}` contains unsupported field `{unknown}`; lexical roots have one canonical definition, not POS-specific meanings"
                    )));
                }
                let definition = fields
                    .get("definition")
                    .and_then(toml::Value::as_str)
                    .map(str::trim)
                    .filter(|definition| !definition.is_empty())
                    .ok_or_else(|| {
                        LanguageError::Dictionary(format!(
                            "dictionary entry `{root}` must contain a non-empty `definition`"
                        ))
                    })?;
                Ok(DictionaryEntry {
                    root: root.to_lowercase(),
                    definition: definition.to_owned(),
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
    morphology: MorphologyEngine,
    syntax: SyntaxEngine,
    dictionary: Dictionary,
    roots: RootInventory,
    semantics: Environment,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexicalWordAnalysis {
    pub morphology: MorphologyAnalysis,
    pub phonology: WordAnalysis,
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
    MorphologyConfig(MorphologyConfigError),
    Morphology(MorphologyError),
    SyntaxConfig(SyntaxConfigError),
    Syntax(SurfaceError),
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
        let morphology = read(path.join("morphology.toml"))?;
        let syntax = read(path.join("syntax.toml"))?;
        let dictionary = read(path.join("dictionary.toml"))?;
        let semantics = compile_path(path.join("semantics")).map_err(LanguageError::Semantics)?;
        Self::from_parts(&alphabet, &phonology, &morphology, &syntax, &dictionary, semantics)
    }

    pub fn from_sources(
        alphabet: &str,
        phonology: &str,
        morphology: &str,
        syntax: &str,
        dictionary: &str,
        semantic_sources: &[(&str, &str)],
    ) -> Result<Self, LanguageError> {
        let semantics = compile_sources(
            semantic_sources
                .iter()
                .map(|(name, source)| ((*name).to_owned(), (*source).to_owned())),
        )
        .map_err(LanguageError::Semantics)?;
        Self::from_parts(alphabet, phonology, morphology, syntax, dictionary, semantics)
    }

    fn from_parts(
        alphabet: &str,
        phonology: &str,
        morphology: &str,
        syntax: &str,
        dictionary: &str,
        semantics: Environment,
    ) -> Result<Self, LanguageError> {
        let phonology = PhonologyConfig::from_toml(alphabet, phonology)
            .map_err(LanguageError::Phonology)?;
        let morphology = MorphologyEngine::new(
            MorphologyConfig::from_toml(morphology).map_err(LanguageError::MorphologyConfig)?,
        );
        let syntax = SyntaxEngine::new(
            SyntaxConfig::from_toml(syntax).map_err(LanguageError::SyntaxConfig)?,
        );
        let dictionary_parsed = Dictionary::from_toml(dictionary)?;
        let roots = RootInventory::from_dictionary_toml(dictionary)
            .map_err(|error| LanguageError::Dictionary(error.to_string()))?;
        validate_roots(&phonology, &roots)?;
        validate_morphology(&morphology, &phonology, &roots)?;
        validate_syntax(&syntax, &phonology, &roots, &semantics)?;
        Ok(Self {
            phonology,
            morphology,
            syntax,
            dictionary: dictionary_parsed,
            roots,
            semantics,
        })
    }

    pub fn phonology(&self) -> &PhonologyConfig {
        &self.phonology
    }

    pub fn morphology(&self) -> &MorphologyEngine {
        &self.morphology
    }

    pub fn syntax(&self) -> &SyntaxEngine {
        &self.syntax
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

    pub fn analyze_word(&self, word: &str) -> Result<LexicalWordAnalysis, LanguageError> {
        let morphology = self
            .morphology
            .analyze(word, &self.phonology, &self.roots)
            .map_err(LanguageError::Morphology)?;
        let phonology = self
            .phonology
            .analyze_word_with_root_text(&morphology.spelling, &morphology.root)
            .map_err(|error| LanguageError::RootInventory(error.to_string()))?;
        Ok(LexicalWordAnalysis {
            morphology,
            phonology,
        })
    }

    pub fn generate_word(&self, root: &str) -> Result<String, LanguageError> {
        self.morphology
            .generate(root, &self.phonology, &self.roots)
            .map_err(LanguageError::Morphology)
    }

    pub fn analyze_surface(&self, expression: &str) -> Result<SurfaceAnalysis, LanguageError> {
        self.syntax
            .analyze(expression, &self.semantics)
            .map_err(LanguageError::Syntax)
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

fn validate_morphology(
    morphology: &MorphologyEngine,
    phonology: &PhonologyConfig,
    roots: &RootInventory,
) -> Result<(), LanguageError> {
    for root in roots.roots() {
        let generated = morphology
            .generate(root, phonology, roots)
            .map_err(LanguageError::Morphology)?;
        let analysis = morphology
            .analyze(&generated, phonology, roots)
            .map_err(LanguageError::Morphology)?;
        if analysis.root != *root || generated != *root {
            return Err(LanguageError::RootInventory(format!(
                "morphology round-trip changed root `{root}` into `{generated}` / `{}`",
                analysis.root
            )));
        }
    }
    Ok(())
}


fn validate_syntax(
    syntax: &SyntaxEngine,
    phonology: &PhonologyConfig,
    roots: &RootInventory,
    semantics: &Environment,
) -> Result<(), LanguageError> {
    syntax.validate_environment(semantics).map_err(LanguageError::Syntax)?;

    for marker in [&syntax.config().scope.open, &syntax.config().scope.close] {
        validate_surface_token("scope marker", marker, phonology)?;
        if roots.roots().iter().any(|root| root == marker) {
            return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
                "scope marker `{marker}` collides with lexical root `{marker}`"
            ))));
        }
    }
    for surface in syntax.config().lexemes.keys() {
        validate_surface_token("surface lexeme", surface, phonology)?;
    }
    Ok(())
}

fn validate_surface_token(
    kind: &str,
    surface: &str,
    phonology: &PhonologyConfig,
) -> Result<(), LanguageError> {
    if surface.is_empty() || surface.chars().any(char::is_whitespace) {
        return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
            "{kind} `{surface}` must be one non-empty written token"
        ))));
    }
    phonology.alphabet.pronounce(surface).map_err(|error| {
        LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
            "{kind} `{surface}` is not phonologically valid: {error}"
        )))
    })?;
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
            Self::MorphologyConfig(error) => write!(f, "morphology config: {error}"),
            Self::Morphology(error) => write!(f, "morphology: {error}"),
            Self::SyntaxConfig(error) => write!(f, "syntax config: {error}"),
            Self::Syntax(error) => write!(f, "syntax: {error}"),
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
