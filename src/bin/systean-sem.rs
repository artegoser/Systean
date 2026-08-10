use std::env;
use std::fs;
use std::process::ExitCode;

use systean::semantics::Checker;
use systean::spec::{compile_specification, lower_term, parse_specification, parse_term};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(spec_path) = args.next() else {
        eprintln!("usage: systean-sem <spec.semsys> <semantic-expression>");
        return ExitCode::from(2);
    };
    let expression = args.collect::<Vec<_>>().join(" ");
    if expression.is_empty() {
        eprintln!("usage: systean-sem <spec.semsys> <semantic-expression>");
        return ExitCode::from(2);
    }

    let source = match fs::read_to_string(&spec_path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("failed to read `{spec_path}`: {error}");
            return ExitCode::FAILURE;
        }
    };

    let specification = match parse_specification(&source) {
        Ok(specification) => specification,
        Err(errors) => {
            for error in errors {
                eprintln!("spec parse error: {error}");
            }
            return ExitCode::FAILURE;
        }
    };

    let environment = match compile_specification(&specification) {
        Ok(environment) => environment,
        Err(errors) => {
            for error in errors {
                eprintln!("spec compile error: {error}");
            }
            return ExitCode::FAILURE;
        }
    };

    let parsed = match parse_term(&expression) {
        Ok(term) => term,
        Err(errors) => {
            for error in errors {
                eprintln!("term parse error: {error}");
            }
            return ExitCode::FAILURE;
        }
    };
    let term = lower_term(parsed);

    match Checker::new(&environment).infer(&term) {
        Ok(ty) => {
            println!("term: {term:#?}");
            println!("type: {ty}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("type error: {error}");
            ExitCode::FAILURE
        }
    }
}
