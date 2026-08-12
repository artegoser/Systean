<script lang="ts">
	import { onMount } from 'svelte';
	import { loadEngine, parseWorkbenchDiagnostic, type SysteanEngine } from '$lib/engine';
	import type {
		DiscourseWorkbenchAnalysis,
		GenerationWorkbenchAnalysis,
		LiteralWorkbenchAnalysis,
		PackageWorkbenchInfo,
		SurfaceWorkbenchAnalysis,
		WordWorkbenchAnalysis,
		WorkbenchDiagnostic
	} from '$lib/types';
	import { app_state } from '$lib/state.svelte';

	let engine = $state<SysteanEngine | null>(null);
	let packageInfo = $state<PackageWorkbenchInfo | null>(null);

	let word = $state('vid');
	let wordAnalysis = $state<WordWorkbenchAnalysis | null>(null);
	let wordError = $state<WorkbenchDiagnostic | null>(null);

	let surfaceExpression = $state('na artemi vid na mari');
	let surfaceAnalysis = $state<SurfaceWorkbenchAnalysis | null>(null);
	let surfaceError = $state<WorkbenchDiagnostic | null>(null);

	let textRealization = $state<'spoken' | 'written'>('written');
	let textSource = $state('na artemi viv. ke na artemi viv.');
	let discourseAnalysis = $state<DiscourseWorkbenchAnalysis | null>(null);
	let discourseError = $state<WorkbenchDiagnostic | null>(null);

	let semanticExpression = $state('see(observed = proper_name(payload = "mari"), observer = proper_name(payload = "artemi"))');
	let generation = $state<GenerationWorkbenchAnalysis | null>(null);
	let generationError = $state<WorkbenchDiagnostic | null>(null);

	let literalSource = $state('2000-12-31');
	let literalAnalysis = $state<LiteralWorkbenchAnalysis | null>(null);
	let literalError = $state<WorkbenchDiagnostic | null>(null);

	app_state.current_tab = 3;

	onMount(async () => {
		try {
			engine = await loadEngine();
			packageInfo = engine.packageInfo();
			analyzeWord();
			analyzeSurface();
			analyzeText();
			generateSurface();
			inspectLiteral();
		} catch (cause) {
			wordError = parseWorkbenchDiagnostic(cause);
		}
	});

	function analyzeWord() {
		if (!engine || !word.trim()) return;
		try {
			wordAnalysis = engine.workbenchWord(word.trim());
			wordError = null;
		} catch (cause) {
			wordAnalysis = null;
			wordError = parseWorkbenchDiagnostic(cause);
		}
	}

	function analyzeSurface() {
		if (!engine || !surfaceExpression.trim()) return;
		try {
			surfaceAnalysis = engine.workbenchSurface(surfaceExpression.trim());
			surfaceError = null;
		} catch (cause) {
			surfaceAnalysis = null;
			surfaceError = parseWorkbenchDiagnostic(cause);
		}
	}

	function analyzeText() {
		if (!engine || !textSource.trim()) return;
		try {
			discourseAnalysis = engine.workbenchText(textSource, textRealization);
			discourseError = null;
		} catch (cause) {
			discourseAnalysis = null;
			discourseError = parseWorkbenchDiagnostic(cause);
		}
	}

	function generateSurface() {
		if (!engine || !semanticExpression.trim()) return;
		try {
			generation = engine.generateSurface(semanticExpression.trim());
			generationError = null;
		} catch (cause) {
			generation = null;
			generationError = parseWorkbenchDiagnostic(cause);
		}
	}

	function inspectLiteral() {
		if (!engine || !literalSource.trim()) return;
		try {
			literalAnalysis = engine.inspectLiteral(literalSource.trim());
			literalError = null;
		} catch (cause) {
			literalAnalysis = null;
			literalError = parseWorkbenchDiagnostic(cause);
		}
	}
</script>

<svelte:head>
	<title>Systean workbench</title>
</svelte:head>

<div class="flex flex-col gap-6 max-w-260 w-full">
	<div class="flex flex-col items-center gap-1">
		<div class="big-text">Systean workbench</div>
		<div class="small-text text-center">One language package, one Rust pipeline, exposed through the browser.</div>
	</div>

	{#if packageInfo}
		<section class="bg-accent/10 border-2 border-accent/10 rounded-xl p-4">
			<div class="flex flex-wrap items-baseline justify-between gap-2">
				<h2 class="text-2xl font-black">Package</h2>
				<div class="font-mono text-sm">{packageInfo.manifest.package.name} {packageInfo.manifest.package.version} · rev {packageInfo.manifest.package.revision}</div>
			</div>
			<div class="font-mono text-sm mt-2 break-all">fingerprint: {packageInfo.provenance.fingerprint}</div>
			<div class="grid sm:grid-cols-2 lg:grid-cols-4 gap-2 mt-3 text-sm">
				<div>owned forms <strong>{packageInfo.validation.owned_forms}</strong></div>
				<div>generated surfaces <strong>{packageInfo.validation.generated_surfaces}</strong></div>
				<div>semantic roundtrips <strong>{packageInfo.validation.semantic_roundtrips}</strong></div>
				<div>corpus <strong>{packageInfo.validation.compatibility_entries + packageInfo.validation.adversarial_entries}</strong></div>
			</div>
			<details class="mt-3">
				<summary class="link small-text">Full provenance ({packageInfo.provenance.sources.length} normative sources)</summary>
				<div class="font-mono text-xs mt-2 grid gap-1">
					{#each packageInfo.provenance.sources as source}
						<div class="break-all">{source.path} · {source.digest} · {source.bytes} B</div>
					{/each}
				</div>
			</details>
		</section>
	{/if}

	<section class="bg-accent/10 border-2 border-accent/10 rounded-xl p-4">
		<h2 class="text-2xl font-black">Unified word analyzer</h2>
		<div class="flex flex-wrap items-center gap-2 mt-2">
			<input class="input m-0" bind:value={word} placeholder="word" onkeydown={(event) => event.key === 'Enter' && analyzeWord()} />
			<button class="small-text link" onclick={analyzeWord}>Analyze</button>
		</div>
		{#if wordError}
			<div class="mt-3 rounded-lg border border-red-400/30 bg-red-400/10 p-3"><strong>{wordError.layer}</strong>: {wordError.message}</div>
		{:else if wordAnalysis}
			<div class="mt-3 grid md:grid-cols-2 gap-4">
				<div>
					<div class="text-3xl font-black">{wordAnalysis.root}</div>
					<div class="mt-2">{wordAnalysis.definition}</div>
					<div class="font-mono text-sm mt-3">/{wordAnalysis.pronunciation}/ → /{wordAnalysis.stressed_pronunciation}/</div>
					<div class="font-mono text-sm">morphology: {wordAnalysis.morphemes.map((part) => `${part.kind}:${part.spelling}`).join(' + ')}</div>
					<div class="mt-2 flex flex-wrap gap-2">
						{#each wordAnalysis.syllables as syllable}
							<span class:font-black={syllable.stressed} class="font-mono">{syllable.spelling} /{syllable.pronunciation}/</span>
						{/each}
					</div>
				</div>
				<div class="font-mono text-sm break-all">
					<div><strong>semantic identity:</strong> <span class="font-mono">{wordAnalysis.root}</span></div>
					<div class="mt-2"><strong>semantic provenance:</strong> {wordAnalysis.semantic_origin ?? 'typed package'}</div>
				</div>
			</div>
		{/if}
	</section>

	<section class="bg-accent/10 border-2 border-accent/10 rounded-xl p-4">
		<h2 class="text-2xl font-black">Surface → typed AST → semantic IR</h2>
		<textarea class="input m-0 mt-2 w-full min-h-20" bind:value={surfaceExpression}></textarea>
		<button class="small-text link mt-2" onclick={analyzeSurface}>Analyze surface</button>
		{#if surfaceError}
			<div class="mt-3 rounded-lg border border-red-400/30 bg-red-400/10 p-3">
				<div><strong>{surfaceError.layer}</strong>: {surfaceError.message}</div>
				{#if surfaceError.candidates?.length}
					<div class="font-mono text-sm mt-2">{surfaceError.candidates.map((candidate) => `${candidate.id}:${candidate.ty}=${candidate.value}`).join('\n')}</div>
				{/if}
			</div>
		{:else if surfaceAnalysis}
			<div class="grid lg:grid-cols-2 gap-4 mt-3">
				<div class="grid gap-2">
					<div><strong>canonical surface:</strong> <span class="font-mono">{surfaceAnalysis.canonical_surface}</span></div>
					<div><strong>resolved surface:</strong> <span class="font-mono">{surfaceAnalysis.canonical_resolved_surface}</span></div>
					<div><strong>type:</strong> <span class="font-mono">{surfaceAnalysis.inferred_type}</span></div>
					<div><strong>typed template:</strong> <span class="font-mono break-all">{surfaceAnalysis.typed_template}</span></div>
					<div><strong>semantic IR:</strong> <span class="font-mono break-all">{surfaceAnalysis.semantic_ir}</span></div>
					<div><strong>canonical IR:</strong> <span class="font-mono break-all">{surfaceAnalysis.canonical_semantic_ir}</span></div>
				</div>
				<div>
					<div class="font-black">Typed surface AST</div>
					<pre class="mt-1 overflow-x-auto whitespace-pre-wrap text-xs">{JSON.stringify(surfaceAnalysis.surface_ast, null, 2)}</pre>
				</div>
			</div>

			<div class="grid md:grid-cols-2 gap-4 mt-4">
				<div>
					<div class="font-black">Scope visualization</div>
					{#if surfaceAnalysis.scopes.length}
						<div class="grid gap-1 mt-2 font-mono text-sm">
							{#each surfaceAnalysis.scopes as scope}
								<div class="rounded-lg bg-black/10 px-2 py-1">{scope.path} → {scope.kind}:{scope.operator}</div>
							{/each}
						</div>
					{:else}
						<div class="small-text text-left mt-2">No scope-forming operator in this expression.</div>
					{/if}
				</div>
				<div>
					<div class="font-black">References / context / aliases</div>
					<div class="grid gap-1 mt-2 font-mono text-sm">
						{#each surfaceAnalysis.references as reference}
							<div class="rounded-lg bg-black/10 px-2 py-1">ref {reference.role}:{reference.expected_type} → {reference.resolution ? `${reference.resolution.id}:${reference.resolution.value}` : 'unresolved'}</div>
						{/each}
						{#each surfaceAnalysis.contexts as context}
							<div class="rounded-lg bg-black/10 px-2 py-1">context {context.key}:{context.expected_type} → {context.resolution?.value ?? 'unresolved'}</div>
						{/each}
						{#each surfaceAnalysis.aliases as alias}
							<div class="rounded-lg bg-black/10 px-2 py-1">alias {alias.alias}:{alias.expected_type} → {alias.resolution?.value ?? 'unresolved'}</div>
						{/each}
						{#if surfaceAnalysis.references.length + surfaceAnalysis.contexts.length + surfaceAnalysis.aliases.length === 0}
							<div class="small-text text-left">No discourse-bound slots.</div>
						{/if}
					</div>
				</div>
			</div>

			<details class="mt-4">
				<summary class="link small-text">Semantic explanation with provenance</summary>
				<pre class="mt-2 overflow-x-auto whitespace-pre-wrap text-xs">{JSON.stringify(surfaceAnalysis.semantic_explanation, null, 2)}</pre>
			</details>
		{/if}
	</section>

	<section class="bg-accent/10 border-2 border-accent/10 rounded-xl p-4">
		<h2 class="text-2xl font-black">Discourse playground</h2>
		<div class="flex gap-3 mt-2 text-sm">
			<label><input type="radio" bind:group={textRealization} value="written" /> written</label>
			<label><input type="radio" bind:group={textRealization} value="spoken" /> spoken</label>
		</div>
		<textarea class="input m-0 mt-2 w-full min-h-24" bind:value={textSource}></textarea>
		<button class="small-text link mt-2" onclick={analyzeText}>Analyze discourse</button>
		{#if discourseError}
			<div class="mt-3 rounded-lg border border-red-400/30 bg-red-400/10 p-3"><strong>{discourseError.layer}</strong>: {discourseError.message}</div>
		{:else if discourseAnalysis}
			<div class="mt-3 grid gap-4">
				{#each discourseAnalysis.turns as turn}
					<div class="rounded-xl bg-black/10 p-3">
						<div class="font-black">{turn.key} · {turn.realization}</div>
						<div class="grid md:grid-cols-2 gap-3 mt-2 text-sm">
							<div><strong>before</strong> · scope {turn.before.scope}, frame {turn.before.frame}, history {turn.before.history.length}, referents {turn.before.referents.length}</div>
							<div><strong>after</strong> · scope {turn.after.scope}, frame {turn.after.frame}, history {turn.after.history.length}, referents {turn.after.referents.length}</div>
						</div>
						<div class="grid gap-2 mt-3">
							{#each turn.events as event}
								<div class="font-mono text-sm border-l-2 border-accent/30 pl-2">
									{event.kind} {event.id ?? ''} · {event.act ?? event.frame ?? ''}<br />
									{event.canonical_surface ?? ''}{event.semantics ? ` → ${event.semantics}` : ''}
								</div>
							{/each}
						</div>
					</div>
				{/each}
				<div class="grid md:grid-cols-2 gap-4">
					<div>
						<div class="font-black">Final references and aliases</div>
						<pre class="mt-1 overflow-x-auto whitespace-pre-wrap text-xs">{JSON.stringify({ referents: discourseAnalysis.final_state.referents, aliases: discourseAnalysis.final_state.aliases }, null, 2)}</pre>
					</div>
					<div>
						<div class="font-black">History and active commitments</div>
						<pre class="mt-1 overflow-x-auto whitespace-pre-wrap text-xs">{JSON.stringify({ history: discourseAnalysis.final_state.history, commitments: discourseAnalysis.final_state.active_commitments }, null, 2)}</pre>
					</div>
				</div>
			</div>
		{/if}
	</section>

	<section class="bg-accent/10 border-2 border-accent/10 rounded-xl p-4">
		<h2 class="text-2xl font-black">Canonical generator</h2>
		<textarea class="input m-0 mt-2 w-full min-h-28 font-mono" bind:value={semanticExpression}></textarea>
		<button class="small-text link mt-2" onclick={generateSurface}>Generate surface</button>
		{#if generationError}
			<div class="mt-3 rounded-lg border border-red-400/30 bg-red-400/10 p-3"><strong>{generationError.layer}</strong>: {generationError.message}</div>
		{:else if generation}
			<div class="mt-3 grid gap-2">
				<div><strong>canonical surface:</strong> <span class="font-mono">{generation.canonical_surface}</span></div>
				<div><strong>type:</strong> <span class="font-mono">{generation.inferred_type}</span></div>
				<div><strong>canonical semantics:</strong> <span class="font-mono break-all">{generation.canonical_semantics}</span></div>
				<div><strong>round-trip:</strong> {generation.roundtrip_verified ? 'verified' : 'failed'}</div>
			</div>
		{/if}
	</section>

	<section class="bg-accent/10 border-2 border-accent/10 rounded-xl p-4">
		<h2 class="text-2xl font-black">Number / quantity / time inspector</h2>
		<div class="flex flex-wrap items-center gap-2 mt-2">
			<input class="input m-0 grow" bind:value={literalSource} placeholder="structured literal" onkeydown={(event) => event.key === 'Enter' && inspectLiteral()} />
			<button class="small-text link" onclick={inspectLiteral}>Inspect</button>
		</div>
		{#if literalError}
			<div class="mt-3 rounded-lg border border-red-400/30 bg-red-400/10 p-3"><strong>{literalError.layer}</strong>: {literalError.message}</div>
		{:else if literalAnalysis}
			<div class="font-mono text-sm mt-3 grid gap-1 break-all">
				<div>family: {literalAnalysis.family} · type: {literalAnalysis.ty} · input: {literalAnalysis.realization}</div>
				<div>semantic: {literalAnalysis.semantic_canonical}</div>
				<div>written: {literalAnalysis.canonical_written}</div>
				<div>spoken: {literalAnalysis.canonical_spoken}</div>
			</div>
		{/if}
	</section>
</div>
