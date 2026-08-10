import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { fileURLToPath, URL } from 'node:url';

const repositoryRoot = fileURLToPath(new URL('..', import.meta.url));

export default defineConfig({
	plugins: [sveltekit()],
	resolve: {
		alias: {
			'$systean-config': fileURLToPath(new URL('../lib/config', import.meta.url))
		}
	},
	server: {
		fs: {
			allow: [repositoryRoot]
		}
	}
});
