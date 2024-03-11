<script lang="ts">
  import { Badge, Button, GradientButton, Input, Modal, Spinner, ToolbarButton } from 'flowbite-svelte';
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
    }

    const addCommentToGate = async () => {
        const newMessage = commentMessage;
        commentMessage = "";
        try {
            addCommentLoading = true;
            gate = await api.addCommentToGate(gate, newMessage);
        } catch (error) {
            showError(error as Error);
        } finally {
            addCommentLoading = false;
        }
    }

    const removeCommentFromGate = async (commentId: string) => {
        try {
            gate = await api.removeCommentFromGate(gate, commentId);
        } catch (error) {
            showError(error as Error);
        }
    }
</script>

<div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-5">
    <Modal open={!!error} size="xs" autoclose>
        <div class="text-center">
            <IconWarn/>
            <h3 class="mb-5 text-lg font-normal text-gray-500 dark:text-gray-400">{error}</h3>
            <Button class="mr-2">Okay</Button>
        </div>
    </Modal>

    <div class="flex items-center space-x-2">
        <h1 class="font-bold text-xl inline-block">{gate.service}</h1>
        <Badge color="blue">{gate.environment}</Badge>
    </div>
    <div class="text-xs text-gray-400 dark:text-gray-500 mt-1 flex space-x-1 items-center">
        <IconClock/>
        <span>{new Date(gate.last_updated).toLocaleString()}</span>
    </div>
    <div class="flex mt-5">
        <GradientButton
                color={gate.state === "open" ? "green" : "red"}
                class="w-32 h-16 rounded-r-none"
                on:click={toggleGateState}>
            {#if toggleGateStateLoading}
                <Spinner class="mr-2" size="3" color="white" />
            {/if}
            {gate.state}
        </GradientButton>

        <div class="bg-gray-100 dark:bg-gray-700 dark:border-gray-600 rounded-lg rounded-tl-none w-full">
            {#if gate.comments.length > 0}
                <div class="p-2 w-full space-y-2">
                    {#each gate.comments as comment}
                        <div class="rounded-lg dark:text-gray-400 bg-gray-200 dark:bg-gray-600 py-2 px-3 flex w-full">
                            <div class="grow flex flex-col">
                                <span class="text-[0.6em] text-gray-500 dark:text-gray-400">{new Date(comment.created).toLocaleString()}</span>
                                <span class="text-gray-600 dark:text-gray-300">{comment.message}</span>
                            </div>
                            <button on:click={() => removeCommentFromGate(comment.id)}
                                    class="opacity-50 hover:opacity-100 transition-opacity">
                                <IconDelete/>
                            </button>
                        </div>
                    {/each}
                    {#if addCommentLoading}
                        <div class="rounded-lg dark:text-gray-400 bg-gray-300 dark:bg-gray-600 py-2 px-3 flex w-full">
                            <div class="grow flex flex-col">
                                <Spinner class="mt-2 mb-2 ml-2 dark:text-gray-700" color="gray" size="4" />
                            </div>
                        </div>
                    {/if}
                </div>
            {/if}
            <form class="flex rounded-lg p-2" on:submit|preventDefault={addCommentToGate}>
                <Input bind:value={commentMessage} rows="1" placeholder="Your message..."/>
                <ToolbarButton type="submit" color="blue" class="ms-2 rounded-full text-blue-600 dark:text-blue-500">
                    <IconSend/>
                    <span class="sr-only">Send message</span>
                </ToolbarButton>
            </form>
        </div>
    </div>
</div>

