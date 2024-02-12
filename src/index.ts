import { Construct, IConstruct } from "constructs";
import {
    CfnResource,
    CustomResource,
    Lazy,
    Stack,
    Stage,
    Token,
} from "aws-cdk-lib";
import * as waf from "aws-cdk-lib/aws-wafv2";
import * as ssm from "aws-cdk-lib/aws-ssm";
import { CrossRegionArnReaderProvider } from "./cross-region-arn-reader-provider";

export interface GatesProps {
    /**
     * A namespace for resources of the deployed gates application.
     * If not specified, the default namespace `default` is used.
     */
    namespace?: string;

    /**
     * A name for the app of the deployed gates application.
     * If not specified, the default app name `gates` is used.
     */
    appName?: string;
}

const GLOBAL_REGION = "us-east-1";
const CROSS_REGION_ARN_READER = "Custom::CrossRegionArnReader";
const DEFAULT_NAMESPACE = "default";
const DEFAULT_APP_NAME = "gates";

export class Gates extends Construct {
    private readonly stack: Stack;

    private readonly namespace: string;
    private readonly appName: string;
    private readonly crossRegionParametersPrefix: string;

    constructor(scope: Construct, id: string, props: GatesProps) {
        super(scope, id);

        const { namespace = DEFAULT_NAMESPACE, appName = DEFAULT_APP_NAME } =
            props;
        this.namespace = namespace;
        this.appName = appName;
        this.crossRegionParametersPrefix = `/${this.namespace}/${this.appName}/cdk`;

        this.stack = Stack.of(this);

        if (Token.isUnresolved(this.stack.region)) {
            throw new Error(
                "stacks which use this construct must have an explicitly set region",
            );
        }

        const globalStack = this.createGlobalStack();

        const ipSet = new waf.CfnIPSet(globalStack, "ip-set", {
            name: "consid-test",
            addresses: ["93.230.172.22/32"],
            ipAddressVersion: "IPV4",
            scope: "CLOUDFRONT",
        });

        // TODO is this unique enough?
        const parameterName = `/${this.crossRegionParametersPrefix}/${this.stack.region}/${this.node.path.replace(/[^\/\w.-]/g, "_")}`;

        new ssm.StringParameter(ipSet, "WafIpSetArnParameter", {
            parameterName,
            stringValue: ipSet.attrArn,
        });

        const arnRef = this.createCrossRegionArnReader(parameterName, ipSet);

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
        parameterName: string,
        construct: IConstruct,
    ) {
        const resource = new CustomResource(this, "arn-reader", {
            resourceType: CROSS_REGION_ARN_READER,
            serviceToken: this.getCrossRegionArnReaderServiceToken(),
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

    private getCrossRegionArnReaderServiceToken() {
        const parameterArnPrefix = this.stack.formatArn({
            service: "ssm",
            region: GLOBAL_REGION,
            resource: "parameter",
            resourceName: this.crossRegionParametersPrefix + "/*",
        });

        return CrossRegionArnReaderProvider.getOrCreate(
            this,
            CROSS_REGION_ARN_READER,
            {
                policyStatements: [
                    {
                        Effect: "Allow",
                        Resource: parameterArnPrefix,
                        Action: ["ssm:GetParameter"],
                    },
                ],
            },
        );
    }
}
