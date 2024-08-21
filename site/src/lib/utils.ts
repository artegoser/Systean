import type { Alphabet } from './types';

export function toPronunciation(alphabet: Alphabet, input: string) {
	let output = '';

	for (const letter of input) {
		const idx = alphabet.letters.findIndex(
			(v) => v.symbol === letter || v.symbol.toLocaleUpperCase() === letter
		);
		if (idx !== -1) {
			output += alphabet.letters[idx].pronunciation;
		} else {
			return `Unsupported letter: '${letter}'`;
		}
	}

	return `/${output}/`;
}
