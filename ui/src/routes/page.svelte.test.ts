import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import page from './+page.svelte';
import { cleanup, fireEvent, render, waitFor } from '@testing-library/svelte';
import { getGroups, type Group } from '$lib/api';

beforeEach(() => {
	vi.mock('$lib/api', () => ({
		getGroups: vi.fn()
	}));
});

afterEach(() => {
	vi.restoreAllMocks();
	cleanup();
});

it('should show loading spinner when loading gates', () => {
	// given
	vi.mocked(getGroups).mockImplementation(() => new Promise(() => {}));

	// when
	const { container } = render(page);

	// then
	const spinner = container.querySelector('.loading-spinner');
	expect(spinner).not.toBeNull();
	const svgElement = spinner?.querySelector('svg');
	expect(svgElement).not.toBeNull();
	expect(svgElement?.classList.contains('animate-spin')).toBeTruthy();
});

it('should show gate groups in tabs', async () => {
	// given
	vi.mocked(getGroups).mockResolvedValue(getTestGates());

	// when
	const { container } = render(page);

	// then
	await waitFor(() => {
		const spinner = container.querySelector('.loading-spinner');
		expect(spinner).toBeNull();
	});

	const tabs = container.querySelectorAll("button[role='tab']");
	expect(tabs.length).toEqual(2);
	expect(tabs.item(0).textContent).toEqual('some-group');
	expect(tabs.item(1).textContent).toEqual('some-other-group');
});

it('should show gates of selected group', async () => {
	// given
	vi.mocked(getGroups).mockResolvedValue(getTestGates());

	// when
	const { container } = render(page);

	// then
	await waitFor(() => {
		const spinner = container.querySelector('.loading-spinner');
		expect(spinner).toBeNull();
	});

	const activeTab = container.querySelector("button[role='tab'].active");
	expect(activeTab?.textContent).toEqual('some-group');
	const gateGroups = container.querySelectorAll('.gates-group');
	expect(gateGroups.length).toEqual(1);
	const serviceTitle = gateGroups.item(0).querySelector('h1');
	expect(serviceTitle?.innerHTML).toEqual('some-service');
	const environment = gateGroups.item(0).querySelector('.gate-environment');
	expect(environment?.innerHTML).toEqual(expect.stringContaining('some-environment'));
});

it('should show gates of other tab', async () => {
	// given
	vi.mocked(getGroups).mockResolvedValue(getTestGates());

	// when
	const { container } = render(page);

	// then
	await waitFor(() => {
		const spinner = container.querySelector('.loading-spinner');
		expect(spinner).toBeNull();
	});

	expect(container.querySelector("button[role='tab'].active")?.textContent).toEqual('some-group');

	// when
	const tab = container.querySelectorAll("button[role='tab']").item(1);
	expect(tab?.textContent).toEqual('some-other-group');
	await fireEvent.click(tab);

	// then
	expect(container.querySelector("button[role='tab'].active")?.textContent).toEqual(
		'some-other-group'
	);

	const gateGroups = container.querySelectorAll('.gates-group');
	expect(gateGroups.length).toEqual(1);
	const serviceTitle = gateGroups.item(0).querySelector('h1');
	expect(serviceTitle?.innerHTML).toEqual('some-other-service');
	const environment = gateGroups.item(0).querySelector('.gate-environment');
	expect(environment?.innerHTML).toEqual(expect.stringContaining('some-other-environment'));
});

it('should show error message if loading gates fails', async () => {
	// given
	vi.mocked(getGroups).mockRejectedValue({
		message: 'Some error occurred when loading gates!'
	});

	// when
	const { container } = render(page);

	// then
	await waitFor(() => {
		const spinner = container.querySelector('.loading-spinner');
		expect(spinner).toBeNull();
	});

	const error = container.querySelector('.error');
	expect(error).not.toBeNull();
	expect(error?.innerHTML).toEqual('Some error occurred when loading gates!');
});

function getTestGates(): Group[] {
	return [
		{
			name: 'some-group',
			services: [
				{
					name: 'some-service',
					environments: [
						{
							name: 'some-environment',
							gate: {
								group: 'some-group',
								service: 'some-service',
								environment: 'some-environment',
								state: 'open',
								comments: [],
								last_updated: '2024-03-13T18:24:14.265799400Z'
							}
						}
					]
				}
			]
		},
		{
			name: 'some-other-group',
			services: [
				{
					name: 'some-other-service',
					environments: [
						{
							name: 'some-environment',
							gate: {
								group: 'some-other-group',
								service: 'some-other-service',
								environment: 'some-other-environment',
								state: 'open',
								comments: [],
								last_updated: '2024-03-14T18:24:14.265799400Z'
							}
						}
					]
				}
			]
		}
	];
}
