import * as cdk from "aws-cdk-lib";
import { Gates } from "../lib/index";
import { IntegTest } from "@aws-cdk/integ-tests-alpha";

const app = new cdk.App();

const stackUnderTest = new cdk.Stack(app, "consid-gates-integ-test", {
    //stackName
    env: {
        region: "eu-central-1",
    },
    permissionsBoundary: cdk.PermissionsBoundary.fromName(
        "consid-aws-cdk-permission-boundary",
    ),
});

new Gates(stackUnderTest, "gates", {});

new IntegTest(app, "test", {
    testCases: [stackUnderTest],
    regions: [stackUnderTest.region],
    cdkCommandOptions: {
        deploy: {
            args: {
                stacks: ["*"],
                context: {
                    "@aws-cdk/core:bootstrapQualifier": "consid",
                },
            },
        },
    },
});
