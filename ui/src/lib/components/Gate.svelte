<script lang="ts">
	import {
		Badge,
		Button,
		GradientButton,
		Input,
		Modal,
		Spinner,
		ToolbarButton
	} from 'flowbite-svelte';
	import IconClock from '$lib/icons/IconClock.svelte';
	import IconDelete from '$lib/icons/IconDelete.svelte';
	import IconSend from '$lib/icons/IconSend.svelte';
	import IconWarn from '$lib/icons/IconWarn.svelte';
	import type { Gate } from '$lib/api';
	import * as api from '$lib/api';

	export let gate: Gate;

	let commentMessage: string;
	let error: Error;
	let toggleGateStateLoading = false;
	let addCommentLoading = false;

	const showError = (newError: Error) => {
		error = newError;
	};

	const toggleGateState = async () => {
		try {
			toggleGateStateLoading = true;
			gate = await api.toggleGateState(gate);
		} catch (error) {
			showError(error as Error);
		} finally {
			toggleGateStateLoading = false;
		}
	};

	const addCommentToGate = async () => {
		const newMessage = commentMessage;
		commentMessage = '';
		try {
			addCommentLoading = true;
			gate = await api.addCommentToGate(gate, newMessage);
		} catch (error) {
			showError(error as Error);
		} finally {
			addCommentLoading = false;
		}
	};

	const removeCommentFromGate = async (commentId: string) => {
		try {
			gate = await api.removeCommentFromGate(gate, commentId);
		} catch (error) {
			showError(error as Error);
		}
	};
</script>

<div class="gate rounded-lg bg-gray-50 p-5 dark:bg-gray-800 dark:text-neutral-200">
	<Modal class="error" open={!!error} size="xs" autoclose>
		<div class="text-center">
			<IconWarn />
			<h3 class="error-text mb-5 text-lg font-normal text-gray-500 dark:text-gray-400">{error}</h3>
			<Button class="error-close-button mr-2">Okay</Button>
		</div>
	</Modal>

	<div class="flex items-center space-x-2">
		<h1 class="gate-service-name inline-block text-xl font-bold">{gate.service}</h1>
		<Badge class="gate-environment" color="blue">{gate.environment}</Badge>
	</div>
	<div class="mt-1 flex items-center space-x-1 text-xs text-gray-400 dark:text-gray-500">
		<IconClock />
		<span class="gate-last-modified">{new Date(gate.last_updated).toLocaleString()}</span>
	</div>
	<div class="mt-5 flex">
		<GradientButton
			color={gate.state === 'open' ? 'green' : 'red'}
			class="gate-state h-16 w-32 rounded-r-none"
			on:click={toggleGateState}
		>
			{#if toggleGateStateLoading}
				<Spinner class="gate-state-loading mr-2" size="3" color="white" />
			{/if}
			{gate.state}
		</GradientButton>

		<div
			class="w-full rounded-lg rounded-tl-none bg-gray-100 dark:border-gray-600 dark:bg-gray-700"
		>
			{#if gate.comments.length > 0}
				<div class="w-full space-y-2 p-2">
					{#each gate.comments as comment}
						<div
							class="flex w-full rounded-lg bg-gray-200 px-3 py-2 dark:bg-gray-600 dark:text-gray-400"
						>
							<div class="flex grow flex-col">
								<span class="gate-comment-created text-[0.6em] text-gray-500 dark:text-gray-400"
									>{new Date(comment.created).toLocaleString()}</span
								>
								<span class="gate-comment-message text-gray-600 dark:text-gray-300"
									>{comment.message}</span
								>
							</div>
							<button
								on:click={() => removeCommentFromGate(comment.id)}
								class="gate-comment-remove-button opacity-50 transition-opacity hover:opacity-100"
							>
								<IconDelete />
							</button>
						</div>
					{/each}
					{#if addCommentLoading}
						<div
							class="flex w-full rounded-lg bg-gray-300 px-3 py-2 dark:bg-gray-600 dark:text-gray-400"
						>
							<div class="flex grow flex-col">
								<Spinner
									class="gate-comment-loading mb-2 ml-2 mt-2 dark:text-gray-700"
									color="gray"
									size="4"
								/>
							</div>
						</div>
					{/if}
				</div>
			{/if}
			<form class="flex rounded-lg p-2" on:submit|preventDefault={addCommentToGate}>
				<Input
					class="gate-new-comment-message"
					bind:value={commentMessage}
					rows="1"
					placeholder="Your message..."
				/>
				<ToolbarButton
					type="submit"
					color="blue"
					class="gate-new-comment-submit ms-2 rounded-full text-blue-600 dark:text-blue-500"
				>
					<IconSend />
					<span class="sr-only">Send message</span>
				</ToolbarButton>
			</form>
		</div>
	</div>
</div>
