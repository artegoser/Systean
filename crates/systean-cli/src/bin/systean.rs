use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use systean_core::language::LanguagePackage;
use systean_core::phonology::{
    LetterKind, RootIssue, RootWarning, Segmentation, SpokenForm, roots_by_pronunciation,
    segment_spoken_stream,
};

fn main() -> ExitCode {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let language_path = match take_language_path(&mut args) {
        Ok(path) => path,
        Err(()) => return ExitCode::from(2),
    };
    let Some(command) = args.first().cloned() else {
        usage();
        return ExitCode::from(2);
    };
    args.remove(0);

    match command.as_str() {
        "check" => check(&language_path, args),
        "explain" => explain(&language_path, args),
        "phonology" => phonology(&language_path, args),
        "morphology" => morphology(&language_path, args),
        "syntax" => syntax(&language_path, args),
        "roots" => roots(&language_path, args),
        _ => {
            usage();
            ExitCode::from(2)
        }
    }
}

fn take_language_path(args: &mut Vec<String>) -> Result<PathBuf, ()> {
    if args.first().map(String::as_str) != Some("--language") {
        return Ok(PathBuf::from("language"));
    }
    if args.len() < 2 {
        usage();
        return Err(());
    }
    let path = PathBuf::from(args[1].clone());
    args.drain(0..2);
    Ok(path)
}

fn load_language(path: &Path) -> Result<LanguagePackage, ExitCode> {
    LanguagePackage::load(path).map_err(|error| {
        eprintln!("language error: {error}");
        ExitCode::FAILURE
    })
}

fn check(language_path: &Path, args: Vec<String>) -> ExitCode {
    if !args.is_empty() {
        usage();
        return ExitCode::from(2);
    }
    let language = match load_language(language_path) {
        Ok(language) => language,
        Err(code) => return code,
    };
    let vowels = language
        .phonology()
        .alphabet
        .letters()
        .iter()
        .filter(|letter| letter.kind == LetterKind::Vowel)
        .count();
    println!("language package OK: {}", language_path.display());
    println!(
        "alphabet: {} letters ({} vowels, {} consonants)",
        language.phonology().alphabet.letters().len(),
        vowels,
        language.phonology().alphabet.letters().len() - vowels
    );
    println!("roots: {}", language.roots().roots().len());
    println!("morphology: {}", language.morphology().config().strategy);
    println!("syntax: compiled ({} lexical roots)", language.syntax().lexicon().len());
    println!("semantics: compiled");
    ExitCode::SUCCESS
}

fn explain(language_path: &Path, args: Vec<String>) -> ExitCode {
    let expression = args.join(" ");
    if expression.is_empty() {
        usage();
        return ExitCode::from(2);
    }
    let language = match load_language(language_path) {
        Ok(language) => language,
        Err(code) => return code,
    };
    match language.explain(&expression) {
        Ok(analysis) => {
            println!("type: {}", analysis.inferred_type);
            println!("canonical: {}", analysis.canonical);
            println!("explanation:\n{}", analysis.explanation);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn phonology(language_path: &Path, mut args: Vec<String>) -> ExitCode {
    if args.is_empty() {
        usage();
        return ExitCode::from(2);
    }
    let command = args.remove(0);
    let language = match load_language(language_path) {
        Ok(language) => language,
        Err(code) => return code,
    };
    match command.as_str() {
        "check" => {
            if !args.is_empty() {
                usage();
                return ExitCode::from(2);
            }
            let vowels = language
                .phonology()
                .alphabet
                .letters()
                .iter()
                .filter(|letter| letter.kind == LetterKind::Vowel)
                .count();
            println!(
                "phonology OK: {} letters ({} vowels, {} consonants)",
                language.phonology().alphabet.letters().len(),
                vowels,
                language.phonology().alphabet.letters().len() - vowels
            );
            println!("stress: first syllable of lexical root");
            ExitCode::SUCCESS
        }
        "pronounce" => {
            let text = args.join(" ");
            if text.is_empty() {
                usage();
                return ExitCode::from(2);
            }
            match language.phonology().alphabet.pronounce(&text) {
                Ok(pronunciation) => {
                    println!("/{pronunciation}/");
                    ExitCode::SUCCESS
                }
                Err(error) => fail("pronunciation", error),
            }
        }
        "spell" => {
            let pronunciation = args.join(" ");
            if pronunciation.is_empty() {
                usage();
                return ExitCode::from(2);
            }
            match language.phonology().alphabet.spell(&pronunciation) {
                Ok(spelling) => {
                    println!("{spelling}");
                    ExitCode::SUCCESS
                }
                Err(error) => fail("spelling", error),
            }
        }
        "analyze" => analyze_word(&language, args),
        _ => {
            usage();
            ExitCode::from(2)
        }
    }
}

fn analyze_word(language: &LanguagePackage, args: Vec<String>) -> ExitCode {
    let Some(word) = args.first() else {
        usage();
        return ExitCode::from(2);
    };
    let root = match args.get(1..) {
        Some([]) | None => None,
        Some([flag, root]) if flag == "--root" => Some(root.as_str()),
        _ => {
            usage();
            return ExitCode::from(2);
        }
    };
    let analysis = match root {
        Some(root) => language
            .phonology()
            .analyze_word_with_root_text(word, root),
        None => language.phonology().analyze_root(word),
    };
    match analysis {
        Ok(analysis) => {
            println!("spelling: {}", analysis.canonical_spelling);
            println!("pronunciation: /{}/", analysis.pronunciation);
            println!("stressed: /{}/", analysis.stressed_pronunciation);
            println!(
                "root graphemes: {}..{}",
                analysis.root_grapheme_range.start, analysis.root_grapheme_range.end
            );
            println!("syllables:");
            for (index, syllable) in analysis.syllables.iter().enumerate() {
                let stress = if syllable.stressed { " stressed" } else { "" };
                println!(
                    "  {}: {} /{}/{}",
                    index + 1,
                    syllable.spelling,
                    syllable.pronunciation,
                    stress
                );
            }
            ExitCode::SUCCESS
        }
        Err(error) => fail("phonology", error),
    }
}


fn morphology(language_path: &Path, mut args: Vec<String>) -> ExitCode {
    if args.is_empty() {
        usage();
        return ExitCode::from(2);
    }
    let command = args.remove(0);
    let language = match load_language(language_path) {
        Ok(language) => language,
        Err(code) => return code,
    };
    match command.as_str() {
        "check" => {
            if !args.is_empty() {
                usage();
                return ExitCode::from(2);
            }
            println!("morphology OK: {}", language.morphology().config().strategy);
            println!("inflection: none");
            println!("POS/class endings: none");
            println!("wide-scope grammar inside words: none");
            ExitCode::SUCCESS
        }
        "analyze" => {
            let [word] = args.as_slice() else {
                usage();
                return ExitCode::from(2);
            };
            match language.analyze_word(word) {
                Ok(analysis) => {
                    println!("word: {}", analysis.morphology.spelling);
                    println!("root: {}", analysis.morphology.root);
                    println!("morphemes:");
                    for morpheme in &analysis.morphology.morphemes {
                        println!(
                            "  {:?}: {} [{}..{}]",
                            morpheme.kind,
                            morpheme.spelling,
                            morpheme.grapheme_range.start,
                            morpheme.grapheme_range.end
                        );
                    }
                    println!("pronunciation: /{}/", analysis.phonology.pronunciation);
                    println!("stressed: /{}/", analysis.phonology.stressed_pronunciation);
                    ExitCode::SUCCESS
                }
                Err(error) => fail("morphology", error),
            }
        }
        "generate" => {
            let [root] = args.as_slice() else {
                usage();
                return ExitCode::from(2);
            };
            match language.generate_word(root) {
                Ok(word) => {
                    println!("{word}");
                    ExitCode::SUCCESS
                }
                Err(error) => fail("morphology", error),
            }
        }
        _ => {
            usage();
            ExitCode::from(2)
        }
    }
}


fn syntax(language_path: &Path, mut args: Vec<String>) -> ExitCode {
    if args.is_empty() {
        usage();
        return ExitCode::from(2);
    }
    let command = args.remove(0);
    let language = match load_language(language_path) {
        Ok(language) => language,
        Err(code) => return code,
    };
    match command.as_str() {
        "check" => {
            if !args.is_empty() {
                usage();
                return ExitCode::from(2);
            }
            let config = language.syntax().config();
            println!("syntax OK");
            println!("frame order: {:?}", config.order.frame);
            println!("free word order: {}", config.order.free_order);
            println!("scope: {} ... {}", config.scope.open, config.scope.close);
            println!("explicit scope: {:?}", config.scope.explicit);
            println!("quantifier scope: {:?}", config.scope.quantifier_order);
            for (operator, precedence) in &config.logic.precedence {
                println!("precedence {operator}: {precedence}");
            }
            println!("lexical roots: {}", language.syntax().lexicon().len());
            ExitCode::SUCCESS
        }
        "analyze" => {
            let expression = args.join(" ");
            if expression.is_empty() {
                usage();
                return ExitCode::from(2);
            }
            match language.analyze_surface(&expression) {
                Ok(analysis) => {
                    println!("surface: {expression}");
                    println!("canonical surface: {}", analysis.canonical_surface);
                    println!("type: {}", analysis.inferred_type);
                    println!("canonical semantics: {}", analysis.canonical_semantics);
                    println!("syntax: {}", analysis.syntax);
                    ExitCode::SUCCESS
                }
                Err(error) => fail("syntax", error),
            }
        }
        _ => {
            usage();
            ExitCode::from(2)
        }
    }
}

fn roots(language_path: &Path, mut args: Vec<String>) -> ExitCode {
    if args.is_empty() {
        usage();
        return ExitCode::from(2);
    }
    let command = args.remove(0);
    let language = match load_language(language_path) {
        Ok(language) => language,
        Err(code) => return code,
    };
    match command.as_str() {
        "check" => root_check(&language, args),
        "audit" => root_audit(&language, args),
        "segment" => root_segment(&language, args),
        _ => {
            usage();
            ExitCode::from(2)
        }
    }
}

fn root_check(language: &LanguagePackage, args: Vec<String>) -> ExitCode {
    let [candidate] = args.as_slice() else {
        usage();
        return ExitCode::from(2);
    };
    let check = language.phonology().check_root(candidate, language.roots());
    println!("root: {}", check.canonical_root);
    if !check.pronunciation.is_empty() {
        println!("pronunciation: /{}/", check.pronunciation);
    }
    for issue in &check.issues {
        match issue {
            RootIssue::Invalid(error) => eprintln!("error: {error}"),
            RootIssue::ExistingSpelling(root) => eprintln!("error: root `{root}` already exists"),
            RootIssue::ExistingPronunciation(root) => {
                eprintln!("error: pronunciation collides with root `{root}`")
            }
        }
    }
    for warning in &check.warnings {
        match warning {
            RootWarning::SimilarSpelling { root, distance } => println!(
                "warning: similar to existing root `{root}` (edit distance {distance})"
            ),
        }
    }
    if check.is_valid() {
        println!("root is phonologically valid");
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn root_audit(language: &LanguagePackage, args: Vec<String>) -> ExitCode {
    if !args.is_empty() {
        usage();
        return ExitCode::from(2);
    }
    let by_pronunciation = roots_by_pronunciation(language.phonology(), language.roots());
    let mut failed = false;
    for root in language.roots().roots() {
        match language.phonology().analyze_root(root) {
            Ok(analysis) => println!(
                "ok: {} /{}/ -> /{}/",
                root, analysis.pronunciation, analysis.stressed_pronunciation
            ),
            Err(error) => {
                eprintln!("error: root `{root}`: {error}");
                failed = true;
            }
        }
    }
    for (pronunciation, roots) in by_pronunciation {
        if roots.len() > 1 {
            eprintln!(
                "error: roots {} share pronunciation /{pronunciation}/",
                roots.join(", ")
            );
            failed = true;
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        println!("root inventory OK: {} roots", language.roots().roots().len());
        ExitCode::SUCCESS
    }
}

fn root_segment(language: &LanguagePackage, args: Vec<String>) -> ExitCode {
    let pronunciation = args.join("");
    if pronunciation.is_empty() {
        usage();
        return ExitCode::from(2);
    }
    let forms = language
        .roots()
        .roots()
        .iter()
        .filter_map(|root| {
            language
                .phonology()
                .analyze_root(root)
                .ok()
                .map(|analysis| SpokenForm {
                    label: root.clone(),
                    pronunciation: analysis.pronunciation,
                })
        })
        .collect::<Vec<_>>();
    match segment_spoken_stream(&pronunciation, &forms) {
        Segmentation::Unique(parts) => {
            println!("unique: {}", parts.join(" | "));
            ExitCode::SUCCESS
        }
        Segmentation::Ambiguous { first, second } => {
            eprintln!("ambiguous spoken segmentation:");
            eprintln!("  {}", first.join(" | "));
            eprintln!("  {}", second.join(" | "));
            ExitCode::FAILURE
        }
        Segmentation::Impossible => {
            eprintln!("no valid root segmentation");
            ExitCode::FAILURE
        }
    }
}

fn fail(label: &str, error: impl std::fmt::Display) -> ExitCode {
    eprintln!("{label} error: {error}");
    ExitCode::FAILURE
}

fn usage() {
    eprintln!(
        "usage:\n  systean [--language <path>] check\n  systean [--language <path>] explain <semantic-expression>\n  systean [--language <path>] phonology check\n  systean [--language <path>] phonology pronounce <text>\n  systean [--language <path>] phonology spell <pronunciation>\n  systean [--language <path>] phonology analyze <word> [--root <root>]\n  systean [--language <path>] morphology check\n  systean [--language <path>] morphology analyze <word>\n  systean [--language <path>] morphology generate <root>\n  systean [--language <path>] syntax check\n  systean [--language <path>] syntax analyze <surface-expression>\n  systean [--language <path>] roots check <candidate>\n  systean [--language <path>] roots audit\n  systean [--language <path>] roots segment <pronunciation>"
    );
}
