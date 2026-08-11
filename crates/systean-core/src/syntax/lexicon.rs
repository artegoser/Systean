use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LexemeConfig {
    Atom {
        semantic: String,
    },
    Reference,
    Information {
        status: String,
        knower_type: Option<String>,
    },
    Alias {
        name: String,
        ty: String,
    },
    Context {
        key: String,
        ty: String,
    },
    Class {
        semantic: String,
        role: String,
    },
    Predicate {
        semantic: String,
        primary_role: Option<String>,
        rest_roles: Vec<String>,
    },
    Prefix {
        semantic: String,
        role: String,
    },
    Infix {
        semantic: String,
        left_role: String,
        right_role: String,
    },
    Quantifier {
        semantic: String,
        binder_role: String,
        variable_type: String,
        restriction_operator: String,
        restriction_role: String,
        body_role: String,
    },
    CountedQuantifier {
        semantic: String,
        binder_role: String,
        count_role: String,
        variable_type: String,
        restriction_operator: String,
        restriction_role: String,
        body_role: String,
    },
    SpeechAct {
        semantic: String,
        role: String,
    },
    Name {
        semantic: String,
        role: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SurfaceLexicon {
    entries: BTreeMap<String, LexemeConfig>,
}

impl SurfaceLexicon {
    pub fn new(entries: BTreeMap<String, LexemeConfig>) -> Self {
        Self { entries }
    }

    pub fn get(&self, surface: &str) -> Option<&LexemeConfig> {
        self.entries.get(surface)
    }

    pub fn contains_key(&self, surface: &str) -> bool {
        self.entries.contains_key(surface)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &LexemeConfig)> {
        self.entries.iter()
    }

    pub fn with_aliases<I>(&self, aliases: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = (String, String)>,
    {
        let mut entries = self.entries.clone();
        for (surface, ty) in aliases {
            if entries.contains_key(&surface) {
                return Err(format!(
                    "discourse alias `{surface}` collides with an existing lexical root"
                ));
            }
            entries.insert(
                surface.clone(),
                LexemeConfig::Alias { name: surface, ty },
            );
        }
        Ok(Self { entries })
    }
}
