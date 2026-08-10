use std::fs;
use std::path::{Path, PathBuf};
use systean_core::semantics::{Checker, Type, canonicalize};
use systean_core::spec::{lower_term, parse_term, parse_type};
fn root()->PathBuf{Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()}
fn repository()->PathBuf{root().join("../..")}
fn environment()->systean_core::semantics::Environment{
 let repo=repository();
 let sources=[
  ("language/semantics/core.semsys".to_owned(),fs::read_to_string(repo.join("language/semantics/core.semsys")).unwrap()),
  ("tests/fixtures/semantics/demo.semsys".to_owned(),fs::read_to_string(repo.join("tests/fixtures/semantics/demo.semsys")).unwrap()),
  ("tests/fixtures/semantics/corpus.semsys".to_owned(),fs::read_to_string(repo.join("tests/fixtures/semantics/corpus.semsys")).unwrap()),
 ];
 systean_core::spec::compile_sources(sources).expect("semantic fixtures should compile")
}
fn data_lines(path:&Path)->Vec<(usize,String)>{fs::read_to_string(path).unwrap().lines().enumerate().filter_map(|(i,l)|{let l=l.trim();(!l.is_empty()&&!l.starts_with('#')).then(||(i+1,l.to_owned()))}).collect()}
#[test]
fn everyday_semantic_corpus_type_checks(){
 let env=environment();let checker=Checker::new(&env);let path=root().join("tests/corpus/everyday.tsv");
 for (line_number,line) in data_lines(&path){let fields=line.splitn(3,'\t').collect::<Vec<_>>();assert_eq!(fields.len(),3,"{}:{line_number}",path.display());let [label,expected,expression]=fields.as_slice() else{unreachable!()};let expected:Type=parse_type(expected).unwrap_or_else(|e|panic!("{label}: {e:?}"));let term=lower_term(parse_term(expression).unwrap_or_else(|e|panic!("{label}: {e:?}")));let actual=checker.infer(&term).unwrap_or_else(|e|panic!("{label}: {e}"));assert_eq!(actual,expected,"{label}");}
}
#[test]
fn adversarial_pairs_remain_canonically_distinct(){
 let env=environment();let checker=Checker::new(&env);let path=root().join("tests/corpus/ambiguity.tsv");
 for (_,line) in data_lines(&path){let fields=line.splitn(3,'\t').collect::<Vec<_>>();let [label,left,right]=fields.as_slice() else{panic!("bad row")};let left=lower_term(parse_term(left).unwrap());let right=lower_term(parse_term(right).unwrap());checker.infer(&left).unwrap();checker.infer(&right).unwrap();assert_ne!(canonicalize(&left),canonicalize(&right),"{label}");}
}
#[test]
fn representation_equivalences_share_one_canonical_ir(){
 let env=environment();let checker=Checker::new(&env);let path=root().join("tests/corpus/equivalence.tsv");
 for (_,line) in data_lines(&path){let fields=line.splitn(3,'\t').collect::<Vec<_>>();let [label,left,right]=fields.as_slice() else{panic!("bad row")};let left=lower_term(parse_term(left).unwrap());let right=lower_term(parse_term(right).unwrap());checker.infer(&left).unwrap();checker.infer(&right).unwrap();assert_eq!(canonicalize(&left),canonicalize(&right),"{label}");}
}
