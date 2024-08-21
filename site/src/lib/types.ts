export interface Letter {
	symbol: string;
	pronunciation: string;
	_type: 'vowel' | 'consonant';
}

export interface Alphabet {
	letters: Letter[];
}
