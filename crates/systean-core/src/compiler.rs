use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::language::{LanguageError, LanguagePackage};
use crate::semantics::canonicalize;
use crate::syntax::{
    Argument, ArgumentOmission, Clause, FrameOrder, InformationKnower, CompiledSurfaceBinding, SurfaceExpr,
};

const REQUIRED_MODULES: [&str; 7] = [
    "alphabet",
    "phonology",
    "morphology",
    "syntax",
    "dictionary",
    "literals",
    "units",
];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackageManifest {
    pub package: PackageIdentity,
    pub provenance: ManifestProvenance,
    pub modules: BTreeMap<String, String>,
    pub compatibility: CompatibilityPolicy,
    pub validation: ValidationPolicy,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackageIdentity {
    pub name: String,
    pub version: String,
    pub revision: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManifestProvenance {
    pub source: String,
    pub specification: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CompatibilityPolicy {
    pub epoch: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ValidationPolicy {
    pub compatibility_corpus: String,
    pub adversarial_corpus: String,
    pub exhaustive_max_tokens: usize,
}

impl PackageManifest {
    pub fn from_toml(source: &str) -> Result<Self, PackageManifestError> {
        let manifest: Self = toml::from_str(source)
            .map_err(|error| PackageManifestError::Toml(error.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn synthetic() -> Self {
        Self {
            package: PackageIdentity {
                name: "in-memory-systean".into(),
                version: "0.0.0".into(),
                revision: 0,
            },
            provenance: ManifestProvenance {
                source: "in-memory sources".into(),
                specification: "unspecified".into(),
            },
            modules: BTreeMap::new(),
            compatibility: CompatibilityPolicy { epoch: 0 },
            validation: ValidationPolicy {
                compatibility_corpus: String::new(),
                adversarial_corpus: String::new(),
                exhaustive_max_tokens: 3,
            },
        }
    }

    fn validate(&self) -> Result<(), PackageManifestError> {
        if self.package.name.trim().is_empty() {
            return Err(PackageManifestError::Invalid(
                "package.name must not be empty".into(),
            ));
        }
        if !is_numeric_triplet(&self.package.version) {
            return Err(PackageManifestError::Invalid(format!(
                "package.version `{}` must be a numeric MAJOR.MINOR.PATCH version",
                self.package.version
            )));
        }
        if self.provenance.source.trim().is_empty()
            || self.provenance.specification.trim().is_empty()
        {
            return Err(PackageManifestError::Invalid(
                "provenance.source and provenance.specification must not be empty".into(),
            ));
        }
        if self.validation.exhaustive_max_tokens == 0
            || self.validation.exhaustive_max_tokens > 4
        {
            return Err(PackageManifestError::Invalid(
                "validation.exhaustive_max_tokens must be in 1..=4".into(),
            ));
        }
        if self.package.revision > 0 {
            for module in REQUIRED_MODULES {
                let Some(version) = self.modules.get(module) else {
                    return Err(PackageManifestError::Invalid(format!(
                        "manifest is missing modules.{module}"
                    )));
                };
                if version.trim().is_empty() {
                    return Err(PackageManifestError::Invalid(format!(
                        "modules.{module} must not be empty"
                    )));
                }
            }
            if let Some(unknown) = self
                .modules
                .keys()
                .find(|name| !REQUIRED_MODULES.contains(&name.as_str()))
            {
                return Err(PackageManifestError::Invalid(format!(
                    "manifest contains unknown normative module `{unknown}`"
                )));
            }
            if self.validation.compatibility_corpus.trim().is_empty()
                || self.validation.adversarial_corpus.trim().is_empty()
            {
                return Err(PackageManifestError::Invalid(
                    "validation corpus paths must not be empty for a versioned package".into(),
                ));
            }
        }
        Ok(())
    }
}

fn is_numeric_triplet(version: &str) -> bool {
    let parts = version.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageManifestError {
    Io { path: PathBuf, message: String },
    Toml(String),
    Invalid(String),
}

impl fmt::Display for PackageManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, message } => write!(f, "{}: {message}", path.display()),
            Self::Toml(message) => write!(f, "package manifest TOML: {message}"),
            Self::Invalid(message) => write!(f, "invalid package manifest: {message}"),
        }
    }
}

impl std::error::Error for PackageManifestError {}

#[derive(Clone, Copy, Debug, Default)]
pub struct WholeLanguageCompiler;

impl WholeLanguageCompiler {
    pub fn compile_path(path: impl AsRef<Path>) -> Result<LanguagePackage, LanguageError> {
        LanguagePackage::compile_directory(path.as_ref())
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct SourceProvenance {
    pub path: String,
    pub digest: String,
    pub bytes: usize,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct PackageProvenance {
    pub source: String,
    pub specification: String,
    pub fingerprint: String,
    pub sources: Vec<SourceProvenance>,
}

impl PackageProvenance {
    pub fn from_directory(
        root: &Path,
        manifest: &PackageManifest,
    ) -> Result<Self, PackageManifestError> {
        let mut files = Vec::new();
        collect_normative_files(root, &mut files)?;
        let mut sources = Vec::new();
        let mut aggregate = Vec::new();
        for path in files {
            let bytes = fs::read(&path).map_err(|error| PackageManifestError::Io {
                path: path.clone(),
                message: error.to_string(),
            })?;
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let digest = stable_digest(&bytes);
            aggregate.extend_from_slice(relative.as_bytes());
            aggregate.push(0);
            aggregate.extend_from_slice(&bytes);
            aggregate.push(0xff);
            sources.push(SourceProvenance {
                path: relative,
                digest,
                bytes: bytes.len(),
            });
        }
        Ok(Self {
            source: manifest.provenance.source.clone(),
            specification: manifest.provenance.specification.clone(),
            fingerprint: stable_digest(&aggregate),
            sources,
        })
    }

    pub fn from_sources(
        manifest: &PackageManifest,
        sources: impl IntoIterator<Item = (String, String)>,
    ) -> Self {
        let mut values = sources.into_iter().collect::<Vec<_>>();
        values.sort_by(|left, right| left.0.cmp(&right.0));
        let mut aggregate = Vec::new();
        let sources = values
            .into_iter()
            .map(|(path, source)| {
                aggregate.extend_from_slice(path.as_bytes());
                aggregate.push(0);
                aggregate.extend_from_slice(source.as_bytes());
                aggregate.push(0xff);
                SourceProvenance {
                    path,
                    digest: stable_digest(source.as_bytes()),
                    bytes: source.len(),
                }
            })
            .collect();
        Self {
            source: manifest.provenance.source.clone(),
            specification: manifest.provenance.specification.clone(),
            fingerprint: stable_digest(&aggregate),
            sources,
        }
    }
}

fn collect_normative_files(
    current: &Path,
    output: &mut Vec<PathBuf>,
) -> Result<(), PackageManifestError> {
    let entries = fs::read_dir(current).map_err(|error| PackageManifestError::Io {
        path: current.to_path_buf(),
        message: error.to_string(),
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| PackageManifestError::Io {
            path: current.to_path_buf(),
            message: error.to_string(),
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| PackageManifestError::Io {
            path: path.clone(),
            message: error.to_string(),
        })?;
        if file_type.is_dir() {
            // `legacy/` is archival migration input only. `typed/` is normative as of Phase 18.
            let directory = path.file_name().and_then(|name| name.to_str());
            if directory == Some("legacy") {
                continue;
            }
            collect_normative_files(&path, output)?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let extension = path.extension().and_then(|value| value.to_str());
        if matches!(extension, Some("toml" | "semsys" | "tsv")) {
            output.push(path);
        }
    }
    output.sort();
    Ok(())
}

fn stable_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct PackageValidationReport {
    pub owned_forms: usize,
    pub generated_asts: usize,
    pub generated_surfaces: usize,
    pub semantic_roundtrips: usize,
    pub exhaustive_candidates: usize,
    pub exhaustive_valid_surfaces: usize,
    pub compatibility_entries: usize,
    pub adversarial_entries: usize,
}

impl PackageValidationReport {
    pub fn empty() -> Self {
        Self {
            owned_forms: 0,
            generated_asts: 0,
            generated_surfaces: 0,
            semantic_roundtrips: 0,
            exhaustive_candidates: 0,
            exhaustive_valid_surfaces: 0,
            compatibility_entries: 0,
            adversarial_entries: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageInvariantError {
    ModuleVersion {
        module: String,
        expected: String,
        actual: String,
    },
    InvalidModuleMetadata {
        module: String,
        message: String,
    },
    SurfaceCollision {
        surface: String,
        channel: String,
        owners: Vec<String>,
    },
    SpokenCollision {
        pronunciation: String,
        forms: Vec<String>,
    },
    GrammarOverlap {
        surface: String,
        analyses: Vec<String>,
    },
    AstRoundTrip {
        surface: String,
        before: String,
        after: String,
    },
    SemanticRoundTrip {
        surface: String,
        before: String,
        after: String,
    },
    Corpus {
        path: String,
        line: usize,
        message: String,
    },
}

impl fmt::Display for PackageInvariantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModuleVersion {
                module,
                expected,
                actual,
            } => write!(
                f,
                "module `{module}` declares version `{actual}`, manifest requires `{expected}`"
            ),
            Self::InvalidModuleMetadata { module, message } => {
                write!(f, "module `{module}` metadata: {message}")
            }
            Self::SurfaceCollision {
                surface,
                channel,
                owners,
            } => write!(
                f,
                "{channel} surface `{surface}` has multiple normative owners: {}",
                owners.join(", ")
            ),
            Self::SpokenCollision {
                pronunciation,
                forms,
            } => write!(
                f,
                "spoken form /{pronunciation}/ maps to multiple normative surfaces: {}",
                forms.join(", ")
            ),
            Self::GrammarOverlap { surface, analyses } => write!(
                f,
                "canonical surface `{surface}` is generated by multiple AST analyses: {}",
                analyses.join(" | ")
            ),
            Self::AstRoundTrip {
                surface,
                before,
                after,
            } => write!(
                f,
                "AST round-trip changed `{surface}` from {before} to {after}"
            ),
            Self::SemanticRoundTrip {
                surface,
                before,
                after,
            } => write!(
                f,
                "semantic round-trip changed `{surface}` from `{before}` to `{after}`"
            ),
            Self::Corpus {
                path,
                line,
                message,
            } => write!(f, "{path}:{line}: {message}"),
        }
    }
}

impl std::error::Error for PackageInvariantError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Channel {
    Spoken,
    Written,
    Both,
}

impl Channel {
    fn overlaps(self, other: Self) -> bool {
        !matches!((self, other), (Self::Spoken, Self::Written) | (Self::Written, Self::Spoken))
    }

    fn spoken(self) -> bool {
        matches!(self, Self::Spoken | Self::Both)
    }
}

#[derive(Clone, Debug)]
struct OwnedSurface {
    surface: String,
    owner: String,
    channel: Channel,
}

pub(crate) fn validate_compiled_package(
    language: &LanguagePackage,
    manifest: &PackageManifest,
    module_sources: &[(&str, &str)],
) -> Result<PackageValidationReport, Vec<PackageInvariantError>> {
    if manifest.package.revision == 0 {
        return Ok(PackageValidationReport::empty());
    }

    let mut errors = Vec::new();
    validate_module_versions(manifest, module_sources, &mut errors);

    let owned = owned_surfaces(language);
    validate_surface_ownership(language, &owned, &mut errors);

    let asts = generated_ast_corpus(language);
    let mut surfaces = BTreeMap::<String, SurfaceExpr>::new();
    let mut semantic_roundtrips = 0usize;
    let mut spoken_streams = BTreeMap::<String, (String, SurfaceExpr)>::new();

    for ast in &asts {
        let surface = match language.syntax().linearize(ast) {
            Ok(surface) => surface,
            Err(error) => {
                errors.push(PackageInvariantError::AstRoundTrip {
                    surface: "<generation failed>".into(),
                    before: format!("{ast:?}"),
                    after: error.to_string(),
                });
                continue;
            }
        };
        match language.syntax().parse(&surface) {
            Ok(reparsed) => {
                if &reparsed != ast {
                    errors.push(PackageInvariantError::AstRoundTrip {
                        surface: surface.clone(),
                        before: format!("{ast:?}"),
                        after: format!("{reparsed:?}"),
                    });
                }
            }
            Err(error) => errors.push(PackageInvariantError::AstRoundTrip {
                surface: surface.clone(),
                before: format!("{ast:?}"),
                after: error.to_string(),
            }),
        }

        if let Some(previous) = surfaces.get(&surface) {
            if previous != ast {
                errors.push(PackageInvariantError::GrammarOverlap {
                    surface: surface.clone(),
                    analyses: vec![format!("{previous:?}"), format!("{ast:?}")],
                });
            }
        } else {
            surfaces.insert(surface.clone(), ast.clone());
        }

        if let Ok(pronunciation) = language.phonology().alphabet.pronounce(&surface) {
            if let Some((previous_surface, previous_ast)) = spoken_streams.get(&pronunciation) {
                if previous_ast != ast {
                    errors.push(PackageInvariantError::GrammarOverlap {
                        surface: format!("/{pronunciation}/"),
                        analyses: vec![
                            format!("{previous_surface} => {previous_ast:?}"),
                            format!("{surface} => {ast:?}"),
                        ],
                    });
                }
            } else {
                spoken_streams.insert(pronunciation, (surface.clone(), ast.clone()));
            }
        }

        let Ok(before) = language.syntax().lower(ast, language.semantics()) else {
            continue;
        };
        let Ok(reparsed) = language.syntax().parse(&surface) else {
            continue;
        };
        let Ok(after) = language.syntax().lower(&reparsed, language.semantics()) else {
            continue;
        };
        let before = canonicalize(&before.term).to_string();
        let after = canonicalize(&after.term).to_string();
        semantic_roundtrips += 1;
        if before != after {
            errors.push(PackageInvariantError::SemanticRoundTrip {
                surface,
                before,
                after,
            });
        }
    }

    let (exhaustive_candidates, exhaustive_valid_surfaces) = validate_short_exhaustive(
        language,
        manifest.validation.exhaustive_max_tokens,
        &mut errors,
    );

    if errors.is_empty() {
        Ok(PackageValidationReport {
            owned_forms: owned.len(),
            generated_asts: asts.len(),
            generated_surfaces: surfaces.len(),
            semantic_roundtrips,
            exhaustive_candidates,
            exhaustive_valid_surfaces,
            compatibility_entries: 0,
            adversarial_entries: 0,
        })
    } else {
        Err(errors)
    }
}

fn validate_module_versions(
    manifest: &PackageManifest,
    module_sources: &[(&str, &str)],
    errors: &mut Vec<PackageInvariantError>,
) {
    let sources = module_sources.iter().copied().collect::<BTreeMap<_, _>>();
    for module in REQUIRED_MODULES {
        let Some(expected) = manifest.modules.get(module) else {
            continue;
        };
        let Some(source) = sources.get(module) else {
            errors.push(PackageInvariantError::InvalidModuleMetadata {
                module: module.into(),
                message: "compiler did not receive the normative module source".into(),
            });
            continue;
        };
        let value: toml::Value = match toml::from_str(source) {
            Ok(value) => value,
            Err(error) => {
                errors.push(PackageInvariantError::InvalidModuleMetadata {
                    module: module.into(),
                    message: error.to_string(),
                });
                continue;
            }
        };
        let actual = value
            .get("meta")
            .and_then(|value| value.get("version"))
            .and_then(toml::Value::as_str);
        let Some(actual) = actual else {
            errors.push(PackageInvariantError::InvalidModuleMetadata {
                module: module.into(),
                message: "missing meta.version".into(),
            });
            continue;
        };
        if actual != expected {
            errors.push(PackageInvariantError::ModuleVersion {
                module: module.into(),
                expected: expected.clone(),
                actual: actual.into(),
            });
        }
    }
}

fn owned_surfaces(language: &LanguagePackage) -> Vec<OwnedSurface> {
    let mut output = Vec::new();
    for root in language.roots().roots() {
        output.push(OwnedSurface {
            surface: root.clone(),
            owner: format!("dictionary root `{root}`"),
            channel: Channel::Both,
        });
    }
    let config = language.syntax().config();
    for (kind, surface) in [
        ("scope open", config.scope.open.as_str()),
        ("scope close", config.scope.close.as_str()),
        ("discourse alias", config.discourse.alias.as_str()),
        ("discourse definition", config.discourse.definition.as_str()),
        ("discourse relative", config.discourse.relative.as_str()),
        ("discourse frame", config.discourse.frame.as_str()),
        ("quotation open", config.quotation.open.as_str()),
        ("quotation close", config.quotation.close.as_str()),
    ] {
        output.push(OwnedSurface {
            surface: surface.to_owned(),
            owner: format!("structural marker `{kind}`"),
            channel: Channel::Both,
        });
    }
    output.push(OwnedSurface {
        surface: config.text.utterance_spoken.clone(),
        owner: "spoken utterance boundary".into(),
        channel: Channel::Spoken,
    });
    output.push(OwnedSurface {
        surface: config.text.utterance_written.clone(),
        owner: "written utterance boundary".into(),
        channel: Channel::Written,
    });
    for punctuation in &config.text.readability_punctuation {
        output.push(OwnedSurface {
            surface: punctuation.clone(),
            owner: "readability punctuation".into(),
            channel: Channel::Written,
        });
    }
    if let Some(literals) = language.literals() {
        for surface in literals.config().reserved_forms() {
            output.push(OwnedSurface {
                surface: surface.to_owned(),
                owner: format!("structured-literal form `{surface}`"),
                channel: Channel::Spoken,
            });
        }
        for unit in &literals.units().config().units {
            output.push(OwnedSurface {
                surface: unit.spoken.to_lowercase(),
                owner: format!("unit `{}` spoken form", unit.id),
                channel: Channel::Spoken,
            });
            output.push(OwnedSurface {
                surface: unit.symbol.to_lowercase(),
                owner: format!("unit `{}` written symbol", unit.id),
                channel: Channel::Written,
            });
        }
    }
    output
}

fn validate_surface_ownership(
    language: &LanguagePackage,
    owned: &[OwnedSurface],
    errors: &mut Vec<PackageInvariantError>,
) {
    for left_index in 0..owned.len() {
        for right_index in (left_index + 1)..owned.len() {
            let left = &owned[left_index];
            let right = &owned[right_index];
            if left.surface == right.surface
                && left.owner != right.owner
                && left.channel.overlaps(right.channel)
            {
                errors.push(PackageInvariantError::SurfaceCollision {
                    surface: left.surface.clone(),
                    channel: channel_overlap_label(left.channel, right.channel).into(),
                    owners: vec![left.owner.clone(), right.owner.clone()],
                });
            }
        }
    }

    let mut by_pronunciation = BTreeMap::<String, BTreeSet<String>>::new();
    for item in owned.iter().filter(|item| item.channel.spoken()) {
        let Ok(pronunciation) = language.phonology().alphabet.pronounce(&item.surface) else {
            continue;
        };
        by_pronunciation
            .entry(pronunciation)
            .or_default()
            .insert(item.surface.clone());
    }
    for (pronunciation, forms) in by_pronunciation {
        if forms.len() > 1 {
            errors.push(PackageInvariantError::SpokenCollision {
                pronunciation,
                forms: forms.into_iter().collect(),
            });
        }
    }
}


fn channel_overlap_label(left: Channel, right: Channel) -> &'static str {
    match (left, right) {
        (Channel::Written, Channel::Written)
        | (Channel::Written, Channel::Both)
        | (Channel::Both, Channel::Written) => "written",
        (Channel::Spoken, Channel::Spoken)
        | (Channel::Spoken, Channel::Both)
        | (Channel::Both, Channel::Spoken) => "spoken",
        (Channel::Both, Channel::Both) => "spoken/written",
        (Channel::Spoken, Channel::Written) | (Channel::Written, Channel::Spoken) => "disjoint",
    }
}

fn generated_ast_corpus(language: &LanguagePackage) -> Vec<SurfaceExpr> {
    let lexicon = language.syntax().lexicon();
    let mut output = Vec::new();
    let mut first_atom = None;
    let mut first_alias = None;
    let mut first_context = None;
    let mut first_capture = None;
    let mut first_class = None;
    let mut prefixes = Vec::new();
    let mut outer_prefixes = Vec::new();
    let mut infixes = Vec::new();
    let mut nested_infixes = Vec::new();
    let mut binders = Vec::new();
    let mut references = Vec::new();
    let mut information = Vec::new();

    for (surface, lexeme) in lexicon.iter() {
        match lexeme {
            CompiledSurfaceBinding::Atom { .. } => {
                first_atom.get_or_insert_with(|| surface.clone());
                output.push(SurfaceExpr::Atom(surface.clone()));
            }
            CompiledSurfaceBinding::Alias { .. } => {
                first_alias.get_or_insert_with(|| surface.clone());
                output.push(SurfaceExpr::Alias(surface.clone()));
            }
            CompiledSurfaceBinding::Context { .. } => {
                first_context.get_or_insert_with(|| surface.clone());
                output.push(SurfaceExpr::Context(surface.clone()));
            }
            CompiledSurfaceBinding::Capture { .. } => {
                first_capture.get_or_insert_with(|| surface.clone());
                output.push(SurfaceExpr::Captured {
                    marker: surface.clone(),
                    payload: "nara".into(),
                });
            }
            CompiledSurfaceBinding::Class { .. } => {
                first_class.get_or_insert_with(|| surface.clone());
            }
            CompiledSurfaceBinding::Prefix { outer_only, .. } => {
                if *outer_only {
                    outer_prefixes.push(surface.clone());
                } else {
                    prefixes.push(surface.clone());
                }
            }
            CompiledSurfaceBinding::Infix { outer_only, .. } => {
                infixes.push(surface.clone());
                if !*outer_only {
                    nested_infixes.push(surface.clone());
                }
            }
            CompiledSurfaceBinding::Binder { direct_parameters, .. } => {
                binders.push((surface.clone(), direct_parameters.len()));
            }
            CompiledSurfaceBinding::Reference => references.push(surface.clone()),
            CompiledSurfaceBinding::Information { knower_type, .. } => {
                information.push((surface.clone(), knower_type.clone()))
            }
            CompiledSurfaceBinding::Predicate { .. } => {}
        }
    }

    let representative_argument = first_atom
        .as_ref()
        .map(|surface| Argument::Atom(surface.clone()))
        .or_else(|| {
            first_context
                .as_ref()
                .map(|surface| Argument::Context(surface.clone()))
        })
        .or_else(|| {
            first_alias
                .as_ref()
                .map(|surface| Argument::Alias(surface.clone()))
        });
    let representative_expr = first_atom
        .as_ref()
        .map(|surface| SurfaceExpr::Atom(surface.clone()))
        .or_else(|| {
            first_context
                .as_ref()
                .map(|surface| SurfaceExpr::Context(surface.clone()))
        })
        .or_else(|| {
            first_alias
                .as_ref()
                .map(|surface| SurfaceExpr::Alias(surface.clone()))
        });

    if let (Some(argument), Some(expr)) = (representative_argument.clone(), representative_expr.clone()) {
        for (surface, lexeme) in lexicon.iter() {
            match lexeme {
                CompiledSurfaceBinding::Class { .. } => output.push(SurfaceExpr::Clause(Clause {
                    primary: Some(argument.clone()),
                    inner_prefixes: Vec::new(),
                    predicate: surface.clone(),
                    rest: Vec::new(),
                })),
                CompiledSurfaceBinding::Predicate {
                    primary_parameter,
                    rest_parameters,
                    ..
                } => {
                    let primary = primary_parameter.as_ref().map(|_| argument.clone());
                    let rest = rest_parameters.iter().map(|_| argument.clone()).collect::<Vec<_>>();
                    output.push(SurfaceExpr::Clause(Clause {
                        primary: primary.clone(),
                        inner_prefixes: Vec::new(),
                        predicate: surface.clone(),
                        rest: rest.clone(),
                    }));
                    if primary_parameter.is_some()
                        && language.syntax().config().arguments.omission
                            == ArgumentOmission::UniqueReferenceOnly
                    {
                        output.push(SurfaceExpr::Clause(Clause {
                            primary: Some(Argument::Omitted),
                            inner_prefixes: Vec::new(),
                            predicate: surface.clone(),
                            rest,
                        }));
                    }
                    if primary_parameter.is_some()
                        && language.syntax().config().order.frame == FrameOrder::PrimaryPredicateRest
                    {
                        for prefix in &prefixes {
                            output.push(SurfaceExpr::Clause(Clause {
                                primary: primary.clone(),
                                inner_prefixes: vec![prefix.clone()],
                                predicate: surface.clone(),
                                rest: rest_parameters.iter().map(|_| argument.clone()).collect(),
                            }));
                        }
                    }
                }
                _ => {}
            }
        }
        for prefix in &prefixes {
            output.push(SurfaceExpr::Prefix {
                operator: prefix.clone(),
                operand: Box::new(expr.clone()),
            });
        }
        for speech_act in &outer_prefixes {
            output.push(SurfaceExpr::Outer {
                operator: speech_act.clone(),
                content: Box::new(expr.clone()),
            });
        }
        for infix in &infixes {
            output.push(SurfaceExpr::Infix {
                operator: infix.clone(),
                operands: vec![expr.clone(), expr.clone()],
            });
        }
        for outer in &infixes {
            for inner in &nested_infixes {
                if outer == inner {
                    continue;
                }
                output.push(SurfaceExpr::Infix {
                    operator: outer.clone(),
                    operands: vec![
                        expr.clone(),
                        SurfaceExpr::Infix {
                            operator: inner.clone(),
                            operands: vec![expr.clone(), expr.clone()],
                        },
                    ],
                });
            }
        }
    }

    let literals = language
        .literals()
        .map(representative_literals)
        .unwrap_or_default();
    for literal in &literals {
        output.push(SurfaceExpr::Literal(literal.clone()));
    }
    let count_literal = language.literals().and_then(representative_number_literal);
    output.push(SurfaceExpr::Quote("nara".into()));

    if let Some(class) = first_class {
        let mut arguments = Vec::new();
        if let Some(atom) = first_atom {
            arguments.push(Argument::Atom(atom));
        }
        if let Some(context) = first_context.clone() {
            arguments.push(Argument::Context(context));
        }
        if let Some(alias) = first_alias {
            arguments.push(Argument::Alias(alias));
        }
        if let Some(name) = first_capture.clone() {
            arguments.push(Argument::Captured {
                marker: name,
                payload: "nara".into(),
            });
        }
        arguments.push(Argument::Quote("nara".into()));
        for literal in &literals {
            arguments.push(Argument::Literal(literal.clone()));
        }
        for reference in references {
            arguments.push(Argument::Reference(reference));
        }
        for (marker, knower_type) in information {
            let knower = if knower_type.is_some() {
                first_context
                    .as_ref()
                    .map(|surface| InformationKnower::Context(surface.clone()))
                    .or_else(|| {
                        first_capture.as_ref().map(|name| InformationKnower::Captured {
                            marker: name.clone(),
                            payload: "nara".into(),
                        })
                    })
            } else {
                None
            };
            if knower_type.is_none() || knower.is_some() {
                arguments.push(Argument::Information { marker, knower });
            }
        }
        for (binder, direct_count) in binders {
            let mut direct = Vec::new();
            if direct_count > 0 {
                if let Some(count) = count_literal.clone() {
                    direct.push(Argument::Literal(count));
                } else {
                    continue;
                }
            }
            if direct.len() == direct_count {
                arguments.push(Argument::Scoped {
                    binder,
                    direct,
                    restriction: class.clone(),
                });
            }
        }
        for argument in arguments {
            output.push(SurfaceExpr::Clause(Clause {
                primary: Some(argument),
                inner_prefixes: Vec::new(),
                predicate: class.clone(),
                rest: Vec::new(),
            }));
        }
    }

    let mut unique = Vec::new();
    for ast in output {
        if !unique.contains(&ast) {
            unique.push(ast);
        }
    }
    unique
}

fn representative_number_literal(
    language: &crate::literals::LiteralEngine,
) -> Option<crate::literals::SurfaceLiteral> {
    language
        .config()
        .number
        .digits
        .keys()
        .find_map(|surface| language.parse_complete(surface).ok())
}

fn representative_literals(
    language: &crate::literals::LiteralEngine,
) -> Vec<crate::literals::SurfaceLiteral> {
    let mut candidates = Vec::new();
    if let Some(digit) = language.config().number.digits.keys().next() {
        candidates.push(digit.clone());
        candidates.push(format!("{} {digit}", language.config().number.approximation));
        for unit in &language.units().config().units {
            candidates.push(format!("{digit} {}", unit.spoken));
        }
    }
    candidates.extend([
        "2000-12-31".to_owned(),
        "12:30:00".to_owned(),
        "2026-08-11T12:00:00Z".to_owned(),
        "PT3600S".to_owned(),
        "2026-08-11T12:00:00Z/2026-08-11T13:00:00Z".to_owned(),
    ]);

    let mut output = Vec::new();
    for candidate in candidates {
        if let Ok(literal) = language.parse_complete(&candidate) {
            let spoken = literal.canonical_spoken.clone();
            if !output.contains(&literal) {
                output.push(literal);
            }
            if let Ok(spoken_literal) = language.parse_complete(&spoken) {
                if !output.contains(&spoken_literal) {
                    output.push(spoken_literal);
                }
            }
        }
    }
    output
}

fn validate_short_exhaustive(
    language: &LanguagePackage,
    max_tokens: usize,
    errors: &mut Vec<PackageInvariantError>,
) -> (usize, usize) {
    let mut representatives = BTreeMap::<&'static str, String>::new();
    for (surface, lexeme) in language.syntax().lexicon().iter() {
        representatives
            .entry(lexeme_family(lexeme))
            .or_insert_with(|| surface.clone());
    }
    let mut tokens = representatives.into_values().collect::<Vec<_>>();
    let config = language.syntax().config();
    tokens.extend([
        config.scope.open.clone(),
        config.scope.close.clone(),
        config.quotation.open.clone(),
        config.quotation.close.clone(),
        config.discourse.alias.clone(),
        config.discourse.definition.clone(),
        config.discourse.relative.clone(),
        config.discourse.frame.clone(),
    ]);
    if let Some(literals) = language.literals() {
        if let Some(surface) = literals.config().number.digits.keys().next() {
            tokens.push(surface.clone());
        }
        tokens.extend([
            literals.config().number.approximation.clone(),
            literals.config().number.digit_sequence_marker.clone(),
            literals.config().calendar.date_marker.clone(),
            literals.config().calendar.timezone_marker.clone(),
        ]);
        if let Some(unit) = literals.units().config().units.first() {
            tokens.push(unit.spoken.clone());
        }
    }
    tokens.sort();
    tokens.dedup();

    let mut candidates = 0usize;
    let mut valid = 0usize;
    let mut current = Vec::new();
    for len in 1..=max_tokens {
        enumerate_sequences(&tokens, len, &mut current, &mut |sequence| {
            candidates += 1;
            let source = sequence.join(" ");
            let Ok(parsed) = language.syntax().parse(&source) else {
                return;
            };
            valid += 1;
            let Ok(canonical) = language.syntax().linearize(&parsed) else {
                return;
            };
            match language.syntax().parse(&canonical) {
                Ok(reparsed) if reparsed == parsed => {}
                Ok(reparsed) => errors.push(PackageInvariantError::AstRoundTrip {
                    surface: canonical,
                    before: format!("{parsed:?}"),
                    after: format!("{reparsed:?}"),
                }),
                Err(error) => errors.push(PackageInvariantError::AstRoundTrip {
                    surface: canonical,
                    before: format!("{parsed:?}"),
                    after: error.to_string(),
                }),
            }
        });
    }
    (candidates, valid)
}

fn enumerate_sequences(
    tokens: &[String],
    remaining: usize,
    current: &mut Vec<String>,
    callback: &mut impl FnMut(&[String]),
) {
    if remaining == 0 {
        callback(current);
        return;
    }
    for token in tokens {
        current.push(token.clone());
        enumerate_sequences(tokens, remaining - 1, current, callback);
        current.pop();
    }
}

fn lexeme_family(lexeme: &CompiledSurfaceBinding) -> &'static str {
    match lexeme {
        CompiledSurfaceBinding::Atom { .. } => "atom",
        CompiledSurfaceBinding::Reference => "reference",
        CompiledSurfaceBinding::Information { knower_type: None, .. } => "information",
        CompiledSurfaceBinding::Information { knower_type: Some(_), .. } => "information_knower",
        CompiledSurfaceBinding::Alias { .. } => "alias",
        CompiledSurfaceBinding::Context { .. } => "context",
        CompiledSurfaceBinding::Class { .. } => "class",
        CompiledSurfaceBinding::Predicate {
            primary_parameter: None, ..
        } => "predicate_no_primary",
        CompiledSurfaceBinding::Predicate {
            primary_parameter: Some(_),
            rest_parameters,
            ..
        } if rest_parameters.is_empty() => "predicate_primary",
        CompiledSurfaceBinding::Predicate { .. } => "predicate_multi",
        CompiledSurfaceBinding::Prefix { outer_only: false, .. } => "prefix",
        CompiledSurfaceBinding::Prefix { outer_only: true, .. } => "outer_prefix",
        CompiledSurfaceBinding::Infix { .. } => "infix",
        CompiledSurfaceBinding::Binder { direct_parameters, .. } if direct_parameters.is_empty() => "binder",
        CompiledSurfaceBinding::Binder { .. } => "binder_direct",
        CompiledSurfaceBinding::Capture { .. } => "capture",
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CompatibilityEntry {
    pub label: String,
    pub source: String,
    pub canonical_surface: String,
    pub inferred_type: String,
    pub canonical_semantics: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CompatibilitySnapshot {
    pub package: String,
    pub version: String,
    pub revision: u32,
    pub compatibility_epoch: u32,
    pub entries: Vec<CompatibilityEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CompatibilityChange {
    pub label: String,
    pub before_surface: String,
    pub after_surface: String,
    pub before_type: String,
    pub after_type: String,
    pub before_semantics: String,
    pub after_semantics: String,
}

impl LanguagePackage {
    pub fn compatibility_snapshot(
        &self,
        entries: impl IntoIterator<Item = (String, String)>,
    ) -> Result<CompatibilitySnapshot, String> {
        let mut compiled = Vec::new();
        for (label, source) in entries {
            let analysis = self
                .analyze_surface(&source)
                .map_err(|error| format!("compatibility entry `{label}`: {error}"))?;
            compiled.push(CompatibilityEntry {
                label,
                source,
                canonical_surface: analysis.canonical_surface,
                inferred_type: analysis.inferred_type,
                canonical_semantics: analysis.canonical_semantics,
            });
        }
        compiled.sort_by(|left, right| left.label.cmp(&right.label));
        Ok(CompatibilitySnapshot {
            package: self.manifest().package.name.clone(),
            version: self.manifest().package.version.clone(),
            revision: self.manifest().package.revision,
            compatibility_epoch: self.manifest().compatibility.epoch,
            entries: compiled,
        })
    }

    pub fn diff_compatibility(
        &self,
        previous: &CompatibilitySnapshot,
    ) -> Result<Vec<CompatibilityChange>, String> {
        if previous.compatibility_epoch != self.manifest().compatibility.epoch {
            return Err(format!(
                "compatibility epoch changed from {} to {}; revisions are not declared compatible",
                previous.compatibility_epoch,
                self.manifest().compatibility.epoch
            ));
        }
        let current = self.compatibility_snapshot(
            previous
                .entries
                .iter()
                .map(|entry| (entry.label.clone(), entry.source.clone())),
        )?;
        let by_label = current
            .entries
            .iter()
            .map(|entry| (entry.label.as_str(), entry))
            .collect::<BTreeMap<_, _>>();
        let mut changes = Vec::new();
        for before in &previous.entries {
            let Some(after) = by_label.get(before.label.as_str()) else {
                return Err(format!("compatibility entry `{}` disappeared", before.label));
            };
            if before.canonical_surface != after.canonical_surface
                || before.inferred_type != after.inferred_type
                || before.canonical_semantics != after.canonical_semantics
            {
                changes.push(CompatibilityChange {
                    label: before.label.clone(),
                    before_surface: before.canonical_surface.clone(),
                    after_surface: after.canonical_surface.clone(),
                    before_type: before.inferred_type.clone(),
                    after_type: after.inferred_type.clone(),
                    before_semantics: before.canonical_semantics.clone(),
                    after_semantics: after.canonical_semantics.clone(),
                });
            }
        }
        Ok(changes)
    }
}

pub(crate) fn validate_declared_corpora(
    language: &LanguagePackage,
    package_root: &Path,
    report: &mut PackageValidationReport,
) -> Result<(), Vec<PackageInvariantError>> {
    if language.manifest().package.revision == 0 {
        return Ok(());
    }
    let compatibility_path = package_root.join(&language.manifest().validation.compatibility_corpus);
    let adversarial_path = package_root.join(&language.manifest().validation.adversarial_corpus);
    let mut errors = Vec::new();
    let compatibility_source = read_corpus_source(&compatibility_path, &mut errors);
    let adversarial_source = read_corpus_source(&adversarial_path, &mut errors);
    if !errors.is_empty() {
        return Err(errors);
    }
    validate_corpus_sources(
        language,
        &compatibility_path.display().to_string(),
        compatibility_source.as_deref().unwrap_or_default(),
        &adversarial_path.display().to_string(),
        adversarial_source.as_deref().unwrap_or_default(),
        report,
    )
}

pub(crate) fn validate_corpus_sources(
    language: &LanguagePackage,
    compatibility_path: &str,
    compatibility_source: &str,
    adversarial_path: &str,
    adversarial_source: &str,
    report: &mut PackageValidationReport,
) -> Result<(), Vec<PackageInvariantError>> {
    if language.manifest().package.revision == 0 {
        return Ok(());
    }
    let mut errors = Vec::new();
    let compatibility = parse_corpus(compatibility_source);
    let mut entries = Vec::new();
    for (line, columns) in compatibility {
        if columns.len() != 5 {
            errors.push(PackageInvariantError::Corpus {
                path: compatibility_path.into(),
                line,
                message: concat!(
                    "expected 5 tab-separated columns: label, source, canonical surface, ",
                    "type, canonical semantics"
                )
                .into(),
            });
            continue;
        }
        let label = &columns[0];
        let source = &columns[1];
        match language.analyze_surface(source) {
            Ok(analysis) => {
                if analysis.canonical_surface != columns[2] {
                    errors.push(PackageInvariantError::Corpus {
                        path: compatibility_path.into(),
                        line,
                        message: format!(
                            "`{label}` canonical surface changed: expected `{}`, got `{}`",
                            columns[2], analysis.canonical_surface
                        ),
                    });
                }
                if analysis.inferred_type != columns[3] {
                    errors.push(PackageInvariantError::Corpus {
                        path: compatibility_path.into(),
                        line,
                        message: format!(
                            "`{label}` inferred type changed: expected `{}`, got `{}`",
                            columns[3], analysis.inferred_type
                        ),
                    });
                }
                if analysis.canonical_semantics != columns[4] {
                    errors.push(PackageInvariantError::Corpus {
                        path: compatibility_path.into(),
                        line,
                        message: format!(
                            "`{label}` semantics changed: expected `{}`, got `{}`",
                            columns[4], analysis.canonical_semantics
                        ),
                    });
                }
                entries.push((label.clone(), source.clone()));
            }
            Err(error) => errors.push(PackageInvariantError::Corpus {
                path: compatibility_path.into(),
                line,
                message: format!("`{label}` no longer analyzes: {error}"),
            }),
        }
    }
    report.compatibility_entries = entries.len();

    let adversarial = parse_corpus(adversarial_source);
    let mut adversarial_entries = 0usize;
    for (line, columns) in adversarial {
        if columns.len() < 3 || columns.len() > 4 {
            errors.push(PackageInvariantError::Corpus {
                path: adversarial_path.into(),
                line,
                message: "expected mode, label, source, and optional canonical surface".into(),
            });
            continue;
        }
        adversarial_entries += 1;
        let mode = columns[0].as_str();
        let label = columns[1].as_str();
        let source = columns[2].as_str();
        match mode {
            "reject" => {
                if let Ok(parsed) = language.syntax().parse(source) {
                    errors.push(PackageInvariantError::Corpus {
                        path: adversarial_path.into(),
                        line,
                        message: format!("`{label}` must be rejected but parsed as {parsed:?}"),
                    });
                }
            }
            "canonical" => match language.syntax().parse(source) {
                Ok(parsed) => match language.syntax().linearize(&parsed) {
                    Ok(canonical) if columns.get(3).is_some_and(|expected| expected == &canonical) => {}
                    Ok(canonical) => errors.push(PackageInvariantError::Corpus {
                        path: adversarial_path.into(),
                        line,
                        message: format!(
                            "`{label}` canonicalized to `{canonical}`, expected `{}`",
                            columns.get(3).map(String::as_str).unwrap_or("<missing>")
                        ),
                    }),
                    Err(error) => errors.push(PackageInvariantError::Corpus {
                        path: adversarial_path.into(),
                        line,
                        message: format!("`{label}` failed generation: {error}"),
                    }),
                },
                Err(error) => errors.push(PackageInvariantError::Corpus {
                    path: adversarial_path.into(),
                    line,
                    message: format!("`{label}` failed to parse: {error}"),
                }),
            },
            other => errors.push(PackageInvariantError::Corpus {
                path: adversarial_path.into(),
                line,
                message: format!("unknown adversarial corpus mode `{other}`"),
            }),
        }
    }
    report.adversarial_entries = adversarial_entries;

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn read_corpus_source(path: &Path, errors: &mut Vec<PackageInvariantError>) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(source) => Some(source),
        Err(error) => {
            errors.push(PackageInvariantError::Corpus {
                path: path.display().to_string(),
                line: 0,
                message: error.to_string(),
            });
            None
        }
    }
}

fn parse_corpus(source: &str) -> Vec<(usize, Vec<String>)> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line = line.trim_end();
            if line.is_empty() || line.trim_start().starts_with('#') {
                return None;
            }
            Some((
                index + 1,
                line.split('\t').map(ToOwned::to_owned).collect::<Vec<_>>(),
            ))
        })
        .collect()
}
