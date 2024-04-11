import createClient from 'openapi-fetch';
import type { components, paths } from './generated/api';
import { PUBLIC_API_BASE_URL } from '$env/static/public';

const client = createClient<paths>({
	baseUrl: PUBLIC_API_BASE_URL
});

export type Group = components['schemas']['Group'];
export type Service = components['schemas']['Service'];
export type Gate = components['schemas']['Gate'];
export type GateState = components['schemas']['GateState'];

export async function getGroups(): Promise<Group[]> {
	const { data: groups, error } = await client.GET('/gates', {});
	if (error) {
		throw new Error(error);
	}
	if (groups === undefined) {
		throw new Error('could not retrieve gates');
	}
	return groups || [];
}

export async function toggleGateState(gate: Gate): Promise<Gate> {
	const { data: updatedGate, error } = await client.PUT(
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
				state: getToggledGateState(gate.state)
			}
		}
	);

	if (error) {
		throw new Error(error);
	}

	if (updatedGate === undefined) {
		throw new Error('could not retrieve updated gate');
	}

	return updatedGate;
}

export async function addCommentToGate(gate: Gate, message: string): Promise<Gate> {
	const { data: updatedGate, error } = await client.POST(
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
				message
			}
		}
	);

	if (error) {
		throw new Error(error);
	}

	if (updatedGate === undefined) {
		throw new Error('could not retrieve updated gate');
	}

	return updatedGate;
}

export const removeCommentFromGate = async (gate: Gate, commentId: string): Promise<Gate> => {
	const { data: updatedGate, error } = await client.DELETE(
		'/gates/{group}/{service}/{environment}/comments/{comment_id}',
		{
			params: {
				path: {
					group: gate.group,
					service: gate.service,
					environment: gate.environment,
					comment_id: commentId
				}
			}
		}
	);

	if (error) {
		throw new Error(error);
	}

	if (updatedGate === undefined) {
		throw new Error('could not retrieve updated gate');
	}

	return updatedGate;
};

function getToggledGateState(gateState: GateState): GateState {
	switch (gateState) {
		case 'open':
			return 'closed';
		case 'closed':
			return 'open';
		default:
			throw new Error(`unknown gate state: ${gateState}`);
	}
}
