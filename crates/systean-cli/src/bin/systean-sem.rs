use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use systean_core::language::LanguagePackage;

fn main() -> ExitCode {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let language = if args.first().map(String::as_str) == Some("--language") {
        if args.len() < 3 {
            usage();
            return ExitCode::from(2);
        }
        let path = PathBuf::from(args[1].clone());
        args.drain(0..2);
        path
    } else {
        PathBuf::from("language")
    };
    let expression = args.join(" ");
    if expression.is_empty() {
        usage();
        return ExitCode::from(2);
    }
    let language = match LanguagePackage::load(language) {
        Ok(language) => language,
        Err(error) => {
            eprintln!("language error: {error}");
            return ExitCode::FAILURE;
        }
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

fn usage() {
    eprintln!("usage: systean-sem [--language <path>] <semantic-expression>");
}
