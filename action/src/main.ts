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

        core.info(String(gateStateResponse.status));
        core.info(await gateStateResponse.json());

        // switch (gateStateResponse.message.statusCode) {
        //     case 200:
        //         await checkGate(await gateStateResponse.readBody());
        //         break;
        //     case 204:
        //         //core.setFailed("Gate could not be found.");
        //         break;
        //     default:
        //         //core.setFailed("Request to check gate state failed");
        //         break;
        // }

        return;

    } catch (error) {
        core.setFailed(`${error}`);
    }
}

async function checkGate(gateStateJson: string) {
    const gateState: GateState = JSON.parse(gateStateJson);
    await core.summary.addDetails("Gate State", gateState.state).write();
    if (gateState.state !== "open") {
        core.setFailed("Gate is closed.");
    }
}
