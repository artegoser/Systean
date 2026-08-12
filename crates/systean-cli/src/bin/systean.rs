use std::env;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use systean_core::discourse::{
    ConversationState, DiscourseState, IntroductionOrigin, ReferentId, TextRealization,
    TextSessionState, TextTurn,
};
use systean_core::language::{LanguagePackage, TextTurnAnalysis, TextTurnEvent};
use systean_core::semantics::{Checker, Term, canonicalize};
use systean_core::spec::{lower_term, parse_term, parse_type};
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
        "english" => english(&language_path, args),
        "explain" => explain(&language_path, args),
        "phonology" => phonology(&language_path, args),
        "morphology" => morphology(&language_path, args),
        "syntax" => syntax(&language_path, args),
        "literals" => literals(&language_path, args),
        "discourse" => discourse(&language_path, args),
        "roots" => roots(&language_path, args),
        "workbench" => workbench(&language_path, args),
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
    println!(
        "English documentation: {} entries ({})",
        language.documentation().len(),
        language.documentation_fingerprint()
    );
    ExitCode::SUCCESS
}

fn english(language_path: &Path, args: Vec<String>) -> ExitCode {
    let expression = args.join(" ");
    if expression.is_empty() {
        usage();
        return ExitCode::from(2);
    }
    let language = match load_language(language_path) {
        Ok(language) => language,
        Err(code) => return code,
    };
    match language.render_english(&expression) {
        Ok(rendering) => {
            println!("{}", rendering.text);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
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
            println!("scope: {} ... {}", config.scope.open, config.scope.close);
            println!("argument omission: {:?}", config.arguments.omission);
            println!("lexical roots: {}", language.syntax().lexicon().len());
            println!("surface rules: {}", language.typed_semantics().surface_rules().count());
            println!("effect programs: {}", language.typed_semantics().effect_programs().count());
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


fn literals(language_path: &Path, mut args: Vec<String>) -> ExitCode {
    if args.is_empty() {
        usage();
        return ExitCode::from(2);
    }
    let command = args.remove(0);
    let language = match load_language(language_path) {
        Ok(language) => language,
        Err(code) => return code,
    };
    let Some(engine) = language.literals() else {
        eprintln!("literal error: language package has no structured-literal codecs");
        return ExitCode::FAILURE;
    };
    match command.as_str() {
        "analyze" => {
            let source = args.join(" ");
            if source.is_empty() {
                usage();
                return ExitCode::from(2);
            }
            match engine.parse_complete(&source) {
                Ok(literal) => {
                    println!("family: {}", literal.semantic.family());
                    println!("type: {}", literal.semantic.ty);
                    println!("canonical written: {}", literal.canonical_written);
                    println!("canonical spoken: {}", literal.canonical_spoken);
                    ExitCode::SUCCESS
                }
                Err(error) => fail("literal", error),
            }
        }
        "convert" => {
            let Some(separator) = args.iter().position(|arg| arg == "--to") else {
                usage();
                return ExitCode::from(2);
            };
            if separator == 0 || separator + 2 != args.len() {
                usage();
                return ExitCode::from(2);
            }
            let source = args[..separator].join(" ");
            let target = &args[separator + 1];
            let literal = match engine.parse_complete(&source) {
                Ok(value) => value,
                Err(error) => return fail("literal", error),
            };
            match engine.convert_quantity_literal(&literal.semantic, target) {
                Ok(converted) => {
                    println!("type: {}", converted.semantic.ty);
                    println!("canonical written: {}", converted.canonical_written);
                    println!("canonical spoken: {}", converted.canonical_spoken);
                    ExitCode::SUCCESS
                }
                Err(error) => fail("literal", error),
            }
        }
        _ => {
            usage();
            ExitCode::from(2)
        }
    }
}

fn discourse(language_path: &Path, args: Vec<String>) -> ExitCode {
    if !args.is_empty() {
        usage();
        return ExitCode::from(2);
    }
    let language = match load_language(language_path) {
        Ok(language) => language,
        Err(code) => return code,
    };
    let mut state = DiscourseState::new();
    let mut conversation = ConversationState::new();
    let mut text_session = TextSessionState::new();
    let stdin = io::stdin();
    let interactive = stdin.is_terminal();
    if interactive {
        println!("Systean discourse playground. Type `help` for commands; `quit` exits.");
    }
    let mut input = stdin.lock();
    loop {
        if interactive {
            print!("> ");
            if io::stdout().flush().is_err() {
                return ExitCode::FAILURE;
            }
        }
        let mut line = String::new();
        match input.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(error) => {
                eprintln!("discourse input error: {error}");
                return ExitCode::FAILURE;
            }
        }
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match run_discourse_line(
            &language,
            &mut state,
            &mut conversation,
            &mut text_session,
            line,
        ) {
            Ok(true) => break,
            Ok(false) => {}
            Err(error) => eprintln!("discourse error: {error}"),
        }
    }
    ExitCode::SUCCESS
}

fn run_discourse_line(
    language: &LanguagePackage,
    state: &mut DiscourseState,
    conversation: &mut ConversationState,
    text_session: &mut TextSessionState,
    line: &str,
) -> Result<bool, String> {
    let tokens = line.split_whitespace().collect::<Vec<_>>();
    let Some(command) = tokens.first().copied() else {
        return Ok(false);
    };
    let config = language.syntax().config();

    if command == config.discourse.frame {
        if tokens.len() != 1 {
            return Err(format!("`{}` takes no arguments", config.discourse.frame));
        }
        let frame = state.advance_frame();
        let section = text_session.advance_section();
        println!("section: {section} frame: {frame}");
        return Ok(false);
    }
    if command == config.discourse.alias {
        if tokens.len() < 3 {
            return Err(format!(
                "usage: {} <alias> <existing surface value>",
                config.discourse.alias
            ));
        }
        let alias = tokens[1];
        let target = tokens[2..].join(" ");
        let id = language
            .bind_alias_to_surface(state, alias, &target)
            .map_err(|error| error.to_string())?;
        println!("alias: {alias} -> {id}");
        return Ok(false);
    }
    if command == config.discourse.definition {
        if tokens.len() < 3 {
            return Err(format!(
                "usage: {} <alias> <surface value>",
                config.discourse.definition
            ));
        }
        let alias = tokens[1];
        let target = tokens[2..].join(" ");
        let id = language
            .define_alias_from_surface(state, alias, &target)
            .map_err(|error| error.to_string())?;
        println!("definition: {alias} -> {id}");
        return Ok(false);
    }
    if command == config.discourse.relative {
        let (alias, target, body) = parse_relative_command(&tokens, config)?;
        let analysis = language
            .analyze_with_relative_binding(state, alias, &target, &body)
            .map_err(|error| error.to_string())?;
        print_discourse_analysis(&analysis);
        return Ok(false);
    }

    match command {
        "quit" | "exit" => Ok(true),
        "help" => {
            print_discourse_help(language);
            Ok(false)
        }
        "state" => {
            print_discourse_state(state);
            Ok(false)
        }
        "scope" => {
            match tokens.as_slice() {
                [_, "enter"] => println!("scope: {}", state.enter_scope()),
                [_, "leave"] => println!(
                    "scope: {}",
                    state.leave_scope().map_err(|error| error.to_string())?
                ),
                _ => return Err("usage: scope enter|leave".into()),
            }
            Ok(false)
        }
        "context" => {
            if tokens.len() < 3 {
                return Err("usage: context <key> <semantic-expression>".into());
            }
            let key = tokens[1];
            let term = parse_semantic_value(language, &tokens[2..].join(" "))?;
            state
                .set_context_value(key, term.clone(), language.semantics())
                .map_err(|error| error.to_string())?;
            println!("context {key}: {}", canonicalize(&term));
            Ok(false)
        }
        "context-surface" => {
            if tokens.len() < 3 {
                return Err("usage: context-surface <key> <surface-expression>".into());
            }
            let key = tokens[1];
            let source = tokens[2..].join(" ");
            let analysis = language
                .analyze_surface(&source)
                .map_err(|error| error.to_string())?;
            let lowered = language
                .syntax()
                .lower(&analysis.syntax, language.semantics())
                .map_err(|error| error.to_string())?;
            state
                .set_context_value(key, lowered.term.clone(), language.semantics())
                .map_err(|error| error.to_string())?;
            println!("context {key}: {}", canonicalize(&lowered.term));
            Ok(false)
        }
        "intro" => {
            if tokens.len() < 2 {
                return Err("usage: intro <surface-expression>".into());
            }
            let source = tokens[1..].join(" ");
            let analysis = language
                .analyze_surface_with_discourse(&source, state)
                .map_err(|error| error.to_string())?;
            let id = state
                .introduce(
                    analysis.resolved.term.clone(),
                    IntroductionOrigin::Surface { source },
                    language.semantics(),
                )
                .map_err(|error| error.to_string())?;
            println!(
                "introduced: {id}:{}={}",
                analysis.resolved.inferred_type,
                canonicalize(&analysis.resolved.term)
            );
            Ok(false)
        }
        "intro-sem" => {
            if tokens.len() < 2 {
                return Err("usage: intro-sem <semantic-expression>".into());
            }
            let term = parse_semantic_value(language, &tokens[1..].join(" "))?;
            let id = state
                .introduce(
                    term.clone(),
                    IntroductionOrigin::External {
                        label: "playground semantic input".into(),
                    },
                    language.semantics(),
                )
                .map_err(|error| error.to_string())?;
            let ty = Checker::new(language.semantics())
                .infer(&term)
                .map_err(|error| error.to_string())?;
            println!("introduced: {id}:{ty}={}", canonicalize(&term));
            Ok(false)
        }
        "bind" => {
            let [_, alias, id] = tokens.as_slice() else {
                return Err("usage: bind <alias> <referent-id>".into());
            };
            let id = parse_referent_id(id)?;
            language
                .bind_alias(state, alias, id)
                .map_err(|error| error.to_string())?;
            println!("alias: {alias} -> {id}");
            Ok(false)
        }
        "resolve" => {
            let [_, ty] = tokens.as_slice() else {
                return Err("usage: resolve <semantic-type>".into());
            };
            let ty = parse_type(ty)
                .map_err(|errors| {
                    errors
                        .into_iter()
                        .map(|error| error.to_string())
                        .collect::<Vec<_>>()
                        .join("; ")
                })?;
            let resolved = state
                .resolve_reference("playground", &ty, language.semantics())
                .map_err(|error| error.to_string())?;
            println!("resolved: {}:{}={}", resolved.id, resolved.ty, resolved.value);
            Ok(false)
        }
        "turn" => {
            if tokens.len() < 4 {
                return Err("usage: turn spoken|written <turn-key> <text-stream>".into());
            }
            let realization = match tokens[1] {
                "spoken" => TextRealization::Spoken,
                "written" => TextRealization::Written,
                _ => return Err("usage: turn spoken|written <turn-key> <text-stream>".into()),
            };
            let turn = TextTurn {
                key: tokens[2].to_owned(),
                realization,
                source: tokens[3..].join(" "),
            };
            let analysis = language
                .apply_text_turn(&turn, text_session, state, conversation)
                .map_err(|error| error.to_string())?;
            print_text_turn_analysis(&analysis);
            Ok(false)
        }
        "say" => {
            if tokens.len() < 2 {
                return Err("usage: say <surface-expression>".into());
            }
            let source = tokens[1..].join(" ");
            let analysis = language
                .analyze_utterance_with_discourse(&source, state)
                .map_err(|error| error.to_string())?;
            let id = conversation
                .apply(
                    source,
                    analysis.surface.canonical_resolved_surface.clone(),
                    analysis.pragmatics.clone(),
                )
                .map_err(|error| error.to_string())?;
            println!("utterance: {id}");
            println!("act: {}", analysis.pragmatics.act.label());
            println!("canonical utterance: {}", analysis.pragmatics.utterance);
            Ok(false)
        }
        "history" => {
            for entry in conversation.history() {
                println!(
                    "{} act={} surface={} semantics={}",
                    entry.id,
                    entry.analysis.act.label(),
                    entry.canonical_surface,
                    entry.analysis.utterance
                );
            }
            Ok(false)
        }
        "commitments" => {
            for commitment in conversation.active_commitments() {
                println!("{} active={}", commitment.entry, commitment.content);
            }
            Ok(false)
        }
        "analyze" => {
            if tokens.len() < 2 {
                return Err("usage: analyze <surface-expression>".into());
            }
            let analysis = language
                .analyze_surface_with_discourse(&tokens[1..].join(" "), state)
                .map_err(|error| error.to_string())?;
            print_discourse_analysis(&analysis);
            Ok(false)
        }
        _ => Err(format!("unknown discourse command `{command}`; use `help`")),
    }
}

fn parse_relative_command<'a>(
    tokens: &'a [&'a str],
    config: &systean_core::syntax::SyntaxConfig,
) -> Result<(&'a str, String, String), String> {
    if tokens.len() < 7 {
        return Err(format!(
            "usage: {} <alias> {} <target> {} {} <body> {}",
            config.discourse.relative,
            config.scope.open,
            config.scope.close,
            config.scope.open,
            config.scope.close
        ));
    }
    let alias = tokens[1];
    let (target, next) = take_structural_group(tokens, 2, &config.scope.open, &config.scope.close)?;
    let (body, next) = take_structural_group(tokens, next, &config.scope.open, &config.scope.close)?;
    if next != tokens.len() {
        return Err("unexpected tokens after relative binding body".into());
    }
    Ok((alias, target.join(" "), body.join(" ")))
}

fn take_structural_group<'a>(
    tokens: &'a [&'a str],
    start: usize,
    open: &str,
    close: &str,
) -> Result<(Vec<&'a str>, usize), String> {
    if tokens.get(start).copied() != Some(open) {
        return Err(format!("expected structural opener `{open}`"));
    }
    let mut depth = 1usize;
    let mut index = start + 1;
    let mut body = Vec::new();
    while index < tokens.len() {
        let token = tokens[index];
        if token == open {
            depth += 1;
            body.push(token);
        } else if token == close {
            depth -= 1;
            if depth == 0 {
                return Ok((body, index + 1));
            }
            body.push(token);
        } else {
            body.push(token);
        }
        index += 1;
    }
    Err(format!("missing structural closer `{close}`"))
}

fn parse_semantic_value(language: &LanguagePackage, source: &str) -> Result<Term, String> {
    let parsed = parse_term(source).map_err(|errors| {
        errors
            .into_iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    let term = lower_term(parsed);
    Checker::new(language.semantics())
        .infer(&term)
        .map_err(|error| error.to_string())?;
    Ok(term)
}

fn parse_referent_id(source: &str) -> Result<ReferentId, String> {
    let number = source
        .strip_prefix('r')
        .ok_or_else(|| format!("referent id `{source}` must look like r0"))?
        .parse::<u64>()
        .map_err(|_| format!("invalid referent id `{source}`"))?;
    Ok(ReferentId::from_raw(number))
}

fn print_text_turn_analysis(analysis: &TextTurnAnalysis) {
    println!(
        "turn: {} ({})",
        analysis.key,
        match analysis.realization {
            TextRealization::Spoken => "spoken",
            TextRealization::Written => "written",
        }
    );
    for event in &analysis.events {
        match event {
            TextTurnEvent::FrameBoundary { section, frame } => {
                println!("section: {section} frame: {frame}");
            }
            TextTurnEvent::Utterance(utterance) => {
                println!(
                    "{} section={} act={}",
                    utterance.id,
                    utterance.section,
                    utterance.pragmatics.act.label()
                );
                println!("  spoken: {}", utterance.canonical_spoken);
                println!("  written: {}", utterance.canonical_written);
                println!("  semantics: {}", utterance.pragmatics.utterance);
            }
        }
    }
}

fn print_discourse_analysis(analysis: &systean_core::language::DiscourseSurfaceAnalysis) {
    println!("canonical surface: {}", analysis.canonical_surface);
    println!("resolved surface: {}", analysis.canonical_resolved_surface);
    println!("type: {}", analysis.inferred_type);
    println!("canonical semantics: {}", analysis.canonical_semantics);
    for binding in &analysis.resolved.contexts {
        println!(
            "context {} -> {}:{}={}",
            binding.slot.surface,
            binding.slot.key,
            binding.value.ty,
            binding.value.value
        );
    }
    for binding in &analysis.resolved.aliases {
        println!(
            "alias {} -> {}:{}={}",
            binding.slot.surface,
            binding.referent.id,
            binding.referent.ty,
            binding.referent.value
        );
    }
    for binding in &analysis.resolved.references {
        println!(
            "reference {} -> {}:{}={}",
            binding.slot.role,
            binding.referent.id,
            binding.referent.ty,
            binding.referent.value
        );
    }
}

fn print_discourse_state(state: &DiscourseState) {
    println!("scope: {}", state.current_scope());
    println!("frame: {}", state.current_frame());
    println!("referents:");
    for referent in state.referents() {
        println!(
            "  {} scope={} frame={} shorthand={} type={} value={}",
            referent.id,
            referent.scope,
            referent.frame,
            referent.shorthand,
            referent.ty,
            referent.value
        );
    }
    println!("aliases:");
    for binding in state.active_aliases() {
        println!(
            "  {} -> {} scope={} type={}",
            binding.surface, binding.referent, binding.scope, binding.ty
        );
    }
}

fn print_discourse_help(language: &LanguagePackage) {
    let config = language.syntax().config();
    println!("commands:");
    println!("  context <key> <semantic-expression>");
    println!("  context-surface <key> <surface-expression>");
    println!("  intro <surface-expression>");
    println!("  intro-sem <semantic-expression>");
    println!("  analyze <surface-expression>");
    println!("  say <surface-expression>");
    println!("  turn spoken|written <turn-key> <text-stream>");
    println!("  history");
    println!("  commitments");
    println!("  resolve <semantic-type>");
    println!("  bind <alias> <referent-id>");
    println!("  scope enter|leave");
    println!("  state");
    println!("  {} <alias> <existing surface value>", config.discourse.alias);
    println!("  {} <alias> <surface value>", config.discourse.definition);
    println!(
        "  {} <alias> {} <target> {} {} <body> {}",
        config.discourse.relative,
        config.scope.open,
        config.scope.close,
        config.scope.open,
        config.scope.close
    );
    println!("  {}", config.discourse.frame);
    println!("  quit");
}


fn workbench(language_path: &Path, mut args: Vec<String>) -> ExitCode {
    if args.is_empty() {
        usage();
        return ExitCode::from(2);
    }
    let command = args.remove(0);
    let language = match load_language(language_path) {
        Ok(language) => language,
        Err(code) => return code,
    };
    let result = match command.as_str() {
        "package" if args.is_empty() => print_workbench_json(&systean_core::workbench::package_info(&language)),
        "word" => {
            let [word] = args.as_slice() else {
                usage();
                return ExitCode::from(2);
            };
            match systean_core::workbench::analyze_word(&language, word) {
                Ok(value) => print_workbench_json(&value),
                Err(error) => return fail_workbench(error),
            }
        }
        "surface" => {
            let source = args.join(" ");
            if source.is_empty() {
                usage();
                return ExitCode::from(2);
            }
            match systean_core::workbench::analyze_surface(
                &language,
                &source,
                &DiscourseState::new(),
            ) {
                Ok(value) => print_workbench_json(&value),
                Err(error) => return fail_workbench(error),
            }
        }
        "utterance" => {
            let source = args.join(" ");
            if source.is_empty() {
                usage();
                return ExitCode::from(2);
            }
            match systean_core::workbench::analyze_utterance(
                &language,
                &source,
                &DiscourseState::new(),
            ) {
                Ok(value) => print_workbench_json(&value),
                Err(error) => return fail_workbench(error),
            }
        }
        "text" => {
            if args.len() < 2 {
                usage();
                return ExitCode::from(2);
            }
            let realization = match args[0].as_str() {
                "spoken" => TextRealization::Spoken,
                "written" => TextRealization::Written,
                _ => {
                    usage();
                    return ExitCode::from(2);
                }
            };
            let source = args[1..].join(" ");
            match systean_core::workbench::analyze_text_stream(&language, &source, realization) {
                Ok(value) => print_workbench_json(&value),
                Err(error) => return fail_workbench(error),
            }
        }
        "generate" => {
            let source = args.join(" ");
            if source.is_empty() {
                usage();
                return ExitCode::from(2);
            }
            match systean_core::workbench::generate_surface(&language, &source) {
                Ok(value) => print_workbench_json(&value),
                Err(error) => return fail_workbench(error),
            }
        }
        "literal" => {
            let source = args.join(" ");
            if source.is_empty() {
                usage();
                return ExitCode::from(2);
            }
            match systean_core::workbench::analyze_literal(&language, &source) {
                Ok(value) => print_workbench_json(&value),
                Err(error) => return fail_workbench(error),
            }
        }
        _ => {
            usage();
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("workbench serialization error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn print_workbench_json(value: &impl serde::Serialize) -> Result<(), serde_json::Error> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn fail_workbench(error: systean_core::workbench::WorkbenchDiagnostic) -> ExitCode {
    match serde_json::to_string_pretty(&error) {
        Ok(json) => eprintln!("{json}"),
        Err(_) => eprintln!("workbench error: {}", error.message),
    }
    ExitCode::FAILURE
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
        "usage:\n  systean [--language <path>] check\n  systean [--language <path>] english <surface-expression>\n  systean [--language <path>] explain <semantic-expression>\n  systean [--language <path>] phonology check\n  systean [--language <path>] phonology pronounce <text>\n  systean [--language <path>] phonology spell <pronunciation>\n  systean [--language <path>] phonology analyze <word> [--root <root>]\n  systean [--language <path>] morphology check\n  systean [--language <path>] morphology analyze <word>\n  systean [--language <path>] morphology generate <root>\n  systean [--language <path>] syntax check\n  systean [--language <path>] syntax analyze <surface-expression>\n  systean [--language <path>] literals analyze <structured-literal>\n  systean [--language <path>] literals convert <quantity> --to <unit-id>\n  systean [--language <path>] discourse\n  systean [--language <path>] roots check <candidate>\n  systean [--language <path>] roots audit\n  systean [--language <path>] roots segment <pronunciation>\n  systean [--language <path>] workbench package\n  systean [--language <path>] workbench word <word>\n  systean [--language <path>] workbench surface <surface-expression>\n  systean [--language <path>] workbench utterance <surface-expression>\n  systean [--language <path>] workbench text spoken|written <text-stream>\n  systean [--language <path>] workbench generate <semantic-expression>\n  systean [--language <path>] workbench literal <structured-literal>"
    );
}
