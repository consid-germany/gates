import { GetSecretValueCommand, SecretsManagerClient } from "@aws-sdk/client-secrets-manager";
import { APIGatewayRequestSimpleAuthorizerHandlerV2 } from "aws-lambda/trigger/api-gateway-authorizer";

const SECRET_VERSION_AWS_PENDING = "AWSPENDING";
const SECRET_VERSION_AWS_CURRENT = "AWSCURRENT";
const SECRET_ID = getEnvVariable("SECRET_ID");
const X_VERIFY_ORIGIN_HEADER_NAME = getEnvVariable("X_VERIFY_ORIGIN_HEADER_NAME");
const SECRETS_MANAGER_CLIENT = new SecretsManagerClient();

export const handler: APIGatewayRequestSimpleAuthorizerHandlerV2 = async (event) => {
    if (
        event.headers === undefined ||
        !Object.prototype.hasOwnProperty.call(event.headers, X_VERIFY_ORIGIN_HEADER_NAME) ||
        event.headers[X_VERIFY_ORIGIN_HEADER_NAME] === undefined
    ) {
        return {
            isAuthorized: false,
        };
    }

    const pendingSecret = await tryGetPendingSecretValue();
    if (pendingSecret !== undefined) {
        if (event.headers[X_VERIFY_ORIGIN_HEADER_NAME] == pendingSecret) {
            return {
                isAuthorized: true,
            };
        }
    }

    const currentSecret = await tryGetCurrentSecretValue();
    if (currentSecret !== undefined) {
        if (event.headers[X_VERIFY_ORIGIN_HEADER_NAME] == currentSecret) {
            return {
                isAuthorized: true,
            };
        }
    }

    return {
        isAuthorized: false,
    };
};

async function tryGetPendingSecretValue() {
    try {
        const pendingSecretResponse = await SECRETS_MANAGER_CLIENT.send(
            new GetSecretValueCommand({
                SecretId: SECRET_ID,
                VersionStage: SECRET_VERSION_AWS_PENDING,
            }),
        );
        return pendingSecretResponse.SecretString;
    } catch (e) {
        console.error(e);
    }
    return undefined;
}

async function tryGetCurrentSecretValue() {
    try {
        const currentSecretResponse = await SECRETS_MANAGER_CLIENT.send(
            new GetSecretValueCommand({
                SecretId: SECRET_ID,
                VersionStage: SECRET_VERSION_AWS_CURRENT,
            }),
        );
        return currentSecretResponse.SecretString;
    } catch (e) {
        console.error(e);
    }
    return undefined;
}

function getEnvVariable(envVarName: string): string {
    const envVar: string | undefined = process.env[envVarName];

    if (!envVar) {
        throw new Error(`${envVarName} environment variable is not set`);
    }

    return envVar;
}
