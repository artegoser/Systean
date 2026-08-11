use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::compiler::{
    PackageInvariantError, PackageManifest, PackageManifestError, PackageProvenance,
    PackageValidationReport, WholeLanguageCompiler, validate_compiled_package,
    validate_corpus_sources, validate_declared_corpora,
};
use crate::discourse::{
    ConversationError, ConversationState, DiscourseFrameId, DiscourseGenerationError,
    DiscourseResolutionError, DiscourseState, ReferentId, ResolvedSurfaceAst, SectionId,
    TextDocument, TextRealization, TextSessionState, TextStreamItem, TextStructureError, TextTurn,
    UtteranceId, materialize_resolved_surface, parse_text_turn, resolve_surface,
};
use crate::literals::{LiteralConfig, LiteralConfigError, LiteralEngine, LiteralError};
use crate::morphology::{
    MorphologyAnalysis, MorphologyConfig, MorphologyConfigError, MorphologyEngine, MorphologyError,
};
use crate::phonology::{ConfigError, PhonologyConfig, RootInventory, WordAnalysis};
use crate::pragmatics::{PragmaticAnalysis, PragmaticError, interpret_pragmatics, validate_pragmatics};
use crate::units::{UnitRegistry, UnitsConfig, UnitsConfigError};
use crate::semantics::{Checker, Environment, Explainer, Term, Type, canonicalize};
use crate::spec::{PackageError, compile_path, compile_sources, lower_term, parse_term, parse_type};
use crate::syntax::{
    LexemeConfig, SurfaceAnalysis, SurfaceError, SurfaceExpr, SurfaceFormConfig, SurfaceLexicon,
    SyntaxConfig, SyntaxConfigError, SyntaxEngine, TypedSurfaceAst, elaborate_surface,
    linearize_surface, parse_surface_with_literals,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LexicalSemantic {
    Constant {
        #[serde(default)]
        name: Option<String>,
        #[serde(rename = "type")]
        ty: String,
    },
    Operator {
        name: String,
    },
    Reference,
    Information {
        status: String,
        #[serde(default)]
        knower_type: Option<String>,
    },
    Context {
        key: String,
        #[serde(rename = "type")]
        ty: String,
    },
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DictionaryEntry {
    pub root: String,
    pub definition: String,
    pub semantic: LexicalSemantic,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax: Option<SurfaceFormConfig>,
}

#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub struct Dictionary {
    entries: Vec<DictionaryEntry>,
}

impl Dictionary {
    pub fn from_toml(source: &str) -> Result<Self, LanguageError> {
        let value: toml::Value = toml::from_str(source)
            .map_err(|error| LanguageError::Dictionary(error.to_string()))?;
        let table = value.as_table().ok_or_else(|| {
            LanguageError::Dictionary("dictionary root must be a TOML table".into())
        })?;
        let mut entries = table
            .iter()
            .filter(|(key, _)| key.as_str() != "meta")
            .map(|(root, value)| parse_dictionary_entry(root, value))
            .collect::<Result<Vec<_>, LanguageError>>()?;
        entries.sort_by(|left, right| left.root.cmp(&right.root));
        Ok(Self { entries })
    }

    pub fn entries(&self) -> &[DictionaryEntry] {
        &self.entries
    }

    pub fn surface_lexicon(&self) -> SurfaceLexicon {
        let entries = self
            .entries
            .iter()
            .map(|entry| (entry.root.clone(), compile_surface_lexeme(entry)))
            .collect::<BTreeMap<_, _>>();
        SurfaceLexicon::new(entries)
    }

    fn install_constants(&self, environment: &mut Environment) -> Result<(), LanguageError> {
        for entry in &self.entries {
            let LexicalSemantic::Constant { name, ty } = &entry.semantic else {
                continue;
            };
            let semantic_name = name.as_deref().unwrap_or(&entry.root);
            let parsed_type = parse_type(ty).map_err(|errors| {
                LanguageError::Dictionary(format!(
                    "dictionary entry `{}` has invalid semantic type `{}`: {}",
                    entry.root,
                    ty,
                    errors
                        .into_iter()
                        .map(|error| error.to_string())
                        .collect::<Vec<_>>()
                        .join("; ")
                ))
            })?;
            environment
                .define_constant(semantic_name.to_owned(), parsed_type)
                .map_err(|error| {
                    LanguageError::Dictionary(format!(
                        "dictionary entry `{}` cannot define semantic constant `{semantic_name}`: {error}",
                        entry.root
                    ))
                })?;
        }
        Ok(())
    }
}

fn parse_dictionary_entry(root: &str, value: &toml::Value) -> Result<DictionaryEntry, LanguageError> {
    let fields = value.as_table().ok_or_else(|| {
        LanguageError::Dictionary(format!("dictionary entry `{root}` must be a TOML table"))
    })?;
    if let Some(unknown) = fields.keys().find(|name| {
        !matches!(name.as_str(), "definition" | "semantic" | "syntax")
    }) {
        return Err(LanguageError::Dictionary(format!(
            "dictionary entry `{root}` contains unsupported field `{unknown}`; lexical roots have one canonical identity, not POS-specific meanings"
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
    let semantic_value = fields.get("semantic").ok_or_else(|| {
        LanguageError::Dictionary(format!(
            "dictionary entry `{root}` must contain one `semantic` binding"
        ))
    })?;
    let semantic: LexicalSemantic = semantic_value.clone().try_into().map_err(|error| {
        LanguageError::Dictionary(format!(
            "dictionary entry `{root}` has invalid `semantic` binding: {error}"
        ))
    })?;
    let syntax = fields
        .get("syntax")
        .map(|value| {
            value.clone().try_into::<SurfaceFormConfig>().map_err(|error| {
                LanguageError::Dictionary(format!(
                    "dictionary entry `{root}` has invalid `syntax` realization: {error}"
                ))
            })
        })
        .transpose()?;

    validate_lexical_binding(root, &semantic, syntax.as_ref())?;

    Ok(DictionaryEntry {
        root: root.to_lowercase(),
        definition: definition.to_owned(),
        semantic,
        syntax,
    })
}

fn validate_lexical_binding(
    root: &str,
    semantic: &LexicalSemantic,
    syntax: Option<&SurfaceFormConfig>,
) -> Result<(), LanguageError> {
    match semantic {
        LexicalSemantic::Constant { name, ty } => {
            if name.as_ref().is_some_and(|name| name.trim().is_empty()) {
                return Err(LanguageError::Dictionary(format!(
                    "dictionary entry `{root}` has an empty semantic constant name"
                )));
            }
            if ty.trim().is_empty() {
                return Err(LanguageError::Dictionary(format!(
                    "dictionary entry `{root}` has an empty semantic type"
                )));
            }
            if syntax.is_some() {
                return Err(LanguageError::Dictionary(format!(
                    "dictionary constant root `{root}` is automatically a surface atom and must not duplicate that in `syntax`"
                )));
            }
        }
        LexicalSemantic::Operator { name } => {
            if name.trim().is_empty() {
                return Err(LanguageError::Dictionary(format!(
                    "dictionary entry `{root}` has an empty semantic operator name"
                )));
            }
            if syntax.is_none() {
                return Err(LanguageError::Dictionary(format!(
                    "dictionary operator root `{root}` requires one explicit `syntax` realization"
                )));
            }
        }
        LexicalSemantic::Reference => {
            if syntax.is_some() {
                return Err(LanguageError::Dictionary(format!(
                    "dictionary reference root `{root}` has intrinsic reference syntax and must not declare `syntax`"
                )));
            }
        }
        LexicalSemantic::Information { status, knower_type } => {
            if status.trim().is_empty() {
                return Err(LanguageError::Dictionary(format!(
                    "dictionary information root `{root}` has an empty status identity"
                )));
            }
            if knower_type.as_ref().is_some_and(|ty| ty.trim().is_empty()) {
                return Err(LanguageError::Dictionary(format!(
                    "dictionary information root `{root}` has an empty knower type"
                )));
            }
            if syntax.is_some() {
                return Err(LanguageError::Dictionary(format!(
                    "dictionary information root `{root}` has intrinsic typed-slot syntax and must not declare `syntax`"
                )));
            }
        }
        LexicalSemantic::Context { key, ty } => {
            if key.trim().is_empty() {
                return Err(LanguageError::Dictionary(format!(
                    "dictionary context root `{root}` has an empty context key"
                )));
            }
            if ty.trim().is_empty() {
                return Err(LanguageError::Dictionary(format!(
                    "dictionary context root `{root}` has an empty semantic type"
                )));
            }
            if syntax.is_some() {
                return Err(LanguageError::Dictionary(format!(
                    "dictionary context root `{root}` has intrinsic context syntax and must not declare `syntax`"
                )));
            }
        }
    }
    Ok(())
}

fn compile_surface_lexeme(entry: &DictionaryEntry) -> LexemeConfig {
    match (&entry.semantic, &entry.syntax) {
        (LexicalSemantic::Constant { name, .. }, None) => LexemeConfig::Atom {
            semantic: name.clone().unwrap_or_else(|| entry.root.clone()),
        },
        (LexicalSemantic::Reference, None) => LexemeConfig::Reference,
        (LexicalSemantic::Information { status, knower_type }, None) => LexemeConfig::Information {
            status: status.clone(),
            knower_type: knower_type.clone(),
        },
        (LexicalSemantic::Context { key, ty }, None) => LexemeConfig::Context {
            key: key.clone(),
            ty: ty.clone(),
        },
        (LexicalSemantic::Operator { name }, Some(surface)) => match surface {
            SurfaceFormConfig::Class { role } => LexemeConfig::Class {
                semantic: name.clone(),
                role: role.clone(),
            },
            SurfaceFormConfig::Predicate {
                primary_role,
                rest_roles,
            } => LexemeConfig::Predicate {
                semantic: name.clone(),
                primary_role: primary_role.clone(),
                rest_roles: rest_roles.clone(),
            },
            SurfaceFormConfig::Prefix { role } => LexemeConfig::Prefix {
                semantic: name.clone(),
                role: role.clone(),
            },
            SurfaceFormConfig::Infix {
                left_role,
                right_role,
            } => LexemeConfig::Infix {
                semantic: name.clone(),
                left_role: left_role.clone(),
                right_role: right_role.clone(),
            },
            SurfaceFormConfig::Quantifier {
                binder_role,
                variable_type,
                restriction_operator,
                restriction_role,
                body_role,
            } => LexemeConfig::Quantifier {
                semantic: name.clone(),
                binder_role: binder_role.clone(),
                variable_type: variable_type.clone(),
                restriction_operator: restriction_operator.clone(),
                restriction_role: restriction_role.clone(),
                body_role: body_role.clone(),
            },
            SurfaceFormConfig::CountedQuantifier {
                binder_role,
                count_role,
                variable_type,
                restriction_operator,
                restriction_role,
                body_role,
            } => LexemeConfig::CountedQuantifier {
                semantic: name.clone(),
                binder_role: binder_role.clone(),
                count_role: count_role.clone(),
                variable_type: variable_type.clone(),
                restriction_operator: restriction_operator.clone(),
                restriction_role: restriction_role.clone(),
                body_role: body_role.clone(),
            },
            SurfaceFormConfig::SpeechAct { role } => LexemeConfig::SpeechAct {
                semantic: name.clone(),
                role: role.clone(),
            },
            SurfaceFormConfig::Name { role } => LexemeConfig::Name {
                semantic: name.clone(),
                role: role.clone(),
            },
        },
        _ => unreachable!("dictionary lexical bindings are validated during parsing"),
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
    manifest: PackageManifest,
    provenance: PackageProvenance,
    validation: PackageValidationReport,
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

#[derive(Clone, Debug, PartialEq)]
pub struct DiscourseSurfaceAnalysis {
    pub syntax: SurfaceExpr,
    pub typed: TypedSurfaceAst,
    pub resolved: ResolvedSurfaceAst,
    pub canonical_surface: String,
    pub canonical_resolved_surface: String,
    pub inferred_type: String,
    pub canonical_semantics: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommunicativeSurfaceAnalysis {
    pub surface: DiscourseSurfaceAnalysis,
    pub pragmatics: PragmaticAnalysis,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextUtteranceAnalysis {
    pub id: UtteranceId,
    pub section: SectionId,
    pub source: String,
    pub canonical_surface: String,
    pub canonical_spoken: String,
    pub canonical_written: String,
    pub pragmatics: PragmaticAnalysis,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextTurnEvent {
    Utterance(TextUtteranceAnalysis),
    FrameBoundary {
        section: SectionId,
        frame: DiscourseFrameId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextTurnAnalysis {
    pub key: String,
    pub realization: TextRealization,
    pub events: Vec<TextTurnEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextDocumentAnalysis {
    pub turns: Vec<TextTurnAnalysis>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LanguageError {
    Io { path: PathBuf, message: String },
    Phonology(ConfigError),
    MorphologyConfig(MorphologyConfigError),
    Morphology(MorphologyError),
    LiteralConfig(LiteralConfigError),
    UnitsConfig(UnitsConfigError),
    Literal(LiteralError),
    SyntaxConfig(SyntaxConfigError),
    Syntax(SurfaceError),
    Discourse(DiscourseResolutionError),
    DiscourseGenerate(DiscourseGenerationError),
    Pragmatics(PragmaticError),
    TextStructure(TextStructureError),
    Conversation(ConversationError),
    Dictionary(String),
    RootInventory(String),
    Semantics(Vec<PackageError>),
    PackageManifest(PackageManifestError),
    Package(Vec<PackageInvariantError>),
    SemanticExpression(String),
}

impl LanguagePackage {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LanguageError> {
        WholeLanguageCompiler::compile_path(path)
    }

    pub(crate) fn compile_directory(path: &Path) -> Result<Self, LanguageError> {
        let manifest_source = read(path.join("package.toml"))?;
        let manifest = PackageManifest::from_toml(&manifest_source)
            .map_err(LanguageError::PackageManifest)?;
        let provenance = PackageProvenance::from_directory(path, &manifest)
            .map_err(LanguageError::PackageManifest)?;
        let alphabet = read(path.join("alphabet.toml"))?;
        let phonology = read(path.join("phonology.toml"))?;
        let morphology = read(path.join("morphology.toml"))?;
        let syntax = read(path.join("syntax.toml"))?;
        let dictionary = read(path.join("dictionary.toml"))?;
        let literals = read(path.join("literals.toml"))?;
        let units = read(path.join("units.toml"))?;
        let semantics = compile_path(path.join("semantics")).map_err(LanguageError::Semantics)?;
        Self::from_parts(
            &alphabet,
            &phonology,
            &morphology,
            &syntax,
            &dictionary,
            Some((&literals, &units)),
            semantics,
            manifest,
            provenance,
            Some(path),
        )
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
        let manifest = PackageManifest::synthetic();
        let provenance = PackageProvenance::from_sources(
            &manifest,
            [
                ("alphabet.toml".to_owned(), alphabet.to_owned()),
                ("phonology.toml".to_owned(), phonology.to_owned()),
                ("morphology.toml".to_owned(), morphology.to_owned()),
                ("syntax.toml".to_owned(), syntax.to_owned()),
                ("dictionary.toml".to_owned(), dictionary.to_owned()),
            ]
            .into_iter()
            .chain(
                semantic_sources
                    .iter()
                    .map(|(name, source)| ((*name).to_owned(), (*source).to_owned())),
            ),
        );
        Self::from_parts(
            alphabet,
            phonology,
            morphology,
            syntax,
            dictionary,
            None,
            semantics,
            manifest,
            provenance,
            None,
        )
    }

    pub fn from_versioned_sources_full(
        manifest_source: &str,
        alphabet: &str,
        phonology: &str,
        morphology: &str,
        syntax: &str,
        dictionary: &str,
        literals: &str,
        units: &str,
        semantic_sources: &[(&str, &str)],
        compatibility_source: &str,
        adversarial_source: &str,
    ) -> Result<Self, LanguageError> {
        let manifest = PackageManifest::from_toml(manifest_source)
            .map_err(LanguageError::PackageManifest)?;
        let semantics = compile_sources(
            semantic_sources
                .iter()
                .map(|(name, source)| ((*name).to_owned(), (*source).to_owned())),
        )
        .map_err(LanguageError::Semantics)?;
        let compatibility_path = manifest.validation.compatibility_corpus.clone();
        let adversarial_path = manifest.validation.adversarial_corpus.clone();
        let provenance = PackageProvenance::from_sources(
            &manifest,
            [
                ("package.toml".to_owned(), manifest_source.to_owned()),
                ("alphabet.toml".to_owned(), alphabet.to_owned()),
                ("phonology.toml".to_owned(), phonology.to_owned()),
                ("morphology.toml".to_owned(), morphology.to_owned()),
                ("syntax.toml".to_owned(), syntax.to_owned()),
                ("dictionary.toml".to_owned(), dictionary.to_owned()),
                ("literals.toml".to_owned(), literals.to_owned()),
                ("units.toml".to_owned(), units.to_owned()),
                (compatibility_path.clone(), compatibility_source.to_owned()),
                (adversarial_path.clone(), adversarial_source.to_owned()),
            ]
            .into_iter()
            .chain(
                semantic_sources
                    .iter()
                    .map(|(name, source)| ((*name).to_owned(), (*source).to_owned())),
            ),
        );
        let mut package = Self::from_parts(
            alphabet,
            phonology,
            morphology,
            syntax,
            dictionary,
            Some((literals, units)),
            semantics,
            manifest,
            provenance,
            None,
        )?;
        let mut report = package.validation.clone();
        validate_corpus_sources(
            &package,
            &compatibility_path,
            compatibility_source,
            &adversarial_path,
            adversarial_source,
            &mut report,
        )
        .map_err(LanguageError::Package)?;
        package.validation = report;
        Ok(package)
    }

    pub fn from_sources_full(
        alphabet: &str,
        phonology: &str,
        morphology: &str,
        syntax: &str,
        dictionary: &str,
        literals: &str,
        units: &str,
        semantic_sources: &[(&str, &str)],
    ) -> Result<Self, LanguageError> {
        let semantics = compile_sources(
            semantic_sources
                .iter()
                .map(|(name, source)| ((*name).to_owned(), (*source).to_owned())),
        )
        .map_err(LanguageError::Semantics)?;
        let manifest = PackageManifest::synthetic();
        let provenance = PackageProvenance::from_sources(
            &manifest,
            [
                ("alphabet.toml".to_owned(), alphabet.to_owned()),
                ("phonology.toml".to_owned(), phonology.to_owned()),
                ("morphology.toml".to_owned(), morphology.to_owned()),
                ("syntax.toml".to_owned(), syntax.to_owned()),
                ("dictionary.toml".to_owned(), dictionary.to_owned()),
                ("literals.toml".to_owned(), literals.to_owned()),
                ("units.toml".to_owned(), units.to_owned()),
            ]
            .into_iter()
            .chain(
                semantic_sources
                    .iter()
                    .map(|(name, source)| ((*name).to_owned(), (*source).to_owned())),
            ),
        );
        Self::from_parts(
            alphabet,
            phonology,
            morphology,
            syntax,
            dictionary,
            Some((literals, units)),
            semantics,
            manifest,
            provenance,
            None,
        )
    }

    fn from_parts(
        alphabet: &str,
        phonology: &str,
        morphology: &str,
        syntax: &str,
        dictionary: &str,
        structured_sources: Option<(&str, &str)>,
        semantics: Environment,
        manifest: PackageManifest,
        provenance: PackageProvenance,
        package_root: Option<&Path>,
    ) -> Result<Self, LanguageError> {
        let phonology_config = PhonologyConfig::from_toml(alphabet, phonology)
            .map_err(LanguageError::Phonology)?;
        let morphology_engine = MorphologyEngine::new(
            MorphologyConfig::from_toml(morphology).map_err(LanguageError::MorphologyConfig)?,
        );
        let syntax_config = SyntaxConfig::from_toml(syntax).map_err(LanguageError::SyntaxConfig)?;
        let dictionary_parsed = Dictionary::from_toml(dictionary)?;
        let roots = RootInventory::from_dictionary_toml(dictionary)
            .map_err(|error| LanguageError::Dictionary(error.to_string()))?;
        validate_roots(&phonology_config, &roots)?;
        validate_morphology(&morphology_engine, &phonology_config, &roots)?;

        let mut semantics = semantics;
        dictionary_parsed.install_constants(&mut semantics)?;
        validate_pragmatics(&syntax_config.pragmatics, &semantics)
            .map_err(LanguageError::Pragmatics)?;
        let syntax_engine = if let Some((literal_source, units_source)) = structured_sources {
            let literal_config = LiteralConfig::from_toml(literal_source)
                .map_err(LanguageError::LiteralConfig)?;
            let units_config = UnitsConfig::from_toml(units_source)
                .map_err(LanguageError::UnitsConfig)?;
            let units_registry = UnitRegistry::new(units_config).map_err(LanguageError::UnitsConfig)?;
            let literal_engine = LiteralEngine::new(
                literal_config,
                units_registry,
                syntax_config.scope.open.clone(),
                syntax_config.scope.close.clone(),
            )
            .map_err(LanguageError::Literal)?;
            SyntaxEngine::new_with_literals(
                syntax_config,
                dictionary_parsed.surface_lexicon(),
                literal_engine,
            )
        } else {
            SyntaxEngine::new(syntax_config, dictionary_parsed.surface_lexicon())
        };
        validate_syntax(&syntax_engine, &phonology_config, &roots, &semantics)?;
        validate_literals(&syntax_engine, &phonology_config, &roots, &semantics)?;

        let mut package = Self {
            phonology: phonology_config,
            morphology: morphology_engine,
            syntax: syntax_engine,
            dictionary: dictionary_parsed,
            roots,
            semantics,
            manifest,
            provenance,
            validation: PackageValidationReport::empty(),
        };
        let (literal_source, units_source) = structured_sources.unwrap_or(("", ""));
        let module_sources = [
            ("alphabet", alphabet),
            ("phonology", phonology),
            ("morphology", morphology),
            ("syntax", syntax),
            ("dictionary", dictionary),
            ("literals", literal_source),
            ("units", units_source),
        ];
        let mut report = validate_compiled_package(&package, &package.manifest, &module_sources)
            .map_err(LanguageError::Package)?;
        if let Some(root) = package_root {
            validate_declared_corpora(&package, root, &mut report)
                .map_err(LanguageError::Package)?;
        }
        package.validation = report;
        Ok(package)
    }

    pub fn manifest(&self) -> &PackageManifest {
        &self.manifest
    }

    pub fn provenance(&self) -> &PackageProvenance {
        &self.provenance
    }

    pub fn validation_report(&self) -> &PackageValidationReport {
        &self.validation
    }

    pub fn package_fingerprint(&self) -> &str {
        &self.provenance.fingerprint
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

    pub fn literals(&self) -> Option<&LiteralEngine> {
        self.syntax.literals()
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
            .validate_environment(&self.semantics)
            .map_err(LanguageError::Syntax)?;
        let syntax = self.syntax.parse(expression).map_err(LanguageError::Syntax)?;
        self.validate_surface_payloads(&syntax)?;
        let canonical_surface = self
            .syntax
            .linearize(&syntax)
            .map_err(LanguageError::Syntax)?;
        let lowered = self
            .syntax
            .lower(&syntax, &self.semantics)
            .map_err(LanguageError::Syntax)?;
        Ok(SurfaceAnalysis {
            syntax,
            canonical_surface,
            inferred_type: lowered.inferred_type.to_string(),
            canonical_semantics: canonicalize(&lowered.term).to_string(),
        })
    }

    pub fn analyze_surface_with_discourse(
        &self,
        expression: &str,
        discourse: &DiscourseState,
    ) -> Result<DiscourseSurfaceAnalysis, LanguageError> {
        self.syntax
            .validate_environment(&self.semantics)
            .map_err(LanguageError::Syntax)?;
        let lexicon = self.discourse_lexicon(discourse)?;
        let syntax = parse_surface_with_literals(
            expression, self.syntax.config(), &lexicon, self.syntax.literals()
        )
            .map_err(|errors| LanguageError::Syntax(SurfaceError::Parse(errors)))?;
        self.validate_surface_payloads(&syntax)?;
        let canonical_surface = linearize_surface(&syntax, self.syntax.config(), &lexicon)
            .map_err(|error| LanguageError::Syntax(SurfaceError::Generate(error)))?;
        let typed = elaborate_surface(&syntax, &lexicon, &self.semantics)
            .map_err(|error| LanguageError::Syntax(SurfaceError::Elaborate(error)))?;
        let resolved = resolve_surface(&typed, discourse, &self.semantics)
            .map_err(LanguageError::Discourse)?;
        let reference_surface = self
            .syntax
            .lexicon()
            .iter()
            .filter_map(|(surface, lexeme)| {
                matches!(lexeme, LexemeConfig::Reference).then_some(surface.as_str())
            })
            .min()
            .unwrap_or("");
        if !typed.references.is_empty() && reference_surface.is_empty() {
            return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(
                "surface language has reference slots but no canonical reference root".into(),
            )));
        }
        let resolved_surface = materialize_resolved_surface(
            &typed,
            &resolved,
            discourse,
            &self.semantics,
            reference_surface,
        )
        .map_err(LanguageError::DiscourseGenerate)?;
        let canonical_resolved_surface =
            linearize_surface(&resolved_surface, self.syntax.config(), &lexicon)
                .map_err(|error| LanguageError::Syntax(SurfaceError::Generate(error)))?;
        Ok(DiscourseSurfaceAnalysis {
            syntax,
            typed,
            canonical_surface,
            canonical_resolved_surface,
            inferred_type: resolved.inferred_type.to_string(),
            canonical_semantics: canonicalize(&resolved.term).to_string(),
            resolved,
        })
    }

    pub fn analyze_utterance(
        &self,
        expression: &str,
    ) -> Result<CommunicativeSurfaceAnalysis, LanguageError> {
        self.analyze_utterance_with_discourse(expression, &DiscourseState::new())
    }

    pub fn analyze_utterance_with_discourse(
        &self,
        expression: &str,
        discourse: &DiscourseState,
    ) -> Result<CommunicativeSurfaceAnalysis, LanguageError> {
        let surface = self.analyze_surface_with_discourse(expression, discourse)?;
        let pragmatics = interpret_pragmatics(
            &surface.resolved.term,
            &surface.resolved.inferred_type,
            &self.syntax.config().pragmatics,
            &self.semantics,
        )
        .map_err(LanguageError::Pragmatics)?;
        Ok(CommunicativeSurfaceAnalysis { surface, pragmatics })
    }

    pub fn apply_text_turn(
        &self,
        turn: &TextTurn,
        session: &mut TextSessionState,
        discourse: &mut DiscourseState,
        conversation: &mut ConversationState,
    ) -> Result<TextTurnAnalysis, LanguageError> {
        let items = parse_text_turn(&turn.source, turn.realization, self.syntax.config())
            .map_err(LanguageError::TextStructure)?;

        let mut next_session = session.clone();
        let mut next_discourse = discourse.clone();
        let mut next_conversation = conversation.clone();
        let mut events = Vec::new();

        for item in items {
            match item {
                TextStreamItem::FrameBoundary => {
                    let frame = next_discourse.advance_frame();
                    let section = next_session.advance_section();
                    events.push(TextTurnEvent::FrameBoundary { section, frame });
                }
                TextStreamItem::Utterance(source) => {
                    let analysis = self
                        .analyze_utterance_with_discourse(&source, &next_discourse)?;
                    let canonical_surface = analysis.surface.canonical_resolved_surface.clone();
                    let id = next_conversation
                        .apply(
                            source.clone(),
                            canonical_surface.clone(),
                            analysis.pragmatics.clone(),
                        )
                        .map_err(LanguageError::Conversation)?;
                    let canonical_spoken = format!(
                        "{} {}",
                        canonical_surface,
                        self.syntax.config().text.utterance_spoken.as_str()
                    );
                    let canonical_written = format!(
                        "{}{}",
                        canonical_surface,
                        self.syntax.config().text.utterance_written.as_str()
                    );
                    events.push(TextTurnEvent::Utterance(TextUtteranceAnalysis {
                        id,
                        section: next_session.current_section(),
                        source,
                        canonical_surface,
                        canonical_spoken,
                        canonical_written,
                        pragmatics: analysis.pragmatics,
                    }));
                }
            }
        }

        *session = next_session;
        *discourse = next_discourse;
        *conversation = next_conversation;
        Ok(TextTurnAnalysis {
            key: turn.key.clone(),
            realization: turn.realization,
            events,
        })
    }

    pub fn analyze_text_document(
        &self,
        document: &TextDocument,
    ) -> Result<TextDocumentAnalysis, LanguageError> {
        let mut session = TextSessionState::new();
        let mut discourse = DiscourseState::new();
        let mut conversation = ConversationState::new();
        self.analyze_text_document_with_state(
            document,
            &mut session,
            &mut discourse,
            &mut conversation,
        )
    }

    pub fn analyze_text_document_with_state(
        &self,
        document: &TextDocument,
        session: &mut TextSessionState,
        discourse: &mut DiscourseState,
        conversation: &mut ConversationState,
    ) -> Result<TextDocumentAnalysis, LanguageError> {
        let mut next_session = session.clone();
        let mut next_discourse = discourse.clone();
        let mut next_conversation = conversation.clone();
        let mut turns = Vec::with_capacity(document.turns.len());

        for turn in &document.turns {
            turns.push(self.apply_text_turn(
                turn,
                &mut next_session,
                &mut next_discourse,
                &mut next_conversation,
            )?);
        }

        *session = next_session;
        *discourse = next_discourse;
        *conversation = next_conversation;
        Ok(TextDocumentAnalysis { turns })
    }

    pub fn validate_alias_surface(&self, surface: &str) -> Result<(), LanguageError> {
        validate_surface_token("discourse alias", surface, &self.phonology)?;
        if self.roots.roots().iter().any(|root| root == surface) {
            return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
                "discourse alias `{surface}` collides with lexical root `{surface}`"
            ))));
        }
        let config = self.syntax.config();
        let structural = [
            config.scope.open.as_str(),
            config.scope.close.as_str(),
            config.discourse.alias.as_str(),
            config.discourse.definition.as_str(),
            config.discourse.relative.as_str(),
            config.discourse.frame.as_str(),
            config.quotation.open.as_str(),
            config.quotation.close.as_str(),
            config.text.utterance_spoken.as_str(),
        ];
        if structural.contains(&surface) {
            return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
                "discourse alias `{surface}` collides with structural marker `{surface}`"
            ))));
        }
        let check = self.phonology.check_root(surface, &self.roots);
        if !check.is_valid() {
            return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
                "discourse alias `{surface}` is not a collision-free spoken form"
            ))));
        }
        Ok(())
    }

    pub fn bind_alias(
        &self,
        discourse: &mut DiscourseState,
        surface: &str,
        referent: ReferentId,
    ) -> Result<(), LanguageError> {
        self.validate_alias_surface(surface)?;
        discourse
            .bind_alias(surface.to_owned(), referent)
            .map_err(|error| LanguageError::Discourse(DiscourseResolutionError::Discourse(error)))
    }

    pub fn define_alias(
        &self,
        discourse: &mut DiscourseState,
        surface: &str,
        value: Term,
    ) -> Result<ReferentId, LanguageError> {
        self.validate_alias_surface(surface)?;
        discourse
            .define_alias(surface.to_owned(), value, &self.semantics)
            .map_err(|error| LanguageError::Discourse(DiscourseResolutionError::Discourse(error)))
    }

    pub fn bind_alias_to_surface(
        &self,
        discourse: &mut DiscourseState,
        alias: &str,
        target: &str,
    ) -> Result<ReferentId, LanguageError> {
        self.validate_alias_surface(alias)?;
        let analysis = self.analyze_surface_with_discourse(target, discourse)?;
        let resolved = discourse
            .resolve_reference_value(
                &analysis.resolved.term,
                &analysis.resolved.inferred_type,
                &self.semantics,
            )
            .map_err(|error| LanguageError::Discourse(DiscourseResolutionError::Discourse(error)))?;
        discourse
            .bind_alias(alias.to_owned(), resolved.id)
            .map_err(|error| LanguageError::Discourse(DiscourseResolutionError::Discourse(error)))?;
        Ok(resolved.id)
    }

    pub fn define_alias_from_surface(
        &self,
        discourse: &mut DiscourseState,
        alias: &str,
        target: &str,
    ) -> Result<ReferentId, LanguageError> {
        let analysis = self.analyze_surface_with_discourse(target, discourse)?;
        self.define_alias(discourse, alias, analysis.resolved.term)
    }

    pub fn analyze_with_relative_binding(
        &self,
        discourse: &mut DiscourseState,
        alias: &str,
        target: &str,
        body: &str,
    ) -> Result<DiscourseSurfaceAnalysis, LanguageError> {
        let target = self.analyze_surface_with_discourse(target, discourse)?;
        discourse.enter_scope();
        let defined = self.define_alias(discourse, alias, target.resolved.term);
        let result = match defined {
            Ok(_) => self.analyze_surface_with_discourse(body, discourse),
            Err(error) => Err(error),
        };
        let leave = discourse.leave_scope();
        match (result, leave) {
            (Ok(analysis), Ok(_)) => Ok(analysis),
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(LanguageError::Discourse(
                DiscourseResolutionError::Discourse(error),
            )),
        }
    }

    fn discourse_lexicon(&self, discourse: &DiscourseState) -> Result<SurfaceLexicon, LanguageError> {
        self.syntax
            .lexicon()
            .with_aliases(
                discourse
                    .active_aliases()
                    .into_iter()
                    .map(|binding| (binding.surface, binding.ty.to_string())),
            )
            .map_err(|message| LanguageError::Syntax(SurfaceError::InvalidBinding(message)))
    }

    fn validate_surface_payloads(&self, expression: &SurfaceExpr) -> Result<(), LanguageError> {
        match expression {
            SurfaceExpr::Name { payload, .. } => self.validate_name_payload(payload),
            SurfaceExpr::Quote(_)
            | SurfaceExpr::Literal(_)
            | SurfaceExpr::Atom(_)
            | SurfaceExpr::Context(_)
            | SurfaceExpr::Alias(_) => Ok(()),
            SurfaceExpr::Clause(clause) => {
                if let Some(primary) = &clause.primary {
                    self.validate_argument_payload(primary)?;
                }
                for argument in &clause.rest {
                    self.validate_argument_payload(argument)?;
                }
                Ok(())
            }
            SurfaceExpr::Prefix { operand, .. } => self.validate_surface_payloads(operand),
            SurfaceExpr::SpeechAct { content, .. } => self.validate_surface_payloads(content),
            SurfaceExpr::Infix { operands, .. } => {
                for operand in operands {
                    self.validate_surface_payloads(operand)?;
                }
                Ok(())
            }
        }
    }

    fn validate_argument_payload(
        &self,
        argument: &crate::syntax::Argument,
    ) -> Result<(), LanguageError> {
        match argument {
            crate::syntax::Argument::Name { payload, .. } => self.validate_name_payload(payload),
            crate::syntax::Argument::Information {
                knower: Some(crate::syntax::InformationKnower::Name { payload, .. }),
                ..
            } => self.validate_name_payload(payload),
            _ => Ok(()),
        }
    }

    fn validate_name_payload(&self, payload: &str) -> Result<(), LanguageError> {
        self.phonology
            .analyze_root(payload)
            .map(|_| ())
            .map_err(|error| {
                LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
                    "proper-name payload `{payload}` is not a canonical Systean spoken form: {error}"
                )))
            })
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

    let config = syntax.config();
    let markers = [
        ("scope marker", config.scope.open.as_str()),
        ("scope marker", config.scope.close.as_str()),
        ("discourse alias marker", config.discourse.alias.as_str()),
        ("discourse definition marker", config.discourse.definition.as_str()),
        ("discourse relative marker", config.discourse.relative.as_str()),
        ("discourse frame marker", config.discourse.frame.as_str()),
        ("quotation open marker", config.quotation.open.as_str()),
        ("quotation close marker", config.quotation.close.as_str()),
        ("spoken utterance boundary", config.text.utterance_spoken.as_str()),
    ];
    for (kind, marker) in markers {
        validate_surface_token(kind, marker, phonology)?;
        if roots.roots().iter().any(|root| root == marker) {
            return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
                "{kind} `{marker}` collides with lexical root `{marker}`"
            ))));
        }
    }
    if syntax.lexicon().len() != roots.roots().len() {
        return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
            "compiled surface lexicon has {} entries for {} dictionary roots",
            syntax.lexicon().len(),
            roots.roots().len()
        ))));
    }
    for root in roots.roots() {
        if !syntax.lexicon().contains_key(root) {
            return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
                "dictionary root `{root}` is missing from the compiled surface lexicon"
            ))));
        }
    }
    Ok(())
}

fn validate_literals(
    syntax: &SyntaxEngine,
    phonology: &PhonologyConfig,
    roots: &RootInventory,
    semantics: &Environment,
) -> Result<(), LanguageError> {
    let Some(literals) = syntax.literals() else { return Ok(()) };
    let structural = [
        syntax.config().scope.open.as_str(),
        syntax.config().scope.close.as_str(),
        syntax.config().discourse.alias.as_str(),
        syntax.config().discourse.definition.as_str(),
        syntax.config().discourse.relative.as_str(),
        syntax.config().discourse.frame.as_str(),
        syntax.config().quotation.open.as_str(),
        syntax.config().quotation.close.as_str(),
        syntax.config().text.utterance_spoken.as_str(),
    ];
    for surface in literals.reserved_forms() {
        validate_surface_token("structured-literal spoken form", surface, phonology)?;
        if roots.roots().iter().any(|root| root == surface) {
            return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
                "structured-literal spoken form `{surface}` collides with lexical root `{surface}`"
            ))));
        }
        if structural.contains(&surface) {
            return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
                "structured-literal spoken form `{surface}` collides with structural marker `{surface}`"
            ))));
        }
    }
    let config = literals.config();
    for ty in [
        &config.number.semantic_type,
        &config.number.digit_type,
        &config.number.digit_sequence_type,
        &config.calendar.date_type,
        &config.calendar.time_of_day_type,
        &config.calendar.instant_type,
        &config.calendar.interval_type,
        &config.calendar.duration_type,
        &config.calendar.timezone_type,
    ] {
        let parsed = parse_type(ty).map_err(|errors| LanguageError::Syntax(SurfaceError::InvalidBinding(
            errors.into_iter().map(|error| error.to_string()).collect::<Vec<_>>().join("; ")
        )))?;
        if !semantics.is_well_formed_type(&parsed) {
            return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
                "structured-literal type `{ty}` is not declared by the semantic package"
            ))));
        }
    }
    let number_type = parse_type(&config.number.semantic_type).map_err(|errors| {
        LanguageError::Syntax(SurfaceError::InvalidBinding(
            errors
                .into_iter()
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("; "),
        ))
    })?;
    let approximate_number_type = Type::Generic {
        name: "Approximate".into(),
        arguments: vec![number_type],
    };
    if !semantics.is_well_formed_type(&approximate_number_type) {
        return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
            "structured-literal type `{approximate_number_type}` is not declared by the semantic package"
        ))));
    }
    for dimension in &literals.units().config().dimensions {
        let ty = parse_type(&dimension.semantic_type).map_err(|errors| LanguageError::Syntax(SurfaceError::InvalidBinding(
            errors.into_iter().map(|error| error.to_string()).collect::<Vec<_>>().join("; ")
        )))?;
        if !semantics.is_well_formed_type(&ty) {
            return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
                "unit dimension `{}` uses undeclared semantic type `{}`",
                dimension.id, dimension.semantic_type
            ))));
        }
    }
    for unit in &literals.units().config().units {
        for ty in [
            literals.units().unit_type(unit),
            literals.units().quantity_type(unit, false),
            literals.units().quantity_type(unit, true),
        ] {
            if !semantics.is_well_formed_type(&ty) {
                return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
                    "unit `{}` produces undeclared semantic type `{ty}`",
                    unit.id
                ))));
            }
        }
        let symbol = unit.symbol.to_lowercase();
        if roots.roots().iter().any(|root| root == &symbol) {
            return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
                "written unit symbol `{}` collides with lexical root `{}`",
                unit.symbol, symbol
            ))));
        }
        if structural.contains(&symbol.as_str()) {
            return Err(LanguageError::Syntax(SurfaceError::InvalidBinding(format!(
                "written unit symbol `{}` collides with structural marker `{}`",
                unit.symbol, symbol
            ))));
        }
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
            Self::LiteralConfig(error) => write!(f, "literals config: {error}"),
            Self::UnitsConfig(error) => write!(f, "units config: {error}"),
            Self::Literal(error) => write!(f, "structured literal: {error}"),
            Self::SyntaxConfig(error) => write!(f, "syntax config: {error}"),
            Self::Syntax(error) => write!(f, "syntax: {error}"),
            Self::Discourse(error) => write!(f, "discourse: {error}"),
            Self::DiscourseGenerate(error) => write!(f, "discourse generation: {error}"),
            Self::Pragmatics(error) => write!(f, "pragmatics: {error}"),
            Self::TextStructure(error) => write!(f, "text structure: {error}"),
            Self::Conversation(error) => write!(f, "conversation: {error}"),
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
            Self::PackageManifest(error) => write!(f, "package manifest: {error}"),
            Self::Package(errors) => {
                for (index, error) in errors.iter().enumerate() {
                    if index > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "package invariant: {error}")?;
                }
                Ok(())
            }
            Self::SemanticExpression(error) => write!(f, "semantic expression: {error}"),
        }
    }
}

impl std::error::Error for LanguageError {}
