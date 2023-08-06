<script>
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import { doStuff } from '../lib/index.ts';

	onMount(() => {
		let promise = invoke('my_custom_command', { value: 'OnMount' });

		let paragraphElement = document.createElement('p');
	});
</script>

<h1>Finanz Analyse<h1>

{#await doStuff()}
	Lädt
{:then value}
	{@html value}
{:catch error}
	<h1>{error}</h1>
{/await}
