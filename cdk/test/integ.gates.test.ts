import * as cdk from "aws-cdk-lib";
import { IntegTest } from "@aws-cdk/integ-tests-alpha";
import { Gates } from "../src";

const app = new cdk.App({
    context: {
        "@aws-cdk/core:bootstrapQualifier": "consid",
        "@aws-cdk/core:permissionsBoundary": {
            name: "consid-aws-cdk-permission-boundary",
        },
    },
});

const stackUnderTest = new cdk.Stack(app, "StackUnderTest", {
    stackName: "consid-gates-integ-test",
    env: {
        region: "eu-central-1",
        account: "669698671509",
    },
    tags: {
        owner: "consid",
    },
});

new Gates(stackUnderTest, "Gates", {
    appName: "consid-gates",
});

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