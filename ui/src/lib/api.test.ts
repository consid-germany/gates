import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { addCommentToGate, type Gate, getGroups, type Group, removeCommentFromGate, toggleGateState } from '$lib/api';
import createClient from 'openapi-fetch';

const clientMock = createClient();

beforeAll(() => {
	vi.mock('openapi-fetch', () => {
		const mockClient = {
			DELETE: vi.fn(),
			POST: vi.fn(),
			PUT: vi.fn(),
			GET: vi.fn(),
		};

		return ({
			default: () => mockClient
		});
	});
});

afterEach(() => {
	vi.restoreAllMocks();
});


describe('should get groups', () => {
	it('should return groups', async () => {
		// given
		const group1: Group = {
			name: 'group1',
			services: []
		};

		const group2: Group = {
			name: 'group2',
			services: []
		};

		const group3: Group = {
			name: 'group3',
			services: []
		};

		vi.mocked(clientMock.GET).mockResolvedValue({
			data: [group1, group2, group3]
		});

		// when
		const result = await getGroups();

		// then
		expect(result).toEqual([group1, group2, group3]);

		expect(clientMock.GET).toHaveBeenCalledOnce();
		expect(clientMock.GET).toBeCalledWith('/gates', {});
	});

	it('should throw an error if there is an error from the server', async () => {
		// given
		vi.mocked(clientMock.GET).mockResolvedValue({
			error: 'some error!'
		});

		// when
		const result = getGroups();

		// then
		expect(result).rejects.toThrow('some error!');

		expect(clientMock.GET).toHaveBeenCalledOnce();
		expect(clientMock.GET).toBeCalledWith('/gates', {});
	});

	it('should throw an error if updated gate is undefined', async () => {
		// given
		vi.mocked(clientMock.GET).mockResolvedValue({});

		// when
		const result = getGroups();

		// then
		expect(result).rejects.toThrow('could not retrieve gates');

		expect(clientMock.GET).toHaveBeenCalledOnce();
		expect(clientMock.GET).toBeCalledWith('/gates', {});
	});

	it('should throw an error if fetch fails', async () => {
		// given
		vi.mocked(clientMock.GET).mockRejectedValue('some error!');

		// when
		const result = getGroups();

		// then
		expect(result).rejects.toThrow('some error!');

		expect(clientMock.GET).toHaveBeenCalledOnce();
		expect(clientMock.GET).toBeCalledWith('/gates', {});
	});
});


describe('should toggle state of gate', () => {
	it('should close gate if state is currently open and return updated gate', async () => {
		// given
		const gate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'open',
			comments: [],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		const updatedGate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'closed',
			comments: [],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		vi.mocked(clientMock.PUT).mockResolvedValue({
			data: updatedGate
		});

		// when
		const result = await toggleGateState(gate);

		// then
		expect(result).toEqual(updatedGate);

		expect(clientMock.PUT).toHaveBeenCalledOnce();
		expect(clientMock.PUT).toBeCalledWith(
			'/gates/{group}/{service}/{environment}/state',
			{
				params: {
					path: {
						group: gate.group,
						service: gate.service,
						environment: gate.environment
					}
				},
				body: {
					state: "closed"
				}
			}
		);
	});

	it('should open gate if state is currently closed and return updated gate', async () => {
		// given
		const gate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'closed',
			comments: [],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		const updatedGate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'open',
			comments: [],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		vi.mocked(clientMock.PUT).mockResolvedValue({
			data: updatedGate
		});

		// when
		const result = await toggleGateState(gate);

		// then
		expect(result).toEqual(updatedGate);

		expect(clientMock.PUT).toHaveBeenCalledOnce();
		expect(clientMock.PUT).toBeCalledWith(
			'/gates/{group}/{service}/{environment}/state',
			{
				params: {
					path: {
						group: gate.group,
						service: gate.service,
						environment: gate.environment
					}
				},
				body: {
					state: "open"
				}
			}
		);
	});

	it('should throw error if state is invalid', async () => {
		// given
		const gate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			// @ts-expect-error test invalid value
			state: 'invalid_state',
			comments: [],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		// when
		const result = toggleGateState(gate);

		// then
		expect(result).rejects.toThrow('unknown gate state: invalid_state');

		expect(clientMock.PUT).toHaveBeenCalledTimes(0);
	});

	it('should throw an error if there is an error from the server', async () => {
		// given
		const gate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'closed',
			comments: [],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		vi.mocked(clientMock.PUT).mockResolvedValue({
			error: 'some error!'
		});

		// when
		const result = toggleGateState(gate);

		// then
		expect(result).rejects.toThrow('some error!');

		expect(clientMock.PUT).toHaveBeenCalledOnce();
		expect(clientMock.PUT).toBeCalledWith(
			'/gates/{group}/{service}/{environment}/state',
			{
				params: {
					path: {
						group: gate.group,
						service: gate.service,
						environment: gate.environment
					}
				},
				body: {
					state: "open"
				}
			}
		);
	});

	it('should throw an error if updated gate is undefined', async () => {
		// given
		const gate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'open',
			comments: [],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		vi.mocked(clientMock.PUT).mockResolvedValue({});

		// when
		const result = toggleGateState(gate);

		// then
		expect(result).rejects.toThrow('could not retrieve updated gate');

		expect(clientMock.PUT).toHaveBeenCalledOnce();
		expect(clientMock.PUT).toBeCalledWith(
			'/gates/{group}/{service}/{environment}/state',
			{
				params: {
					path: {
						group: gate.group,
						service: gate.service,
						environment: gate.environment
					}
				},
				body: {
					state: "closed"
				}
			}
		);
	});

	it('should throw an error if fetch fails', async () => {
		// given
		const gate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'open',
			comments: [],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		vi.mocked(clientMock.PUT).mockRejectedValue('some error!');

		// when
		const result = toggleGateState(gate);

		// then
		expect(result).rejects.toThrow('some error!');

		expect(clientMock.PUT).toHaveBeenCalledOnce();
		expect(clientMock.PUT).toBeCalledWith(
			'/gates/{group}/{service}/{environment}/state',
			{
				params: {
					path: {
						group: gate.group,
						service: gate.service,
						environment: gate.environment
					}
				},
				body: {
					state: "closed"
				}
			}
		);
	});
});


describe('should add comment to gate', () => {
	it('should return updated gate', async () => {
		// given
		const gate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'open',
			comments: [],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		const updatedGate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'open',
			comments: [
				{
					id: 'comment-id-1',
					message: 'Some comment message.',
					created: '2024-03-15T18:24:14.265799400Z'
				}
			],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		vi.mocked(clientMock.POST).mockResolvedValue({
			data: updatedGate
		});

		// when
		const result = await addCommentToGate(gate, 'some new comment message');

		// then
		expect(result).toEqual(updatedGate);

		expect(clientMock.POST).toHaveBeenCalledOnce();
		expect(clientMock.POST).toBeCalledWith(
			'/gates/{group}/{service}/{environment}/comments',
			{
				params: {
					path: {
						group: gate.group,
						service: gate.service,
						environment: gate.environment
					}
				},
				body: {
					message: 'some new comment message'
				}
			}
		);
	});

	it('should throw an error if there is an error from the server', async () => {
		// given
		const gate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'open',
			comments: [],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		vi.mocked(clientMock.POST).mockResolvedValue({
			error: 'some error!'
		});

		// when
		const result = addCommentToGate(gate, 'some new comment message');

		// then
		expect(result).rejects.toThrow('some error!');

		expect(clientMock.POST).toHaveBeenCalledOnce();
		expect(clientMock.POST).toBeCalledWith(
			'/gates/{group}/{service}/{environment}/comments',
			{
				params: {
					path: {
						group: gate.group,
						service: gate.service,
						environment: gate.environment
					}
				},
				body: {
					message: 'some new comment message'
				}
			}
		);
	});

	it('should throw an error if updated gate is undefined', async () => {
		// given
		const gate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'open',
			comments: [],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		vi.mocked(clientMock.POST).mockResolvedValue({});

		// when
		const result = addCommentToGate(gate, 'some new comment message');

		// then
		expect(result).rejects.toThrow('could not retrieve updated gate');

		expect(clientMock.POST).toHaveBeenCalledOnce();
		expect(clientMock.POST).toBeCalledWith(
			'/gates/{group}/{service}/{environment}/comments',
			{
				params: {
					path: {
						group: gate.group,
						service: gate.service,
						environment: gate.environment
					}
				},
				body: {
					message: 'some new comment message'
				}
			}
		);
	});

	it('should throw an error if fetch fails', async () => {
		// given
		const gate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'open',
			comments: [],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		vi.mocked(clientMock.POST).mockRejectedValue('some error!');

		// when
		const result = addCommentToGate(gate, 'some new comment message');

		// then
		expect(result).rejects.toThrow('some error!');

		expect(clientMock.POST).toHaveBeenCalledOnce();
		expect(clientMock.POST).toBeCalledWith(
			'/gates/{group}/{service}/{environment}/comments',
			{
				params: {
					path: {
						group: gate.group,
						service: gate.service,
						environment: gate.environment
					}
				},
				body: {
					message: 'some new comment message'
				}
			}
		);
	});
});


describe('should remove comment from gate', () => {
	it('should return updated gate', async () => {
		// given
		const gate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'open',
			comments: [
				{
					id: 'comment-id-1',
					message: 'Some comment message.',
					created: '2024-03-15T18:24:14.265799400Z'
				}
			],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		const updatedGate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'open',
			comments: [],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		vi.mocked(clientMock.DELETE).mockResolvedValue({
			data: updatedGate
		});

		// when
		const result = await removeCommentFromGate(gate, 'comment-id-1');

		// then
		expect(result).toEqual(updatedGate);

		expect(clientMock.DELETE).toHaveBeenCalledOnce();
		expect(clientMock.DELETE).toBeCalledWith(
			'/gates/{group}/{service}/{environment}/comments/{comment_id}',
			{
				params: {
					path: {
						group: gate.group,
						service: gate.service,
						environment: gate.environment,
						comment_id: 'comment-id-1'
					}
				}
			}
		);
	});

	it('should throw an error if there is an error from the server', async () => {
		// given
		const gate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'open',
			comments: [
				{
					id: 'comment-id-1',
					message: 'Some comment message.',
					created: '2024-03-15T18:24:14.265799400Z'
				}
			],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		vi.mocked(clientMock.DELETE).mockResolvedValue({
			error: 'some error!'
		});

		// when
		const result = removeCommentFromGate(gate, 'comment-id-1');

		// then
		expect(result).rejects.toThrow('some error!');

		expect(clientMock.DELETE).toHaveBeenCalledOnce();
		expect(clientMock.DELETE).toBeCalledWith(
			'/gates/{group}/{service}/{environment}/comments/{comment_id}',
			{
				params: {
					path: {
						group: gate.group,
						service: gate.service,
						environment: gate.environment,
						comment_id: 'comment-id-1'
					}
				}
			}
		);
	});

	it('should throw an error if updated gate is undefined', async () => {
		// given
		const gate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'open',
			comments: [
				{
					id: 'comment-id-1',
					message: 'Some comment message.',
					created: '2024-03-15T18:24:14.265799400Z'
				}
			],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		vi.mocked(clientMock.DELETE).mockResolvedValue({});

		// when
		const result = removeCommentFromGate(gate, 'comment-id-1');

		// then
		expect(result).rejects.toThrow('could not retrieve updated gate');

		expect(clientMock.DELETE).toHaveBeenCalledOnce();
		expect(clientMock.DELETE).toBeCalledWith(
			'/gates/{group}/{service}/{environment}/comments/{comment_id}',
			{
				params: {
					path: {
						group: gate.group,
						service: gate.service,
						environment: gate.environment,
						comment_id: 'comment-id-1'
					}
				}
			}
		);
	});

	it('should throw an error if fetch fails', async () => {
		// given
		const gate: Gate = {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment',
			state: 'open',
			comments: [
				{
					id: 'comment-id-1',
					message: 'Some comment message.',
					created: '2024-03-15T18:24:14.265799400Z'
				}
			],
			last_updated: '2024-03-13T18:24:14.265799400Z'
		};

		vi.mocked(clientMock.DELETE).mockRejectedValue('some error!');

		// when
		const result = removeCommentFromGate(gate, 'comment-id-1');

		// then
		expect(result).rejects.toThrow('some error!');

		expect(clientMock.DELETE).toHaveBeenCalledOnce();
		expect(clientMock.DELETE).toBeCalledWith(
			'/gates/{group}/{service}/{environment}/comments/{comment_id}',
			{
				params: {
					path: {
						group: gate.group,
						service: gate.service,
						environment: gate.environment,
						comment_id: 'comment-id-1'
					}
				}
			}
		);
	});
});

