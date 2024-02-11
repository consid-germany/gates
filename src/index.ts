import { Construct } from "constructs";
import {
    CfnResource,
    CustomResource,
    CustomResourceProvider,
    CustomResourceProviderRuntime,
    Lazy,
    Stack,
    Stage,
    Token,
} from "aws-cdk-lib";
import * as waf from "aws-cdk-lib/aws-wafv2";
import * as ssm from "aws-cdk-lib/aws-ssm";
import * as path from "path";
import { IConstruct } from "constructs/lib/construct";

export interface GatesProps {}

const GLOBAL_REGION = "us-east-1";
const CROSS_REGION_ARN_READER = "Custom::CrossRegionArnReader";

export class Gates extends Construct {
    private readonly stack: Stack;

    constructor(scope: Construct, id: string, _props: GatesProps) {
        super(scope, id);

        this.stack = Stack.of(this);

        if (Token.isUnresolved(this.stack.region)) {
            throw new Error(
                "stacks which use this construct must have an explicitly set region",
            );
        }

        const globalStack = this.createGlobalStack();

        const ipSet = new waf.CfnIPSet(globalStack, "ip-set", {
            name: "consid-test",
            addresses: ["93.230.173.30/32"],
            ipAddressVersion: "IPV4",
            scope: "CLOUDFRONT",
        });

        const parameterNamePrefix = "cdk/WafIpSetArn";
        const sanitizedPath = this.node.path.replace(/[^\/\w.-]/g, "_");
        const parameterName = `/${parameterNamePrefix}/${this.stack.region}/${sanitizedPath}`;

        new ssm.StringParameter(ipSet, "WafIpSetArnParameter", {
            parameterName,
            stringValue: ipSet.attrArn,
        });

        const arnRef = this.createCrossRegionArnReader(
            parameterNamePrefix,
            parameterName,
            ipSet,
        );

        new ssm.StringParameter(this, "InRegionTest", {
            parameterName: "/consid/integ-test-exported",
            stringValue: arnRef,
        });

        // const ipAllowList: waf.CfnWebACL.RuleProperty = {
        //     visibilityConfig: {
        //         cloudWatchMetricsEnabled: false,
        //         metricName: "verify-ip-allowlist-rule",
        //         sampledRequestsEnabled: false,
        //     },
        //     name: "verify-ip-allowlist-rule",
        //     priority: 0,
        //     action: { allow: {} },
        //     statement: {
        //         ipSetReferenceStatement: {
        //             arn: arnRef,
        //         },
        //     },
        // };
        //
        // new waf.CfnWebACL(globalStack, "waf", {
        //     defaultAction: {
        //         block: {},
        //     },
        //     scope: "CLOUDFRONT",
        //     visibilityConfig: {
        //         cloudWatchMetricsEnabled: false,
        //         metricName: "webACL",
        //         sampledRequestsEnabled: false,
        //     },
        //     rules: [ipAllowList],
        // });
    }

    private createGlobalStack() {
        const stage = Stage.of(this);
        if (!stage) {
            throw new Error(
                "stacks which use this construct must be part of a CDK app or stage",
            );
        }

        const globalStackId = `${this.stack.stackName}-global`;
        let globalStack = stage.node.tryFindChild(globalStackId) as Stack;
        if (!globalStack) {
            globalStack = new Stack(stage, globalStackId, {
                env: {
                    region: GLOBAL_REGION,
                    account: this.stack.account,
                },
            });
        }
        this.stack.addDependency(globalStack);
        return globalStack;
    }

    private createCrossRegionArnReader(
        parameterNamePrefix: string,
        parameterName: string,
        construct: IConstruct,
    ) {
        const parameterArnPrefix = this.stack.formatArn({
            service: "ssm",
            region: GLOBAL_REGION,
            resource: "parameter",
            resourceName: parameterNamePrefix + "/*",
        });

        const serviceToken = CustomResourceProvider.getOrCreate(
            this,
            CROSS_REGION_ARN_READER,
            {
                codeDirectory: path.join(
                    __dirname,
                    "function",
                    "cross-region-arn-reader",
                    "build",
                ),
                runtime: CustomResourceProviderRuntime.NODEJS_18_X,
                policyStatements: [
                    {
                        Effect: "Allow",
                        Resource: parameterArnPrefix,
                        Action: ["ssm:GetParameter"],
                    },
                ],
            },
        );

        const resource = new CustomResource(this, "arn-reader", {
            resourceType: CROSS_REGION_ARN_READER,
            serviceToken,
            properties: {
                Region: GLOBAL_REGION,
                ParameterName: parameterName,
                RefreshToken: Lazy.uncachedString({
                    produce: () => {
                        if (construct instanceof CfnResource) {
                            return this.stack.resolve(construct.logicalId);
                        }
                        const cfn = construct.node.defaultChild as CfnResource;
                        return this.stack.resolve(cfn.logicalId);
                    },
                }),
            },
        });

        return resource.getAttString("Arn");
    }
}
