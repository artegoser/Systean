use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Origin {
    pub source: String,
    pub declaration: usize,
}

impl Origin {
    pub fn new(source: impl Into<String>, declaration: usize) -> Self {
        Self { source: source.into(), declaration }
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.source, self.declaration)
    }
}
