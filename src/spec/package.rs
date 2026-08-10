use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use crate::semantics::Environment;
use super::{CompileError, ParseError, SourceSpecification, compile_specifications, parse_specification};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageError {
    Io { path: PathBuf, message: String },
    NoSemanticSpecifications(PathBuf),
    Parse { source: String, error: ParseError },
    Compile(CompileError),
}

pub fn compile_path(path: impl AsRef<Path>) -> Result<Environment, Vec<PackageError>> {
    let sources = load_specifications(path)?;
    compile_specifications(&sources).map_err(|errors| errors.into_iter().map(PackageError::Compile).collect())
}

pub fn load_specifications(path: impl AsRef<Path>) -> Result<Vec<SourceSpecification>, Vec<PackageError>> {
    let path = path.as_ref();
    let files = match semantic_files(path) { Ok(files) => files, Err(error) => return Err(vec![error]) };
    if files.is_empty() { return Err(vec![PackageError::NoSemanticSpecifications(path.to_path_buf())]); }
    let mut sources = Vec::new();
    let mut errors = Vec::new();
    for file in files {
        let source_name = file.to_string_lossy().into_owned();
        let text = match fs::read_to_string(&file) {
            Ok(text) => text,
            Err(error) => { errors.push(PackageError::Io { path: file, message: error.to_string() }); continue; }
        };
        match parse_specification(&text) {
            Ok(specification) => sources.push(SourceSpecification::new(source_name, specification)),
            Err(parse_errors) => errors.extend(parse_errors.into_iter().map(|error| PackageError::Parse { source: source_name.clone(), error })),
        }
    }
    if errors.is_empty() { Ok(sources) } else { Err(errors) }
}

fn semantic_files(path: &Path) -> Result<Vec<PathBuf>, PackageError> {
    let metadata = fs::metadata(path).map_err(|error| PackageError::Io { path: path.to_path_buf(), message: error.to_string() })?;
    let mut files = Vec::new();
    if metadata.is_file() {
        if path.extension().and_then(|e| e.to_str()) == Some("semsys") { files.push(path.to_path_buf()); }
    } else if metadata.is_dir() { collect_semantic_files(path, &mut files)?; }
    files.sort();
    Ok(files)
}

fn collect_semantic_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), PackageError> {
    let entries = fs::read_dir(directory).map_err(|error| PackageError::Io { path: directory.to_path_buf(), message: error.to_string() })?;
    for entry in entries {
        let entry = entry.map_err(|error| PackageError::Io { path: directory.to_path_buf(), message: error.to_string() })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| PackageError::Io { path: path.clone(), message: error.to_string() })?;
        if file_type.is_dir() { collect_semantic_files(&path, files)?; }
        else if file_type.is_file() && path.extension().and_then(|e| e.to_str()) == Some("semsys") { files.push(path); }
    }
    Ok(())
}

impl fmt::Display for PackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, message } => write!(f, "{}: {message}", path.display()),
            Self::NoSemanticSpecifications(path) => write!(f, "{} contains no `.semsys` semantic specification files", path.display()),
            Self::Parse { source, error } => write!(f, "{source}: {error}"),
            Self::Compile(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for PackageError {}
