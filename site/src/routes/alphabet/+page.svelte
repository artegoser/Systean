<script lang="ts">
	import { onMount } from 'svelte';
	import Letter from '$lib/components/Letter.svelte';
	import { loadEngine, type SysteanEngine } from '$lib/engine';
	import type { Alphabet } from '$lib/types';
	import { app_state } from '$lib/state.svelte';

	let engine = $state<SysteanEngine | null>(null);
	let alphabet = $state<Alphabet | null>(null);
	let input = $state('Systean');
	let pronunciation = $state('');
	let error = $state('');

	app_state.current_tab = 1;

	onMount(async () => {
		try {
			engine = await loadEngine();
			alphabet = engine.alphabet();
			pronunciation = `/${engine.pronounce(input)}/`;
		} catch (cause) {
			error = cause instanceof Error ? cause.message : String(cause);
		}
	});

	function updatePronunciation() {
		if (!engine) return;
		try {
			pronunciation = `/${engine.pronounce(input)}/`;
			error = '';
		} catch (cause) {
			pronunciation = '';
			error = cause instanceof Error ? cause.message : String(cause);
		}
	}
</script>

<svelte:head>
	<title>Systean alphabet</title>
</svelte:head>

<div class="flex flex-col gap-2 max-w-150">
	<div class="flex flex-col gap-1 items-center">
		<div class="big-text">Systean alphabet</div>
		<div class="small-text">
			Letters always have the same sound. Pronunciation below is produced by the Rust Systean
			engine used by the CLI and language validator.
		</div>
	</div>

	<div class="flex flex-col items-center">
		<input
			type="text"
			class="input"
			placeholder="Type any word..."
			bind:value={input}
			oninput={updatePronunciation}
		/>
		{#if error}
			<div class="text-red-400 text-center font-bold">{error}</div>
		{:else if pronunciation}
			<div class="text-accent/75 text-3xl font-extrabold mx-10 break-all">
				{pronunciation}
			</div>
		{:else}
			<div class="small-text">Loading language engine…</div>
		{/if}
	</div>

	{#if alphabet}
		<div class="flex flex-wrap gap-2 justify-center">
			{#each alphabet.letters as letter}
				<Letter {letter} />
			{/each}
		</div>
	{/if}
</div>
