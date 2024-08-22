<script lang="ts">
	import '../app.css';
</script>

<div
	class="flex gap-1 m-1 overflow-auto sticky top-2 bg-stone-800/30 backdrop-blur-lg z-10 p-1 rounded-xl"
>
	<a href="/" class="small-text link max-w-64">Main</a>
	<a href="/alphabet" class="small-text link max-w-64">Alphabet</a>
	<a href="/dictionary" class="small-text link max-w-64">Dictionary</a>
</div>

<div class="flex flex-col justify-center items-center gap-6 p-4">
	<slot />
</div>
