import { expect, it } from 'vitest';
import GroupComponent from './Group.svelte';
import { type Group } from '$lib/api';
import { render } from '@testing-library/svelte';

it('should show services of group', () => {
	// given
	const group: Group = {
		name: 'some-group',
		services: [
			{
				name: 'some-service',
				environments: []
			},
			{
				name: 'some-other-service',
				environments: []
			},
			{
				name: 'some-third-service',
				environments: []
			}
		]
	};

	// when
	const { container } = render(GroupComponent, {
		group
	});

	// then
	const gateServices = container.querySelectorAll('.gate-service');
	expect(gateServices.length).toEqual(3);
});
