<script lang="ts">
	import Header from '$lib/components/Header.svelte';
	import '../app.css';
	interface Props {
		children?: import('svelte').Snippet;
	}

	let { children }: Props = $props();
</script>

<Header />

<div class="flex flex-col justify-center items-center gap-6 p-4">
	{@render children?.()}
</div>
