import { Construct } from "constructs";
import { Duration, Stack } from "aws-cdk-lib";
import * as wafv2 from "aws-cdk-lib/aws-wafv2";
import * as route53 from "aws-cdk-lib/aws-route53";
import * as route53_targets from "aws-cdk-lib/aws-route53-targets";
import * as acm from "aws-cdk-lib/aws-certificatemanager";
import * as lambda from "aws-cdk-lib/aws-lambda";
import * as logs from "aws-cdk-lib/aws-logs";
import * as apigatewayv2 from "aws-cdk-lib/aws-apigatewayv2";
import * as apigatewayv2_integrations from "aws-cdk-lib/aws-apigatewayv2-integrations";
import * as dynamodb from "aws-cdk-lib/aws-dynamodb";
import * as secretsmanager from "aws-cdk-lib/aws-secretsmanager";
import * as cloudfront from "aws-cdk-lib/aws-cloudfront";
import * as s3 from "aws-cdk-lib/aws-s3";
import * as s3_deployment from "aws-cdk-lib/aws-s3-deployment";
import GlobalStackProvider from "./global-stack";
import CrossRegionStringRef from "./cross-region-string-ref";
import * as path from "path";
import * as apigatewayv2_authorizers from "aws-cdk-lib/aws-apigatewayv2-authorizers";

const SCOPE_CLOUDFRONT = "CLOUDFRONT";

export interface Domain {
    readonly domainName: string;
    readonly zoneDomainName?: string;
}

export interface GatesProps {
    /**
     * A name for the application.
     * If not specified, the default app name `gates` is used.
     */
    readonly appName?: string;

    readonly domain?: Domain;

    readonly globalStackName?: string;
}

const DEFAULT_APP_NAME = "gates";
const X_VERIFY_ORIGIN_HEADER_NAME = "x-verify-origin";

export class Gates extends Construct {
    private readonly stack: Stack;

    private readonly globalStack: Stack;

    constructor(scope: Construct, id: string, props: GatesProps) {
        super(scope, id);

        const { appName = DEFAULT_APP_NAME } = props;

        this.stack = Stack.of(this);
        this.globalStack = GlobalStackProvider.getOrCreate(this, {
            stackName: props.globalStackName || `${this.stack.stackName}-global`,
            tags: this.stack.tags.tagValues(),
        });

        const certificate = this.createCertificate(props.domain);
        const webAclArn = this.createWebAcl(appName);

        const gatesTable = new dynamodb.TableV2(this, "GatesTable", {
            tableName: `${appName}`,
            partitionKey: { name: "group", type: dynamodb.AttributeType.STRING },
            sortKey: { name: "service_environment", type: dynamodb.AttributeType.STRING },
        });

        const apiFunction = new lambda.Function(this, "ApiFunction", {
            functionName: `${appName}-api`,
            runtime: lambda.Runtime.PROVIDED_AL2023,
            architecture: lambda.Architecture.ARM_64,
            code: lambda.Code.fromAsset(
                path.join(__dirname, "..", "..", "api/target/lambda/gates-api"),
            ),
            handler: "provided",
            environment: {
                GATES_DYNAMO_DB_TABLE_NAME: gatesTable.tableName,
            },
            logRetention: logs.RetentionDays.ONE_WEEK,
        });

        gatesTable.grantReadWriteData(apiFunction);

        const verifyOriginSecret = new secretsmanager.Secret(this, "VerifyOriginSecret", {
            secretName: `${appName}-verify-origin-secret`,
            generateSecretString: {
                excludePunctuation: true,
            },
        });

        const verifyOriginAuthFunction = new lambda.Function(this, "VerifyOriginAuthFunction", {
            functionName: `${appName}-verify-origin-auth`,
            runtime: lambda.Runtime.NODEJS_20_X,
            code: lambda.Code.fromAsset(
                path.join(__dirname, "..", "lib", "function", "verify-origin-authorizer"),
            ),
            handler: "index.handler",
            logRetention: logs.RetentionDays.ONE_WEEK,
            environment: {
                SECRET_ID: verifyOriginSecret.secretName,
                X_VERIFY_ORIGIN_HEADER_NAME,
            },
        });

        verifyOriginSecret.grantRead(verifyOriginAuthFunction);

        const apiFunctionIntegration = new apigatewayv2_integrations.HttpLambdaIntegration(
            "ApiFunctionIntegration",
            apiFunction,
        );

        const httpApi = new apigatewayv2.HttpApi(this, "HttpApi", {
            apiName: `${appName}-api`,
            defaultIntegration: apiFunctionIntegration,
            defaultAuthorizer: new apigatewayv2_authorizers.HttpLambdaAuthorizer(
                "VerifyOriginAuthorizer",
                verifyOriginAuthFunction,
                {
                    responseTypes: [apigatewayv2_authorizers.HttpLambdaResponseType.SIMPLE],
                    identitySource: [`$request.header.${X_VERIFY_ORIGIN_HEADER_NAME}`],
                },
            ),
        });

        const frontendAssetsBucket = new s3.Bucket(this, "FrontendAssetsBucket", {
            // TODO name?
            blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
            objectOwnership: s3.ObjectOwnership.BUCKET_OWNER_ENFORCED,
        });

        const cloudfrontOAI = new cloudfront.OriginAccessIdentity(this, "OriginAccessIdentity");

        frontendAssetsBucket.grantRead(cloudfrontOAI);

        const webDistribution = new cloudfront.CloudFrontWebDistribution(this, "WebDistribution", {
            webACLId: webAclArn,
            enableIpV6: false,
            viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
            viewerCertificate:
                props.domain && certificate
                    ? cloudfront.ViewerCertificate.fromAcmCertificate(certificate, {
                          aliases: [props.domain.domainName],
                      })
                    : undefined,
            originConfigs: [
                {
                    customOriginSource: {
                        domainName: `${httpApi.apiId}.execute-api.${this.stack.region}.amazonaws.com`,
                        originHeaders: {
                            [X_VERIFY_ORIGIN_HEADER_NAME]:
                                verifyOriginSecret.secretValue.unsafeUnwrap(),
                        },
                    },
                    behaviors: [
                        {
                            pathPattern: "/api",
                            allowedMethods: cloudfront.CloudFrontAllowedMethods.ALL,
                            defaultTtl: Duration.seconds(0),
                        },
                        {
                            pathPattern: "/api/*",
                            allowedMethods: cloudfront.CloudFrontAllowedMethods.ALL,
                            defaultTtl: Duration.seconds(0),
                        },
                    ],
                },
                {
                    s3OriginSource: {
                        s3BucketSource: frontendAssetsBucket,
                        originAccessIdentity: cloudfrontOAI,
                    },
                    behaviors: [
                        {
                            isDefaultBehavior: true,
                            allowedMethods: cloudfront.CloudFrontAllowedMethods.GET_HEAD_OPTIONS,
                        },
                    ],
                },
            ],
        });

        if (props.domain) {
            const hostedZone = route53.HostedZone.fromLookup(this, "HostedZone", {
                domainName: props.domain.domainName,
            });

            new route53.ARecord(this, "ARecord", {
                recordName: props.domain.zoneDomainName || props.domain.domainName,
                target: route53.RecordTarget.fromAlias(
                    new route53_targets.CloudFrontTarget(webDistribution),
                ),
                zone: hostedZone,
            });
        }

        const verifyOriginSecretRotationFunction = new lambda.Function(
            this,
            "VerifyOriginSecretRotationFunction",
            {
                functionName: `${appName}-verify-origin-secret-rotation`,
                runtime: lambda.Runtime.NODEJS_20_X,
                code: lambda.Code.fromAsset(
                    path.join(__dirname, "..", "lib", "function", "verify-origin-secret-rotation"),
                ),
                handler: "index.handler",
                logRetention: logs.RetentionDays.ONE_WEEK,
                timeout: Duration.seconds(30),
                environment: {
                    CLOUDFRONT_DISTRIBUTION_ID: webDistribution.distributionId,
                    X_VERIFY_ORIGIN_HEADER_NAME,
                    ORIGIN_TEST_URL: `https://${httpApi.apiId}.execute-api.${this.stack.region}.amazonaws.com/api`,
                },
            },
        );

        webDistribution.grant(
            verifyOriginSecretRotationFunction,
            "cloudfront:GetDistribution",
            "cloudfront:GetDistributionConfig",
            "cloudfront:UpdateDistribution",
        );

        verifyOriginSecret.addRotationSchedule("RotationSchedule", {
            rotationLambda: verifyOriginSecretRotationFunction,
            automaticallyAfter: Duration.days(1),
        });

        new s3_deployment.BucketDeployment(this, "BucketDeployment", {
            sources: [s3_deployment.Source.asset(path.join(__dirname, "..", "..", "ui/build"))],
            destinationBucket: frontendAssetsBucket,
            distribution: webDistribution,
        });
    }

    private createCertificate(domain: Domain | undefined) {
        if (domain === undefined) {
            return undefined;
        }

        const hostedZone = route53.HostedZone.fromLookup(this.globalStack, "HostedZone", {
            domainName: domain.zoneDomainName || domain.domainName,
        });
        const certificate = new acm.Certificate(this.globalStack, "Certificate", {
            domainName: domain.domainName,
            validation: acm.CertificateValidation.fromDns(hostedZone),
        });

        const certificateArn = new CrossRegionStringRef(this, "CertificateArn", {
            constructInOtherRegion: certificate,
            value: (certificate) => certificate.certificateArn,
        }).value;

        return acm.Certificate.fromCertificateArn(this, "Certificate", certificateArn);
    }

    private createWebAcl(appName: string) {
        const ipSet = new wafv2.CfnIPSet(this.globalStack, "IpSet", {
            name: `${appName}-ip-allow-list`,
            addresses: [],
            ipAddressVersion: "IPV4",
            scope: "CLOUDFRONT",
        });

        const ipAllowList: wafv2.CfnWebACL.RuleProperty = {
            name: `${appName}-waf-ip-allow-list-rule`,
            visibilityConfig: {
                cloudWatchMetricsEnabled: true,
                metricName: `${appName}-waf-ip-allow-list`,
                sampledRequestsEnabled: true,
            },
            priority: 0,
            action: { allow: {} },
            statement: {
                ipSetReferenceStatement: {
                    arn: ipSet.attrArn,
                },
            },
        };

        const webAcl = new wafv2.CfnWebACL(this.globalStack, "WebAcl", {
            name: `${appName}-waf`,
            defaultAction: { block: {} },
            visibilityConfig: {
                cloudWatchMetricsEnabled: true,
                metricName: `${appName}-waf`,
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
