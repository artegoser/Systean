<script lang="ts">
	import { onMount } from 'svelte';
	import { loadEngine, type SysteanEngine } from '$lib/engine';
	import type { SemanticAnalysis, SurfaceAnalysis, SyntaxPolicy, WordAnalysis } from '$lib/types';
	import { app_state } from '$lib/state.svelte';

	let engine = $state<SysteanEngine | null>(null);
	let word = $state('sol');
	let wordAnalysis = $state<WordAnalysis | null>(null);
	let surfaceExpression = $state('');
	let surfaceAnalysis = $state<SurfaceAnalysis | null>(null);
	let syntaxPolicy = $state<SyntaxPolicy | null>(null);
	let semanticExpression = $state('equal(left = 1, right = 1)');
	let semanticAnalysis = $state<SemanticAnalysis | null>(null);
	let wordError = $state('');
	let surfaceError = $state('');
	let semanticError = $state('');

	app_state.current_tab = 3;

	onMount(async () => {
		try {
			engine = await loadEngine();
			syntaxPolicy = engine.syntaxPolicy();
			analyzeWord();
			explainSemantics();
		} catch (cause) {
			wordError = errorText(cause);
			surfaceError = wordError;
			semanticError = wordError;
		}
	});

	function analyzeWord() {
		if (!engine) return;
		try {
			wordAnalysis = engine.analyzeWord(word);
			wordError = '';
		} catch (cause) {
			wordAnalysis = null;
			wordError = errorText(cause);
		}
	}

	function analyzeSurface() {
		if (!engine || !surfaceExpression.trim()) return;
		try {
			surfaceAnalysis = engine.analyzeSurface(surfaceExpression);
			surfaceError = '';
		} catch (cause) {
			surfaceAnalysis = null;
			surfaceError = errorText(cause);
		}
	}

	function explainSemantics() {
		if (!engine) return;
		try {
			semanticAnalysis = engine.explain(semanticExpression);
			semanticError = '';
		} catch (cause) {
			semanticAnalysis = null;
			semanticError = errorText(cause);
		}
	}

	function errorText(cause: unknown) {
		return cause instanceof Error ? cause.message : String(cause);
	}
</script>

<svelte:head>
	<title>Systean analyzer</title>
</svelte:head>

<div class="flex flex-col gap-6 max-w-220 w-full">
	<div class="flex flex-col items-center gap-1">
		<div class="big-text">Systean analyzer</div>
		<div class="small-text text-center">
			This page calls the same Rust engine as the native CLI.
		</div>
	</div>


	<section class="bg-accent/10 border-2 border-accent/10 rounded-xl p-4">
		<h2 class="text-2xl font-black">Word analysis</h2>
		<div class="flex flex-wrap items-center gap-2 mt-2">
			<input class="input m-0" bind:value={word} placeholder="word" />
			<button class="small-text link" onclick={analyzeWord}>Analyze</button>
		</div>
		{#if wordError}
			<div class="text-red-400 mt-3 font-bold">{wordError}</div>
		{:else if wordAnalysis}
			<div class="mt-3 font-mono break-all">
				<div>spelling: {wordAnalysis.spelling}</div>
				<div>root: {wordAnalysis.root}</div>
				<div>morphology: {wordAnalysis.morphemes.map((part) => `${part.kind}:${part.spelling}`).join(' + ')}</div>
				<div>pronunciation: /{wordAnalysis.pronunciation}/</div>
				<div>stressed: /{wordAnalysis.stressedPronunciation}/</div>
				<div class="mt-2">
					{#each wordAnalysis.syllables as syllable}
						<span class:font-black={syllable.stressed} class="mr-2">
							{syllable.spelling} /{syllable.pronunciation}/
						</span>
					{/each}
				</div>
			</div>
		{/if}
	</section>


	<section class="bg-accent/10 border-2 border-accent/10 rounded-xl p-4">
		<h2 class="text-2xl font-black">Surface syntax</h2>
		{#if syntaxPolicy}
			<div class="mt-3 grid gap-1 font-mono text-sm">
				<div>frame: {syntaxPolicy.frameOrder}</div>
				<div>scope: {syntaxPolicy.scopeOpen} ... {syntaxPolicy.scopeClose}</div>
				<div>quote: {syntaxPolicy.quoteOpen} ... {syntaxPolicy.quoteClose}</div>
				<div>scope policy: {syntaxPolicy.explicitScope}</div>
				<div>quantifier scope: {syntaxPolicy.quantifierScope}</div>
				<div>precedence: {Object.entries(syntaxPolicy.precedence).sort(([, left], [, right]) => right - left).map(([operator, precedence]) => `${operator.toUpperCase()}=${precedence}`).join(' > ')}</div>
				<div>lexical roots: {syntaxPolicy.lexicalRoots}</div>
			</div>
			{#if syntaxPolicy.lexicalRoots === 0}
				<div class="small-text mt-3 text-left">
					The structural parser is active, but the dictionary contains no lexical roots.
				</div>
			{:else}
				<textarea class="input m-0 mt-3 w-full min-h-20" bind:value={surfaceExpression} placeholder="Systean surface expression"></textarea>
				<button class="small-text link mt-2" onclick={analyzeSurface}>Analyze surface</button>
				{#if surfaceError}
					<div class="text-red-400 mt-3 font-bold">{surfaceError}</div>
				{:else if surfaceAnalysis}
					<div class="mt-3 font-mono break-all">
						<div>canonical surface: {surfaceAnalysis.canonicalSurface}</div>
						<div>type: {surfaceAnalysis.inferredType}</div>
						<div>semantics: {surfaceAnalysis.canonicalSemantics}</div>
						<div>syntax: {surfaceAnalysis.syntax}</div>
					</div>
				{/if}
			{/if}
		{/if}
	</section>

	<section class="bg-accent/10 border-2 border-accent/10 rounded-xl p-4">
		<h2 class="text-2xl font-black">Semantic IR</h2>
		<textarea class="input m-0 mt-2 w-full min-h-28" bind:value={semanticExpression}></textarea>
		<button class="small-text link mt-2" onclick={explainSemantics}>Explain</button>
		{#if semanticError}
			<div class="text-red-400 mt-3 font-bold">{semanticError}</div>
		{:else if semanticAnalysis}
			<div class="mt-3 flex flex-col gap-2">
				<div><strong>type:</strong> {semanticAnalysis.inferred_type}</div>
				<div class="font-mono break-all"><strong>canonical:</strong> {semanticAnalysis.canonical}</div>
				<pre class="overflow-x-auto whitespace-pre-wrap text-sm">{semanticAnalysis.explanation}</pre>
			</div>
		{/if}
	</section>
</div>
