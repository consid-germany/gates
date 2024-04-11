<script lang="ts">
	import { Spinner, TabItem, Tabs } from 'flowbite-svelte';

	import { getGroups } from '$lib/api';
	import Group from '$lib/components/Group.svelte';

	let groups = getGroups();
</script>

{#await groups}
	<div class="loading-spinner mt-10 flex justify-center">
		<Spinner />
	</div>
{:then groups}
	<Tabs contentClass="mt-10">
		{#each groups as group, i}
			<TabItem title={group.name} open={i === 0}>
				<div class="gates-group">
					<Group {group} />
				</div>
			</TabItem>
		{/each}
	</Tabs>
{:catch error}
	<p class="error text-red-500">{error.message}</p>
{/await}
