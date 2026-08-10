<script lang="ts">
	import { onMount } from 'svelte';
	import { loadEngine } from '$lib/engine';
	import type { DictionaryEntry } from '$lib/types';
	import { app_state } from '$lib/state.svelte';

	let entries = $state<DictionaryEntry[]>([]);
	let error = $state('');
	let loading = $state(true);

	app_state.current_tab = 2;

	onMount(async () => {
		try {
			entries = (await loadEngine()).dictionary().entries;
		} catch (cause) {
			error = cause instanceof Error ? cause.message : String(cause);
		} finally {
			loading = false;
		}
	});
</script>

<svelte:head>
	<title>Systean dictionary</title>
</svelte:head>

<div class="flex flex-col items-center gap-4 max-w-180 w-full">
	<div class="big-text mt-6">Systean dictionary</div>
	<div class="small-text text-center">
		The dictionary is loaded and parsed by the same Rust language package as the CLI.
	</div>

	{#if loading}
		<div class="small-text">Loading language engine…</div>
	{:else if error}
		<div class="text-red-400 text-center font-bold">{error}</div>
	{:else}
		<div class="flex flex-col gap-3 w-full">
			{#each entries as entry}
				<article class="bg-accent/10 border-2 border-accent/10 rounded-xl p-4">
					<div class="text-3xl font-black">{entry.root}</div>
					<div class="mt-3 text-stone-200/80 whitespace-pre-line">{entry.definition}</div>
				</article>
			{/each}
		</div>
	{/if}
</div>
