import * as core from "@actions/core";
import * as http from "@actions/http-client";
import * as httpAuth from '@actions/http-client/lib/auth'

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

        const idToken = await core.getIDToken(AUDIENCE);
        const auth = new httpAuth.BearerCredentialHandler(idToken);
        const client = new http.HttpClient(USER_AGENT, [auth]);

        const gateStateResponse = await client.get(`${gitHubApiBaseUrl}/gates/${group}/${service}/${environment}/state`);

        switch (gateStateResponse.message.statusCode) {
            case 200:
                checkGate(await gateStateResponse.readBody());
                break;
            case 204:
                core.setFailed("Gate could not be found.");
                break;
            default:
                core.setFailed("Request to check gate state failed");
                break;
        }
    } catch (error) {
        core.setFailed(`${error}`);
    }
}

function checkGate(gateStateJson: string) {
    const gateState: GateState = JSON.parse(gateStateJson);
    core.summary.addDetails("Gate State", gateState.state);
    if (gateState.state !== "open") {
        core.setFailed("Gate is closed.");
    }
}
