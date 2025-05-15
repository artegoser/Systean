<script lang="ts">
	import Letter from '$lib/components/Letter.svelte';
	import type { Alphabet } from '$lib/types';
	import { toPronunciation } from '$lib/utils';
	import { app_state } from '$lib/state.svelte';

	interface Props {
		data: Alphabet;
	}

	let { data }: Props = $props();

	let input = $state('Sistean');

	app_state.current_tab = 1;
</script>

<svelte:head>
	<title>Systean alphabet</title>
</svelte:head>

<div class="flex flex-col items-center justify-center gap-2 mt-6">
	<div class="flex flex-col gap-1 items-center">
		<div class="big-text">Systean alphabet</div>
		<div class="small-text max-w-(--breakpoint-sm)">
			In the Sistean language, letters always have the same sound as written in the alphabet. For
			any letter combination, you can get its pronunciation automatically.
		</div>
	</div>

	<div class="flex flex-col items-center">
		<input type="text" class="input max-w-64" placeholder="Type any word..." bind:value={input} />
		<div class="text-accent/75 text-3xl font-extrabold mx-10">{toPronunciation(data, input)}</div>
	</div>

	<div class="flex flex-wrap gap-2 m-2 justify-center max-w-(--breakpoint-sm)">
		{#each data.letters as letter}
			<Letter {letter} />
		{/each}
	</div>
</div>
