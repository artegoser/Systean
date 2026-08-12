use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::discourse::{DiscourseResolutionError, DiscourseState, IntroductionOrigin};
use crate::language::{LanguageError, LanguagePackage};
use crate::semantics::Term;
use crate::spec::{
    CompiledSymbolKind, CompiledTerm, TypedSemanticPackage, legacy_typed_type, lower_term, parse_term,
};

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentationDeclarationKind {
    Primitive,
    Defined,
    IntrinsicBacked,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DocumentationParameter {
    pub name: String,
    pub ty: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DocumentationExample {
    pub source: String,
    pub canonical_surface: String,
    pub canonical_semantics: String,
    pub english: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DocumentationEntry {
    pub root: String,
    pub gloss: String,
    pub explain: String,
    pub declaration_kind: DocumentationDeclarationKind,
    pub signature: String,
    pub parameters: Vec<DocumentationParameter>,
    pub result_type: String,
    pub pronunciation: String,
    pub stressed_pronunciation: String,
    pub tags: Vec<String>,
    pub examples: Vec<DocumentationExample>,
    pub source: String,
    pub source_line: usize,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DocumentationSearchEntry {
    pub root: String,
    pub gloss: String,
    pub explain: String,
    pub declaration_kind: DocumentationDeclarationKind,
    pub result_type: String,
    pub argument_count: usize,
    pub argument_types: Vec<String>,
    pub tags: Vec<String>,
    pub examples: Vec<String>,
    pub source: String,
    pub source_line: usize,
}

#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub struct DocumentationPackage {
    fingerprint: String,
    entries: BTreeMap<String, DocumentationEntry>,
}

impl DocumentationPackage {
    pub fn empty() -> Self {
        Self {
            fingerprint: stable_digest(&[]),
            entries: BTreeMap::new(),
        }
    }

    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    pub fn entry(&self, root: &str) -> Option<&DocumentationEntry> {
        self.entries.get(root)
    }

    pub fn entries(&self) -> impl Iterator<Item = &DocumentationEntry> {
        self.entries.values()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn search_index(&self) -> Vec<DocumentationSearchEntry> {
        self.entries
            .values()
            .map(|entry| DocumentationSearchEntry {
                root: entry.root.clone(),
                gloss: entry.gloss.clone(),
                explain: entry.explain.clone(),
                declaration_kind: entry.declaration_kind,
                result_type: entry.result_type.clone(),
                argument_count: entry.parameters.len(),
                argument_types: entry.parameters.iter().map(|parameter| parameter.ty.clone()).collect(),
                tags: entry.tags.clone(),
                examples: entry.examples.iter().map(|example| example.source.clone()).collect(),
                source: entry.source.clone(),
                source_line: entry.source_line,
            })
            .collect()
    }

    pub(crate) fn finalize_examples(
        mut self,
        language: &LanguagePackage,
    ) -> Result<Self, LanguageError> {
        // Documentation examples are executable language fixtures. A small, fixed
        // discourse fixture is necessary for context words, ordinary `ref`, and
        // examples whose typed target is intentionally introduced as an alias.
        // Nothing in this fixture is inferred from the host machine or current time.
        let discourse = documentation_example_discourse(language)?;
        let roots = self.entries.keys().cloned().collect::<Vec<_>>();
        for root in roots {
            let expected_examples = self
                .entries
                .get(&root)
                .expect("documentation root")
                .examples
                .iter()
                .map(|example| (example.source.clone(), example.canonical_semantics.clone()))
                .collect::<Vec<_>>();
            let mut compiled = Vec::with_capacity(expected_examples.len());
            for (source, expected_semantics) in expected_examples {
                let analysis = language
                    .analyze_surface_with_discourse(&source, &discourse)
                    .map_err(|error| {
                        LanguageError::Documentation(format!(
                            "documentation example for `{root}` does not parse and resolve: `{source}`: {error}"
                        ))
                    })?;
                if source.trim() != analysis.canonical_surface {
                    return Err(LanguageError::Documentation(format!(
                        "documentation example for `{root}` is not canonical: `{source}` -> `{}`",
                        analysis.canonical_surface
                    )));
                }
                if expected_semantics != analysis.canonical_semantics {
                    return Err(LanguageError::Documentation(format!(
                        "documentation example for `{root}` changed canonical semantics: `{source}`: expected `{expected_semantics}`, got `{}`",
                        analysis.canonical_semantics
                    )));
                }
                let roundtrip = language
                    .analyze_surface_with_discourse(&analysis.canonical_surface, &discourse)
                    .map_err(|error| {
                        LanguageError::Documentation(format!(
                            "canonical documentation example for `{root}` does not round-trip: `{source}`: {error}"
                        ))
                    })?;
                if roundtrip.canonical_semantics != analysis.canonical_semantics {
                    return Err(LanguageError::Documentation(format!(
                        "documentation example for `{root}` changed semantics on canonical round-trip: `{source}`: `{}` -> `{}`",
                        analysis.canonical_semantics, roundtrip.canonical_semantics
                    )));
                }
                let english = language
                    .render_english_term(&analysis.resolved.term)
                    .map_err(|error| {
                        LanguageError::Documentation(format!(
                            "documentation example for `{root}` cannot render to English: `{source}`: {error}"
                        ))
                    })?;
                compiled.push(DocumentationExample {
                    source,
                    canonical_surface: analysis.canonical_surface,
                    canonical_semantics: analysis.canonical_semantics,
                    english: english.text,
                });
            }
            self.entries
                .get_mut(&root)
                .expect("documentation root")
                .examples = compiled;
        }
        Ok(self)
    }
}


fn documentation_example_discourse(language: &LanguagePackage) -> Result<DiscourseState, LanguageError> {
    let mut discourse = DiscourseState::new();
    let sun = Term::Const("sol".into());

    discourse
        .set_context_value("speaker", sun.clone(), language.semantics())
        .map_err(discourse_language_error)?;
    discourse
        .set_context_value("addressee", sun.clone(), language.semantics())
        .map_err(discourse_language_error)?;

    let instant = language.analyze_surface("2026-08-12T00:00:00Z")?;
    let instant = language
        .syntax()
        .lower(&instant.syntax, language.semantics())
        .map_err(LanguageError::Syntax)?
        .term;
    discourse
        .set_context_value("now", instant, language.semantics())
        .map_err(discourse_language_error)?;

    for (alias, source) in [
        ("prun", "process_of(content = mov(mover = sol))"),
        ("habit", "activity_of(content = mov(mover = sol))"),
        ("prop", "viv(entity = sol)"),
    ] {
        let term = parse_term(source)
            .map(lower_term)
            .map_err(|errors| {
                LanguageError::Documentation(format!(
                    "internal Phase 20 documentation fixture `{source}` is invalid: {}",
                    errors
                        .into_iter()
                        .map(|error| error.to_string())
                        .collect::<Vec<_>>()
                        .join("; ")
                ))
            })?;
        language.define_alias(&mut discourse, alias, term)?;
    }

    discourse
        .introduce(
            sun,
            IntroductionOrigin::External {
                label: "Phase 20 documentation referent".into(),
            },
            language.semantics(),
        )
        .map_err(discourse_language_error)?;
    Ok(discourse)
}

fn discourse_language_error(error: crate::discourse::DiscourseError) -> LanguageError {
    LanguageError::Discourse(DiscourseResolutionError::Discourse(error))
}

#[derive(Clone, Debug)]
struct RawDocumentationEntry {
    root: String,
    gloss: String,
    explain: String,
    tags: Vec<String>,
    examples: Vec<RawDocumentationExample>,
    source: String,
    source_line: usize,
}

#[derive(Clone, Debug)]
struct RawDocumentationExample {
    source: String,
    expected_semantics: String,
}

pub(crate) fn compile_directory(
    root: &Path,
    semantics: &TypedSemanticPackage,
    language: &LanguagePackage,
) -> Result<DocumentationPackage, LanguageError> {
    if !root.exists() {
        return Err(LanguageError::Documentation(format!(
            "English documentation directory `{}` is missing",
            root.display()
        )));
    }
    let mut paths = Vec::new();
    collect_sources(root, &mut paths)?;
    if paths.is_empty() {
        return Err(LanguageError::Documentation(format!(
            "English documentation directory `{}` contains no .sydoc files",
            root.display()
        )));
    }
    let mut sources = Vec::new();
    for path in paths {
        let text = fs::read_to_string(&path).map_err(|error| LanguageError::Io {
            path: path.clone(),
            message: error.to_string(),
        })?;
        let relative = path
            .strip_prefix(root.parent().unwrap_or(root))
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        sources.push((relative, text));
    }
    compile_sources(sources, semantics, language)
}

pub(crate) fn compile_sources(
    sources: impl IntoIterator<Item = (String, String)>,
    semantics: &TypedSemanticPackage,
    language: &LanguagePackage,
) -> Result<DocumentationPackage, LanguageError> {
    let mut source_values = sources.into_iter().collect::<Vec<_>>();
    source_values.sort_by(|left, right| left.0.cmp(&right.0));
    let fingerprint = documentation_fingerprint(&source_values);

    let mut raw_entries = BTreeMap::new();
    for (source, text) in &source_values {
        for entry in Parser::new(source, text).parse()? {
            let root = entry.root.clone();
            if raw_entries.insert(root.clone(), entry).is_some() {
                return Err(LanguageError::Documentation(format!(
                    "duplicate English documentation entry for `{root}`"
                )));
            }
        }
    }

    let public_roots = semantics
        .symbols()
        .filter(|symbol| symbol.kind == CompiledSymbolKind::Word)
        .filter_map(|symbol| semantics.source_name_for_symbol(symbol.id).map(str::to_owned))
        .collect::<BTreeSet<_>>();
    let documented_roots = raw_entries.keys().cloned().collect::<BTreeSet<_>>();
    let missing = public_roots
        .difference(&documented_roots)
        .cloned()
        .collect::<Vec<_>>();
    let unknown = documented_roots
        .difference(&public_roots)
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() || !unknown.is_empty() {
        return Err(LanguageError::Documentation(format!(
            "English documentation coverage mismatch; missing public roots: {missing:?}; unknown roots: {unknown:?}"
        )));
    }

    let mut entries = BTreeMap::new();
    for (root, raw) in raw_entries {
        let symbol_id = semantics.symbol_id(&root).ok_or_else(|| {
            LanguageError::Documentation(format!("unknown documented root `{root}`"))
        })?;
        let symbol = semantics.symbol(symbol_id).expect("documented word symbol");
        let debug = semantics.debug_symbol(symbol_id).ok_or_else(|| {
            LanguageError::Documentation(format!("documented root `{root}` has no debug signature"))
        })?;
        let declaration_kind = if symbol.definition.as_ref().is_some_and(|term| contains_intrinsic(term, semantics)) {
            DocumentationDeclarationKind::IntrinsicBacked
        } else if symbol.definition.is_some() {
            DocumentationDeclarationKind::Defined
        } else {
            DocumentationDeclarationKind::Primitive
        };
        let parameters = symbol
            .signature
            .parameters
            .iter()
            .enumerate()
            .map(|(index, ty)| DocumentationParameter {
                name: debug
                    .parameter_names
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| format!("arg{}", index + 1)),
                ty: legacy_typed_type(semantics, ty).to_string(),
            })
            .collect::<Vec<_>>();
        let result_type = legacy_typed_type(semantics, &symbol.signature.returns).to_string();
        let signature = render_signature(&root, symbol.signature.type_parameter_count, &parameters, &result_type);
        let word = language.analyze_word(&root)?;
        entries.insert(
            root.clone(),
            DocumentationEntry {
                root,
                gloss: raw.gloss,
                explain: raw.explain,
                declaration_kind,
                signature,
                parameters,
                result_type,
                pronunciation: word.phonology.pronunciation,
                stressed_pronunciation: word.phonology.stressed_pronunciation,
                tags: raw.tags,
                examples: raw
                    .examples
                    .into_iter()
                    .map(|example| DocumentationExample {
                        source: example.source,
                        canonical_surface: String::new(),
                        canonical_semantics: example.expected_semantics,
                        english: String::new(),
                    })
                    .collect(),
                source: raw.source,
                source_line: raw.source_line,
            },
        );
    }

    Ok(DocumentationPackage { fingerprint, entries })
}

fn render_signature(
    root: &str,
    type_parameter_count: u32,
    parameters: &[DocumentationParameter],
    result_type: &str,
) -> String {
    let generics = if type_parameter_count == 0 {
        String::new()
    } else {
        let names = (0..type_parameter_count)
            .map(|index| format!("T{}", index + 1))
            .collect::<Vec<_>>()
            .join(", ");
        format!("<{names}>")
    };
    if parameters.is_empty() {
        return format!("{root}{generics} : {result_type}");
    }
    let parameters = parameters
        .iter()
        .map(|parameter| format!("{}: {}", parameter.name, parameter.ty))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{root}{generics}({parameters}) -> {result_type}")
}

fn contains_intrinsic(term: &CompiledTerm, semantics: &TypedSemanticPackage) -> bool {
    match term {
        CompiledTerm::Symbol(symbol) => semantics.intrinsic_id(*symbol).is_some(),
        CompiledTerm::Apply { function, arguments } => {
            semantics.intrinsic_id(*function).is_some()
                || arguments.iter().any(|argument| contains_intrinsic(argument, semantics))
        }
        CompiledTerm::Invoke { function, arguments } => {
            contains_intrinsic(function, semantics)
                || arguments.iter().any(|argument| contains_intrinsic(argument, semantics))
        }
        CompiledTerm::Constructor { fields, .. } => {
            fields.iter().any(|field| contains_intrinsic(field, semantics))
        }
        CompiledTerm::Lambda { body, .. } => contains_intrinsic(body, semantics),
        CompiledTerm::Context(_)
        | CompiledTerm::Unit(_)
        | CompiledTerm::Bound(_)
        | CompiledTerm::Scalar(_) => false,
    }
}

fn collect_sources(current: &Path, output: &mut Vec<PathBuf>) -> Result<(), LanguageError> {
    let entries = fs::read_dir(current).map_err(|error| LanguageError::Io {
        path: current.to_path_buf(),
        message: error.to_string(),
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| LanguageError::Io {
            path: current.to_path_buf(),
            message: error.to_string(),
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| LanguageError::Io {
            path: path.clone(),
            message: error.to_string(),
        })?;
        if file_type.is_dir() {
            collect_sources(&path, output)?;
        } else if file_type.is_file() && path.extension().and_then(|value| value.to_str()) == Some("sydoc") {
            output.push(path);
        }
    }
    output.sort();
    Ok(())
}

fn documentation_fingerprint(sources: &[(String, String)]) -> String {
    let mut aggregate = Vec::new();
    for (path, source) in sources {
        aggregate.extend_from_slice(path.as_bytes());
        aggregate.push(0);
        aggregate.extend_from_slice(source.as_bytes());
        aggregate.push(0xff);
    }
    stable_digest(&aggregate)
}

fn stable_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

struct Parser<'a> {
    source_name: &'a str,
    source: &'a str,
    offset: usize,
}

impl<'a> Parser<'a> {
    fn new(source_name: &'a str, source: &'a str) -> Self {
        Self { source_name, source, offset: 0 }
    }

    fn parse(mut self) -> Result<Vec<RawDocumentationEntry>, LanguageError> {
        let mut entries = Vec::new();
        self.skip_trivia();
        while !self.eof() {
            let line = self.line();
            self.expect_keyword("word")?;
            self.skip_trivia();
            let root = self.ident()?;
            self.skip_trivia();
            self.expect_char('{')?;
            let mut gloss = None;
            let mut explain = None;
            let mut tags = Vec::new();
            let mut example_sources = Vec::new();
            let mut example_semantics = Vec::new();
            loop {
                self.skip_trivia();
                if self.consume_char('}') {
                    break;
                }
                let field = self.ident()?;
                self.skip_trivia();
                match field.as_str() {
                    "gloss" => set_once(&mut gloss, self.string()?, &root, "gloss")?,
                    "explain" => set_once(&mut explain, self.string()?, &root, "explain")?,
                    "example" => example_sources.push(self.string()?),
                    "expect_semantics" => example_semantics.push(self.string()?),
                    "tag" => tags.push(self.string()?),
                    unknown => {
                        return Err(self.error(format!(
                            "unsupported documentation field `{unknown}` in word `{root}`"
                        )))
                    }
                }
                self.skip_trivia();
                self.expect_char(';')?;
            }
            let gloss = gloss
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| self.error(format!("word `{root}` requires a non-empty gloss")))?;
            let explain = explain
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| self.error(format!("word `{root}` requires a non-empty explain field")))?;
            if example_sources.is_empty() {
                return Err(self.error(format!("word `{root}` requires at least one example")));
            }
            if example_sources.iter().any(|example| example.trim().is_empty()) {
                return Err(self.error(format!("word `{root}` contains an empty example")));
            }
            if example_sources.len() != example_semantics.len() {
                return Err(self.error(format!(
                    "word `{root}` requires one `expect_semantics` field for every example"
                )));
            }
            if example_semantics.iter().any(|semantics| semantics.trim().is_empty()) {
                return Err(self.error(format!(
                    "word `{root}` contains an empty expected semantic value"
                )));
            }
            let examples = example_sources
                .into_iter()
                .zip(example_semantics)
                .map(|(source, expected_semantics)| RawDocumentationExample {
                    source,
                    expected_semantics: expected_semantics.trim().to_owned(),
                })
                .collect();
            tags.sort();
            tags.dedup();
            entries.push(RawDocumentationEntry {
                root,
                gloss,
                explain,
                tags,
                examples,
                source: self.source_name.to_owned(),
                source_line: line,
            });
            self.skip_trivia();
        }
        Ok(entries)
    }

    fn string(&mut self) -> Result<String, LanguageError> {
        self.skip_trivia();
        if self.remaining().starts_with("\"\"\"") {
            self.offset += 3;
            let rest = self.remaining();
            let Some(end) = rest.find("\"\"\"") else {
                return Err(self.error("unterminated triple-quoted string"));
            };
            let value = rest[..end].to_owned();
            self.offset += end + 3;
            return Ok(dedent(&value));
        }
        self.expect_char('"')?;
        let mut value = String::new();
        while !self.eof() {
            let ch = self.next_char().expect("string character");
            match ch {
                '"' => return Ok(value),
                '\\' => {
                    let Some(escaped) = self.next_char() else {
                        return Err(self.error("unterminated string escape"));
                    };
                    value.push(match escaped {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '"' => '"',
                        '\\' => '\\',
                        other => {
                            return Err(self.error(format!("unsupported string escape `\\{other}`")))
                        }
                    });
                }
                other => value.push(other),
            }
        }
        Err(self.error("unterminated string"))
    }

    fn ident(&mut self) -> Result<String, LanguageError> {
        self.skip_trivia();
        let start = self.offset;
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                self.offset += ch.len_utf8();
            } else {
                break;
            }
        }
        if self.offset == start {
            return Err(self.error("expected identifier"));
        }
        Ok(self.source[start..self.offset].to_owned())
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<(), LanguageError> {
        let actual = self.ident()?;
        if actual == keyword {
            Ok(())
        } else {
            Err(self.error(format!("expected `{keyword}`, found `{actual}`")))
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), LanguageError> {
        self.skip_trivia();
        if self.consume_char(expected) {
            Ok(())
        } else {
            Err(self.error(format!("expected `{expected}`")))
        }
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.offset += expected.len_utf8();
            true
        } else {
            false
        }
    }

    fn skip_trivia(&mut self) {
        loop {
            while self.peek_char().is_some_and(char::is_whitespace) {
                self.next_char();
            }
            if self.remaining().starts_with("//") {
                while let Some(ch) = self.next_char() {
                    if ch == '\n' {
                        break;
                    }
                }
                continue;
            }
            if self.remaining().starts_with('#') {
                while let Some(ch) = self.next_char() {
                    if ch == '\n' {
                        break;
                    }
                }
                continue;
            }
            break;
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }

    fn peek_char(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn remaining(&self) -> &str {
        &self.source[self.offset..]
    }

    fn eof(&self) -> bool {
        self.offset >= self.source.len()
    }

    fn line(&self) -> usize {
        self.source[..self.offset].bytes().filter(|byte| *byte == b'\n').count() + 1
    }

    fn error(&self, message: impl Into<String>) -> LanguageError {
        LanguageError::Documentation(format!(
            "{}:{}: {}",
            self.source_name,
            self.line(),
            message.into()
        ))
    }
}

fn set_once(
    target: &mut Option<String>,
    value: String,
    root: &str,
    field: &str,
) -> Result<(), LanguageError> {
    if target.replace(value).is_some() {
        return Err(LanguageError::Documentation(format!(
            "word `{root}` contains duplicate `{field}` field"
        )));
    }
    Ok(())
}

fn dedent(value: &str) -> String {
    let mut lines = value.lines().collect::<Vec<_>>();
    while lines.first().is_some_and(|line| line.trim().is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }
    let indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|ch| ch.is_whitespace()).count())
        .min()
        .unwrap_or(0);
    lines
        .into_iter()
        .map(|line| line.chars().skip(indent).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::Parser;

    #[test]
    fn parser_accepts_split_friendly_word_blocks() {
        let entries = Parser::new(
            "docs/en/core.sydoc",
            r#"
            word vid {
                gloss "see";
                explain """
                    Expresses visual perception.
                    It does not imply recognition.
                """;
                tag "perception";
                example "na artemi vid na mari";
                expect_semantics """
                    vid(observed = na(payload = "mari"), observer = na(payload = "artemi"))
                """;
            }
            "#,
        )
        .parse()
        .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].root, "vid");
        assert_eq!(entries[0].gloss, "see");
        assert_eq!(entries[0].explain, "Expresses visual perception.\nIt does not imply recognition.");
        assert_eq!(entries[0].examples.len(), 1);
        assert_eq!(entries[0].examples[0].source, "na artemi vid na mari");
        assert_eq!(
            entries[0].examples[0].expected_semantics,
            "vid(observed = na(payload = \"mari\"), observer = na(payload = \"artemi\"))"
        );
    }
}
