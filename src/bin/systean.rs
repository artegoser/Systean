use std::env;
use std::process::ExitCode;
use systean::phonology::{
    PhonologyConfig, RootInventory, RootIssue, RootWarning, Segmentation, SpokenForm,
    segment_spoken_stream,
};
use systean::semantics::{Checker, Explainer, canonicalize};
use systean::spec::{compile_path, lower_term, parse_term};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("check") => check_semantics(args),
        Some("explain") => explain_semantics(args),
        Some("phonology") => phonology(args),
        Some("roots") => roots(args),
        _ => {
            usage();
            ExitCode::from(2)
        }
    }
}

fn check_semantics(mut args: impl Iterator<Item = String>) -> ExitCode {
    let Some(path) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    if args.next().is_some() {
        usage();
        return ExitCode::from(2);
    }
    match compile_path(&path) {
        Ok(_) => {
            println!("semantic package OK: {path}");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            ExitCode::FAILURE
        }
    }
}

fn explain_semantics(mut args: impl Iterator<Item = String>) -> ExitCode {
    let Some(path) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    let expression = args.collect::<Vec<_>>().join(" ");
    if expression.is_empty() {
        usage();
        return ExitCode::from(2);
    }
    let environment = match compile_path(&path) {
        Ok(value) => value,
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            return ExitCode::FAILURE;
        }
    };
    let term = match parse_term(&expression) {
        Ok(value) => lower_term(value),
        Err(errors) => {
            for error in errors {
                eprintln!("term parse error: {error}");
            }
            return ExitCode::FAILURE;
        }
    };
    let ty = match Checker::new(&environment).infer(&term) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("type error: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!("type: {ty}");
    println!("canonical: {}", canonicalize(&term));
    match Explainer::new(&environment).explain(&term) {
        Ok(value) => {
            println!("explanation:\n{}", value.render());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("explanation error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn phonology(mut args: impl Iterator<Item = String>) -> ExitCode {
    match args.next().as_deref() {
        Some("check") => phonology_check(args),
        Some("pronounce") => phonology_pronounce(args),
        Some("spell") => phonology_spell(args),
        Some("analyze") => phonology_analyze(args),
        _ => {
            usage();
            ExitCode::from(2)
        }
    }
}

fn phonology_check(mut args: impl Iterator<Item = String>) -> ExitCode {
    let Some(alphabet_path) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    let Some(rules_path) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    if args.next().is_some() {
        usage();
        return ExitCode::from(2);
    }
    let phonology = match load_phonology(&alphabet_path, &rules_path) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let vowels = phonology
        .alphabet
        .letters()
        .iter()
        .filter(|letter| letter.kind.to_string() == "vowel")
        .count();
    println!(
        "phonology OK: {} letters ({} vowels, {} consonants)",
        phonology.alphabet.letters().len(),
        vowels,
        phonology.alphabet.letters().len() - vowels
    );
    println!("stress: first syllable of lexical root");
    ExitCode::SUCCESS
}

fn phonology_pronounce(mut args: impl Iterator<Item = String>) -> ExitCode {
    let Some(alphabet_path) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    let text = args.collect::<Vec<_>>().join(" ");
    if text.is_empty() {
        usage();
        return ExitCode::from(2);
    }
    let alphabet = match systean::phonology::Alphabet::load(&alphabet_path) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("alphabet error: {error}");
            return ExitCode::FAILURE;
        }
    };
    match alphabet.pronounce(&text) {
        Ok(pronunciation) => {
            println!("/{pronunciation}/");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("pronunciation error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn phonology_spell(mut args: impl Iterator<Item = String>) -> ExitCode {
    let Some(alphabet_path) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    let pronunciation = args.collect::<Vec<_>>().join(" ");
    if pronunciation.is_empty() {
        usage();
        return ExitCode::from(2);
    }
    let alphabet = match systean::phonology::Alphabet::load(&alphabet_path) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("alphabet error: {error}");
            return ExitCode::FAILURE;
        }
    };
    match alphabet.spell(&pronunciation) {
        Ok(spelling) => {
            println!("{spelling}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("spelling error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn phonology_analyze(mut args: impl Iterator<Item = String>) -> ExitCode {
    let Some(alphabet_path) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    let Some(rules_path) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    let Some(word) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    let tail = args.collect::<Vec<_>>();
    let root = match tail.as_slice() {
        [] => None,
        [flag, root] if flag == "--root" => Some(root.as_str()),
        _ => {
            usage();
            return ExitCode::from(2);
        }
    };
    let phonology = match load_phonology(&alphabet_path, &rules_path) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let analysis = match root {
        Some(root) => phonology.analyze_word_with_root_text(&word, root),
        None => phonology.analyze_root(&word),
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
        Err(error) => {
            eprintln!("phonology error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn roots(mut args: impl Iterator<Item = String>) -> ExitCode {
    match args.next().as_deref() {
        Some("check") => root_check(args),
        Some("segment") => roots_segment(args),
        _ => {
            usage();
            ExitCode::from(2)
        }
    }
}

fn root_check(mut args: impl Iterator<Item = String>) -> ExitCode {
    let Some(alphabet_path) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    let Some(rules_path) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    let Some(dictionary_path) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    let Some(candidate) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    if args.next().is_some() {
        usage();
        return ExitCode::from(2);
    }
    let phonology = match load_phonology(&alphabet_path, &rules_path) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let inventory = match RootInventory::load_dictionary(&dictionary_path) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("dictionary error: {error}");
            return ExitCode::FAILURE;
        }
    };
    let check = phonology.check_root(&candidate, &inventory);
    println!("root: {}", check.canonical_root);
    if !check.pronunciation.is_empty() {
        println!("pronunciation: /{}/", check.pronunciation);
    }
    for issue in &check.issues {
        match issue {
            RootIssue::Invalid(error) => eprintln!("error: {error}"),
            RootIssue::ExistingSpelling(root) => {
                eprintln!("error: root `{root}` already exists")
            }
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

fn roots_segment(mut args: impl Iterator<Item = String>) -> ExitCode {
    let Some(alphabet_path) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    let Some(rules_path) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    let Some(dictionary_path) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    let Some(stream) = args.next() else {
        usage();
        return ExitCode::from(2);
    };
    if args.next().is_some() {
        usage();
        return ExitCode::from(2);
    }
    let phonology = match load_phonology(&alphabet_path, &rules_path) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let inventory = match RootInventory::load_dictionary(&dictionary_path) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("dictionary error: {error}");
            return ExitCode::FAILURE;
        }
    };
    let forms = inventory
        .roots()
        .iter()
        .filter_map(|root| {
            phonology.analyze_root(root).ok().map(|analysis| SpokenForm {
                label: root.clone(),
                pronunciation: analysis.pronunciation,
            })
        })
        .collect::<Vec<_>>();
    match segment_spoken_stream(&stream, &forms) {
        Segmentation::Impossible => {
            println!("no segmentation");
            ExitCode::FAILURE
        }
        Segmentation::Unique(path) => {
            println!("unique: {}", path.join(" | "));
            ExitCode::SUCCESS
        }
        Segmentation::Ambiguous { first, second } => {
            println!("ambiguous:");
            println!("  1: {}", first.join(" | "));
            println!("  2: {}", second.join(" | "));
            ExitCode::FAILURE
        }
    }
}

fn load_phonology(alphabet_path: &str, rules_path: &str) -> Result<PhonologyConfig, ExitCode> {
    PhonologyConfig::load(alphabet_path, rules_path).map_err(|error| {
        eprintln!("phonology config error: {error}");
        ExitCode::FAILURE
    })
}

fn usage() {
    eprintln!(
        "usage:\n  \
         systean check <semantic-spec-path>\n  \
         systean explain <semantic-spec-path> <semantic-expression>\n  \
         systean phonology check <alphabet.toml> <phonology.toml>\n  \
         systean phonology pronounce <alphabet.toml> <text>\n  \
         systean phonology spell <alphabet.toml> <pronunciation>\n  \
         systean phonology analyze <alphabet.toml> <phonology.toml> <word> [--root <root>]\n  \
         systean roots check <alphabet.toml> <phonology.toml> <dictionary.toml> <root>\n  \
         systean roots segment <alphabet.toml> <phonology.toml> <dictionary.toml> <phoneme-stream>"
    );
}
