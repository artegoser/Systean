<script lang="ts">
	import { onMount } from 'svelte';
	import { loadEngine } from '$lib/engine';
	import type { DictionaryEntry } from '$lib/types';
	import { app_state } from '$lib/state.svelte';

	let entries = $state<DictionaryEntry[]>([]);
	let query = $state('');
	let error = $state('');
	let loading = $state(true);

	let filtered = $derived(
		entries.filter((entry) => {
			const needle = query.trim().toLowerCase();
			if (!needle) return true;
			return `${entry.root} ${entry.definition}`
				.toLowerCase()
				.includes(needle);
		})
	);

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

<div class="flex flex-col items-center gap-4 max-w-220 w-full">
	<div class="big-text mt-6">Systean dictionary</div>
	<div class="small-text text-center">Lexical definitions and canonical surface metadata from the compiled typed package.</div>

	{#if loading}
		<div class="small-text">Loading language engine…</div>
	{:else if error}
		<div class="text-red-400 text-center font-bold">{error}</div>
	{:else}
		<input class="input m-0 w-full" bind:value={query} placeholder="Search roots or definitions" />
		<div class="small-text">{filtered.length} / {entries.length} entries</div>
		<div class="flex flex-col gap-3 w-full">
			{#each filtered as entry}
				<article class="bg-accent/10 border-2 border-accent/10 rounded-xl p-4">
					<div class="flex flex-wrap gap-3 items-baseline justify-between">
						<div class="text-3xl font-black">{entry.root}</div>
						<div class="font-mono text-sm">typed semantic word</div>
					</div>
					<div class="mt-3 text-stone-200/80 whitespace-pre-line">{entry.definition}</div>
				</article>
			{/each}
		</div>
	{/if}
</div>
