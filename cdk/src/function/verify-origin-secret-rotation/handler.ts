import { SecretsManagerRotationHandler } from "aws-lambda/trigger/secretsmanager";
import {
    DescribeSecretCommand,
    GetRandomPasswordCommand,
    GetSecretValueCommand,
    InvalidRequestException,
    PutSecretValueCommand,
    ResourceNotFoundException,
    SecretsManagerClient,
    UpdateSecretVersionStageCommand,
} from "@aws-sdk/client-secrets-manager";
import {
    CloudFrontClient,
    GetDistributionCommand,
    GetDistributionConfigCommand,
    UpdateDistributionCommand,
} from "@aws-sdk/client-cloudfront";

const SECRET_VERSION_AWS_PENDING = "AWSPENDING";
const SECRET_VERSION_AWS_CURRENT = "AWSCURRENT";
const CLOUDFRONT_DISTRIBUTION_STATUS_DEPLOYED = "Deployed";
const CLOUDFRONT_DISTRIBUTION_ID = getEnvVariable("CLOUDFRONT_DISTRIBUTION_ID");
const X_VERIFY_ORIGIN_HEADER_NAME = getEnvVariable("X_VERIFY_ORIGIN_HEADER_NAME");
const ORIGIN_TEST_URL = getEnvVariable("ORIGIN_TEST_URL");
const SECRETS_MANAGER_CLIENT = new SecretsManagerClient();

const CLOUDFRONT_CLIENT = new CloudFrontClient();

export const handler: SecretsManagerRotationHandler = async (event) => {
    switch (event.Step) {
        case "createSecret":
            await createSecret(event.SecretId, event.ClientRequestToken);
            break;
        case "setSecret":
            await setSecret(event.SecretId, event.ClientRequestToken);
            break;
        case "testSecret":
            await testSecret(event.SecretId, event.ClientRequestToken);
            break;
        case "finishSecret":
            await finishSecret(event.SecretId, event.ClientRequestToken);
            break;
    }
};

async function createSecret(secretId: string, clientRequestToken: string) {
    await SECRETS_MANAGER_CLIENT.send(
        new GetSecretValueCommand({
            SecretId: secretId,
        }),
    );
    try {
        await SECRETS_MANAGER_CLIENT.send(
            new GetSecretValueCommand({
                SecretId: secretId,
                VersionId: clientRequestToken,
                VersionStage: SECRET_VERSION_AWS_PENDING,
            }),
        );
    } catch (e) {
        if (e instanceof ResourceNotFoundException || e instanceof InvalidRequestException) {
            const secretValueResponse = await SECRETS_MANAGER_CLIENT.send(
                new GetRandomPasswordCommand({
                    ExcludePunctuation: true,
                }),
            );

            await SECRETS_MANAGER_CLIENT.send(
                new PutSecretValueCommand({
                    SecretId: secretId,
                    ClientRequestToken: clientRequestToken,
                    SecretString: secretValueResponse.RandomPassword,
                    VersionStages: [SECRET_VERSION_AWS_PENDING],
                }),
            );
        }
    }
}

async function setSecret(secretId: string, clientRequestToken: string) {
    const cloudFrontDistributionResponse = await CLOUDFRONT_CLIENT.send(
        new GetDistributionCommand({
            Id: CLOUDFRONT_DISTRIBUTION_ID,
        }),
    );

    if (
        cloudFrontDistributionResponse.Distribution?.Status !=
        CLOUDFRONT_DISTRIBUTION_STATUS_DEPLOYED
    ) {
        throw new Error(
            `cloudfront distribution is not in state ${CLOUDFRONT_DISTRIBUTION_STATUS_DEPLOYED}`,
        );
    }

    const pendingSecretResponse = await SECRETS_MANAGER_CLIENT.send(
        new GetSecretValueCommand({
            SecretId: secretId,
            VersionId: clientRequestToken,
            VersionStage: SECRET_VERSION_AWS_PENDING,
        }),
    );
    const cloudFrontDistributionConfigResponse = await CLOUDFRONT_CLIENT.send(
        new GetDistributionConfigCommand({
            Id: CLOUDFRONT_DISTRIBUTION_ID,
        }),
    );

    const distributionConfig = cloudFrontDistributionConfigResponse.DistributionConfig;

    distributionConfig?.Origins?.Items?.forEach((origin) => {
        origin.CustomHeaders?.Items?.filter(
            (header) => header.HeaderName === X_VERIFY_ORIGIN_HEADER_NAME,
        ).forEach((header) => {
            header.HeaderValue = pendingSecretResponse.SecretString;
        });
    });
    await CLOUDFRONT_CLIENT.send(
        new UpdateDistributionCommand({
            Id: CLOUDFRONT_DISTRIBUTION_ID,
            IfMatch: cloudFrontDistributionConfigResponse.ETag,
            DistributionConfig: distributionConfig,
        }),
    );
}

async function testSecret(secretId: string, clientRequestToken: string) {
    const pendingSecretResponse = await SECRETS_MANAGER_CLIENT.send(
        new GetSecretValueCommand({
            SecretId: secretId,
            VersionId: clientRequestToken,
            VersionStage: SECRET_VERSION_AWS_PENDING,
        }),
    );

    if (pendingSecretResponse.SecretString === undefined) {
        throw new Error("could not find pending secret value");
    }

    const response = await fetch(ORIGIN_TEST_URL, {
        headers: {
            [X_VERIFY_ORIGIN_HEADER_NAME]: pendingSecretResponse.SecretString,
        },
    });

    if (!response.ok) {
        throw new Error("failed to access origin test url");
    }
}

async function finishSecret(secretId: string, clientRequestToken: string) {
    const describeSecretResponse = await SECRETS_MANAGER_CLIENT.send(
        new DescribeSecretCommand({
            SecretId: secretId,
        }),
    );

    if (describeSecretResponse.VersionIdsToStages === undefined) {
        throw new Error("could not find versions of secret");
    }

    const currentVersionToStages = Object.entries(describeSecretResponse.VersionIdsToStages).find(
        ([_, stages]) => stages.includes(SECRET_VERSION_AWS_CURRENT),
    );

    if (currentVersionToStages === undefined) {
        throw new Error("could not find current version of secret");
    }

    const currentVersion = currentVersionToStages[0];

    if (currentVersion == clientRequestToken) {
        return;
    }

    await SECRETS_MANAGER_CLIENT.send(
        new UpdateSecretVersionStageCommand({
            SecretId: secretId,
            VersionStage: SECRET_VERSION_AWS_CURRENT,
            MoveToVersionId: clientRequestToken,
            RemoveFromVersionId: currentVersionToStages[0],
        }),
    );
}

function getEnvVariable(envVarName: string): string {
    const envVar: string | undefined = process.env[envVarName];

    if (!envVar) {
        throw new Error(`${envVarName} environment variable is not set`);
    }

    return envVar;
}
