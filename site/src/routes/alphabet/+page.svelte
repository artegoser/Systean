<script lang="ts">
	import Letter from '$lib/components/Letter.svelte';
	import type { Alphabet } from '$lib/types';
	import { toPronunciation } from '$lib/utils';

	export let data: Alphabet;

	let input = 'test';
</script>

<svelte:head>
	<title>Systeon alphabet</title>
</svelte:head>

<div class="flex flex-col items-center justify-center">
	<div class="text-stone-50/50 text-center text-5xl font-extrabold mt-6">Systeon alphabet</div>

	<div class="flex flex-col items-center my-6">
		<input type="text" class="input max-w-64" placeholder="Type word..." bind:value={input} />
		<div class="text-stone-700 text-3xl font-extrabold mx-10">{toPronunciation(data, input)}</div>
	</div>

	<div class="flex flex-wrap gap-2 m-2 justify-items-stretch place-content-center">
		{#each data.letters as letter}
			<Letter {letter} />
		{/each}
	</div>
</div>
