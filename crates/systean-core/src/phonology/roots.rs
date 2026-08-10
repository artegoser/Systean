use super::{PhonologyConfig, PhonologyError};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RootIssue {
    Invalid(PhonologyError),
    ExistingSpelling(String),
    ExistingPronunciation(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RootWarning {
    SimilarSpelling { root: String, distance: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootCheck {
    pub canonical_root: String,
    pub pronunciation: String,
    pub issues: Vec<RootIssue>,
    pub warnings: Vec<RootWarning>,
}

impl RootCheck {
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InventoryError {
    Io(String),
    Toml(String),
    InvalidRoot(String),
}

impl fmt::Display for InventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Toml(error) => write!(f, "TOML error: {error}"),
            Self::InvalidRoot(root) => write!(f, "invalid root key `{root}` in dictionary"),
        }
    }
}

impl std::error::Error for InventoryError {}

#[derive(Clone, Debug, Default)]
pub struct RootInventory {
    roots: Vec<String>,
}

impl RootInventory {
    pub fn new(roots: impl IntoIterator<Item = String>) -> Self {
        let mut roots = roots.into_iter().map(|root| root.to_lowercase()).collect::<Vec<_>>();
        roots.sort();
        roots.dedup();
        Self { roots }
    }

    pub fn load_dictionary(path: impl AsRef<Path>) -> Result<Self, InventoryError> {
        let source = fs::read_to_string(path.as_ref()).map_err(|error| InventoryError::Io(error.to_string()))?;
        Self::from_dictionary_toml(&source)
    }

    pub fn from_dictionary_toml(source: &str) -> Result<Self, InventoryError> {
        let value: toml::Value = toml::from_str(source).map_err(|error| InventoryError::Toml(error.to_string()))?;
        let table = value
            .as_table()
            .ok_or_else(|| InventoryError::Toml("dictionary root must be a TOML table".into()))?;
        let roots = table
            .keys()
            .filter(|key| key.as_str() != "meta")
            .map(|key| {
                if key.trim().is_empty() {
                    Err(InventoryError::InvalidRoot(key.clone()))
                } else {
                    Ok(key.to_lowercase())
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::new(roots))
    }

    pub fn roots(&self) -> &[String] {
        &self.roots
    }
}

impl PhonologyConfig {
    pub fn check_root(&self, candidate: &str, inventory: &RootInventory) -> RootCheck {
        let canonical_root = candidate.to_lowercase();
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        let candidate_analysis = match self.analyze_root(candidate) {
            Ok(analysis) => Some(analysis),
            Err(error) => {
                issues.push(RootIssue::Invalid(error));
                None
            }
        };
        let pronunciation = candidate_analysis
            .as_ref()
            .map(|analysis| analysis.pronunciation.clone())
            .unwrap_or_default();

        if inventory.roots.iter().any(|root| root == &canonical_root) {
            issues.push(RootIssue::ExistingSpelling(canonical_root.clone()));
        }

        if let Some(candidate_analysis) = &candidate_analysis {
            let existing_by_pronunciation = inventory
                .roots
                .iter()
                .filter_map(|root| self.analyze_root(root).ok().map(|analysis| (root, analysis)))
                .find(|(_, analysis)| analysis.pronunciation == candidate_analysis.pronunciation);
            if let Some((root, _)) = existing_by_pronunciation {
                if root != &canonical_root {
                    issues.push(RootIssue::ExistingPronunciation(root.clone()));
                }
            }
        }

        let threshold = self.roots.similarity_warning_distance;
        if threshold > 0 {
            for root in &inventory.roots {
                if root == &canonical_root {
                    continue;
                }
                let distance = levenshtein_graphemes(&canonical_root, root);
                if distance <= threshold {
                    warnings.push(RootWarning::SimilarSpelling {
                        root: root.clone(),
                        distance,
                    });
                }
            }
        }

        warnings.sort_by(|left, right| warning_key(left).cmp(&warning_key(right)));
        RootCheck {
            canonical_root,
            pronunciation,
            issues,
            warnings,
        }
    }
}

fn warning_key(warning: &RootWarning) -> (usize, &str) {
    match warning {
        RootWarning::SimilarSpelling { root, distance } => (*distance, root),
    }
}

fn levenshtein_graphemes(left: &str, right: &str) -> usize {
    let left = left.chars().collect::<Vec<_>>();
    let right = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    for (left_index, left_character) in left.iter().enumerate() {
        let mut current = vec![left_index + 1];
        for (right_index, right_character) in right.iter().enumerate() {
            let substitution = previous[right_index] + usize::from(left_character != right_character);
            let insertion = current[right_index] + 1;
            let deletion = previous[right_index + 1] + 1;
            current.push(substitution.min(insertion).min(deletion));
        }
        previous = current;
    }
    previous[right.len()]
}

pub fn roots_by_pronunciation(
    phonology: &PhonologyConfig,
    inventory: &RootInventory,
) -> BTreeMap<String, Vec<String>> {
    let mut output = BTreeMap::<String, Vec<String>>::new();
    for root in inventory.roots() {
        if let Ok(analysis) = phonology.analyze_root(root) {
            output
                .entry(analysis.pronunciation)
                .or_default()
                .push(root.clone());
        }
    }
    output
}
