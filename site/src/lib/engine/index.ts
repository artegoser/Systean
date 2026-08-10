import type {
	Alphabet,
	Dictionary,
	SemanticAnalysis,
	WordAnalysis
} from './types';

interface WasmModule {
	default: () => Promise<unknown>;
	alphabet_json: () => string;
	dictionary_json: () => string;
	pronounce: (text: string) => string;
	spell: (pronunciation: string) => string;
	analyze_word_json: (word: string) => string;
	generate_word: (root: string) => string;
	explain_json: (expression: string) => string;
}

export interface SysteanEngine {
	alphabet(): Alphabet;
	dictionary(): Dictionary;
	pronounce(text: string): string;
	spell(pronunciation: string): string;
	analyzeWord(word: string): WordAnalysis;
	generateWord(root: string): string;
	explain(expression: string): SemanticAnalysis;
}

let enginePromise: Promise<SysteanEngine> | undefined;

export function loadEngine(): Promise<SysteanEngine> {
	enginePromise ??= createEngine();
	return enginePromise;
}

async function createEngine(): Promise<SysteanEngine> {
	const wasm = (await import('$lib/wasm/pkg/systean_wasm.js')) as WasmModule;
	await wasm.default();

	return {
		alphabet: () => JSON.parse(wasm.alphabet_json()) as Alphabet,
		dictionary: () => JSON.parse(wasm.dictionary_json()) as Dictionary,
		pronounce: (text) => wasm.pronounce(text),
		spell: (pronunciation) => wasm.spell(pronunciation),
		analyzeWord: (word) => JSON.parse(wasm.analyze_word_json(word)) as WordAnalysis,
		generateWord: (root) => wasm.generate_word(root),
		explain: (expression) => JSON.parse(wasm.explain_json(expression)) as SemanticAnalysis
	};
}
