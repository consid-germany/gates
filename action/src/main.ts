import * as core from "@actions/core";
import * as http from "@actions/http-client";
import * as httpAuth from '@actions/http-client/lib/auth'

export async function run(): Promise<void> {
    try {
        const group = core.getInput("group");
        core.info(`Group: ${group}`);

        const idToken = await core.getIDToken("consid-germany/gates");
        core.info(`idToken: ${idToken}`);


        const auth = new httpAuth.BearerCredentialHandler(idToken);
        const client = new http.HttpClient("consid-germany/gates", [auth]);

        const response = await client.get("https://github.gates.consid.tech/api/");

        console.log((await response.readBody()));


        core.setFailed("some error");
    } catch (error) {
        core.setFailed(`${error}`);
    }
}
