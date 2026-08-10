use std::env;
use std::process::ExitCode;
use systean::semantics::{Checker, Explainer, canonicalize};
use systean::spec::{compile_path, lower_term, parse_term};

fn main()->ExitCode {
    let mut args=env::args().skip(1);
    match args.next().as_deref() { Some("check")=>check(args), Some("explain")=>explain(args), _=>{usage();ExitCode::from(2)} }
}
fn check(mut args:impl Iterator<Item=String>)->ExitCode {
    let Some(path)=args.next() else {usage();return ExitCode::from(2)};
    if args.next().is_some(){usage();return ExitCode::from(2)}
    match compile_path(&path){Ok(_)=>{println!("semantic package OK: {path}");ExitCode::SUCCESS},Err(errors)=>{for e in errors{eprintln!("{e}");}ExitCode::FAILURE}}
}
fn explain(mut args:impl Iterator<Item=String>)->ExitCode {
    let Some(path)=args.next() else {usage();return ExitCode::from(2)};
    let expression=args.collect::<Vec<_>>().join(" "); if expression.is_empty(){usage();return ExitCode::from(2)}
    let environment=match compile_path(&path){Ok(v)=>v,Err(errors)=>{for e in errors{eprintln!("{e}");}return ExitCode::FAILURE}};
    let term=match parse_term(&expression){Ok(v)=>lower_term(v),Err(errors)=>{for e in errors{eprintln!("term parse error: {e}");}return ExitCode::FAILURE}};
    let ty=match Checker::new(&environment).infer(&term){Ok(v)=>v,Err(e)=>{eprintln!("type error: {e}");return ExitCode::FAILURE}};
    println!("type: {ty}"); println!("canonical: {}",canonicalize(&term));
    match Explainer::new(&environment).explain(&term){Ok(v)=>{println!("explanation:\n{}",v.render());ExitCode::SUCCESS},Err(e)=>{eprintln!("explanation error: {e}");ExitCode::FAILURE}}
}
fn usage(){eprintln!("usage:\n  systean check <spec-path>\n  systean explain <spec-path> <semantic-expression>");}
