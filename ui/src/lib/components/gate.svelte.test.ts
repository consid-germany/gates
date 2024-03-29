import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import GateComponent from './Gate.svelte';
import userEvent from '@testing-library/user-event';
import { addCommentToGate, type Gate, type GateState, removeCommentFromGate, toggleGateState } from '$lib/api';
import { cleanup, render, waitFor } from '@testing-library/svelte';

beforeEach(() => {
	vi.mock('$lib/api', () => ({
		toggleGateState: vi.fn(),
		addCommentToGate: vi.fn(),
		removeCommentFromGate: vi.fn(),
	}));
});

afterEach(() => {
	vi.restoreAllMocks();
	cleanup();
});

it('should show service of gate', () => {
	// given
	const gate = someGate("open");

	// when
	const { container } = render(GateComponent, {
		gate
	});

	// then
	const gateService = container.querySelector('.gate-service-name');
	expect(gateService?.innerHTML).toEqual("some-service");
});

it('should show environment of gate', () => {
	// given
	const gate = someGate("open");

	// when
	const { container } = render(GateComponent, {
		gate
	});

	// then
	const environment = container.querySelector('.gate-environment');
	expect(environment?.innerHTML).toEqual(expect.stringContaining('some-environment'));
});

it('should show state of gate', () => {
	// given
	const gate = someGate("open");

	// when
	const { container } = render(GateComponent, {
		gate
	});

	// then
	const gateState = container.querySelector('.gate-state');
	expect(gateState?.innerHTML).toEqual(expect.stringContaining('open'));
});

it('should show last modification of gate', () => {
	// given
	const gate = someGate("open");

	// when
	const { container } = render(GateComponent, {
		gate
	});

	// then
	const lastModified = container.querySelector('.gate-last-modified');
	expect(lastModified?.innerHTML).toEqual(new Date(gate.last_updated).toLocaleString());
});

it('should show comments of gate', () => {
	// given
	const gate = someGate("open");

	// when
	const { container } = render(GateComponent, {
		gate
	});

	// then
	const commentMessages = container.querySelectorAll('.gate-comment-message');
	expect(commentMessages.length).toBe(2);
	expect(commentMessages.item(0).innerHTML).toEqual("Some comment message 1.");
	expect(commentMessages.item(1).innerHTML).toEqual("Some comment message 2.");

	const commentCreatedDates = container.querySelectorAll('.gate-comment-created');
	expect(commentCreatedDates.length).toBe(2);
	expect(commentCreatedDates.item(0).innerHTML).toEqual(new Date(gate.comments[0].created).toLocaleString());
	expect(commentCreatedDates.item(1).innerHTML).toEqual(new Date(gate.comments[1].created).toLocaleString());
});

it('should show gate state loading when clicking gate state button', async () => {
	// given
	const user = userEvent.setup();

	vi.mocked(toggleGateState).mockImplementation(() => new Promise(() => {}));
	const gate = someGate("open");

	// when
	const { container } = render(GateComponent, {
		gate
	});

	const gateState = container.querySelector('.gate-state');
	await user.click(gateState!);

	// then
	const gateStateLoading = container.querySelector(".gate-state-loading");
	expect(gateStateLoading).not.toBeNull();
	expect(gateStateLoading?.classList.contains('animate-spin')).toBeTruthy();
});

it('should toggle gate state when clicking gate state button', async () => {
	// given
	const user = userEvent.setup();

	const toggledGate = someGate("closed", "2025-03-13T18:24:14.265799400Z");
	vi.mocked(toggleGateState).mockResolvedValue(toggledGate);
	const gate = someGate("open");

	// when
	const { container } = render(GateComponent, {
		gate
	});

	const gateState = container.querySelector('.gate-state');
	expect(gateState?.innerHTML).toEqual(expect.stringContaining('open'));

	const lastModified = container.querySelector('.gate-last-modified');
	expect(lastModified?.innerHTML).toEqual(new Date("2024-03-13T18:24:14.265799400Z").toLocaleString());

	await user.click(gateState!);

	// then
	await waitFor(() => {
		const gateStateLoading = container.querySelector('.gate-state-loading');
		expect(gateStateLoading).toBeNull();
	});

	const error = container.querySelector(".error");
	expect(error).toBeNull();

	expect(gateState?.innerHTML).toEqual(expect.stringContaining('closed'));
	const lastModifiedAfterToggle = container.querySelector('.gate-last-modified');
	expect(lastModifiedAfterToggle?.innerHTML).toEqual(new Date("2025-03-13T18:24:14.265799400Z").toLocaleString());
});

it('should should show error if toggling gate state fails', async () => {
	// given
	const user = userEvent.setup();

	vi.mocked(toggleGateState).mockRejectedValue("Could not toggle gate state because of some error!");
	const gate = someGate("open");

	// when
	const { container } = render(GateComponent, {
		gate
	});

	const gateState = container.querySelector('.gate-state');
	expect(gateState?.innerHTML).toEqual(expect.stringContaining('open'));

	const lastModified = container.querySelector('.gate-last-modified');
	expect(lastModified?.innerHTML).toEqual(new Date("2024-03-13T18:24:14.265799400Z").toLocaleString());

	await user.click(gateState!);

	// then
	await waitFor(() => {
		const gateStateLoading = container.querySelector('.gate-state-loading');
		expect(gateStateLoading).toBeNull();
	});

	const error = container.querySelector(".error");
	expect(error).not.toBeNull();
	const errorText = error?.querySelector(".error-text");
	expect(errorText?.innerHTML).toEqual("Could not toggle gate state because of some error!");

	// when (close error)
	const errorCloseButton = error?.querySelector(".error-close-button");
	await user.click(errorCloseButton!);

	// then
	expect(container.querySelector(".error")).toBeNull();

	expect(gateState?.innerHTML).toEqual(expect.stringContaining('open'));
	const lastModifiedAfterToggle = container.querySelector('.gate-last-modified');
	expect(lastModifiedAfterToggle?.innerHTML).toEqual(new Date("2024-03-13T18:24:14.265799400Z").toLocaleString());
});

it('should show comment loading when adding new comment', async () => {
	// given
	const user = userEvent.setup();

	vi.mocked(addCommentToGate).mockImplementation(() => new Promise(() => {}));
	const gate = someGate("open");

	// when
	const { container } = render(GateComponent, {
		gate
	});

	const newCommentMessage = container.querySelector('.gate-new-comment-message');
	await user.type(newCommentMessage!, "Some new comment message.");

	const newCommentSubmit = container.querySelector(".gate-new-comment-submit");
	await user.click(newCommentSubmit!);

	// then
	const gateCommentLoading = container.querySelector(".gate-comment-loading");
	expect(gateCommentLoading).not.toBeNull();
	expect(gateCommentLoading?.classList.contains('animate-spin')).toBeTruthy();
});

it('should show clear comment message input when submitting new comment', async () => {
	// given
	const user = userEvent.setup();

	vi.mocked(addCommentToGate).mockImplementation(() => new Promise(() => {}));
	const gate = someGate("open");

	// when
	const { container } = render(GateComponent, {
		gate
	});

	const newCommentMessage: HTMLInputElement | null = container.querySelector('.gate-new-comment-message');
	await user.type(newCommentMessage!, "Some new comment message.");

	// then
	expect(newCommentMessage?.value).toEqual("Some new comment message.");

	// when (submit comment)
	const newCommentSubmit = container.querySelector(".gate-new-comment-submit");
	await user.click(newCommentSubmit!);

	// then
	expect(newCommentMessage?.value).toEqual("");
});

it('should show updated gate comments when adding new comment', async () => {
	// given
	const user = userEvent.setup();

	const updatedGate = someGate("open", "2025-03-13T18:24:14.265799400Z");
	updatedGate.comments.push({
		id: 'new-comment-id',
		message: 'Some stored new comment message.',
		created: '2025-03-15T18:24:14.265799400Z'
	});
	vi.mocked(addCommentToGate).mockResolvedValue(updatedGate);
	const gate = someGate("open");

	// when
	const { container } = render(GateComponent, {
		gate
	});

	const commentMessages = container.querySelectorAll('.gate-comment-message');
	expect(commentMessages.length).toBe(2);

	const newCommentMessage = container.querySelector('.gate-new-comment-message');
	await user.type(newCommentMessage!, "Some new comment message.");

	const newCommentSubmit = container.querySelector(".gate-new-comment-submit");
	await user.click(newCommentSubmit!);

	// then
	await waitFor(() => {
		const gateCommentLoading = container.querySelector(".gate-comment-loading");
		expect(gateCommentLoading).toBeNull();
	});

	const commentMessagesAfterSubmit = container.querySelectorAll('.gate-comment-message');
	expect(commentMessagesAfterSubmit.length).toBe(3);
	expect(commentMessagesAfterSubmit.item(0).innerHTML).toEqual("Some comment message 1.");
	expect(commentMessagesAfterSubmit.item(1).innerHTML).toEqual("Some comment message 2.");
	expect(commentMessagesAfterSubmit.item(2).innerHTML).toEqual("Some stored new comment message.");

	const commentCreatedDates = container.querySelectorAll('.gate-comment-created');
	expect(commentCreatedDates.length).toBe(3);
	expect(commentCreatedDates.item(0).innerHTML).toEqual(new Date(updatedGate.comments[0].created).toLocaleString());
	expect(commentCreatedDates.item(1).innerHTML).toEqual(new Date(updatedGate.comments[1].created).toLocaleString());
	expect(commentCreatedDates.item(2).innerHTML).toEqual(new Date(updatedGate.comments[2].created).toLocaleString());

	const lastModifiedAfterSubmit = container.querySelector('.gate-last-modified');
	expect(lastModifiedAfterSubmit?.innerHTML).toEqual(new Date("2025-03-13T18:24:14.265799400Z").toLocaleString());
});

it('should should show error if adding new comment fails', async () => {
	// given
	const user = userEvent.setup();

	vi.mocked(addCommentToGate).mockRejectedValue("Could not add comment because of some error!");
	const gate = someGate("open");

	// when
	const { container } = render(GateComponent, {
		gate
	});

	const commentMessages = container.querySelectorAll('.gate-comment-message');
	expect(commentMessages.length).toBe(2);

	const lastModified = container.querySelector('.gate-last-modified');
	expect(lastModified?.innerHTML).toEqual(new Date("2024-03-13T18:24:14.265799400Z").toLocaleString());

	const newCommentMessage = container.querySelector('.gate-new-comment-message');
	await user.type(newCommentMessage!, "Some new comment message.");

	const newCommentSubmit = container.querySelector(".gate-new-comment-submit");
	await user.click(newCommentSubmit!);

	// then
	await waitFor(() => {
		const gateStateLoading = container.querySelector('.gate-state-loading');
		expect(gateStateLoading).toBeNull();
	});

	const error = container.querySelector(".error");
	expect(error).not.toBeNull();
	const errorText = error?.querySelector(".error-text");
	expect(errorText?.innerHTML).toEqual("Could not add comment because of some error!");

	// when (close error)
	const errorCloseButton = error?.querySelector(".error-close-button");
	await user.click(errorCloseButton!);

	// then
	expect(container.querySelector(".error")).toBeNull();

	const commentMessagesAfterSubmit = container.querySelectorAll('.gate-comment-message');
	expect(commentMessagesAfterSubmit.length).toBe(2);
	expect(commentMessagesAfterSubmit.item(0).innerHTML).toEqual("Some comment message 1.");
	expect(commentMessagesAfterSubmit.item(1).innerHTML).toEqual("Some comment message 2.");
	const lastModifiedAfterSubmit = container.querySelector('.gate-last-modified');
	expect(lastModifiedAfterSubmit?.innerHTML).toEqual(new Date("2024-03-13T18:24:14.265799400Z").toLocaleString());
});

it('should show updated gate comments when removing comment', async () => {
	// given
	const user = userEvent.setup();

	const updatedGate = someGate("open", "2025-03-13T18:24:14.265799400Z");
	updatedGate.comments.pop();
	vi.mocked(removeCommentFromGate).mockResolvedValue(updatedGate);
	const gate = someGate("open");

	// when
	const { container } = render(GateComponent, {
		gate
	});

	const commentMessages = container.querySelectorAll('.gate-comment-message');
	expect(commentMessages.length).toBe(2);

	const removeCommentButton = container.querySelector(".gate-comment-remove-button");
	await user.click(removeCommentButton!);

	// then
	const commentMessagesAfterDelete = container.querySelectorAll('.gate-comment-message');
	expect(commentMessagesAfterDelete.length).toBe(1);
	expect(commentMessagesAfterDelete.item(0).innerHTML).toEqual("Some comment message 1.");

	const commentCreatedDates = container.querySelectorAll('.gate-comment-created');
	expect(commentCreatedDates.length).toBe(1);
	expect(commentCreatedDates.item(0).innerHTML).toEqual(new Date(updatedGate.comments[0].created).toLocaleString());

	const lastModifiedAfterDelete = container.querySelector('.gate-last-modified');
	expect(lastModifiedAfterDelete?.innerHTML).toEqual(new Date("2025-03-13T18:24:14.265799400Z").toLocaleString());
});

it('should should show error if removing comment fails', async () => {
	// given
	const user = userEvent.setup();

	vi.mocked(removeCommentFromGate).mockRejectedValue("Could not remove comment because of some error!");
	const gate = someGate("open");

	// when
	const { container } = render(GateComponent, {
		gate
	});

	const commentMessages = container.querySelectorAll('.gate-comment-message');
	expect(commentMessages.length).toBe(2);

	const lastModified = container.querySelector('.gate-last-modified');
	expect(lastModified?.innerHTML).toEqual(new Date("2024-03-13T18:24:14.265799400Z").toLocaleString());

	const removeCommentButton = container.querySelector(".gate-comment-remove-button");
	await user.click(removeCommentButton!);

	// then
	const error = container.querySelector(".error");
	expect(error).not.toBeNull();
	const errorText = error?.querySelector(".error-text");
	expect(errorText?.innerHTML).toEqual("Could not remove comment because of some error!");

	// when (close error)
	const errorCloseButton = error?.querySelector(".error-close-button");
	await user.click(errorCloseButton!);

	// then
	expect(container.querySelector(".error")).toBeNull();

	const commentMessagesAfterDelete = container.querySelectorAll('.gate-comment-message');
	expect(commentMessagesAfterDelete.length).toBe(2);
	expect(commentMessagesAfterDelete.item(0).innerHTML).toEqual("Some comment message 1.");
	expect(commentMessagesAfterDelete.item(1).innerHTML).toEqual("Some comment message 2.");
	const lastModifiedAfterDelete = container.querySelector('.gate-last-modified');
	expect(lastModifiedAfterDelete?.innerHTML).toEqual(new Date("2024-03-13T18:24:14.265799400Z").toLocaleString());
});

function someGate(gateState: GateState, lastUpdated = '2024-03-13T18:24:14.265799400Z'): Gate {
	return {
		group: 'some-group',
		service: 'some-service',
		environment: 'some-environment',
		state: gateState,
		comments: [
			{
				id: 'c1',
				message: 'Some comment message 1.',
				created: '2024-03-14T18:24:14.265799400Z'
			},
			{
				id: 'c2',
				message: 'Some comment message 2.',
				created: '2024-03-15T18:24:14.265799400Z'
			}
		],
		last_updated: lastUpdated
	};
}
