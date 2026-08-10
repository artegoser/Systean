import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { fileURLToPath, URL } from 'node:url';

const repositoryRoot = fileURLToPath(new URL('..', import.meta.url));

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		fs: {
			allow: [repositoryRoot]
		}
	}
});
