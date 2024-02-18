import { Construct } from "constructs";
import { Stack } from "aws-cdk-lib";
import * as wafv2 from "aws-cdk-lib/aws-wafv2";
import * as route53 from "aws-cdk-lib/aws-route53";
import * as acm from "aws-cdk-lib/aws-certificatemanager";
import GlobalStackProvider from "./global-stack";
import CrossRegionStringRef from "./cross-region-string-ref";

const SCOPE_CLOUDFRONT = "CLOUDFRONT";

export interface GatesProps {
    /**
     * A namespace for resources of the deployed application.
     * If not specified, the default namespace `default` is used.
     */
    namespace?: string;

    /**
     * A name for the application.
     * If not specified, the default app name `gates` is used.
     */
    appName?: string;

    globalStackName?: string;
}

const DEFAULT_NAMESPACE = "default";
const DEFAULT_APP_NAME = "gates";

export class Gates extends Construct {
    private readonly stack: Stack;

    private readonly globalStack: Stack;

    constructor(scope: Construct, id: string, props: GatesProps) {
        super(scope, id);

        this.stack = Stack.of(this);
        this.globalStack = GlobalStackProvider.getOrCreate(this, {
            stackName: props.globalStackName ?? `${this.stack.stackName}-global`,
        });

        this.createCertificate();
        this.createWebAcl(props);
    }

    private createCertificate() {
        // TODO
        const domainName = `consid.tech`;

        const hostedZone = route53.HostedZone.fromLookup(this.globalStack, "HostedZone", {
            domainName: domainName,
        });
        const certificate = new acm.Certificate(this.globalStack, "Certificate", {
            domainName: `gates.${domainName}`,
            validation: acm.CertificateValidation.fromDns(hostedZone),
        });

        return new CrossRegionStringRef(this, "CertificateArn", {
            constructInOtherRegion: certificate,
            value: (certificate) => certificate.certificateArn,
        }).value;
    }

    private createWebAcl(props: GatesProps) {
        // TODO
        const { namespace = DEFAULT_NAMESPACE, appName = DEFAULT_APP_NAME } = props;

        const ipSet = new wafv2.CfnIPSet(this.globalStack, "IpSet", {
            name: "consid-test",
            addresses: ["93.230.172.22/32"],
            ipAddressVersion: "IPV4",
            scope: "CLOUDFRONT",
        });

        const ipAllowList: wafv2.CfnWebACL.RuleProperty = {
            visibilityConfig: {
                cloudWatchMetricsEnabled: true,
                metricName: `${namespace}-${appName}-waf-ip-allow-list`,
                sampledRequestsEnabled: true,
            },
            name: `${namespace}-${appName}-waf-ip-allow-list-rule`,
            priority: 0,
            action: { allow: {} },
            statement: {
                ipSetReferenceStatement: {
                    arn: ipSet.attrArn,
                },
            },
        };

        const webAcl = new wafv2.CfnWebACL(this.globalStack, "WebAcl", {
            name: `${namespace}-${appName}-waf`,
            defaultAction: { block: {} },
            visibilityConfig: {
                cloudWatchMetricsEnabled: true,
                metricName: `${namespace}-${appName}-waf`,
                sampledRequestsEnabled: true,
            },
            scope: SCOPE_CLOUDFRONT,
            rules: [ipAllowList],
        });

        return new CrossRegionStringRef(this, "WebAclArn", {
            constructInOtherRegion: webAcl,
            value: (webAcl) => webAcl.attrArn,
        }).value;
    }
}
