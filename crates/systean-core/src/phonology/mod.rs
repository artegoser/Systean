mod analysis;
mod config;
mod roots;
mod segmentation;

pub use analysis::{Grapheme, PhonologyError, Syllable, WordAnalysis};
pub use config::{
    Alphabet, ConfigError, Letter, LetterKind, PhonologyConfig, RootRules, StressRules,
    StressStrategy, SyllableRules,
};
pub use roots::{
    InventoryError, RootCheck, RootInventory, RootIssue, RootWarning, roots_by_pronunciation,
};
pub use segmentation::{Segmentation, SpokenForm, segment_spoken_stream};
