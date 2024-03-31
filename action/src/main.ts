import * as core from "@actions/core";

export async function run(): Promise<void> {
    try {
        const group = core.getInput("group");
        core.info(`Group: ${group}`);

        const idToken = await core.getIDToken();
        core.info(`idToken: ${idToken}`);

        core.setFailed("some error");
    } catch (error) {
        core.setFailed(`${error}`);
    }
}