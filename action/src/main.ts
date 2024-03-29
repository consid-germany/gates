import core from "@actions/core";

export async function run(): Promise<void> {
    // eslint-disable-next-line no-useless-catch
    try {
        const group = core.getInput("group");
        core.info(`Group: ${group}`);

        const idToken = await core.getIDToken();
        core.info(`idToken: ${idToken}`);

        core.setFailed("some error");
    } catch (error) {
        throw error;
    }
}
