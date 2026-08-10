use std::env;
use std::process::ExitCode;
use systean::semantics::{Checker, Explainer, canonicalize};
use systean::spec::{compile_path, lower_term, parse_term};
fn main()->ExitCode{
    let mut args=env::args().skip(1); let Some(path)=args.next() else {eprintln!("usage: systean-sem <spec-path> <semantic-expression>");return ExitCode::from(2)};
    let expression=args.collect::<Vec<_>>().join(" "); if expression.is_empty(){eprintln!("usage: systean-sem <spec-path> <semantic-expression>");return ExitCode::from(2)}
    let environment=match compile_path(&path){Ok(v)=>v,Err(errors)=>{for e in errors{eprintln!("spec error: {e}");}return ExitCode::FAILURE}};
    let term=match parse_term(&expression){Ok(v)=>lower_term(v),Err(errors)=>{for e in errors{eprintln!("term parse error: {e}");}return ExitCode::FAILURE}};
    let ty=match Checker::new(&environment).infer(&term){Ok(v)=>v,Err(e)=>{eprintln!("type error: {e}");return ExitCode::FAILURE}};
    println!("type: {ty}");println!("canonical: {}",canonicalize(&term));
    match Explainer::new(&environment).explain(&term){Ok(v)=>{println!("explanation:\n{}",v.render());ExitCode::SUCCESS},Err(e)=>{eprintln!("explanation error: {e}");ExitCode::FAILURE}}
}
