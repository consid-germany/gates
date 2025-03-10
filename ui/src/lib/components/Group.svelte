<script lang="ts">
	import Service from './Service.svelte';
	import type { Group } from '$lib/api';

	export let group: Group;
</script>

<div class="space-y-14">
	{#each group.services as service (service.name)}
		<Service {service} />
	{/each}
</div>
