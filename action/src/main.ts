import * as core from "@actions/core";
import * as http from "@actions/http-client";
import * as httpAuth from '@actions/http-client/lib/auth'

export async function run(): Promise<void> {
    try {
        const gitHubApiBaseUrl = core.getInput("gitHubApiBaseUrl");
        const group = core.getInput("group");
        const service = core.getInput("service");
        const environment = core.getInput("environment");

        const idToken = await core.getIDToken("consid-germany/gates");

        const auth = new httpAuth.BearerCredentialHandler(idToken);
        const client = new http.HttpClient("consid-germany/gates", [auth]);

        const response = await client.get(`https://${gitHubApiBaseUrl}/gates/${group}/${service}/${environment}/state`);

        console.log(response.message.statusCode);
        console.log((await response.readBody()));


        core.setFailed("some error");
    } catch (error) {
        core.setFailed(`${error}`);
    }
}
