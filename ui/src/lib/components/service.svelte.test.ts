import { expect, it } from 'vitest';
import ServiceComponent from './Service.svelte';
import { type Service } from '$lib/api';
import { render } from '@testing-library/svelte';

it('should show gate for each environment of service', () => {
	// given
	const service: Service = {
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
			},
			{
				name: 'some-other-environment',
				gate: {
					group: 'some-group',
					service: 'some-service',
					environment: 'some-other-environment',
					state: 'open',
					comments: [],
					last_updated: '2024-03-13T18:24:14.265799400Z'
				}
			},
			{
				name: 'some-third-environment',
				gate: {
					group: 'some-group',
					service: 'some-service',
					environment: 'some-third-environment',
					state: 'open',
					comments: [],
					last_updated: '2024-03-13T18:24:14.265799400Z'
				}
			}
		],
	}

	// when
	const { container } = render(ServiceComponent, {
		service
	});

	// then
	const gates = container.querySelectorAll('.gate');
	expect(gates.length).toEqual(3);
});
