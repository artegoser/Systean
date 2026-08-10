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
	fields: Record<string, string>;
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

export interface WordAnalysis {
	spelling: string;
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
