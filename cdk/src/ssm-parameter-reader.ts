import { Arn, Stack } from "aws-cdk-lib";
import * as CustomResource from "aws-cdk-lib/custom-resources";
import { Construct } from "constructs";

interface SSMParameterReaderProps {
    parameterName: string;
    region: string;
    resourceType: string;
}

export class SSMParameterReader extends CustomResource.AwsCustomResource {
    constructor(scope: Construct, id: string, props: SSMParameterReaderProps) {
        const onUpdate: CustomResource.AwsSdkCall = {
            service: "SSM",
            action: "getParameter",
            parameters: {
                Name: props.parameterName,
            },
            region: props.region,
            physicalResourceId: CustomResource.PhysicalResourceId.of(Date.now().toString()),
        };

        const policy = CustomResource.AwsCustomResourcePolicy.fromSdkCalls({
            resources: [
                Arn.format(
                    {
                        service: "ssm",
                        region: props.region,
                        resource: "parameter",
                        resourceName: removeLeadingSlash(props.parameterName),
                    },
                    Stack.of(scope),
                ),
            ],
        });

        super(scope, id, { onUpdate, policy, resourceType: props.resourceType });
    }

    get value(): string {
        return this.getResponseField("Parameter.Value");
    }
}

function removeLeadingSlash(value: string): string {
    return value.slice(0, 1) == "/" ? value.slice(1) : value;
}
