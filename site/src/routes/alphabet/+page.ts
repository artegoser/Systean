import type { Alphabet } from '$lib/types';
import type { PageLoad } from './$types';
import toml from 'toml';
import alphabetSource from '$systean-config/alphabet.toml?raw';

export const load: PageLoad = async (): Promise<Alphabet> => {
	return { ...toml.parse(alphabetSource) };
};
