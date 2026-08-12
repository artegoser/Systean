import type {
	Alphabet,
	Dictionary,
	DocumentationSearchEntry,
	EnglishRendering,
	SemanticAnalysis,
	SurfaceAnalysis,
	SyntaxPolicy,
	WordAnalysis,
	PackageWorkbenchInfo,
	WordWorkbenchAnalysis,
	SurfaceWorkbenchAnalysis,
	UtteranceWorkbenchAnalysis,
	DiscourseWorkbenchAnalysis,
	GenerationWorkbenchAnalysis,
	LiteralWorkbenchAnalysis,
	WorkbenchDiagnostic
} from './types';

interface WasmModule {
	default: () => Promise<unknown>;
	alphabet_json: () => string;
	dictionary_json: () => string;
	documentation_json: () => string;
	english_json: (expression: string) => string;
	pronounce: (text: string) => string;
	spell: (pronunciation: string) => string;
	analyze_word_json: (word: string) => string;
	generate_word: (root: string) => string;
	explain_json: (expression: string) => string;
	syntax_policy_json: () => string;
	analyze_surface_json: (expression: string) => string;
	package_info_json: () => string;
	workbench_word_json: (word: string) => string;
	workbench_surface_json: (expression: string) => string;
	workbench_utterance_json: (expression: string) => string;
	workbench_text_json: (source: string, realization: 'spoken' | 'written') => string;
	generate_surface_json: (semantics: string) => string;
	inspect_literal_json: (source: string) => string;
}

export interface SysteanEngine {
	alphabet(): Alphabet;
	dictionary(): Dictionary;
	documentation(): DocumentationSearchEntry[];
	english(expression: string): EnglishRendering;
	pronounce(text: string): string;
	spell(pronunciation: string): string;
	analyzeWord(word: string): WordAnalysis;
	generateWord(root: string): string;
	explain(expression: string): SemanticAnalysis;
	syntaxPolicy(): SyntaxPolicy;
	analyzeSurface(expression: string): SurfaceAnalysis;
	packageInfo(): PackageWorkbenchInfo;
	workbenchWord(word: string): WordWorkbenchAnalysis;
	workbenchSurface(expression: string): SurfaceWorkbenchAnalysis;
	workbenchUtterance(expression: string): UtteranceWorkbenchAnalysis;
	workbenchText(source: string, realization: 'spoken' | 'written'): DiscourseWorkbenchAnalysis;
	generateSurface(semantics: string): GenerationWorkbenchAnalysis;
	inspectLiteral(source: string): LiteralWorkbenchAnalysis;
}

let enginePromise: Promise<SysteanEngine> | undefined;

export function loadEngine(): Promise<SysteanEngine> {
	enginePromise ??= createEngine();
	return enginePromise;
}

async function createEngine(): Promise<SysteanEngine> {
	const wasm = (await import('../wasm/pkg/systean_wasm.js')) as unknown as WasmModule;
	await wasm.default();

	return {
		alphabet: () => JSON.parse(wasm.alphabet_json()) as Alphabet,
		dictionary: () => JSON.parse(wasm.dictionary_json()) as Dictionary,
		documentation: () => JSON.parse(wasm.documentation_json()) as DocumentationSearchEntry[],
		english: (expression) => JSON.parse(wasm.english_json(expression)) as EnglishRendering,
		pronounce: (text) => wasm.pronounce(text),
		spell: (pronunciation) => wasm.spell(pronunciation),
		analyzeWord: (word) => JSON.parse(wasm.analyze_word_json(word)) as WordAnalysis,
		generateWord: (root) => wasm.generate_word(root),
		explain: (expression) => JSON.parse(wasm.explain_json(expression)) as SemanticAnalysis,
		syntaxPolicy: () => JSON.parse(wasm.syntax_policy_json()) as SyntaxPolicy,
		analyzeSurface: (expression) => JSON.parse(wasm.analyze_surface_json(expression)) as SurfaceAnalysis,
		packageInfo: () => JSON.parse(wasm.package_info_json()) as PackageWorkbenchInfo,
		workbenchWord: (word) => parseWorkbench<WordWorkbenchAnalysis>(() => wasm.workbench_word_json(word)),
		workbenchSurface: (expression) => parseWorkbench<SurfaceWorkbenchAnalysis>(() => wasm.workbench_surface_json(expression)),
		workbenchUtterance: (expression) => parseWorkbench<UtteranceWorkbenchAnalysis>(() => wasm.workbench_utterance_json(expression)),
		workbenchText: (source, realization) => parseWorkbench<DiscourseWorkbenchAnalysis>(() => wasm.workbench_text_json(source, realization)),
		generateSurface: (semantics) => parseWorkbench<GenerationWorkbenchAnalysis>(() => wasm.generate_surface_json(semantics)),
		inspectLiteral: (source) => parseWorkbench<LiteralWorkbenchAnalysis>(() => wasm.inspect_literal_json(source))
	};
}

function parseWorkbench<T>(call: () => string): T {
	try {
		return JSON.parse(call()) as T;
	} catch (cause) {
		throw parseWorkbenchDiagnostic(cause);
	}
}

export function parseWorkbenchDiagnostic(cause: unknown): WorkbenchDiagnostic {
	if (cause && typeof cause === 'object' && 'layer' in cause && 'message' in cause) {
		return cause as WorkbenchDiagnostic;
	}
	const raw = cause instanceof Error ? cause.message : String(cause);
	const candidates = [raw, raw.replace(/^RuntimeError:\s*/, ''), raw.replace(/^JsValue\(/, '').replace(/\)$/, '')];
	for (const candidate of candidates) {
		try {
			const parsed = JSON.parse(candidate) as Partial<WorkbenchDiagnostic>;
			if (parsed.layer && parsed.message) return parsed as WorkbenchDiagnostic;
		} catch {
			// wasm-bindgen may wrap the thrown string; fall through to a generic diagnostic.
		}
	}
	return { layer: 'package', message: raw };
}
