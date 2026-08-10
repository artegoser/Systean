export interface Letter {
	symbol: string;
	pronunciation: string;
	_type: 'vowel' | 'consonant';
}

export interface Alphabet {
	letters: Letter[];
}

export interface DictionaryEntry {
	root: string;
	definition: string;
}

export interface Dictionary {
	entries: DictionaryEntry[];
}

export interface SyllableAnalysis {
	spelling: string;
	pronunciation: string;
	stressed: boolean;
	graphemeStart: number;
	graphemeEnd: number;
}

export interface MorphemeAnalysis {
	kind: 'root';
	spelling: string;
	graphemeStart: number;
	graphemeEnd: number;
}

export interface WordAnalysis {
	spelling: string;
	root: string;
	morphemes: MorphemeAnalysis[];
	pronunciation: string;
	stressedPronunciation: string;
	rootStart: number;
	rootEnd: number;
	stressedSyllable: number;
	syllables: SyllableAnalysis[];
}

export interface SemanticAnalysis {
	inferred_type: string;
	canonical: string;
	explanation: string;
}
