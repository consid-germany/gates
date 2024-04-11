import {Construct} from "constructs";
import * as cdk from "aws-cdk-lib";
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

export interface Domain {
    readonly domainName: string;
    readonly zoneDomainName?: string;
    readonly gitHubApiSubdomain?: string;
}

export interface GitHubApi {
    readonly allowedSubPatterns: string[];
}

export interface GatesProps {
    /**
     * A name for the application.
     * If not specified, the default app name `gates` is used.
     */
    readonly appName?: string;
    readonly domain?: Domain;

    readonly ipAllowList?: string[];

    readonly gitHubApi?: GitHubApi;

    readonly globalStackName?: string;
    readonly frontendAssetsBucketName?: string;

    readonly demoMode?: boolean;
}

const SCOPE_CLOUDFRONT = "CLOUDFRONT";
const DEFAULT_APP_NAME = "gates";
const DEFAULT_GITHUB_SUBDOMAIN = "github";
const X_VERIFY_ORIGIN_HEADER_NAME = "x-verify-origin";

export class Gates extends Construct {
    private readonly stack: cdk.Stack;
    private readonly globalStack: cdk.Stack;

    constructor(scope: Construct, id: string, props: GatesProps) {
        super(scope, id);

        const { appName = DEFAULT_APP_NAME } = props;

        this.stack = cdk.Stack.of(this);
        this.globalStack = GlobalStackProvider.getOrCreate(this, {
            stackName: props.globalStackName || `${this.stack.stackName}-global`,
            tags: this.stack.tags.tagValues(),
        });

        const hostedZone = this.lookupHostedZone(props.domain);

        const gatesTable = this.createGatesTable(appName);
        const apiFunction = this.createApiFunction(appName, gatesTable, props.demoMode);

        const verifyOriginSecret = this.createVerifyOriginSecret(appName);
        const verifyOriginAuthFunction = this.createVerifyOriginAuthFunction(
            appName,
            verifyOriginSecret,
        );

        const gatesApi = this.createGatesApi(
            appName,
            apiFunction,
            verifyOriginAuthFunction,
        );

        if (props.gitHubApi !== undefined) {
            const gitHubJwtAuthFunction = this.createGitHubJwtAuthFunction(appName, props.gitHubApi.allowedSubPatterns);

            this.createGitHubApi(
                appName,
                apiFunction,
                gitHubJwtAuthFunction,
                hostedZone,
                props.domain
            );
        }

        const frontendAssetsBucket = this.createFrontendAssetsBucket(
            props.frontendAssetsBucketName,
        );

        const webDistribution = this.createWebDistribution(
            appName,
            frontendAssetsBucket,
            gatesApi,
            verifyOriginSecret,
            hostedZone,
            props.domain,
            props.ipAllowList,
        );

        this.createVerifyOriginSecretRotation(verifyOriginSecret, webDistribution, gatesApi);
        this.createFrontendAssetsDeployment(frontendAssetsBucket, webDistribution);
    }

    private lookupHostedZone(domain?: Domain) {
        if (domain === undefined) {
            return undefined;
        }
        return route53.HostedZone.fromLookup(this, "HostedZone", {
            domainName: domain.zoneDomainName || domain.domainName,
        });
    }

    private createGitHubJwtAuthFunction(appName: string, allowedSubPatterns: string[]) {
        return new lambda.Function(this, "GitHubJwtAuthFunction", {
            functionName: `${appName}-github-jwt-auth`,
            runtime: lambda.Runtime.NODEJS_20_X,
            code: lambda.Code.fromAsset(
                path.join(__dirname, "..", "build", "function", "github-jwt-authorizer"),
            ),
            handler: "index.handler",
            logRetention: logs.RetentionDays.ONE_WEEK,
            environment: {
                ALLOWED_SUB_PATTERNS: JSON.stringify(allowedSubPatterns),
            },
        });
    }

    private createGitHubApi(
        appName: string,
        apiFunction: lambda.Function,
        gitHubJwtAuthFunction: lambda.Function,
        hostedZone?: route53.IHostedZone,
        domain?: Domain,
    ) {
        const apiFunctionIntegration = new apigatewayv2_integrations.HttpLambdaIntegration(
            "ApiFunctionGitHubHttpApiIntegration",
            apiFunction,
        );

        const gitHubJwtAuthorizer = new apigatewayv2_authorizers.HttpLambdaAuthorizer(
            "GitHubJwtAuthorizer",
            gitHubJwtAuthFunction,
            {
                responseTypes: [apigatewayv2_authorizers.HttpLambdaResponseType.SIMPLE],
            },
        );

        const httpApi = new apigatewayv2.HttpApi(this, "GitHubHttpApi", {
            apiName: `${appName}-github-api`,
            defaultDomainMapping: this.createGitHubDomainMapping(hostedZone, domain),
        });

        httpApi.addRoutes({
            integration: apiFunctionIntegration,
            authorizer: gitHubJwtAuthorizer,
            path: "/api/gates/{group}/{service}/{environment}/state",
            methods: [apigatewayv2.HttpMethod.GET]
        });
    }

    private createGitHubDomainMapping(hostedZone?: route53.IHostedZone, domain?: Domain): apigatewayv2.DomainMappingOptions | undefined {
        if (hostedZone === undefined || domain === undefined) {
            return undefined;
        }

        const gitHubApiDomainName = `${domain.gitHubApiSubdomain || DEFAULT_GITHUB_SUBDOMAIN}.${domain.domainName}`;

        const gitHubSubdomainCertificate = new acm.Certificate(this, "GitHubHttpApiCertificate", {
            domainName: gitHubApiDomainName,
            validation: acm.CertificateValidation.fromDns(hostedZone),
        });

        const gitHubHttpApiDomainName = new apigatewayv2.DomainName(this, "GitHubHttpApiDomain", {
            domainName: gitHubApiDomainName,
            certificate: gitHubSubdomainCertificate,
        });

        new route53.ARecord(this, "GitHubHttpApiARecord", {
            recordName: gitHubApiDomainName,
            target: route53.RecordTarget.fromAlias(
                new route53_targets.ApiGatewayv2DomainProperties(
                    gitHubHttpApiDomainName.regionalDomainName,
                    gitHubHttpApiDomainName.regionalHostedZoneId),
            ),
            zone: hostedZone,
        });

        return {
            domainName: gitHubHttpApiDomainName
        };
    }

    private createFrontendAssetsDeployment(
        frontendAssetsBucket: s3.Bucket,
        webDistribution: cloudfront.CloudFrontWebDistribution,
    ) {
        new s3_deployment.BucketDeployment(this, "BucketDeployment", {
            sources: [s3_deployment.Source.asset(path.join(__dirname, "..", "build", "ui"))],
            destinationBucket: frontendAssetsBucket,
            distribution: webDistribution,
        });
    }

    private createFrontendAssetsBucket(bucketName?: string) {
        return new s3.Bucket(this, "FrontendAssetsBucket", {
            bucketName,
            blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
            objectOwnership: s3.ObjectOwnership.BUCKET_OWNER_ENFORCED,
            autoDeleteObjects: true,
            removalPolicy: cdk.RemovalPolicy.DESTROY,
        });
    }

    private createWebDistribution(
        appName: string,
        frontendAssetsBucket: s3.Bucket,
        httpApi: apigatewayv2.HttpApi,
        verifyOriginSecret: secretsmanager.Secret,
        hostedZone?: route53.IHostedZone,
        domain?: Domain,
        ipAllowList?: string[],
    ) {
        const cloudfrontOAI = new cloudfront.OriginAccessIdentity(this, "OriginAccessIdentity");
        frontendAssetsBucket.grantRead(cloudfrontOAI);

        const webDistribution = new cloudfront.CloudFrontWebDistribution(this, "WebDistribution", {
            webACLId: this.createGlobalWebAcl(appName, ipAllowList),
            enableIpV6: false,
            viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
            viewerCertificate: this.createViewerCertificate(domain),
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
                            defaultTtl: cdk.Duration.seconds(0),
                        },
                        {
                            pathPattern: "/api/*",
                            allowedMethods: cloudfront.CloudFrontAllowedMethods.ALL,
                            defaultTtl: cdk.Duration.seconds(0),
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

        if (domain !== undefined && hostedZone !== undefined) {
            new route53.ARecord(this, "ARecord", {
                recordName: domain.domainName,
                target: route53.RecordTarget.fromAlias(
                    new route53_targets.CloudFrontTarget(webDistribution),
                ),
                zone: hostedZone,
            });
        }

        return webDistribution;
    }

    private createGatesApi(
        appName: string,
        apiFunction: lambda.Function,
        verifyOriginAuthFunction: lambda.Function,
    ) {
        const apiFunctionIntegration = new apigatewayv2_integrations.HttpLambdaIntegration(
            "ApiFunctionApiIntegration",
            apiFunction,
        );

        return new apigatewayv2.HttpApi(this, "HttpApi", {
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
    }

    private createVerifyOriginSecretRotation(
        verifyOriginSecret: secretsmanager.Secret,
        webDistribution: cloudfront.CloudFrontWebDistribution,
        httpApi: apigatewayv2.HttpApi,
    ) {
        const verifyOriginSecretRotationFunction = new lambda.Function(
            this,
            "VerifyOriginSecretRotationFunction",
            {
                functionName: `${verifyOriginSecret.secretName}-rotation`,
                runtime: lambda.Runtime.NODEJS_20_X,
                code: lambda.Code.fromAsset(
                    path.join(__dirname, "..", "build", "function", "verify-origin-secret-rotation"),
                ),
                handler: "index.handler",
                logRetention: logs.RetentionDays.ONE_WEEK,
                timeout: cdk.Duration.seconds(30),
                environment: {
                    CLOUDFRONT_DISTRIBUTION_ID: webDistribution.distributionId,
                    X_VERIFY_ORIGIN_HEADER_NAME,
                    ORIGIN_TEST_URL: `https://${httpApi.apiId}.execute-api.${this.stack.region}.amazonaws.com/api/`,
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
            automaticallyAfter: cdk.Duration.days(1),
        });
    }

    private createVerifyOriginAuthFunction(
        appName: string,
        verifyOriginSecret: secretsmanager.Secret,
    ) {
        const verifyOriginAuthFunction = new lambda.Function(this, "VerifyOriginAuthFunction", {
            functionName: `${appName}-verify-origin-auth`,
            runtime: lambda.Runtime.NODEJS_20_X,
            code: lambda.Code.fromAsset(
                path.join(__dirname, "..", "build", "function", "verify-origin-authorizer"),
            ),
            handler: "index.handler",
            logRetention: logs.RetentionDays.ONE_WEEK,
            environment: {
                SECRET_ID: verifyOriginSecret.secretName,
                X_VERIFY_ORIGIN_HEADER_NAME,
            },
        });

        verifyOriginSecret.grantRead(verifyOriginAuthFunction);

        return verifyOriginAuthFunction;
    }

    private createVerifyOriginSecret(appName: string) {
        return new secretsmanager.Secret(this, "VerifyOriginSecret", {
            secretName: `${appName}-verify-origin-secret`,
            generateSecretString: {
                excludePunctuation: true,
            },
        });
    }

    private createApiFunction(appName: string, gatesTable: dynamodb.TableV2, demoMode?: boolean) {
        const apiFunction = new lambda.Function(this, "ApiFunction", {
            functionName: `${appName}-api`,
            runtime: lambda.Runtime.PROVIDED_AL2023,
            architecture: lambda.Architecture.ARM_64,
            code: lambda.Code.fromAsset(
                path.join(__dirname, "..", "build", "api"),
            ),
            handler: "provided",
            environment: {
                GATES_DYNAMO_DB_TABLE_NAME: gatesTable.tableName,
                ...(demoMode && { DEMO_MODE: "true" }),
            },
            logRetention: logs.RetentionDays.ONE_WEEK,
        });

        gatesTable.grantReadWriteData(apiFunction);

        return apiFunction;
    }

    private createViewerCertificate(domain?: Domain) {
        if (domain === undefined) {
            return undefined;
        }

        return cloudfront.ViewerCertificate.fromAcmCertificate(
            this.createGlobalCertificate(domain),
            {
                aliases: [domain.domainName],
            },
        );
    }

    private createGatesTable(appName: string) {
        return new dynamodb.TableV2(this, "GatesTable", {
            tableName: `${appName}`,
            partitionKey: { name: "group", type: dynamodb.AttributeType.STRING },
            sortKey: { name: "service_environment", type: dynamodb.AttributeType.STRING },
        });
    }

    private createGlobalCertificate(domain: Domain) {
        const hostedZone = route53.HostedZone.fromLookup(this.globalStack, "HostedZone", {
            domainName: domain.zoneDomainName || domain.domainName,
        });

        const certificate = new acm.Certificate(this.globalStack, "GlobalCertificate", {
            domainName: domain.domainName,
            validation: acm.CertificateValidation.fromDns(hostedZone),
        });

        const certificateArn = new CrossRegionStringRef(this, "GlobalCertificateArn", {
            constructInOtherRegion: certificate,
            value: (certificate) => certificate.certificateArn,
        }).value;

        return acm.Certificate.fromCertificateArn(this, "GlobalCertificate", certificateArn);
    }

    private createGlobalWebAcl(appName: string, ipAllowList?: string[]) {
        if (ipAllowList === undefined) {
            return undefined;
        }

        const ipSet = new wafv2.CfnIPSet(this.globalStack, "IpSet", {
            name: `${appName}-ip-allow-list`,
            addresses: [...ipAllowList],
            ipAddressVersion: "IPV4",
            scope: "CLOUDFRONT",
        });

        const ipAllowListRule: wafv2.CfnWebACL.RuleProperty = {
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
            rules: [ipAllowListRule],
        });

        return new CrossRegionStringRef(this, "WebAclArn", {
            constructInOtherRegion: webAcl,
            value: (webAcl) => webAcl.attrArn,
        }).value;
    }
}
