<script lang="ts">
	import { app_state } from '$lib/state.svelte';
	import { onMount } from 'svelte';

	const sections = [
		{ name: 'About', link: '/' },
		{ name: 'Alphabet', link: '/alphabet' },
		{ name: 'Dictionary', link: '/dictionary' },
		{ name: 'Analyzer', link: '/analyzer' }
	];

	let scrollY = $state(0);

	const anim = $derived(Math.min(scrollY / 150, 1));

	let current_anim = $state(0);

	const fn = () => {
		if (current_anim !== anim) current_anim = anim;

		requestAnimationFrame(fn);
	};

	onMount(() => {
		fn();
	});
</script>

<svelte:window bind:scrollY />

<div
	class="header z-50"
	style="background-color: rgba(0, 0, 0, {Math.max(current_anim, 0.4)}); padding: {2 -
		1.2 * current_anim}rem; -webkit-backdrop-filter: blur({12 *
		current_anim}px); backdrop-filter: blur({12 * current_anim}px)"
>
	{#each sections as section, i}
		<a class="section-link{app_state.current_tab === i ? ' active' : ''}" href={section.link}
			>{section.name}</a
		>
	{/each}
</div>
