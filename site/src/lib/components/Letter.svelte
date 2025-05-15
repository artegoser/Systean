<script lang="ts">
	import type { Letter } from '$lib/types';

	interface Props {
		letter: Letter;
	}

	let { letter }: Props = $props();
</script>

<div class="letter">
	<div class={letter._type === 'consonant' ? 'text-cyan-500' : 'text-red-500'}>
		{letter.symbol.toLocaleUpperCase()}{letter.symbol}
	</div>
	<div class="text-accent/50">/{letter.pronunciation}/</div>
</div>
