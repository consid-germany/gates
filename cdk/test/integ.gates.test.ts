import * as cdk from "aws-cdk-lib";
import {IntegTest} from "@aws-cdk/integ-tests-alpha";
import {ApplyDestroyPolicyAspect} from "./destroy-aspect";
import {createTestBuilds} from "./assets";
import {execSync} from "child_process";
import {Gates} from "../src";
import {getEnvVariable} from "./env-variable";

Date.now = function() {
    return 0;
};

const AWS_CDK_BOOTSTRAP_QUALIFIER = getEnvVariable("AWS_CDK_BOOTSTRAP_QUALIFIER");
const AWS_ACCOUNT_ID = getEnvVariable("AWS_ACCOUNT_ID");
const AWS_REGION = getEnvVariable("AWS_REGION");
const AWS_CDK_PERMISSION_BOUNDARY = getEnvVariable("AWS_CDK_PERMISSION_BOUNDARY");

execSync("npm run build:function");
createTestBuilds();

const app = new cdk.App({
    context: {
        "@aws-cdk/core:bootstrapQualifier": AWS_CDK_BOOTSTRAP_QUALIFIER,
        "@aws-cdk/core:permissionsBoundary": {
            name: AWS_CDK_PERMISSION_BOUNDARY,
        },
    },
});

const stackUnderTest = new cdk.Stack(app, "StackUnderTest", {
    stackName: "consid-gates-integ-test",
    env: {
        region: AWS_REGION,
        account: AWS_ACCOUNT_ID,
    },
    tags: {
        owner: AWS_CDK_BOOTSTRAP_QUALIFIER,
    },
});

new Gates(stackUnderTest, "Gates", {
    appName: "consid-gates-integ",
    ipAllowList: [],
    gitHubApi: {
        allowedSubPatterns: []
    }
});

cdk.Aspects.of(stackUnderTest).add(new ApplyDestroyPolicyAspect());

new IntegTest(app, "IntegTest", {
    testCases: [stackUnderTest],
    regions: [stackUnderTest.region],
    cdkCommandOptions: {
        deploy: {
            args: {
                stacks: ["*"],
            },
        },
        destroy: {
            args: {
                force: true,
            },
        },
    },
});
