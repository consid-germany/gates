<script lang="ts">
	import { Spinner, TabItem, Tabs } from 'flowbite-svelte';

	import { getGroups } from '$lib/api';
	import Group from '$lib/components/Group.svelte';

	let groups = getGroups();
</script>

{#await groups}
	<div class="flex justify-center mt-10">
		<Spinner />
	</div>
{:then groups}
	<Tabs contentClass="mt-10">
		{#each groups as group, i}
			<TabItem title={group.name} open={i===0}>
				<Group {group} />
			</TabItem>
		{/each}
	</Tabs>
{:catch error}
	<p class="text-red-500">{error.message}</p>
{/await}
