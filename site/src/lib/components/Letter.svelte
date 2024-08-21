<script lang="ts">
	import type { Letter } from '$lib/types';

	export let letter: Letter;
</script>

<div class="letter">
	<div class={letter._type === 'consonant' ? 'text-red-400' : 'text-cyan-500'}>
		{letter.symbol}
	</div>
	<div class="text-stone-700">/{letter.pronunciation}/</div>
</div>
