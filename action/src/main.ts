import * as core from "@actions/core";

const USER_AGENT = "consid-germany/gates";
const AUDIENCE = "consid-germany/gates";

interface GateState {
    state: "open" | "closed";
}

export async function run(): Promise<void> {
    try {
        const gitHubApiBaseUrl = core.getInput("gitHubApiBaseUrl", { required: true });
        const group = core.getInput("group", { required: true });
        const service = core.getInput("service", { required: true });
        const environment = core.getInput("environment", { required: true });
        const create_gate_if_not_exists = core.getBooleanInput("create_gate_if_not_exists", {
            required: true,
        });

        const gateStateResponse = await fetch(
            `${gitHubApiBaseUrl}/gates/${group}/${service}/${environment}/state`,
            {
                method: "GET",
                headers: {
                    Accept: "application/json",
                    "User-Agent": USER_AGENT,
                    Authorization: `Bearer ${await core.getIDToken(AUDIENCE)}`,
                },
            },
        );

        switch (gateStateResponse.status) {
            case 200:
                if (isClosed(await gateStateResponse.json())) {
                    core.setFailed(`Gate ${group}/${service}/${environment} is closed.`);
                } else {
                    core.notice(`Gate ${group}/${service}/${environment} is open.`);
                }
                break;
            case 204:
                core.setFailed(`Gate ${group}/${service}/${environment} could not be found.`);
                if (create_gate_if_not_exists) {
                    await create_gate(gitHubApiBaseUrl, group, service, environment);
                    core.notice(`Created gate ${group}/${service}/${environment}.`);
                }
                break;
            default:
                core.setFailed(
                    `Request to check gate ${group}/${service}/${environment} failed: ${gateStateResponse.status} ${gateStateResponse.statusText}`,
                );
                break;
        }
    } catch (error) {
        core.setFailed(`${error}`);
    }
}

async function create_gate(
    gitHubApiBaseUrl: string,
    group: string,
    service: string,
    environment: string,
) {
    const createGateResponse = await fetch(`${gitHubApiBaseUrl}/gates`, {
        method: "POST",
        body: JSON.stringify({ group, service, environment }),
        headers: {
            Accept: "application/json",
            "User-Agent": USER_AGENT,
            Authorization: `Bearer ${await core.getIDToken(AUDIENCE)}`,
        },
    });
    if (createGateResponse.status !== 200) {
        throw new Error(
            `Request to create gate ${group}/${service}/${environment} failed: ${createGateResponse.status} ${createGateResponse.statusText}`,
        );
    }
}

function isClosed(gateState: GateState) {
    return gateState.state !== "open";
}
