import * as core from "@actions/core";

const USER_AGENT = "consid-germany/gates";

const AUDIENCE = "consid-germany/gates";

interface GateState {
    state: "open" | "closed";
}

export async function run(): Promise<void> {
    try {
        const gitHubApiBaseUrl = core.getInput("gitHubApiBaseUrl");
        const group = core.getInput("group");
        const service = core.getInput("service");
        const environment = core.getInput("environment");

        const gateStateResponse = await fetch(`${gitHubApiBaseUrl}/gates/${group}/${service}/${environment}/state`, {
            method: "GET",
            headers: {
                "Accept" : "application/json",
                "User-Agent" : USER_AGENT,
                "Authorization": `Bearer ${await core.getIDToken(AUDIENCE)}`,
            }
        })

        switch (gateStateResponse.status) {
            case 200:
                if (isClosed(await gateStateResponse.json())) {
                    core.setFailed(`Gate ${group}/${service}/${environment} is closed.`);
                }
                break;
            case 204:
                core.setFailed(`Gate ${group}/${service}/${environment} could not be found.`);
                break;
            default:
                core.setFailed(`Request to check gate state of ${group}/${service}/${environment} failed`);
                break;
        }
    } catch (error) {
        core.setFailed(`${error}`);
    }
}

function isClosed(gateState: GateState) {
    return gateState.state !== "open";

}
