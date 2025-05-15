import type { Alphabet } from '$lib/types';
import type { PageLoad } from './$types';
import toml from 'toml';

export const load: PageLoad = async ({ fetch }): Promise<Alphabet> => {
	const alphabet = await fetch('/config/alphabet.toml').then((res) => res.text());

	return { ...toml.parse(alphabet) };
};
