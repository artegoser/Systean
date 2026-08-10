<script lang="ts">
	import Letter from '$lib/components/Letter.svelte';
	import type { Alphabet } from '$lib/types';
	import { toPronunciation } from '$lib/utils';
	import { app_state } from '$lib/state.svelte';

	interface Props {
		data: Alphabet;
	}

	let { data }: Props = $props();

	let input = $state('Systean');

	app_state.current_tab = 1;
</script>

<svelte:head>
	<title>Systean alphabet</title>
</svelte:head>

<div class="flex flex-col gap-2 max-w-150">
	<div class="flex flex-col gap-1 items-center">
		<div class="big-text">Systean alphabet</div>
		<div class="small-text">
			In the Systean language, letters always have the same sound as written in the alphabet. For
			any letter combination, you can get its pronunciation automatically.
		</div>
	</div>

	<div class="flex flex-col items-center">
		<input type="text" class="input" placeholder="Type any word..." bind:value={input} />
		<div class="text-accent/75 text-3xl font-extrabold mx-10 break-all">
			{toPronunciation(data, input)}
		</div>
	</div>

	<div class="flex flex-wrap gap-2 justify-center">
		{#each data.letters as letter}
			<Letter {letter} />
		{/each}
	</div>
</div>
